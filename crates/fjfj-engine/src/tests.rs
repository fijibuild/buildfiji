use super::*;
use std::sync::atomic::AtomicUsize;

/// A value read from outside the graph, which a test changes.
#[derive(Clone, Default)]
struct Cell(Arc<AtomicU64>);

impl Cell {
    fn set(&self, v: u64) {
        self.0.store(v, Ordering::SeqCst);
    }
}

#[derive(Clone, Debug)]
struct Input {
    cell: Cell,
    reads: Arc<AtomicUsize>,
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cell")
    }
}

// The key is its name; the cell is how the test reaches in.
#[derive(Clone, Debug)]
struct Leaf(&'static str, Input);

impl PartialEq for Leaf {
    fn eq(&self, o: &Leaf) -> bool {
        self.0 == o.0
    }
}
impl Eq for Leaf {}
impl Hash for Leaf {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.hash(h)
    }
}

impl Key for Leaf {
    type Value = u64;
    async fn compute(&self, _: &Ctx) -> Result<u64, Error> {
        self.1.reads.fetch_add(1, Ordering::SeqCst);
        Ok(self.1.cell.0.load(Ordering::SeqCst))
    }
}

/// Depends on a leaf and reduces it to its parity, counting its runs.
#[derive(Clone, Debug)]
struct Parity(Leaf, Arc<AtomicUsize>);

impl PartialEq for Parity {
    fn eq(&self, o: &Parity) -> bool {
        self.0 == o.0
    }
}
impl Eq for Parity {}
impl Hash for Parity {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.hash(h)
    }
}

impl Key for Parity {
    type Value = u64;
    async fn compute(&self, ctx: &Ctx) -> Result<u64, Error> {
        self.1.fetch_add(1, Ordering::SeqCst);
        Ok(*ctx.get(self.0.clone()).await? % 2)
    }
}

#[derive(Clone, Debug)]
struct Top(Parity, Arc<AtomicUsize>);

impl PartialEq for Top {
    fn eq(&self, o: &Top) -> bool {
        self.0 == o.0
    }
}
impl Eq for Top {}
impl Hash for Top {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.hash(h)
    }
}

impl Key for Top {
    type Value = u64;
    async fn compute(&self, ctx: &Ctx) -> Result<u64, Error> {
        self.1.fetch_add(1, Ordering::SeqCst);
        Ok(*ctx.get(self.0.clone()).await? + 100)
    }
}

struct Chain {
    cell: Cell,
    leaf_reads: Arc<AtomicUsize>,
    parity_runs: Arc<AtomicUsize>,
    top_runs: Arc<AtomicUsize>,
    leaf: Leaf,
    top: Top,
}

fn chain() -> Chain {
    let cell = Cell::default();
    let leaf_reads = Arc::new(AtomicUsize::new(0));
    let parity_runs = Arc::new(AtomicUsize::new(0));
    let top_runs = Arc::new(AtomicUsize::new(0));
    let leaf = Leaf(
        "leaf",
        Input {
            cell: cell.clone(),
            reads: leaf_reads.clone(),
        },
    );
    let top = Top(Parity(leaf.clone(), parity_runs.clone()), top_runs.clone());
    Chain {
        cell,
        leaf_reads,
        parity_runs,
        top_runs,
        leaf,
        top,
    }
}

#[tokio::test]
async fn a_value_is_computed_once_however_often_it_is_asked_for() {
    let c = chain();
    c.cell.set(4);
    let engine = Engine::new();
    assert_eq!(*engine.get(c.top.clone()).await.unwrap(), 100);
    assert_eq!(*engine.get(c.top.clone()).await.unwrap(), 100);
    assert_eq!(c.top_runs.load(Ordering::SeqCst), 1);
    assert_eq!(c.parity_runs.load(Ordering::SeqCst), 1);
    assert_eq!(c.leaf_reads.load(Ordering::SeqCst), 1);
    assert_eq!(engine.len(), 3);
}

#[tokio::test]
async fn a_new_version_with_no_change_recomputes_nothing() {
    let c = chain();
    let engine = Engine::new();
    engine.get(c.top.clone()).await.unwrap();
    engine.next_version();
    assert_eq!(*engine.get(c.top.clone()).await.unwrap(), 100);
    assert_eq!(c.top_runs.load(Ordering::SeqCst), 1);
    assert_eq!(c.parity_runs.load(Ordering::SeqCst), 1);
    assert_eq!(c.leaf_reads.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_change_that_does_not_change_a_value_stops_there() {
    let c = chain();
    c.cell.set(2);
    let engine = Engine::new();
    assert_eq!(*engine.get(c.top.clone()).await.unwrap(), 100);
    // 2 to 4: the leaf changes, its parity does not, so the top is kept.
    c.cell.set(4);
    assert!(engine.invalidate(&c.leaf));
    assert_eq!(*engine.get(c.top.clone()).await.unwrap(), 100);
    assert_eq!(c.leaf_reads.load(Ordering::SeqCst), 2);
    assert_eq!(c.parity_runs.load(Ordering::SeqCst), 2);
    assert_eq!(c.top_runs.load(Ordering::SeqCst), 1);
    // 4 to 5: the parity changes, and so does the top.
    c.cell.set(5);
    engine.invalidate(&c.leaf);
    assert_eq!(*engine.get(c.top.clone()).await.unwrap(), 101);
    assert_eq!(c.top_runs.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn invalidating_a_key_the_graph_does_not_have_says_so() {
    let c = chain();
    let engine = Engine::new();
    assert!(!engine.invalidate(&c.leaf));
}

/// A key that takes a while, counting its runs in the counter it carries.
#[derive(Clone, Debug)]
struct Slow(u32, &'static AtomicUsize);

impl PartialEq for Slow {
    fn eq(&self, o: &Slow) -> bool {
        self.0 == o.0
    }
}
impl Eq for Slow {}
impl Hash for Slow {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.0.hash(h)
    }
}

static SHARED_RUNS: AtomicUsize = AtomicUsize::new(0);
static ABANDONED_RUNS: AtomicUsize = AtomicUsize::new(0);

impl Key for Slow {
    type Value = u32;
    async fn compute(&self, _: &Ctx) -> Result<u32, Error> {
        self.1.fetch_add(1, Ordering::SeqCst);
        for _ in 0..50 {
            tokio::task::yield_now().await;
        }
        Ok(self.0 * 2)
    }
}

#[tokio::test]
async fn requests_made_together_share_one_computation() {
    let engine = Engine::new();
    let (a, b, c) = tokio::join!(
        engine.get(Slow(21, &SHARED_RUNS)),
        engine.get(Slow(21, &SHARED_RUNS)),
        engine.get(Slow(21, &SHARED_RUNS))
    );
    assert_eq!((*a.unwrap(), *b.unwrap(), *c.unwrap()), (42, 42, 42));
    assert_eq!(SHARED_RUNS.load(Ordering::SeqCst), 1);
}

/// `Link(n)` depends on `Link(n + 1)` up to `Link(last)`, which depends on
/// `Link(first)` when `closes`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Link {
    n: u32,
    last: u32,
    closes: bool,
}

impl Key for Link {
    type Value = u32;
    async fn compute(&self, ctx: &Ctx) -> Result<u32, Error> {
        if self.n < self.last {
            let next = Link {
                n: self.n + 1,
                ..self.clone()
            };
            Ok(*ctx.get(next).await? + 1)
        } else if self.closes {
            let first = Link {
                n: 0,
                ..self.clone()
            };
            Ok(*ctx.get(first).await? + 1)
        } else {
            Ok(0)
        }
    }
}

#[tokio::test]
async fn a_chain_is_not_a_cycle() {
    let engine = Engine::new();
    let end = engine
        .get(Link {
            n: 0,
            last: 5,
            closes: false,
        })
        .await
        .unwrap();
    assert_eq!(*end, 5);
}

#[tokio::test]
async fn a_cycle_is_an_error_not_a_hang() {
    let engine = Engine::new();
    let err = engine
        .get(Link {
            n: 0,
            last: 3,
            closes: true,
        })
        .await
        .unwrap_err();
    let Error::Cycle(keys) = &err else {
        panic!("{err}");
    };
    assert_eq!(keys.len(), 5, "{keys:?}");
    // The last link closes it: it first, then the way back to it.
    assert!(
        keys[0].contains("n: 3") && keys[4].contains("n: 3"),
        "{keys:?}"
    );
    assert!(keys[1].contains("n: 0"), "{keys:?}");
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Pair {
    me: &'static str,
    other: &'static str,
}

impl Key for Pair {
    type Value = ();
    async fn compute(&self, ctx: &Ctx) -> Result<(), Error> {
        // Let the other side start before asking for it.
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
        ctx.get(Pair {
            me: self.other,
            other: self.me,
        })
        .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Both;

impl Key for Both {
    type Value = usize;
    async fn compute(&self, ctx: &Ctx) -> Result<usize, Error> {
        // Two branches that wait on each other, started together.
        let results = futures::future::join(
            ctx.get(Pair {
                me: "a",
                other: "b",
            }),
            ctx.get(Pair {
                me: "b",
                other: "a",
            }),
        )
        .await;
        Ok(usize::from(results.0.is_err()) + usize::from(results.1.is_err()))
    }
}

#[tokio::test]
async fn a_cycle_between_branches_that_run_together_is_found() {
    let engine = Engine::new();
    let errors = engine.get(Both).await.unwrap();
    assert_eq!(*errors, 2);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Fails;

static FAILS_RUNS: AtomicUsize = AtomicUsize::new(0);

impl Key for Fails {
    type Value = u32;
    async fn compute(&self, _: &Ctx) -> Result<u32, Error> {
        FAILS_RUNS.fetch_add(1, Ordering::SeqCst);
        Err(Error::msg("no good"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DependsOnFailure;

impl Key for DependsOnFailure {
    type Value = u32;
    async fn compute(&self, ctx: &Ctx) -> Result<u32, Error> {
        Ok(*ctx.get(Fails).await? + 1)
    }
}

#[tokio::test]
async fn an_error_is_a_memoised_value_that_dependents_propagate() {
    let engine = Engine::new();
    assert_eq!(
        engine.get(DependsOnFailure).await.unwrap_err().to_string(),
        "no good"
    );
    assert_eq!(
        engine.get(DependsOnFailure).await.unwrap_err().to_string(),
        "no good"
    );
    assert_eq!(engine.get(Fails).await.unwrap_err().to_string(), "no good");
    assert_eq!(FAILS_RUNS.load(Ordering::SeqCst), 1);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Many(u32);

impl Key for Many {
    type Value = u32;
    async fn compute(&self, ctx: &Ctx) -> Result<u32, Error> {
        if self.0 == 0 {
            let kids = ctx.get_all((1..=8).map(Many).collect()).await;
            Ok(kids.into_iter().map(|k| *k.unwrap()).sum())
        } else {
            tokio::task::yield_now().await;
            Ok(self.0)
        }
    }
}

#[tokio::test]
async fn keys_are_asked_for_in_parallel() {
    let engine = Engine::new();
    assert_eq!(*engine.get(Many(0)).await.unwrap(), 36);
}

#[tokio::test]
async fn a_request_given_up_on_does_not_wedge_the_node() {
    let engine = Engine::new();
    let first = engine.get(Slow(7, &ABANDONED_RUNS));
    // Poll it until the computation has started, then drop it.
    let mut first = Box::pin(first);
    while ABANDONED_RUNS.load(Ordering::SeqCst) == 0 {
        let _ = futures::poll!(first.as_mut());
        tokio::task::yield_now().await;
    }
    drop(first);
    assert_eq!(*engine.get(Slow(7, &ABANDONED_RUNS)).await.unwrap(), 14);
    // The abandoned computation was stopped, and the new request computed it again.
    assert_eq!(ABANDONED_RUNS.load(Ordering::SeqCst), 2);
}

/// A tree of keys that never finish by themselves, counting those running.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Forever(u32);

static FOREVER_STARTED: AtomicUsize = AtomicUsize::new(0);
static FOREVER_LIVE: AtomicUsize = AtomicUsize::new(0);

struct Live;

impl Live {
    fn enter() -> Live {
        FOREVER_STARTED.fetch_add(1, Ordering::SeqCst);
        FOREVER_LIVE.fetch_add(1, Ordering::SeqCst);
        Live
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        FOREVER_LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Key for Forever {
    type Value = u32;
    async fn compute(&self, ctx: &Ctx) -> Result<u32, Error> {
        let _live = Live::enter();
        if self.0 < 3 {
            let kids = ctx
                .get_all(vec![Forever(self.0 + 1), Forever(self.0 + 1 + 10)])
                .await;
            return Ok(kids.len() as u32);
        }
        std::future::pending().await
    }
}

#[tokio::test]
async fn dropping_the_root_request_stops_every_task_beneath_it() {
    let engine = Engine::new();
    let mut root = Box::pin(engine.get(Forever(0)));
    while FOREVER_LIVE.load(Ordering::SeqCst) < 7 {
        let _ = futures::poll!(root.as_mut());
        tokio::task::yield_now().await;
    }
    drop(root);
    for _ in 0..100 {
        if FOREVER_LIVE.load(Ordering::SeqCst) == 0 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(FOREVER_LIVE.load(Ordering::SeqCst), 0);
    // The nodes can be asked for again: nothing is wedged in `Running`.
    let started = FOREVER_STARTED.load(Ordering::SeqCst);
    let mut again = Box::pin(engine.get(Forever(0)));
    while FOREVER_STARTED.load(Ordering::SeqCst) == started {
        let _ = futures::poll!(again.as_mut());
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn a_computation_another_request_still_waits_on_keeps_running() {
    static RUNS: AtomicUsize = AtomicUsize::new(0);
    let engine = Engine::new();
    let first = engine.get(Slow(9, &RUNS));
    let other = engine.clone();
    let mut first = Box::pin(first);
    let _ = futures::poll!(first.as_mut());
    let second = tokio::spawn(async move { other.get(Slow(9, &RUNS)).await });
    tokio::task::yield_now().await;
    drop(first);
    assert_eq!(*second.await.unwrap().unwrap(), 18);
    assert_eq!(RUNS.load(Ordering::SeqCst), 1);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Greeting;

impl Key for Greeting {
    type Value = String;
    async fn compute(&self, ctx: &Ctx) -> Result<String, Error> {
        Ok(format!("hello, {}", ctx.data::<&'static str>()?))
    }
}

#[tokio::test]
async fn computations_read_the_data_the_engine_was_given() {
    let engine = Engine::new();
    assert!(
        engine
            .get(Greeting)
            .await
            .unwrap_err()
            .to_string()
            .contains("no &str")
    );
    let engine = Engine::new();
    engine.set_data("world");
    assert_eq!(*engine.get(Greeting).await.unwrap(), "hello, world");
}
