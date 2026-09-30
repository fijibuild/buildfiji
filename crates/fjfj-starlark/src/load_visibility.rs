//! Load visibility (buildfiji-ps4): the `visibility()` a `.bzl` file may call
//! once at its top level, and the check a loader makes before a file is
//! loaded.
//!
//! What Bazel 9.2.0 does, read off probes:
//!
//! - `visibility(value)` (`value` positional-only) takes `"public"`,
//!   `"private"` or a list of strings, each of which is `public`, `private`
//!   or a package specification (`//pkg`, `//pkg:__pkg__`, `//pkg/...`,
//!   `//...`, `@repo//pkg`, `@@repo//pkg`), as in a `package_group`;
//!   a `-` spec is refused. A file that never calls it is public, and
//!   `private` or `[]` lets no other package load it; a list that contains
//!   `public` is public. A package may always load its own files, and a
//!   spec never reaches a subpackage unless it says `/...`.
//! - It may be called once (`load visibility may not be set more than
//!   once`), not inside a function (`load visibility may only be set at the
//!   top level, not inside a function`), and only while a `.bzl` loads.
//!   Where in the file it is does not matter.
//! - A load that is not allowed is `Starlark file //a:lib.bzl is not visible
//!   for loading from package //b. Check the file's `visibility()`
//!   declaration.` (the main repo's root package is `//`), and
//!   `--check_bzl_visibility=false` turns the check off.
//!
//! The declaration is kept in the module as its extra value, so it is there
//! for a loader to read once the file is frozen ([`load_visibility`]).

use crate::args::{Wording, bind, fatal, positional_only};
use crate::label::{caller, display_label, evaluating_bzl};
use allocative::Allocative;
use fjfj_graph::Label;
use fjfj_graph::visibility::{PackageScope, PackageSpec, SpecError};
use starlark::environment::{FrozenModule, GlobalsBuilder};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::none::NoneType;
use starlark::values::{NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::fmt;

/// Who may load a `.bzl` file.
#[derive(Debug, Clone, PartialEq, Eq, ProvidesStaticType, NoSerialize, Allocative)]
pub struct LoadVisibility {
    /// What is allowed. Empty is private.
    #[allocative(skip)]
    scopes: Vec<PackageScope>,
}

impl LoadVisibility {
    pub fn public() -> LoadVisibility {
        LoadVisibility {
            scopes: vec![PackageScope::Public],
        }
    }

    pub fn private() -> LoadVisibility {
        LoadVisibility { scopes: Vec::new() }
    }

    /// Whether a file in `package` of `repo` may load the file.
    pub fn allows(&self, repo: &str, package: &str) -> bool {
        self.scopes.iter().any(|s| s.contains(repo, package))
    }
}

starlark_simple_value!(LoadVisibility);

impl fmt::Display for LoadVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<load visibility>")
    }
}

#[starlark_value(type = "load_visibility")]
impl<'v> StarlarkValue<'v> for LoadVisibility {}

/// Who may load the file `module` is: what its `visibility()` said, public
/// if it never called it.
pub fn load_visibility(module: &FrozenModule) -> LoadVisibility {
    module
        .extra_value()
        .and_then(|v| v.to_value().downcast_ref::<LoadVisibility>().cloned())
        .unwrap_or_else(LoadVisibility::public)
}

/// The error for a load a file in `importer`'s package may not make, or
/// `Ok` if it may (or `check` is false, which is
/// `--check_bzl_visibility=false`).
pub fn check_load_visibility(
    importer: &Label,
    file: &Label,
    visibility: &LoadVisibility,
    check: bool,
) -> Result<(), String> {
    if !check
        || (importer.repo == file.repo && importer.package == file.package)
        || visibility.allows(&importer.repo, &importer.package)
    {
        return Ok(());
    }
    let package = Label {
        repo: importer.repo.clone(),
        package: importer.package.clone(),
        name: String::new(),
    };
    // A package is written like a label with no name: `//b`, `//`.
    let shown = display_label(&package);
    let shown = shown.strip_suffix(':').unwrap_or(&shown);
    Err(format!(
        "Starlark file {} is not visible for loading from package {shown}. Check the file's \
         `visibility()` declaration.",
        display_label(file)
    ))
}

/// One string of a `visibility()`: a package specification, in the repo of
/// the file that wrote it. `None` for what matches nothing.
fn scope_of<'v>(
    text: &str,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Option<PackageScope>> {
    if text.starts_with('-') {
        return Err(fatal("Cannot use negative package patterns here"));
    }
    // `@repo//p` names a repo as the file's own mapping does.
    let (file, mappings) = caller(eval, "visibility")?;
    let repo = file.repo;
    let text = if text.starts_with('@') && !text.starts_with("@@") {
        let apparent = text[1..].split("//").next().unwrap_or_default();
        let canonical = mappings.resolve_apparent(&repo, apparent);
        if canonical.starts_with('[') {
            // A repo the file cannot name: the spec reaches nobody.
            return Ok(None);
        }
        let rest = text.get(1 + apparent.len()..).unwrap_or_default();
        format!("@@{canonical}{rest}")
    } else {
        text.to_owned()
    };
    match PackageSpec::parse(&text, &repo) {
        Ok(Some(spec)) => Ok(Some(spec.scope)),
        Ok(None) => Ok(None),
        Err(SpecError::Label(e)) => Err(fatal(format!("invalid package name '{text}': {e}"))),
        Err(e) => Err(fatal(e.to_string())),
    }
}

fn make_visibility<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    if !evaluating_bzl(eval) {
        return Err(fatal(
            "visibility() can only be used during .bzl initialization (top-level evaluation)",
        ));
    }
    let bound = bind(
        "visibility",
        Wording::Signature,
        &[positional_only("value", true)],
        args,
        eval,
    )?;
    let value = bound[0].expect("required");
    let items: Vec<Value<'v>> = if value.unpack_str().is_some() {
        vec![value]
    } else if value.get_type() == "list" {
        value.iterate(eval.heap())?.collect()
    } else {
        return Err(fatal(format!(
            "Invalid visibility: got '{}', want string or list of strings",
            value.get_type()
        )));
    };
    let mut scopes = Vec::new();
    for (i, item) in items.iter().enumerate() {
        let Some(text) = item.unpack_str() else {
            return Err(fatal(format!(
                "at index {i} of visibility list, got element of type {}, want string",
                item.get_type()
            )));
        };
        if let Some(scope) = scope_of(text, eval)? {
            scopes.push(scope);
        }
    }
    // `public` beside other entries is all of them.
    if scopes.contains(&PackageScope::Public) {
        scopes = vec![PackageScope::Public];
    }
    if eval.module().extra_value().is_some() {
        return Err(fatal("load visibility may not be set more than once"));
    }
    if eval.call_stack_count() > 2 {
        return Err(fatal(
            "load visibility may only be set at the top level, not inside a function",
        ));
    }
    let declared = eval.heap().alloc(LoadVisibility { scopes });
    eval.module().set_extra_value(declared);
    Ok(NoneType)
}

#[starlark_module]
pub(crate) fn visibility_globals(builder: &mut GlobalsBuilder) {
    /// `visibility(value)`.
    fn visibility<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        make_visibility(args, eval)
    }
}
