//! Analysis: from labels to configured targets and the actions they register
//! (buildfiji-136.1, 136.3).
//!
//! The phase is keys of [`fjfj_engine`]: a [`PackageKey`] is a package with its
//! BUILD file evaluated, and a [`ConfiguredTargetKey`] is a target in a
//! configuration, with its files, its actions and the targets it read. The
//! repositories and the main repository's name come to the keys as [`Env`], the
//! engine's data.

mod expand;
mod native;
mod target;

pub use expand::{ExpandError, Expander, Prerequisite};
pub use target::{ConfiguredTarget, ConfiguredTargetKey, Env, PackageKey, engine};

#[cfg(test)]
mod tests;
