//! Analysis: from labels to configured targets and the actions they register
//! (buildfiji-136.1, 136.3).
//!
//! The phase is keys of [`fjfj_engine`]: a [`PackageKey`] is a package with its
//! BUILD file evaluated, and a [`ConfiguredTargetKey`] is a target in a
//! configuration, with its files, its actions and the targets it read. The
//! repositories and the main repository's name come to the keys as [`Env`], the
//! engine's data.

mod aspect;
mod native;
mod runfiles_tree;
mod select;
mod starlark_rule;
mod target;
mod test_action;
mod toolchain;
mod transition;

pub use aspect::AspectKey;
pub use select::ConfigMatching;
pub use starlark_rule::dep_info;
pub use target::{ConfiguredTarget, ConfiguredTargetKey, Env, PackageKey, engine};
pub use target::{Incompatible, PlatformDecl, ToolchainDecl};

#[cfg(test)]
mod tests;

/// `//p:n` for the main repository, `@@repo//p:n` for another.
pub(crate) fn expand_label_text(label: &fjfj_graph::Label) -> String {
    fjfj_graph::expand::label_text(label)
}

/// The constraint values of the platform `label`, analysed in `engine`: what
/// the configuration of `--platforms=<label>` holds.
pub async fn platform_constraints(
    engine: &fjfj_engine::Engine,
    label: &fjfj_graph::Label,
) -> Result<std::collections::BTreeSet<fjfj_graph::Label>, String> {
    let done = engine
        .get(ConfiguredTargetKey {
            label: label.clone(),
            configuration: fjfj_graph::Configuration::default(),
        })
        .await
        .map_err(|e| e.to_string())?;
    match &done.platform {
        Some(platform) => Ok(platform.values()),
        None => Err(format!(
            "{} is not a platform",
            fjfj_graph::expand::label_text(label)
        )),
    }
}

/// [`platform_constraints`] from inside an analysis.
pub(crate) async fn platform_constraints_in(
    ctx: &fjfj_engine::Ctx,
    label: &fjfj_graph::Label,
) -> Result<std::collections::BTreeSet<fjfj_graph::Label>, fjfj_engine::Error> {
    let done = ctx
        .get(ConfiguredTargetKey {
            label: label.clone(),
            configuration: fjfj_graph::Configuration::default(),
        })
        .await?;
    done.platform.as_ref().map(|p| p.values()).ok_or_else(|| {
        fjfj_engine::Error::msg(format!(
            "{} is not a platform",
            fjfj_graph::expand::label_text(label)
        ))
    })
}
