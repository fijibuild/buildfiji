//! The loading phase's view of the filesystem: which directories of a repo
//! are packages (buildfiji-mum.5).
//!
//! `glob` (buildfiji-mum.4) reads the same tree the same way.

mod glob;
mod lookup;

pub use glob::{GlobError, GlobOptions, glob};
pub use lookup::{LookupError, PackageLookup};
