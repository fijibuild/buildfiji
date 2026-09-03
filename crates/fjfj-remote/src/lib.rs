//! Remote execution and caching via the Bazel Remote Execution API (REAPI v2).
//!
//! Plan: depend on the `bazel-remote-apis` crate (prost/tonic bindings) rather
//! than vendoring protos. Wire compatibility with Bazel means fjfj can share
//! a CAS/action cache (bazel-remote, Buildbarn, BuildBuddy, EngFlow, NativeLink)
//! with Bazel builds of the same repo, and even hit the same cache keys when
//! action digests match.

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
