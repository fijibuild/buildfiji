//! bzlmod: Bazel's module system.
//!
//! `MODULE.bazel` declares what a workspace depends on; this crate turns
//! that declaration into the set of external repositories the build will
//! use. Bazel 9.2.0 is the compatibility target, and the algorithms here
//! are ports of its implementation rather than reconstructions from the
//! documentation — a resolution that differs from Bazel's by one version
//! is a different build.
//!
//! The pipeline, and where each step lives:
//!
//! | Step | Module | Bazel's name |
//! |---|---|---|
//! | Evaluate one `MODULE.bazel` | [`eval`] | `ModuleFileFunction` |
//! | Walk out to the whole graph | [`discovery`] | `Discovery` |
//! | Pick one version per module | [`selection`] | `Selection` |
//! | Ask a registry for files | [`registry`] | `IndexRegistry` |
//! | All of the above, in order | [`resolve`] | `BazelDepGraphFunction` |
//!
//! What is deliberately *not* here, because it needs repositories fetched
//! or a lockfile format: running module extensions and repository rules
//! (buildfiji-mum.8), reading and writing `MODULE.bazel.lock`
//! (buildfiji-mum.7), and the apparent-name side of repo mapping
//! (buildfiji-mum.15). `WORKSPACE` is out of scope permanently — Bazel 9
//! removed it.

pub mod attrs;
pub mod discovery;
pub mod error;
pub mod eval;
pub mod module;
pub mod overrides;
pub mod registry;
pub mod resolve;
pub mod selection;
pub mod version;

pub use error::{BzlmodError, Result};
pub use module::{DepSpec, Module, ModuleKey};
pub use registry::{BAZEL_CENTRAL_REGISTRY, Registry};
pub use resolve::{Resolution, ResolveOptions, WorkspaceIncludeSource, YankedPolicy, resolve};
pub use selection::Selection;
pub use version::Version;
