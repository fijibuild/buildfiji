//! The loading phase's view of the filesystem: which directories of a repo
//! are packages (buildfiji-mum.5).
//!
//! `glob` (buildfiji-mum.4) reads the same tree the same way.

mod glob;
mod lookup;
mod resolve;

pub use glob::{GlobError, GlobOptions, glob, subpackages};
pub use lookup::{LookupError, PackageLookup};
pub use resolve::{Failure, PackageSource, Resolved, declared_target, resolve, resolve_with};
