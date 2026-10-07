//! `File`, the type of a source file or an output a rule's code sees
//! (buildfiji-136.2, 136.13). All of it was read off `bazel build` of a rule
//! that printed each member of a file (Bazel 9.2.0):
//!
//! - `path` is the exec path, `short_path` what follows the root (with
//!   `../repo/` in front for another repository's file), `basename` and
//!   `dirname` the last component and the rest of `path` (`.` for a source
//!   in the root package), `extension` what follows the last `.`;
//! - `root.path` is `bazel-out/k8-fastbuild/bin` for an output and empty for a
//!   source; `owner` is the label of the target that makes it;
//! - `repr` is `<generated file sub/x.tar.gz>` or `<source file s.txt>`.

use allocative::Allocative;
use fjfj_graph::{Artifact, Label};
use starlark::collections::StarlarkHasher;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::fmt;
use std::hash::Hash;

/// A file of the build.
#[derive(Debug, Clone, PartialEq, Eq, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct FileValue {
    #[allocative(skip)]
    pub(crate) artifact: Artifact,
    /// The target that makes it; a source file's is the target of that file.
    #[allocative(skip)]
    pub(crate) owner: Label,
}

starlark_simple_value!(FileValue);

impl fmt::Display for FileValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = if self.artifact.is_source() {
            "source"
        } else {
            "generated"
        };
        write!(f, "<{kind} file {}>", self.short_path())
    }
}

impl FileValue {
    pub(crate) fn short_path(&self) -> String {
        let a = &self.artifact;
        if a.is_source() {
            match a.root.prefix.strip_prefix("external/") {
                Some(repo) => format!("../{repo}/{}", a.path),
                None => a.path.clone(),
            }
        } else {
            match a.path.strip_prefix("external/") {
                Some(rest) => format!("../{rest}"),
                None => a.path.clone(),
            }
        }
    }

    fn dirname(&self) -> String {
        let path = self.artifact.exec_path();
        match path.rsplit_once('/') {
            Some((dir, _)) => dir.to_owned(),
            None => ".".to_owned(),
        }
    }

    fn basename(&self) -> String {
        let path = &self.artifact.path;
        path.rsplit('/').next().unwrap_or(path).to_owned()
    }

    fn extension(&self) -> String {
        let base = self.basename();
        match base.rsplit_once('.') {
            Some((stem, ext)) if !stem.is_empty() => ext.to_owned(),
            _ => String::new(),
        }
    }
}

/// `file.root`.
#[derive(Debug, Clone, PartialEq, Eq, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct RootValue {
    pub(crate) path: String,
    pub(crate) source: bool,
}

starlark_simple_value!(RootValue);

impl fmt::Display for RootValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.source {
            "<source root>"
        } else {
            "<derived root>"
        })
    }
}

#[starlark_value(type = "root")]
impl<'v> StarlarkValue<'v> for RootValue {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("root", root_members);
        Some(RES.methods())
    }
}

#[starlark_module]
fn root_members(builder: &mut MethodsBuilder) {
    #[starlark(attribute)]
    fn path(this: Value) -> starlark::Result<String> {
        Ok(this
            .downcast_ref::<RootValue>()
            .expect("a root")
            .path
            .clone())
    }
}

#[starlark_value(type = "File")]
impl<'v> StarlarkValue<'v> for FileValue {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("File", file_members);
        Some(RES.methods())
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        self.artifact.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .downcast_ref::<FileValue>()
            .is_some_and(|o| o.artifact == self.artifact))
    }
}

fn file<'v>(this: Value<'v>) -> &'v FileValue {
    this.downcast_ref::<FileValue>().expect("a File")
}

#[starlark_module]
fn file_members(builder: &mut MethodsBuilder) {
    #[starlark(attribute)]
    fn path<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(file(this).artifact.exec_path())
    }

    #[starlark(attribute)]
    fn short_path<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(file(this).short_path())
    }

    #[starlark(attribute)]
    fn basename<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(file(this).basename())
    }

    #[starlark(attribute)]
    fn dirname<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(file(this).dirname())
    }

    #[starlark(attribute)]
    fn extension<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(file(this).extension())
    }

    #[starlark(attribute)]
    fn is_source<'v>(this: Value<'v>) -> starlark::Result<bool> {
        Ok(file(this).artifact.is_source())
    }

    #[starlark(attribute)]
    fn is_directory<'v>(this: Value<'v>) -> starlark::Result<bool> {
        Ok(file(this).artifact.tree)
    }

    #[starlark(attribute)]
    fn is_symlink<'v>(this: Value<'v>) -> starlark::Result<bool> {
        Ok(file(this).artifact.symlink)
    }

    #[starlark(attribute)]
    fn owner<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(crate::label::StarlarkLabel::from(file(this).owner.clone())))
    }

    #[starlark(attribute)]
    fn root<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let artifact = &file(this).artifact;
        Ok(heap.alloc(RootValue {
            path: artifact.root.prefix.clone(),
            source: artifact.is_source(),
        }))
    }
}

/// The file of an artifact, owned by `owner`.
pub(crate) fn alloc_file<'v>(heap: Heap<'v>, artifact: Artifact, owner: Label) -> Value<'v> {
    heap.alloc(FileValue { artifact, owner })
}

/// The artifact a `File` value is.
pub(crate) fn artifact_of(value: Value<'_>) -> Option<Artifact> {
    value
        .downcast_ref::<FileValue>()
        .map(|f| f.artifact.clone())
}
