//! The loader (buildfiji-mum.19, mum.20, mum.21): reads `.bzl` files, evaluates
//! each once however many files and threads load it, and keeps the frozen
//! module and never the syntax tree or the source.
//!
//! One [`BzlLoader`] serves a build. It is `Sync`: any number of threads may
//! load packages through it at once ([`BzlLoader::load_package`]), and a
//! `.bzl` that two of them need is evaluated by one while the other waits for
//! its frozen module. A load that comes back to a file being evaluated is a
//! cycle, whether the same thread or two that wait on each other's files
//! closes it; the error names the files, and neither thread waits forever.
//!
//! What Bazel 9.2.0 says of a load, read off probes: the label must name a
//! `.bzl` or `.scl` file (`in load statement: ...`); the `.bzl`'s package must
//! exist (`Every .bzl file must have a corresponding package, but ...`); the
//! file must exist (`cannot load '//a:x.bzl': no such file`, `... is a
//! directory`); a label may not reach into a subpackage; an unknown repo is
//! `Unable to find package for ...: The repository '...' could not be
//! resolved: No repository visible as '@nope' from main repository.`; and
//! then the file's load visibility decides (`ps4`), unless
//! `--check_bzl_visibility=false`.

use crate::label::{BzlFile, RepoMappings, bzl_name, evaluate_bzl};
use crate::load_visibility::{check_load_visibility, load_visibility};
use crate::native::{BuildFile, BuildFileError, BuildFileOutput, bzl_globals, evaluate_build_file};
use fjfj_graph::package::check_subpackage_crossing;
use fjfj_graph::{Label, LabelContext};
use fjfj_loading::PackageLookup;
use starlark::environment::{FrozenModule, Globals};
use starlark::eval::FileLoader;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;

/// Where one load stands.
enum SlotState {
    Empty,
    Running(ThreadId),
    Done(Result<FrozenModule, Arc<String>>),
}

/// The one evaluation of one `.bzl` file.
struct Slot {
    state: Mutex<SlotState>,
    done: Condvar,
}

/// A `.bzl` evaluated, or why it was not.
type Loaded = Result<FrozenModule, Arc<String>>;

thread_local! {
    /// The files this thread is evaluating, outermost first.
    static EVALUATING: RefCell<Vec<Label>> = const { RefCell::new(Vec::new()) };
}

/// What loads the files of a build.
pub struct BzlLoader {
    repos: HashMap<String, PackageLookup>,
    mappings: RepoMappings,
    globals: Globals,
    check_bzl_visibility: bool,
    slots: Mutex<HashMap<(String, String, String), Arc<Slot>>>,
    /// The thread each thread is waiting for a file from.
    waiting: Mutex<HashMap<ThreadId, ThreadId>>,
    /// How many files have been evaluated.
    evaluations: AtomicUsize,
}

/// A loader for the loads one file makes: who is loading decides what a
/// relative label means, which repos it can name, and what it may load.
pub struct Importing<'a> {
    loader: &'a BzlLoader,
    importer: Label,
}

impl FileLoader for Importing<'_> {
    fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
        self.loader
            .load(&self.importer, path)
            .map_err(|message| starlark::Error::new_other(anyhow::anyhow!("{message}")))
    }
}

impl BzlLoader {
    /// A loader over the repos in `repos` (canonical name to the directory
    /// that holds it, as a [`PackageLookup`]), whose `.bzl` files name
    /// repos through `mappings`.
    pub fn new(
        repos: HashMap<String, PackageLookup>,
        mappings: RepoMappings,
        check_bzl_visibility: bool,
    ) -> BzlLoader {
        BzlLoader {
            repos,
            mappings,
            globals: bzl_globals(),
            check_bzl_visibility,
            slots: Mutex::new(HashMap::new()),
            waiting: Mutex::new(HashMap::new()),
            evaluations: AtomicUsize::new(0),
        }
    }

    /// How many `.bzl` files have been evaluated: one for each, however many
    /// files loaded it.
    pub fn evaluations(&self) -> usize {
        self.evaluations.load(Ordering::Relaxed)
    }

    /// The loader for the loads a file of `importer`'s package makes.
    pub fn importing(&self, importer: Label) -> Importing<'_> {
        Importing {
            loader: self,
            importer,
        }
    }

    /// Evaluate the BUILD file of `package` in `repo`: what `fjfj build`
    /// does for each package, from any number of threads.
    pub fn load_package(
        &self,
        repo: &str,
        package: &str,
    ) -> Result<BuildFileOutput, BuildFileError> {
        let Some(lookup) = self.repos.get(repo) else {
            return Err(BuildFileError::Eval(anyhow::anyhow!(
                "no such repository '@@{repo}'"
            )));
        };
        let build = lookup
            .build_file(package)
            .map_err(|e| BuildFileError::Eval(anyhow::anyhow!("{e}")))?;
        let source = {
            let _span = tracing::debug_span!("read", file = %build.display()).entered();
            std::fs::read_to_string(&build)
                .map_err(|e| BuildFileError::Eval(anyhow::anyhow!("{}: {e}", build.display())))?
        };
        let importer = Label {
            repo: repo.to_owned(),
            package: package.to_owned(),
            name: build
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        };
        let path = if package.is_empty() {
            importer.name.clone()
        } else {
            format!("{package}/{}", importer.name)
        };
        let loader = self.importing(importer);
        evaluate_build_file(&BuildFile {
            repo,
            package,
            lookup,
            mappings: &self.mappings,
            path: &path,
            source: &source,
            loader: &loader,
        })
    }

    /// Whether `file` (canonical) is a package of its repo.
    fn is_package(&self, repo: &str, package: &str) -> bool {
        self.repos.get(repo).is_some_and(|l| l.is_package(package))
    }

    /// Serve one `load("<written>", ...)` made by a file of `importer`'s
    /// package.
    fn load(&self, importer: &Label, written: &str) -> Result<FrozenModule, String> {
        let file = self.parse(importer, written)?;
        let module = self.loaded(&file).map_err(|e| (*e).clone())?;
        check_load_visibility(
            importer,
            &file,
            &load_visibility(&module),
            self.check_bzl_visibility,
        )?;
        Ok(module)
    }

    /// The file a load statement names, checked as Bazel checks it before
    /// reading anything.
    fn parse(&self, importer: &Label, written: &str) -> Result<Label, String> {
        let ctx = LabelContext {
            repo: &importer.repo,
            package: &importer.package,
        };
        let file = Label::parse_mapped(written, ctx, &mut |apparent| {
            self.mappings.resolve_apparent(&importer.repo, apparent)
        })
        .map_err(|e| format!("in load statement: {e}"))?;
        if !(file.name.ends_with(".bzl") || file.name.ends_with(".scl")) {
            return Err(
                "in load statement: The label must reference a file with extension \".bzl\" or \
                 \".scl\""
                    .to_owned(),
            );
        }
        if let Some(unknown) = file.repo.strip_prefix('[') {
            let apparent = unknown.split('\'').nth(1).unwrap_or_default();
            let from = if importer.repo.is_empty() {
                "main repository".to_owned()
            } else {
                format!("repository '@@{}'", importer.repo)
            };
            return Err(format!(
                "Unable to find package for {}: The repository '@@[{}' could not be resolved: No \
                 repository visible as '@{apparent}' from {from}.",
                crate::label::display_label(&file),
                unknown.trim_end_matches('/'),
            ));
        }
        if !self.repos.contains_key(&file.repo) {
            return Err(format!(
                "Unable to find package for {}: The repository '@@{}' could not be resolved: \
                 Repository '@@{}' is not defined.",
                crate::label::display_label(&file),
                file.repo,
                file.repo
            ));
        }
        check_subpackage_crossing(&file.repo, &file.package, &file.name, &|p| {
            self.is_package(&file.repo, p)
        })
        .map_err(|e| e.to_string())?;
        if !self.is_package(&file.repo, &file.package) {
            return Err(format!(
                "Every .bzl file must have a corresponding package, but '{}' does not have one. \
                 Please create a BUILD file in the same or any parent directory. Note that this \
                 BUILD file does not need to do anything except exist.",
                crate::label::display_label(&file)
            ));
        }
        Ok(file)
    }

    /// The frozen module of `file`, evaluated by the first to ask and shared.
    fn loaded(&self, file: &Label) -> Loaded {
        let key = (file.repo.clone(), file.package.clone(), file.name.clone());
        let slot = self
            .slots
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(|| {
                Arc::new(Slot {
                    state: Mutex::new(SlotState::Empty),
                    done: Condvar::new(),
                })
            })
            .clone();
        let me = std::thread::current().id();
        let mut state = slot.state.lock().unwrap();
        loop {
            match &*state {
                SlotState::Done(result) => return result.clone(),
                SlotState::Running(owner) if *owner == me => {
                    return Err(Arc::new(cycle(file)));
                }
                SlotState::Running(owner) => {
                    let owner = *owner;
                    {
                        // Wait for its owner, unless that thread is, through
                        // others, waiting for this one.
                        let mut waiting = self.waiting.lock().unwrap();
                        let mut at = owner;
                        while let Some(&next) = waiting.get(&at) {
                            if next == me {
                                return Err(Arc::new(cycle(file)));
                            }
                            at = next;
                        }
                        waiting.insert(me, owner);
                    }
                    state = slot.done.wait(state).unwrap();
                    self.waiting.lock().unwrap().remove(&me);
                }
                SlotState::Empty => {
                    *state = SlotState::Running(me);
                    break;
                }
            }
        }
        drop(state);
        EVALUATING.with(|stack| stack.borrow_mut().push(file.clone()));
        let result = self.evaluate(file).map_err(Arc::new);
        EVALUATING.with(|stack| stack.borrow_mut().pop());
        *slot.state.lock().unwrap() = SlotState::Done(result.clone());
        slot.done.notify_all();
        result
    }

    /// Read and evaluate `file`.
    fn evaluate(&self, file: &Label) -> Result<FrozenModule, String> {
        let lookup = &self.repos[&file.repo];
        let path = lookup.package_dir(&file.package).join(&file.name);
        let source = {
            let _span = tracing::debug_span!("read", file = %bzl_name(file)).entered();
            match std::fs::metadata(&path) {
                Ok(m) if m.is_dir() => {
                    return Err(format!(
                        "cannot load '{}': is a directory",
                        crate::label::display_label(file)
                    ));
                }
                Ok(_) => std::fs::read_to_string(&path).map_err(|e| {
                    format!("cannot load '{}': {e}", crate::label::display_label(file))
                })?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(format!(
                        "cannot load '{}': no such file",
                        crate::label::display_label(file)
                    ));
                }
                Err(e) => {
                    return Err(format!(
                        "cannot load '{}': {e}",
                        crate::label::display_label(file)
                    ));
                }
            }
        };
        let loader = self.importing(file.clone());
        self.evaluations.fetch_add(1, Ordering::Relaxed);
        let module = evaluate_bzl(&BzlFile {
            file,
            source: &source,
            globals: &self.globals,
            mappings: &self.mappings,
            loader: &loader,
            print: None,
        });
        // The source is dropped here, and the syntax tree was gone when the
        // module was evaluated: what stays is the frozen module.
        module.map_err(|e| format!("{:#}", e.into_anyhow()))
    }
}

/// The cycle a load of `file` closes, through the files this thread is
/// evaluating.
fn cycle(file: &Label) -> String {
    let show = crate::label::display_label;
    let mut path: Vec<String> = EVALUATING.with(|stack| {
        let stack = stack.borrow();
        let start = stack.iter().position(|l| l == file).unwrap_or(0);
        stack[start..].iter().map(show).collect()
    });
    if path.is_empty() {
        path.push(show(file));
    }
    let mut text = String::from("cycle detected in extension files: \n");
    for (i, step) in path.iter().enumerate() {
        text.push_str(if i == 0 { ".-> " } else { "|   " });
        text.push_str(step);
        text.push('\n');
    }
    text.push_str(&format!("`-- {}", show(file)));
    text
}
