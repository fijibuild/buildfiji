//! Remote execution and caching via the Bazel Remote Execution API (REAPI v2).
//!
//! Wire types come from the `bazel-remote-apis` crate. `action_key` holds the
//! canonical encoding of `Directory`, `Command` and `Action` messages, which
//! must be byte-identical to Bazel's so that both tools share cache entries.
//! See `docs/design/remote-execution.md` and `spec/Fjfj/ActionKey.lean`.

pub mod action_key;

pub use bazel_remote_apis::build::bazel::remote::execution::v2 as reapi;

/// A content-addressable store. Implemented locally (disk cache) and remotely (gRPC CAS).
pub trait ContentAddressableStore: Send + Sync {
    fn contains(&self, hash: &str) -> bool;
}

/// Local disk cache layout compatible with `--disk_cache` is a follow-up; see beads.
pub struct DiskCache;

impl ContentAddressableStore for DiskCache {
    fn contains(&self, _hash: &str) -> bool {
        false
    }
}
