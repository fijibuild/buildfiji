//! `repository_ctx`: what a repository rule's implementation is given
//! (buildfiji-mum.8.2), and [`run_repository_rule`], which runs one.
//!
//! This is the local half: naming, attributes, paths, files, templates,
//! symlinks, processes and the environment. Downloading, extracting and
//! patching are `mum.8.3`'s and are refused here.
//!
//! What Bazel 9.2.0 does, all read off probes of repository rules run by
//! `bazel fetch`:
//!
//! - **The directory.** The repository directory does not exist when the
//!   implementation starts (`ctx.path(".").exists` is false); the first write
//!   into it, or the first `execute`, creates it, and a rule that ends
//!   without it existing has failed (`<name> must create a directory`). The
//!   framework writes `REPO.bazel` afterwards, which is not done here.
//! - **Paths.** A string is relative to the repository directory unless it is
//!   absolute; `..` is resolved by the text of the path alone. A `Label` is
//!   the file it names, which the caller says where to find. Nothing may be
//!   written (files, templates, renamed or linked to) outside the repository
//!   directory: `Cannot write outside of the repository directory for path
//!   <path>`. Files are executable unless `executable = False`.
//! - **Values.** `repository_ctx` and its `attr` print as unknown Java
//!   objects; `ctx.attr` lists only `name` in `dir()` though every attribute
//!   can be read (an unknown one is an error), and `name` is the canonical
//!   name. A `path` prints as itself and is a dict key as its string.
//! - **Failures.** The system calls' errors are Bazel's own words
//!   (`java.io.FileNotFoundException: <path> (No such file or directory)`,
//!   `Could not rename <from> to <to>: already exists`, ...) and are
//!   reproduced for the common ones.
//! - **Processes.** `execute` runs the arguments with the client environment
//!   plus `environment` (`None` removes a variable), in the repository
//!   directory or `working_directory` (created if need be), and returns
//!   `return_code`, `stdout` and `stderr`: 256 and `Timed out` when the
//!   timeout (600 seconds unless given) passes, and 1 with the process
//!   wrapper's complaint when the program cannot be started.

use crate::args::fatal;
use crate::attr::sequence;
use crate::decl::{P, bind_checked, is_bool, is_dict, is_sequence, p};
use crate::ext::repository_rule_arg;
use crate::label::{BzlEval, RepoMappings, StarlarkLabel, display_label, label_of_value};
use allocative::Allocative;
use fjfj_graph::Label;
use starlark::collections::StarlarkHasher;
use starlark::environment::{FrozenModule, Methods, MethodsBuilder, MethodsStatic, Module};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::{AllocDict, DictRef};
use starlark::values::list::AllocList;
use starlark::values::none::NoneType;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ---- what a run is given --------------------------------------------------------------

/// An attribute value a repository rule is called with, as `ctx.attr` shows it.
#[derive(Debug, Clone, PartialEq)]
pub enum RepoAttr {
    /// An attribute with no value, such as a label that was not given.
    None,
    Bool(bool),
    Int(i64),
    String(String),
    StringList(Vec<String>),
    StringDict(Vec<(String, String)>),
    StringListDict(Vec<(String, Vec<String>)>),
    IntList(Vec<i64>),
    Label(Label),
    LabelList(Vec<Label>),
    StringKeyedLabelDict(Vec<(String, Label)>),
    LabelKeyedStringDict(Vec<(Label, String)>),
    LabelListDict(Vec<(String, Vec<Label>)>),
}

/// Where the file a label names is, if the caller can say.
pub type LabelPaths = dyn Fn(&Label) -> Option<PathBuf> + Send + Sync;

/// What a repository rule runs with.
pub struct RepoEnv {
    /// The canonical name of the repository, `ctx.name`.
    pub name: String,
    /// The name it was declared under, `ctx.original_name`.
    pub original_name: String,
    /// The directory the repository is made in; it need not exist yet.
    pub output: PathBuf,
    /// The main repository's directory, `ctx.workspace_root`.
    pub workspace_root: PathBuf,
    /// The environment of the Bazel client: `ctx.os.environ`, `ctx.getenv`, and
    /// what `execute` starts from.
    pub environ: BTreeMap<String, String>,
    /// Where a label's file is.
    pub labels: Box<LabelPaths>,
    /// The attributes the rule was called with, with defaults filled in.
    pub attrs: Vec<(String, RepoAttr)>,
    /// How `download` fetches an `http` or `https` URL.
    pub downloader: Option<Arc<dyn crate::repo_download::Downloader>>,
    /// The repository cache: a directory with `content_addressable/sha256/`.
    pub repository_cache: Option<PathBuf>,
    /// `--distdir`: directories that may hold a download's file by its name.
    pub distdirs: Vec<PathBuf>,
    /// What the rule or extension read from outside its own directory.
    pub recorded: Recorder,
}

/// Something a module extension read that its result depends on: what
/// `recordedInputs` of `MODULE.bazel.lock` lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordedInput {
    /// An environment variable and its value (`None`: not set).
    Env { name: String, value: Option<String> },
    /// A file and the SHA-256 (hex) of its content.
    File { path: PathBuf, sha256: String },
    /// A directory and the digest of the names in it (Bazel's `Fingerprint` of
    /// the sorted names: the count, then each name's length and bytes, as
    /// protobuf varints).
    Dirents { path: PathBuf, sha256: String },
    /// A repo name a label written in `repo` used, and the canonical repo it
    /// named (`\0` if none).
    RepoMapping {
        repo: String,
        apparent: String,
        canonical: String,
    },
}

impl RecordedInput {
    fn same_input(&self, other: &RecordedInput) -> bool {
        match (self, other) {
            (RecordedInput::Env { name: a, .. }, RecordedInput::Env { name: b, .. }) => a == b,
            (RecordedInput::File { path: a, .. }, RecordedInput::File { path: b, .. }) => a == b,
            (RecordedInput::Dirents { path: a, .. }, RecordedInput::Dirents { path: b, .. }) => {
                a == b
            }
            (
                RecordedInput::RepoMapping {
                    repo: a,
                    apparent: x,
                    ..
                },
                RecordedInput::RepoMapping {
                    repo: b,
                    apparent: y,
                    ..
                },
            ) => a == b && x == y,
            _ => false,
        }
    }
}

/// Where [`RecordedInput`]s are collected, in the order they were first
/// read.
#[derive(Debug, Clone, Default)]
pub struct Recorder(Arc<std::sync::Mutex<Vec<RecordedInput>>>);

impl PartialEq for Recorder {
    fn eq(&self, _: &Recorder) -> bool {
        true
    }
}

impl Eq for Recorder {}

impl Recorder {
    pub fn push(&self, input: RecordedInput) {
        let mut inputs = self.0.lock().expect("recorder");
        if !inputs.iter().any(|i| i.same_input(&input)) {
            inputs.push(input);
        }
    }

    /// What was recorded so far.
    pub fn inputs(&self) -> Vec<RecordedInput> {
        self.0.lock().expect("recorder").clone()
    }
}

/// The digest `RecordedInput::Dirents` holds for a directory with these names.
fn dirents_digest(names: &mut [String]) -> String {
    use sha2::Digest as _;
    names.sort();
    let mut bytes = Vec::new();
    let varint = |out: &mut Vec<u8>, mut n: usize| {
        loop {
            let low = (n & 0x7f) as u8;
            n >>= 7;
            if n == 0 {
                out.push(low);
                return;
            }
            out.push(low | 0x80);
        }
    };
    varint(&mut bytes, names.len());
    for name in names.iter() {
        varint(&mut bytes, name.len());
        bytes.extend_from_slice(name.as_bytes());
    }
    hex_of(&sha2::Sha256::digest(&bytes))
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Why a repository rule failed: Bazel's message for it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct RepoError {
    pub message: String,
}

// ---- paths ----------------------------------------------------------------------------

/// `path` of a repository rule: a file name, which may not exist.
#[derive(Debug, Clone, PartialEq, Eq, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct RepoPath {
    #[allocative(skip)]
    path: PathBuf,
    /// Where reads through this path are recorded.
    #[allocative(skip)]
    recorder: Recorder,
}

starlark_simple_value!(RepoPath);

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `repr`, which is what a path shows as inside a container.
        write!(f, "{:?}", self.path.display().to_string())
    }
}

#[starlark_value(type = "path")]
impl<'v> StarlarkValue<'v> for RepoPath {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("path", path_members);
        Some(RES.methods())
    }

    fn collect_str(&self, collector: &mut String) {
        collector.push_str(&self.path.display().to_string());
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        self.path.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .downcast_ref::<RepoPath>()
            .is_some_and(|other| other.path == self.path))
    }
}

fn this_path<'v>(this: Value<'v>) -> &'v RepoPath {
    this.downcast_ref::<RepoPath>()
        .expect("a path method is called on a path")
}

#[starlark_module]
fn path_members(builder: &mut MethodsBuilder) {
    /// The last component of the path.
    #[starlark(attribute)]
    fn basename<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(this_path(this)
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default())
    }

    /// The path of the directory it is in.
    #[starlark(attribute)]
    fn dirname<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let parent = this_path(this).path.parent().map(Path::to_path_buf);
        Ok(match parent {
            Some(path) => heap.alloc(RepoPath {
                path,
                recorder: this_path(this).recorder.clone(),
            }),
            None => Value::new_none(),
        })
    }

    /// Whether there is anything at the path.
    #[starlark(attribute)]
    fn exists<'v>(this: Value<'v>) -> starlark::Result<bool> {
        Ok(std::fs::symlink_metadata(&this_path(this).path).is_ok()
            && std::fs::metadata(&this_path(this).path).is_ok())
    }

    /// Whether it is a directory.
    #[starlark(attribute)]
    fn is_dir<'v>(this: Value<'v>) -> starlark::Result<bool> {
        Ok(this_path(this).path.is_dir())
    }

    /// The path with symbolic links resolved.
    #[starlark(attribute)]
    fn realpath<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let path = &this_path(this).path;
        let real = std::fs::canonicalize(path).map_err(|_| {
            // The message names the first place on the way that is missing.
            let missing = path
                .ancestors()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .find(|a| std::fs::symlink_metadata(a).is_err())
                .unwrap_or(path);
            fatal(format!(
                "[unix_jni.cc:382] {} (No such file or directory)",
                missing.display()
            ))
        })?;
        Ok(heap.alloc(RepoPath {
            path: real,
            recorder: this_path(this).recorder.clone(),
        }))
    }

    /// The entries of a directory.
    fn readdir<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        const PARAMS: &[P] = &[p("watch", false, false, "string", is_str)];
        let bound = bind_checked("readdir", PARAMS, args, eval)?;
        let bound_no_watch = bound[0].and_then(|w| w.unpack_str()) == Some("no");
        let path = &this_path(this).path;
        if !path.is_dir() {
            return Err(fatal(format!(
                "can't readdir(), not a directory: {}",
                path.display()
            )));
        }
        let mut entries = Vec::new();
        let mut names = Vec::new();
        for entry in
            std::fs::read_dir(path).map_err(|e| fatal(format!("java.io.IOException: {e}")))?
        {
            let entry = entry.map_err(|e| fatal(format!("java.io.IOException: {e}")))?;
            names.push(entry.file_name().to_string_lossy().into_owned());
            entries.push(eval.heap().alloc(RepoPath {
                path: entry.path(),
                recorder: this_path(this).recorder.clone(),
            }));
        }
        if !bound_no_watch {
            this_path(this).recorder.push(RecordedInput::Dirents {
                path: path.clone(),
                sha256: dirents_digest(&mut names),
            });
        }
        Ok(eval.heap().alloc(AllocList(entries)))
    }

    /// `path.get_child(*relative)`.
    fn get_child<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let mut path = this_path(this).path.clone();
        if let Some((name, _)) = args.names_map()?.iter().next() {
            return Err(fatal(format!(
                "get_child() got unexpected keyword argument '{}'",
                name.as_str()
            )));
        }
        for child in args.positions(eval.heap())? {
            let child = child.unpack_str().ok_or_else(|| {
                fatal(format!(
                    "at index 0 of relative_paths, got element of type {}, want string",
                    child.get_type()
                ))
            })?;
            path = lexical(&path.join(child));
        }
        Ok(eval.heap().alloc(RepoPath {
            path,
            recorder: this_path(this).recorder.clone(),
        }))
    }
}

fn is_str(v: Value<'_>) -> bool {
    v.unpack_str().is_some()
}

/// `path` with `.` and `..` resolved by its text alone.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other),
        }
    }
    out
}

// ---- the other values -----------------------------------------------------------------

/// `ctx.os`.
#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct RepoOs {
    #[allocative(skip)]
    environ: BTreeMap<String, String>,
}

starlark_simple_value!(RepoOs);

impl fmt::Display for RepoOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.bazel.repository.starlark.StarlarkOS>"
        )
    }
}

#[starlark_value(type = "repository_os")]
impl<'v> StarlarkValue<'v> for RepoOs {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("repository_os", os_members);
        Some(RES.methods())
    }
}

#[starlark_module]
fn os_members(builder: &mut MethodsBuilder) {
    /// The operating system's name, as Bazel spells it.
    #[starlark(attribute)]
    fn name<'v>(this: Value<'v>) -> starlark::Result<String> {
        let _ = this;
        Ok(match std::env::consts::OS {
            "macos" => "mac os x".to_owned(),
            "windows" => "windows".to_owned(),
            other => other.to_owned(),
        })
    }

    /// The architecture, as Bazel spells it.
    #[starlark(attribute)]
    fn arch<'v>(this: Value<'v>) -> starlark::Result<String> {
        let _ = this;
        Ok(match std::env::consts::ARCH {
            "x86_64" => "amd64".to_owned(),
            other => other.to_owned(),
        })
    }

    /// The client's environment.
    #[starlark(attribute)]
    fn environ<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let os = this.downcast_ref::<RepoOs>().expect("an os");
        Ok(heap.alloc(AllocDict(
            os.environ.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        )))
    }
}

/// What `execute` returns.
#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
struct ExecResult {
    return_code: i32,
    stdout: String,
    stderr: String,
}

starlark_simple_value!(ExecResult);

impl fmt::Display for ExecResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "exec_result(return_code = {}, stdout = {:?}, stderr = {:?})",
            self.return_code, self.stdout, self.stderr
        )
    }
}

#[starlark_value(type = "exec_result")]
impl<'v> StarlarkValue<'v> for ExecResult {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("exec_result", exec_members);
        Some(RES.methods())
    }
}

#[starlark_module]
fn exec_members(builder: &mut MethodsBuilder) {
    #[starlark(attribute)]
    fn return_code<'v>(this: Value<'v>) -> starlark::Result<i32> {
        Ok(this
            .downcast_ref::<ExecResult>()
            .expect("a result")
            .return_code)
    }

    #[starlark(attribute)]
    fn stdout<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(this
            .downcast_ref::<ExecResult>()
            .expect("a result")
            .stdout
            .clone())
    }

    #[starlark(attribute)]
    fn stderr<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(this
            .downcast_ref::<ExecResult>()
            .expect("a result")
            .stderr
            .clone())
    }
}

/// What `repo_metadata()` returns: nothing a rule can look at.
#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
struct RepoMetadata;

starlark_simple_value!(RepoMetadata);

impl fmt::Display for RepoMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.bazel.repository.starlark.RepoMetadata>"
        )
    }
}

#[starlark_value(type = "repo_metadata")]
impl<'v> StarlarkValue<'v> for RepoMetadata {}

/// `ctx.attr`: every attribute the rule was called with, and `name`.
#[derive(ProvidesStaticType, NoSerialize, Allocative)]
struct RepoAttrs {
    #[allocative(skip)]
    env: Arc<RepoEnv>,
}

starlark_simple_value!(RepoAttrs);

impl fmt::Debug for RepoAttrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RepoAttrs")
    }
}

impl fmt::Display for RepoAttrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.bazel.repository.RepoDefinition>"
        )
    }
}

fn string_value<'v>(heap: Heap<'v>, s: &str) -> Value<'v> {
    heap.alloc_str(s).to_value()
}

fn label_value<'v>(heap: Heap<'v>, label: &Label) -> Value<'v> {
    heap.alloc(StarlarkLabel::from(label.clone()))
}

impl RepoAttr {
    pub(crate) fn to_value<'v>(&self, heap: Heap<'v>) -> Value<'v> {
        match self {
            RepoAttr::None => Value::new_none(),
            RepoAttr::Bool(b) => Value::new_bool(*b),
            RepoAttr::Int(i) => heap.alloc(*i as i32),
            RepoAttr::String(s) => string_value(heap, s),
            RepoAttr::StringList(items) => heap.alloc(AllocList(
                items
                    .iter()
                    .map(|s| string_value(heap, s))
                    .collect::<Vec<_>>(),
            )),
            RepoAttr::StringDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(k, v)| (string_value(heap, k), string_value(heap, v)))
                    .collect::<Vec<_>>(),
            )),
            RepoAttr::StringListDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(k, vs)| {
                        (
                            string_value(heap, k),
                            heap.alloc(AllocList(
                                vs.iter().map(|s| string_value(heap, s)).collect::<Vec<_>>(),
                            )),
                        )
                    })
                    .collect::<Vec<_>>(),
            )),
            RepoAttr::IntList(items) => heap.alloc(AllocList(
                items
                    .iter()
                    .map(|i| heap.alloc(*i as i32))
                    .collect::<Vec<_>>(),
            )),
            RepoAttr::StringKeyedLabelDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(k, l)| (string_value(heap, k), label_value(heap, l)))
                    .collect::<Vec<_>>(),
            )),
            RepoAttr::LabelKeyedStringDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(l, v)| (label_value(heap, l), string_value(heap, v)))
                    .collect::<Vec<_>>(),
            )),
            RepoAttr::LabelListDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(k, ls)| {
                        (
                            string_value(heap, k),
                            heap.alloc(AllocList(
                                ls.iter().map(|l| label_value(heap, l)).collect::<Vec<_>>(),
                            )),
                        )
                    })
                    .collect::<Vec<_>>(),
            )),
            RepoAttr::Label(label) => label_value(heap, label),
            RepoAttr::LabelList(labels) => heap.alloc(AllocList(
                labels
                    .iter()
                    .map(|l| label_value(heap, l))
                    .collect::<Vec<_>>(),
            )),
        }
    }
}

#[starlark_value(type = "RepoDefinition")]
impl<'v> StarlarkValue<'v> for RepoAttrs {
    fn get_attr(&self, attribute: &str, heap: Heap<'v>) -> Option<Value<'v>> {
        if attribute == "name" {
            return Some(string_value(heap, &self.env.name));
        }
        self.env
            .attrs
            .iter()
            .find(|(name, _)| name == attribute)
            .map(|(_, value)| value.to_value(heap))
    }

    fn dir_attr(&self) -> Vec<String> {
        vec!["name".to_owned()]
    }
}

// ---- the context ----------------------------------------------------------------------

/// `repository_ctx`.
#[derive(ProvidesStaticType, NoSerialize, Allocative)]
struct RepositoryCtx {
    #[allocative(skip)]
    env: Arc<RepoEnv>,
}

starlark_simple_value!(RepositoryCtx);

impl fmt::Debug for RepositoryCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RepositoryCtx")
    }
}

impl fmt::Display for RepositoryCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<unknown object com.google.devtools.build.lib.bazel.repository.starlark.\
             StarlarkRepositoryContext>"
        )
    }
}

#[starlark_value(type = "repository_ctx")]
impl<'v> StarlarkValue<'v> for RepositoryCtx {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("repository_ctx", ctx_members);
        Some(RES.methods())
    }
}

pub(crate) fn env_of<'v>(this: Value<'v>) -> &'v Arc<RepoEnv> {
    if let Some(ctx) = this.downcast_ref::<RepositoryCtx>() {
        return &ctx.env;
    }
    &this
        .downcast_ref::<crate::module_ctx::ModuleCtx>()
        .expect("a context method is called on a repository_ctx or module_ctx")
        .env
}

/// The `os` of a context.
pub(crate) fn os_members_for(env: &RepoEnv) -> RepoOs {
    RepoOs {
        environ: env.environ.clone(),
    }
}

/// A path a rule named, and how it was named if by label.
pub(crate) struct Resolved {
    pub(crate) path: PathBuf,
    /// The label's file as a path in its repository, which is how an error
    /// names a file that came from a label.
    pub(crate) label_path: Option<String>,
}

impl Resolved {
    /// How an error message names the path.
    fn shown(&self) -> String {
        self.label_path
            .clone()
            .unwrap_or_else(|| self.path.display().to_string())
    }
}

/// A parameter that names a file (as a string, `Label` or `path`) and can
/// only be given by position.
pub(crate) const fn path_arg(name: &'static str) -> P {
    p(name, true, true, "string, Label, or path", is_path_like).positional_only()
}

pub(crate) fn is_path_like(v: Value<'_>) -> bool {
    is_str(v) || v.get_type() == "Label" || v.get_type() == "path"
}

fn is_string_or_path(v: Value<'_>) -> bool {
    is_str(v) || v.get_type() == "path"
}

fn label_relative(label: &Label) -> String {
    if label.package.is_empty() {
        label.name.clone()
    } else {
        format!("{}/{}", label.package, label.name)
    }
}

pub(crate) fn resolve(env: &RepoEnv, value: Value<'_>) -> starlark::Result<Resolved> {
    if let Some(text) = value.unpack_str() {
        let joined = if Path::new(text).is_absolute() {
            PathBuf::from(text)
        } else {
            env.output.join(text)
        };
        return Ok(Resolved {
            path: lexical(&joined),
            label_path: None,
        });
    }
    if let Some(path) = value.downcast_ref::<RepoPath>() {
        return Ok(Resolved {
            path: path.path.clone(),
            label_path: None,
        });
    }
    if let Some(label) = label_of_value(value) {
        let path = (env.labels)(&label).ok_or_else(|| {
            fatal(format!(
                "cannot find the file of label {}: its repository is not available",
                display_label(&label)
            ))
        })?;
        return Ok(Resolved {
            path,
            label_path: Some(label_relative(&label)),
        });
    }
    Err(fatal("expected a string, Label or path"))
}

pub(crate) fn inside(env: &RepoEnv, path: &Path) -> bool {
    path.starts_with(&env.output)
}

pub(crate) fn writable(env: &RepoEnv, resolved: &Resolved) -> starlark::Result<()> {
    if inside(env, &resolved.path) {
        Ok(())
    } else {
        Err(fatal(format!(
            "Cannot write outside of the repository directory for path {}",
            resolved.shown()
        )))
    }
}

pub(crate) fn io_error(e: &std::io::Error) -> starlark::Error {
    fatal(format!("java.io.IOException: {e}"))
}

/// The error for something that cannot be read because it is not there.
fn not_found(resolved: &Resolved) -> starlark::Error {
    fatal(format!(
        "java.io.FileNotFoundException: {} (No such file or directory)",
        resolved.shown()
    ))
}

/// Make the directories `path` is in, naming the file that is in the way.
fn make_parents(path: &Path) -> starlark::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if let Err(e) = std::fs::create_dir_all(parent) {
        let in_the_way = parent
            .ancestors()
            .find(|a| a.exists() && !a.is_dir())
            .unwrap_or(parent);
        return Err(
            if in_the_way != parent || e.kind() == std::io::ErrorKind::AlreadyExists {
                fatal(format!(
                    "java.io.IOException: {} (File exists)",
                    in_the_way.display()
                ))
            } else {
                io_error(&e)
            },
        );
    }
    Ok(())
}

fn write_file(
    env: &RepoEnv,
    resolved: &Resolved,
    content: &[u8],
    executable: bool,
) -> starlark::Result<()> {
    writable(env, resolved)?;
    let path = &resolved.path;
    if path == &env.output || path.is_dir() {
        return Err(fatal(format!(
            "java.io.IOException: {} (File exists)",
            path.display()
        )));
    }
    make_parents(path)?;
    // A file that is being replaced is replaced, not written through.
    let _ = std::fs::remove_file(path);
    std::fs::write(path, content).map_err(|e| io_error(&e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if executable { 0o755 } else { 0o644 };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(|e| io_error(&e))?;
    }
    #[cfg(not(unix))]
    let _ = executable;
    Ok(())
}

pub(crate) fn read_text(resolved: &Resolved) -> starlark::Result<String> {
    if resolved.path.is_dir() {
        return Err(fatal(format!(
            "attempting to read() a directory: {}",
            resolved.path.display()
        )));
    }
    match std::fs::read(&resolved.path) {
        Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(not_found(resolved)),
        Err(e) => Err(io_error(&e)),
    }
}

/// A file being watched: one under the repository directory is refused.
pub(crate) fn check_watch(
    env: &RepoEnv,
    resolved: &Resolved,
    watch: Option<Value<'_>>,
) -> starlark::Result<()> {
    let watching = match watch.and_then(|w| w.unpack_str()) {
        Some("yes") => true,
        Some("no") | Some("auto") | None => false,
        Some(other) => {
            return Err(fatal(format!(
                "bad value for 'watch' parameter; want 'yes', 'no', or 'auto', got {other}"
            )));
        }
    };
    if watching && inside(env, &resolved.path) {
        return Err(fatal("attempted to watch path under working directory"));
    }
    Ok(())
}

const FILE_PARAMS: &[P] = &[
    path_arg("path"),
    p("content", true, false, "string", is_str),
    p("executable", true, false, "bool", is_bool),
    p("legacy_utf8", true, false, "bool", is_bool),
];

const TEMPLATE_PARAMS: &[P] = &[
    path_arg("path"),
    path_arg("template"),
    p("substitutions", true, false, "dict", is_dict),
    p("executable", true, false, "bool", is_bool),
    p("watch_template", false, false, "string", is_str),
];

fn arg<'v>(params: &[P], bound: &[Option<Value<'v>>], name: &str) -> Option<Value<'v>> {
    bound[params.iter().position(|p| p.name == name).expect("known")]
}

pub(crate) fn flag(value: Option<Value<'_>>, default: bool) -> bool {
    value.and_then(|v| v.unpack_bool()).unwrap_or(default)
}

pub(crate) fn op_path<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    const PARAMS: &[P] = &[path_arg("path")];
    let bound = bind_checked("path", PARAMS, args, eval)?;
    let resolved = resolve(env_of(this), bound[0].expect("required"))?;
    Ok(eval.heap().alloc(RepoPath {
        path: resolved.path,
        recorder: env_of(this).recorded.clone(),
    }))
}

pub(crate) fn op_file<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    let env = env_of(this);
    let bound = bind_checked("file", FILE_PARAMS, args, eval)?;
    let resolved = resolve(env, arg(FILE_PARAMS, &bound, "path").expect("required"))?;
    let content = arg(FILE_PARAMS, &bound, "content")
        .and_then(|c| c.unpack_str())
        .unwrap_or_default();
    let executable = flag(arg(FILE_PARAMS, &bound, "executable"), true);
    write_file(env, &resolved, content.as_bytes(), executable)?;
    Ok(NoneType)
}

pub(crate) fn op_template<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    let env = env_of(this);
    let bound = bind_checked("template", TEMPLATE_PARAMS, args, eval)?;
    let substitutions = match arg(TEMPLATE_PARAMS, &bound, "substitutions") {
        Some(dict) => {
            let dict = DictRef::from_value(dict).expect("checked");
            if let Some((k, v)) = dict
                .iter()
                .find(|(k, v)| k.unpack_str().is_none() || v.unpack_str().is_none())
            {
                return Err(fatal(format!(
                    "got dict<{}, {}> for 'substitutions', want dict<string, string>",
                    k.get_type(),
                    v.get_type()
                )));
            }
            dict.iter()
                .map(|(k, v)| {
                    (
                        k.unpack_str().expect("checked").to_owned(),
                        v.unpack_str().expect("checked").to_owned(),
                    )
                })
                .collect::<Vec<_>>()
        }
        None => Vec::new(),
    };
    let target = resolve(env, arg(TEMPLATE_PARAMS, &bound, "path").expect("required"))?;
    let template = resolve(
        env,
        arg(TEMPLATE_PARAMS, &bound, "template").expect("required"),
    )?;
    check_watch(
        env,
        &template,
        arg(TEMPLATE_PARAMS, &bound, "watch_template"),
    )?;
    let mut text = read_text(&template)?;
    for (from, to) in &substitutions {
        if !from.is_empty() {
            text = text.replace(from.as_str(), to);
        }
    }
    let executable = flag(arg(TEMPLATE_PARAMS, &bound, "executable"), true);
    write_file(env, &target, text.as_bytes(), executable)?;
    Ok(NoneType)
}

pub(crate) fn op_read<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<String> {
    const PARAMS: &[P] = &[path_arg("path"), p("watch", false, false, "string", is_str)];
    let env = env_of(this);
    let bound = bind_checked("read", PARAMS, args, eval)?;
    let resolved = resolve(env, arg(PARAMS, &bound, "path").expect("required"))?;
    check_watch(env, &resolved, arg(PARAMS, &bound, "watch"))?;
    let text = read_text(&resolved)?;
    if arg(PARAMS, &bound, "watch").and_then(|w| w.unpack_str()) != Some("no") {
        record_file(env, &resolved.path);
    }
    Ok(text)
}

/// Note that the file at `path` was read, unless it is in the directory being
/// made.
pub(crate) fn record_file(env: &RepoEnv, path: &Path) {
    use sha2::Digest as _;
    if inside(env, path) {
        return;
    }
    if let Ok(bytes) = std::fs::read(path) {
        env.recorded.push(RecordedInput::File {
            path: path.to_path_buf(),
            sha256: hex_of(&sha2::Sha256::digest(&bytes)),
        });
    }
}

pub(crate) fn op_delete<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<bool> {
    const PARAMS: &[P] =
        &[p("path", true, true, "string or path", is_string_or_path).positional_only()];
    let bound = bind_checked("delete", PARAMS, args, eval)?;
    let resolved = resolve(env_of(this), bound[0].expect("required"))?;
    let path = &resolved.path;
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => std::fs::remove_dir_all(path)
            .map(|()| true)
            .map_err(|e| io_error(&e)),
        Ok(_) => std::fs::remove_file(path)
            .map(|()| true)
            .map_err(|e| io_error(&e)),
        Err(_) => Ok(false),
    }
}

pub(crate) fn op_rename<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    const PARAMS: &[P] = &[path_arg("src"), path_arg("dst")];
    let env = env_of(this);
    let bound = bind_checked("rename", PARAMS, args, eval)?;
    let from = resolve(env, bound[0].expect("required"))?;
    let to = resolve(env, bound[1].expect("required"))?;
    writable(env, &to)?;
    let (src, dst) = (from.path.display(), to.path.display());
    if std::fs::symlink_metadata(&to.path).is_ok() {
        return Err(fatal(format!(
            "java.io.IOException: Could not rename {src} to {dst}: already exists"
        )));
    }
    make_parents(&to.path)?;
    std::fs::rename(&from.path, &to.path).map_err(|e| {
        fatal(format!(
            "java.io.IOException: Could not rename {src} to {dst}: {}",
            errno_text(&e, &format!("{src} -> {dst}"), 638)
        ))
    })?;
    Ok(NoneType)
}

pub(crate) fn op_symlink<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    const PARAMS: &[P] = &[path_arg("target"), path_arg("link_name")];
    let env = env_of(this);
    let bound = bind_checked("symlink", PARAMS, args, eval)?;
    let target = resolve(env, bound[0].expect("required"))?;
    let link = resolve(env, bound[1].expect("required"))?;
    writable(env, &link)?;
    make_parents(&link.path)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target.path, &link.path).map_err(|e| {
        fatal(format!(
            "java.io.IOException: Could not create symlink from {} to {}: {}",
            target.path.display(),
            link.path.display(),
            errno_text(&e, &link.path.display().to_string(), 297)
        ))
    })?;
    #[cfg(not(unix))]
    return Err(fatal("symlink() is not supported on this platform"));
    Ok(NoneType)
}

pub(crate) fn op_execute<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    const PARAMS: &[P] = &[
        p("arguments", true, true, "sequence", is_sequence).positional_only(),
        p("timeout", true, false, "int", is_int),
        p("environment", true, false, "dict", is_dict),
        p("quiet", true, false, "bool", is_bool),
        p("working_directory", true, false, "string", is_str),
    ];
    let env = env_of(this);
    let bound = bind_checked("execute", PARAMS, args, eval)?;
    let heap = eval.heap();
    let mut argv: Vec<String> = Vec::new();
    for (i, item) in sequence(arg(PARAMS, &bound, "arguments").expect("required"), heap)
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        if !is_path_like(*item) {
            return Err(fatal(format!(
                "Argument {i} of execute is neither a path, label, nor string."
            )));
        }
        argv.push(resolve_text(env, *item)?);
    }
    let timeout = arg(PARAMS, &bound, "timeout")
        .and_then(|t| t.unpack_i32())
        .unwrap_or(600);
    let mut vars = env.environ.clone();
    if let Some(dict) = arg(PARAMS, &bound, "environment").and_then(DictRef::from_value) {
        for (k, v) in dict.iter() {
            let Some(k) = k.unpack_str() else {
                return Err(fatal(format!(
                    "environment keys must be strings, got {}",
                    k.get_type()
                )));
            };
            if v.is_none() {
                vars.remove(k);
            } else if let Some(v) = v.unpack_str() {
                vars.insert(k.to_owned(), v.to_owned());
            } else {
                return Err(fatal(format!(
                    "environment values must be strings or None, got {v}"
                )));
            }
        }
    }
    let quiet = flag(arg(PARAMS, &bound, "quiet"), true);
    let directory = match arg(PARAMS, &bound, "working_directory").and_then(|d| d.unpack_str()) {
        Some(d) if !d.is_empty() => lexical(&env.output.join(d)),
        _ => env.output.clone(),
    };
    std::fs::create_dir_all(&directory).map_err(|e| io_error(&e))?;
    let result = run_process(&argv, &vars, &directory, timeout, quiet);
    Ok(heap.alloc(result))
}

pub(crate) fn op_which<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    const PARAMS: &[P] = &[p("program", true, true, "string", is_str).positional_only()];
    let env = env_of(this);
    let bound = bind_checked("which", PARAMS, args, eval)?;
    let program = bound[0].expect("required").unpack_str().expect("checked");
    if program.contains('/') || program.contains('\\') {
        return Err(fatal(format!(
            "Program argument of which() may not contain a / or a \\ ('{program}' given)"
        )));
    }
    let found = env
        .environ
        .get("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(path))
        .map(|dir| dir.join(program))
        .find(|candidate| is_executable(candidate));
    Ok(match found {
        Some(path) => eval.heap().alloc(RepoPath {
            path,
            recorder: env_of(this).recorded.clone(),
        }),
        None => Value::new_none(),
    })
}

pub(crate) fn op_getenv<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    const PARAMS: &[P] = &[
        p("name", true, true, "string", is_str).positional_only(),
        p("default", true, false, "string or NoneType", is_str_or_none).positional_only(),
    ];
    let env = env_of(this);
    let bound = bind_checked("getenv", PARAMS, args, eval)?;
    let name = bound[0].expect("required").unpack_str().expect("checked");
    env.recorded.push(RecordedInput::Env {
        name: name.to_owned(),
        value: env.environ.get(name).cloned(),
    });
    Ok(match env.environ.get(name) {
        Some(value) => string_value(eval.heap(), value),
        None => bound[1].unwrap_or_else(Value::new_none),
    })
}

pub(crate) fn op_report_progress<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    const PARAMS: &[P] = &[p("status", true, false, "string", is_str).positional_only()];
    let _ = this;
    bind_checked("report_progress", PARAMS, args, eval)?;
    Ok(NoneType)
}

pub(crate) fn op_watch<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    watch_under(this, "watch", args, eval)
}

pub(crate) fn op_watch_tree<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    watch_under(this, "watch_tree", args, eval)
}

pub(crate) fn op_repo_metadata<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    const PARAMS: &[P] = &[
        p("reproducible", false, false, "bool", is_bool),
        p("attrs_for_reproducibility", false, false, "dict", is_dict),
    ];
    let _ = this;
    bind_checked("repo_metadata", PARAMS, args, eval)?;
    Ok(eval.heap().alloc(RepoMetadata))
}

#[starlark_module]
fn ctx_members(builder: &mut MethodsBuilder) {
    /// The canonical name of the repository.

    #[starlark(attribute)]
    fn name<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(env_of(this).name.clone())
    }

    /// The name the repository was declared under.

    #[starlark(attribute)]
    fn original_name<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(env_of(this).original_name.clone())
    }

    /// The attributes the rule was called with.

    #[starlark(attribute)]
    fn attr<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(RepoAttrs {
            env: Arc::clone(env_of(this)),
        }))
    }

    /// The main repository's directory.

    #[starlark(attribute)]
    fn workspace_root<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(RepoPath {
            path: env_of(this).workspace_root.clone(),
            recorder: env_of(this).recorded.clone(),
        }))
    }

    /// The operating system.

    #[starlark(attribute)]
    fn os<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(RepoOs {
            environ: env_of(this).environ.clone(),
        }))
    }

    /// `ctx.path(path)`.
    fn path<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        op_path(this, args, eval)
    }

    /// `ctx.file(path, content, executable, legacy_utf8)`.
    fn file<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        op_file(this, args, eval)
    }

    /// `ctx.template(path, template, substitutions, executable, watch_template)`.
    fn template<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        op_template(this, args, eval)
    }

    /// `ctx.read(path, watch)`.
    fn read<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        op_read(this, args, eval)
    }

    /// `ctx.delete(path)`: whether there was something to delete.
    fn delete<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<bool> {
        op_delete(this, args, eval)
    }

    /// `ctx.rename(src, dst)`.
    fn rename<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        op_rename(this, args, eval)
    }

    /// `ctx.symlink(target, link_name)`.
    fn symlink<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        op_symlink(this, args, eval)
    }

    /// `ctx.execute(arguments, timeout, environment, quiet, working_directory)`.
    fn execute<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        op_execute(this, args, eval)
    }

    /// `ctx.which(program)`.
    fn which<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        op_which(this, args, eval)
    }

    /// `ctx.getenv(name, default)`.
    fn getenv<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        op_getenv(this, args, eval)
    }

    /// `ctx.report_progress(status)`.
    fn report_progress<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        op_report_progress(this, args, eval)
    }

    /// `ctx.watch(path)`.
    fn watch<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        op_watch(this, args, eval)
    }

    /// `ctx.watch_tree(path)`.
    fn watch_tree<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        op_watch_tree(this, args, eval)
    }

    /// `ctx.repo_metadata(reproducible, attrs_for_reproducibility)`.
    fn repo_metadata<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        op_repo_metadata(this, args, eval)
    }

    /// `ctx.download(...)`: buildfiji-mum.8.3.
    fn download<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        crate::repo_download::op_download(this, args, eval)
    }

    /// `ctx.download_and_extract(...)`: buildfiji-mum.8.3.
    fn download_and_extract<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        crate::repo_download::op_download_and_extract(this, args, eval)
    }

    /// `ctx.extract(...)`: buildfiji-mum.8.3.
    fn extract<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        crate::repo_download::op_extract(this, args, eval)
    }

    /// `ctx.patch(...)`: buildfiji-mum.8.3.
    fn patch<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        crate::repo_download::op_patch(this, args, eval)
    }
}

fn is_int(v: Value<'_>) -> bool {
    v.unpack_i32().is_some()
}

fn is_str_or_none(v: Value<'_>) -> bool {
    v.is_none() || is_str(v)
}

fn watch_under<'v>(
    this: Value<'v>,
    function: &str,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    const PARAMS: &[P] = &[path_arg("path")];
    let env = env_of(this);
    let bound = bind_checked(function, PARAMS, args, eval)?;
    let resolved = resolve(env, bound[0].expect("required"))?;
    if inside(env, &resolved.path) {
        return Err(fatal("attempted to watch path under working directory"));
    }
    if resolved.path.is_file() {
        record_file(env, &resolved.path);
    }
    Ok(NoneType)
}

/// The text of an argument of `execute`: a label is the file it names.
fn resolve_text(env: &RepoEnv, value: Value<'_>) -> starlark::Result<String> {
    if let Some(text) = value.unpack_str() {
        return Ok(text.to_owned());
    }
    Ok(resolve(env, value)?.path.display().to_string())
}

/// The message the JNI layer gives a failed system call: `[unix_jni.cc:LINE]`,
/// what it was doing, and the reason.
fn errno_text(error: &std::io::Error, doing: &str, line: u32) -> String {
    let reason = match error.kind() {
        std::io::ErrorKind::NotFound => "No such file or directory".to_owned(),
        std::io::ErrorKind::AlreadyExists => "File exists".to_owned(),
        std::io::ErrorKind::InvalidInput => "Invalid argument".to_owned(),
        _ => error
            .to_string()
            .split(" (os error")
            .next()
            .unwrap_or_default()
            .to_owned(),
    };
    format!("[unix_jni.cc:{line}] {doing} ({reason})")
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// The code a shell would report: the exit status, or 128 and the signal.
fn exit_code(status: std::process::ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }
    status.code().unwrap_or(-1)
}

/// Run `argv` to completion or until `timeout` seconds pass.
fn run_process(
    argv: &[String],
    vars: &BTreeMap<String, String>,
    directory: &Path,
    timeout: i32,
    quiet: bool,
) -> ExecResult {
    let Some((program, rest)) = argv.split_first() else {
        return ExecResult {
            return_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
    };
    let mut command = Command::new(program);
    command
        .args(rest)
        .env_clear()
        .envs(vars)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            let reason = match e.kind() {
                std::io::ErrorKind::NotFound => "No such file or directory".to_owned(),
                std::io::ErrorKind::PermissionDenied => "Permission denied".to_owned(),
                _ => e.to_string(),
            };
            return ExecResult {
                return_code: 1,
                stdout: String::new(),
                stderr: format!(
                    "src/main/tools/process-wrapper-legacy.cc:80: \"execvp({program}, ...)\": \
                     {reason}\n"
                ),
            };
        }
    };
    let read = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut pipe) = pipe {
                let _ = pipe.read_to_end(&mut bytes);
            }
            bytes
        })
    };
    let out = read(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let err = read(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let deadline = Instant::now() + Duration::from_secs(timeout.max(0) as u64);
    let (code, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (exit_code(status), false),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break (256, true);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(_) => break (-1, false),
        }
    };
    let stdout = String::from_utf8_lossy(&out.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&err.join().unwrap_or_default()).into_owned();
    if !quiet {
        print!("{stdout}");
        eprint!("{stderr}");
    }
    ExecResult {
        return_code: code,
        stdout: if timed_out { String::new() } else { stdout },
        stderr: if timed_out {
            "Timed out".to_owned()
        } else {
            stderr
        },
    }
}

// ---- running a rule -------------------------------------------------------------------

/// Run the implementation of the repository rule `rule_name` of `module`
/// with `env`, which is what fetching the repository does. `print` gets what
/// the implementation prints.
pub fn run_repository_rule(
    module: &FrozenModule,
    rule_name: &str,
    env: RepoEnv,
    mappings: &RepoMappings,
    print: Option<&dyn starlark::PrintHandler>,
) -> Result<(), RepoError> {
    let rule = module
        .get_any_visibility(rule_name)
        .map(|(value, _)| value)
        .map_err(|_| RepoError {
            message: format!("no repository rule named {rule_name}"),
        })?;
    let env = Arc::new(env);
    let result = Module::with_temp_heap(|scratch| {
        scratch.frozen_heap().add_reference(rule.owner());
        let rule_value = rule
            .value()
            .unpack_frozen()
            .expect("a global is frozen")
            .to_value();
        let implementation =
            repository_rule_arg(rule_value, "implementation").ok_or_else(|| RepoError {
                message: format!("{rule_name} is not a repository rule"),
            })?;
        let running = BzlEval::running(mappings);
        let mut eval = Evaluator::new(&scratch);
        eval.extra = Some(&running);
        if let Some(print) = print {
            eval.set_print_handler(print);
        }
        let ctx = scratch.heap().alloc(RepositoryCtx {
            env: Arc::clone(&env),
        });
        eval.eval_function(implementation, &[ctx], &[])
            .map(|_| ())
            .map_err(|e| RepoError {
                message: format!("{e}"),
            })
    });
    // A repository that failed is not left behind half made.
    if let Err(e) = result {
        let _ = std::fs::remove_dir_all(&env.output);
        return Err(e);
    }
    // A repository has to exist when its rule is done.
    if !env.output.is_dir() {
        return Err(RepoError {
            message: format!("{} must create a directory", env.name),
        });
    }
    Ok(())
}

/// What each attribute of the repository rule `rule_name` of `module` is when
/// the call does not say: its default, or the zero of its type. Attributes
/// that have neither (a label) are left out.
pub fn repository_rule_defaults(module: &FrozenModule, rule_name: &str) -> Vec<(String, RepoAttr)> {
    use crate::attr::view as attribute_view;
    use fjfj_graph::rule::AttrValue;
    let Ok((rule, _)) = module.get_any_visibility(rule_name) else {
        return Vec::new();
    };
    let Some(attrs) = repository_rule_arg(rule.value(), "attrs").and_then(DictRef::from_value)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (name, descriptor) in attrs.iter() {
        let (Some(name), Some(view)) = (name.unpack_str(), attribute_view(descriptor)) else {
            continue;
        };
        let value = view.def.default.clone().or_else(|| view.def.ty.zero());
        let converted = match value {
            None => RepoAttr::None,
            Some(AttrValue::Bool(b)) => RepoAttr::Bool(b),
            Some(AttrValue::Int(i)) => RepoAttr::Int(i64::from(i)),
            Some(AttrValue::String(s)) => RepoAttr::String(s),
            Some(AttrValue::StringList(items)) => RepoAttr::StringList(items),
            Some(AttrValue::IntList(items)) => {
                RepoAttr::IntList(items.into_iter().map(i64::from).collect())
            }
            Some(AttrValue::Label(label)) => RepoAttr::Label(label),
            Some(AttrValue::LabelList(labels)) => RepoAttr::LabelList(labels),
            Some(AttrValue::StringDict(items)) => RepoAttr::StringDict(items),
            Some(AttrValue::StringListDict(items)) => RepoAttr::StringListDict(items),
            Some(AttrValue::StringKeyedLabelDict(items)) => RepoAttr::StringKeyedLabelDict(items),
            Some(AttrValue::LabelKeyedStringDict(items)) => RepoAttr::LabelKeyedStringDict(items),
            Some(AttrValue::LabelListDict(items)) => RepoAttr::LabelListDict(items),
            _ => continue,
        };
        out.push((name.to_owned(), converted));
    }
    out
}
