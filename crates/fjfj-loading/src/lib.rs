//! The loading phase's view of the filesystem: which directories of a repo
//! are packages (buildfiji-mum.5).
//!
//! Evaluating a BUILD file, `glob` and the rest of `native.*` build on this
//! and land with buildfiji-mum.4.

mod lookup;

pub use lookup::{LookupError, PackageLookup};
