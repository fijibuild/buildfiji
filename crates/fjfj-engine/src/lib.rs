//! A demand-driven, memoising key/value graph with versioned dependency
//! edges and early cutoff (buildfiji-23d.3).
//!
//! The rules are the ones `spec/Fjfj/Engine.lean` states:
//!
//! - every node has the version at which its value last *changed* and the
//!   version at which it was last *verified*; re-verifying does not move
//!   `changed_at`;
//! - a dependent records, for each dependency, the `changed_at` it observed.
//!   At a later version it is up to date if no dependency has changed since
//!   it observed it, and then only its `verified_at` moves (early cutoff).
//!   A recomputation that gives an equal value leaves `changed_at` alone, so
//!   its own dependents are cut off too;
//! - all reads of one evaluation are taken at one version.
//!
//! Keys are Rust types implementing [`Key`]; the graph holds them all, so a
//! package, a configured target and an action can depend on each other. A
//! key's computation is `async` and asks for its dependencies through
//! [`Ctx::get`], which records them. An in-flight computation is shared by
//! everything that asks for it, and a request that would wait on itself, by
//! any route, gets [`Error::Cycle`] instead of hanging.
//!
//! Errors are values: a failed computation is memoised like any other, and a
//! dependent that propagates it fails the same way.
//!
//! A computation runs as a task of its own, and stops when the last request
//! waiting on it is dropped: the task is aborted, which drops the requests it
//! was making in turn, and the node goes back to what it was (so a later
//! request computes it again). Work handed to `spawn_blocking` is not
//! interrupted.
//!
//! Not here yet: persistence and a bound on memory.

use futures::future::{BoxFuture, FutureExt, Shared};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::task::AbortHandle;

/// A point in the graph's history. Moves forward when inputs may have changed.
pub type Version = u64;

/// Why a computation did not give a value. Cheap to clone and memoised.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Failed(Arc<dyn std::error::Error + Send + Sync>),
    /// The keys of a cycle, each waiting on the next, the first on the last.
    #[error("cycle: {}", .0.join(" -> "))]
    Cycle(Vec<String>),
}

impl Error {
    pub fn new(error: impl std::error::Error + Send + Sync + 'static) -> Error {
        Error::Failed(Arc::new(error))
    }

    pub fn msg(message: impl Into<String>) -> Error {
        #[derive(Debug, thiserror::Error)]
        #[error("{0}")]
        struct Message(String);
        Error::new(Message(message.into()))
    }
}

/// Errors are equal when they say the same, which is what early cutoff needs.
impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        self.to_string() == other.to_string()
    }
}

/// A kind of node. The value type is what the key computes; equal values
/// are what lets a dependent keep its own.
pub trait Key: Debug + Hash + Eq + Clone + Send + Sync + 'static {
    type Value: PartialEq + Debug + Send + Sync + 'static;

    fn compute(&self, ctx: &Ctx) -> impl Future<Output = Result<Self::Value, Error>> + Send;
}

type AnyValue = Arc<dyn Any + Send + Sync>;
type Outcome = Result<AnyValue, Error>;

/// A [`Key`] with its type forgotten.
trait ErasedKey: Send + Sync + Debug {
    fn compute<'a>(&'a self, ctx: &'a Ctx) -> BoxFuture<'a, Outcome>;
    fn values_equal(&self, a: &AnyValue, b: &AnyValue) -> bool;
    fn key_eq(&self, other: &dyn ErasedKey) -> bool;
    fn key_hash(&self, state: &mut dyn Hasher);
    fn as_any(&self) -> &dyn Any;
}

impl<K: Key> ErasedKey for K {
    fn compute<'a>(&'a self, ctx: &'a Ctx) -> BoxFuture<'a, Outcome> {
        use tracing::Instrument as _;
        let span = tracing::debug_span!("key", kind = std::any::type_name::<K>(), key = ?self);
        Box::pin(
            async move {
                let value = Key::compute(self, ctx).await?;
                Ok(Arc::new(value) as AnyValue)
            }
            .instrument(span),
        )
    }

    fn values_equal(&self, a: &AnyValue, b: &AnyValue) -> bool {
        match (a.downcast_ref::<K::Value>(), b.downcast_ref::<K::Value>()) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    fn key_eq(&self, other: &dyn ErasedKey) -> bool {
        other.as_any().downcast_ref::<K>() == Some(self)
    }

    fn key_hash(&self, mut state: &mut dyn Hasher) {
        TypeId::of::<K>().hash(&mut state);
        self.hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A key as a map key.
#[derive(Clone)]
struct KeyBox(Arc<dyn ErasedKey>);

impl PartialEq for KeyBox {
    fn eq(&self, other: &KeyBox) -> bool {
        self.0.key_eq(&*other.0)
    }
}

impl Eq for KeyBox {}

impl Hash for KeyBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.key_hash(state);
    }
}

/// What a node last computed and what it depended on.
struct Evaluated {
    outcome: Outcome,
    changed_at: Version,
    verified_at: Version,
    /// Each dependency and its `changed_at` when this node read it.
    deps: Vec<(Arc<Node>, Version)>,
}

type Running = Shared<BoxFuture<'static, Result<Arc<Evaluated>, Error>>>;

enum State {
    Empty,
    Done(Arc<Evaluated>),
    Running(Arc<Computation>),
}

/// A computation in flight, and how many requests are waiting for it.
struct Computation {
    version: Version,
    future: Running,
    waiters: AtomicUsize,
    abort: AbortHandle,
    /// What the node held before, to go back to if this is abandoned.
    previous: Option<Arc<Evaluated>>,
    /// The node was marked dirty when this started.
    dirty: bool,
}

/// One request waiting for a [`Computation`]; the last to go stops it.
struct Waiter {
    node: Arc<Node>,
    computation: Arc<Computation>,
}

impl Waiter {
    /// Join `computation`; call with the node's state lock held.
    fn join(node: &Arc<Node>, computation: &Arc<Computation>) -> Waiter {
        computation.waiters.fetch_add(1, Ordering::AcqRel);
        Waiter {
            node: node.clone(),
            computation: computation.clone(),
        }
    }
}

impl Drop for Waiter {
    fn drop(&mut self) {
        if self.computation.waiters.fetch_sub(1, Ordering::AcqRel) != 1 {
            return;
        }
        let abandoned = {
            let mut state = self.node.state.lock().unwrap();
            // Someone may have joined, or the computation finished, since.
            let ours = matches!(&*state, State::Running(c) if Arc::ptr_eq(c, &self.computation))
                && self.computation.waiters.load(Ordering::Acquire) == 0;
            if !ours {
                return;
            }
            self.computation.abort.abort();
            if self.computation.dirty {
                self.node.dirty.store(true, Ordering::Release);
            }
            let back = match &self.computation.previous {
                Some(previous) => State::Done(previous.clone()),
                None => State::Empty,
            };
            std::mem::replace(&mut *state, back)
        };
        // Dropped with no lock held.
        drop(abandoned);
    }
}

struct Node {
    id: u64,
    key: Arc<dyn ErasedKey>,
    state: Mutex<State>,
    /// An input that may have changed: recompute, do not verify.
    dirty: AtomicBool,
}

struct Inner {
    version: AtomicU64,
    next_id: AtomicU64,
    nodes: Mutex<HashMap<KeyBox, Arc<Node>>>,
    /// The nodes each running node is waiting on, for finding cycles.
    waits: Mutex<HashMap<u64, Vec<Arc<Node>>>>,
    /// What computations read that is not a key: the repositories of the
    /// build, the options. One value of each type.
    data: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

/// The graph.
#[derive(Clone)]
pub struct Engine {
    inner: Arc<Inner>,
}

impl Default for Engine {
    fn default() -> Engine {
        Engine::new()
    }
}

/// What a computation reads its dependencies through.
pub struct Ctx {
    inner: Arc<Inner>,
    node: Option<Arc<Node>>,
    version: Version,
    deps: Mutex<Vec<(Arc<Node>, Version)>>,
}

/// A wait edge, removed when the wait ends or is given up.
struct Waiting {
    inner: Arc<Inner>,
    from: u64,
    to: u64,
}

impl Drop for Waiting {
    fn drop(&mut self) {
        let mut waits = self.inner.waits.lock().unwrap();
        if let Some(list) = waits.get_mut(&self.from) {
            if let Some(at) = list.iter().position(|n| n.id == self.to) {
                list.swap_remove(at);
            }
            if list.is_empty() {
                waits.remove(&self.from);
            }
        }
    }
}

impl Inner {
    fn node(self: &Arc<Self>, key: KeyBox) -> Arc<Node> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes
            .entry(key.clone())
            .or_insert_with(|| {
                Arc::new(Node {
                    id: self.next_id.fetch_add(1, Ordering::Relaxed),
                    key: key.0.clone(),
                    state: Mutex::new(State::Empty),
                    dirty: AtomicBool::new(false),
                })
            })
            .clone()
    }

    /// Wait on `to` for `from` (a computation, or nothing for a request
    /// from outside), refusing a wait that would close a cycle.
    fn wait_on(
        self: &Arc<Self>,
        from: Option<&Arc<Node>>,
        to: &Arc<Node>,
    ) -> Result<Option<Waiting>, Error> {
        let Some(from) = from else {
            return Ok(None);
        };
        let mut waits = self.waits.lock().unwrap();
        // Is `from` reachable from `to`? The path back is the cycle.
        let mut path = vec![to.clone()];
        let mut seen = vec![to.id];
        if reaches(&waits, to, from.id, &mut path, &mut seen) || to.id == from.id {
            let mut keys = vec![format!("{:?}", from.key)];
            keys.extend(path.iter().map(|n| format!("{:?}", n.key)));
            return Err(Error::Cycle(keys));
        }
        waits.entry(from.id).or_default().push(to.clone());
        drop(waits);
        Ok(Some(Waiting {
            inner: self.clone(),
            from: from.id,
            to: to.id,
        }))
    }
}

fn reaches(
    waits: &HashMap<u64, Vec<Arc<Node>>>,
    at: &Arc<Node>,
    target: u64,
    path: &mut Vec<Arc<Node>>,
    seen: &mut Vec<u64>,
) -> bool {
    if at.id == target {
        return true;
    }
    for next in waits.get(&at.id).into_iter().flatten() {
        if seen.contains(&next.id) {
            continue;
        }
        seen.push(next.id);
        path.push(next.clone());
        if reaches(waits, next, target, path, seen) {
            return true;
        }
        path.pop();
    }
    false
}

/// Bring `node` up to date at `version`, sharing the work with whoever else
/// asks.
///
/// Boxed, because it and `run` call each other.
fn evaluate(
    inner: Arc<Inner>,
    node: Arc<Node>,
    version: Version,
) -> BoxFuture<'static, Result<Arc<Evaluated>, Error>> {
    async move { evaluate_now(&inner, &node, version).await }.boxed()
}

async fn evaluate_now(
    inner: &Arc<Inner>,
    node: &Arc<Node>,
    version: Version,
) -> Result<Arc<Evaluated>, Error> {
    /// What the state of the node says to do, decided with its lock held.
    enum Step {
        Done(Arc<Evaluated>),
        Await(Waiter),
        /// Another version is being worked out; let it finish, then look again.
        Other(Running),
    }
    loop {
        let step = {
            let mut state = node.state.lock().unwrap();
            match &*state {
                State::Done(done)
                    if done.verified_at >= version && !node.dirty.load(Ordering::Acquire) =>
                {
                    Step::Done(done.clone())
                }
                State::Running(computation) => {
                    if computation.version == version {
                        Step::Await(Waiter::join(node, computation))
                    } else {
                        Step::Other(computation.future.clone())
                    }
                }
                State::Empty | State::Done(_) => {
                    let previous = match &*state {
                        State::Done(done) => Some(done.clone()),
                        _ => None,
                    };
                    let dirty = node.dirty.swap(false, Ordering::AcqRel);
                    // A task of its own, so that independent keys run on
                    // different workers instead of inside whichever poll
                    // first asked for them.
                    let task = tokio::spawn(run(
                        inner.clone(),
                        node.clone(),
                        previous.clone(),
                        dirty,
                        version,
                    ));
                    let abort = task.abort_handle();
                    let future: Running = async move {
                        task.await
                            .map_err(|e| Error::msg(format!("a computation panicked: {e}")))?
                    }
                    .boxed()
                    .shared();
                    let computation = Arc::new(Computation {
                        version,
                        future,
                        waiters: AtomicUsize::new(0),
                        abort,
                        previous,
                        dirty,
                    });
                    let waiter = Waiter::join(node, &computation);
                    *state = State::Running(computation);
                    Step::Await(waiter)
                }
            }
        };
        match step {
            Step::Done(done) => return Ok(done),
            Step::Await(waiter) => {
                let result = waiter.computation.future.clone().await;
                drop(waiter);
                return result;
            }
            Step::Other(future) => {
                let _ = future.await;
            }
        }
    }
}

/// Verify a stale node or compute it.
async fn run(
    inner: Arc<Inner>,
    node: Arc<Node>,
    previous: Option<Arc<Evaluated>>,
    dirty: bool,
    version: Version,
) -> Result<Arc<Evaluated>, Error> {
    if let (Some(previous), false) = (&previous, dirty) {
        let mut unchanged = true;
        for (dep, observed) in &previous.deps {
            let _waiting = inner.wait_on(Some(&node), dep)?;
            match evaluate(inner.clone(), dep.clone(), version).await {
                Ok(done) if done.changed_at <= *observed => {}
                _ => {
                    unchanged = false;
                    break;
                }
            }
        }
        if unchanged {
            let kept = Arc::new(Evaluated {
                outcome: previous.outcome.clone(),
                changed_at: previous.changed_at,
                verified_at: version,
                deps: previous.deps.clone(),
            });
            *node.state.lock().unwrap() = State::Done(kept.clone());
            return Ok(kept);
        }
    }
    let ctx = Ctx {
        inner: inner.clone(),
        node: Some(node.clone()),
        version,
        deps: Mutex::new(Vec::new()),
    };
    let outcome = node.key.compute(&ctx).await;
    let deps = ctx.deps.into_inner().unwrap();
    let same = previous
        .as_ref()
        .is_some_and(|p| match (&p.outcome, &outcome) {
            (Ok(a), Ok(b)) => node.key.values_equal(a, b),
            (Err(a), Err(b)) => a == b,
            _ => false,
        });
    let changed_at = match (&previous, same) {
        (Some(p), true) => p.changed_at,
        _ => version,
    };
    let done = Arc::new(Evaluated {
        outcome,
        changed_at,
        verified_at: version,
        deps,
    });
    *node.state.lock().unwrap() = State::Done(done.clone());
    Ok(done)
}

fn typed<K: Key>(done: &Evaluated) -> Result<Arc<K::Value>, Error> {
    match &done.outcome {
        Ok(value) => value
            .clone()
            .downcast::<K::Value>()
            .map_err(|_| Error::msg("a key's value has another type than its key says")),
        Err(e) => Err(e.clone()),
    }
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            inner: Arc::new(Inner {
                version: AtomicU64::new(0),
                next_id: AtomicU64::new(0),
                nodes: Mutex::new(HashMap::new()),
                waits: Mutex::new(HashMap::new()),
                data: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Make `value` available to computations as [`Ctx::data`], replacing
    /// the one of its type. It is not a dependency: a computation that read
    /// the old one is not recomputed, so give the graph a new version and
    /// invalidate what read it.
    pub fn set_data<T: Send + Sync + 'static>(&self, value: T) {
        self.inner
            .data
            .lock()
            .unwrap()
            .insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn version(&self) -> Version {
        self.inner.version.load(Ordering::Acquire)
    }

    /// Start a new version. Nodes are brought up to date on demand.
    pub fn next_version(&self) -> Version {
        self.inner.version.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// Say that what `key` reads from outside the graph may have changed; its
    /// next evaluation recomputes it, and its dependents are recomputed only
    /// if it gives another value. Starts a new version. Returns whether the
    /// graph had the key.
    pub fn invalidate<K: Key>(&self, key: &K) -> bool {
        let node = self
            .inner
            .nodes
            .lock()
            .unwrap()
            .get(&KeyBox(Arc::new(key.clone())))
            .cloned();
        self.next_version();
        match node {
            Some(node) => {
                node.dirty.store(true, Ordering::Release);
                true
            }
            None => false,
        }
    }

    /// The value of `key` at the current version.
    pub async fn get<K: Key>(&self, key: K) -> Result<Arc<K::Value>, Error> {
        let node = self.inner.node(KeyBox(Arc::new(key)));
        let done = evaluate(self.inner.clone(), node, self.version()).await?;
        typed::<K>(&done)
    }

    /// How many nodes the graph holds.
    pub fn len(&self) -> usize {
        self.inner.nodes.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Ctx {
    pub fn version(&self) -> Version {
        self.version
    }

    /// The value of type `T` given to [`Engine::set_data`].
    pub fn data<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, Error> {
        self.inner
            .data
            .lock()
            .unwrap()
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(|value| value.downcast::<T>().ok())
            .ok_or_else(|| Error::msg(format!("the engine has no {}", std::any::type_name::<T>())))
    }

    /// The value of `key`, which this computation now depends on.
    pub async fn get<K: Key>(&self, key: K) -> Result<Arc<K::Value>, Error> {
        let node = self.inner.node(KeyBox(Arc::new(key)));
        tracing::trace!(target: "fjfj_engine::request", from = ?self.node.as_ref().map(|n| &n.key), to = ?node.key);
        let _waiting = self.inner.wait_on(self.node.as_ref(), &node)?;
        let done = evaluate(self.inner.clone(), node.clone(), self.version).await?;
        self.deps.lock().unwrap().push((node, done.changed_at));
        typed::<K>(&done)
    }

    /// Several keys at once, in parallel.
    pub async fn get_all<K: Key>(&self, keys: Vec<K>) -> Vec<Result<Arc<K::Value>, Error>> {
        futures::future::join_all(keys.into_iter().map(|key| self.get(key))).await
    }
}

#[cfg(test)]
mod tests;
