//! The order a Java `HashMap<ModuleKey, _>` iterates in, which is the order
//! Bazel 9.2.0 writes `selectedYankedVersions` in (buildfiji-avh) and the one
//! in which it finds the first yanked version to complain about.
//!
//! Read off probes (modules `yk0`..`yk11`, `mod..`, `c..`, `e..` of a local
//! registry, all yanked) and the classes of `A-server.jar`:
//!
//! - `ModuleKey` is a record: its hash is `31 * name.hashCode() +
//!   version.hashCode()`, and `Version.hashCode` is
//!   `Arrays.hashCode({"version", normalized.hashCode()})`, that is
//!   `31 * (31 + "version".hashCode()) + normalized.hashCode()`. Strings hash
//!   over UTF-16 code units.
//! - The map holds every selected module, not just the yanked ones (the others
//!   are dropped when written), and grows by doubling from 16 when it is more
//!   than three quarters full: 47 modules have room for 64 buckets, 49 need 128.
//! - Iteration is by bucket, the bucket being `(h ^ h >>> 16) & (capacity - 1)`.
//!   Keys in one bucket come out in ascending order of their hash as an
//!   unsigned number, whatever order they were declared in.

use crate::module::ModuleKey;

fn string_hash(s: &str) -> i32 {
    s.encode_utf16()
        .fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(i32::from(c)))
}

/// `ModuleKey.hashCode()`.
pub(crate) fn module_key_hash(key: &ModuleKey) -> i32 {
    let version = 31i32
        .wrapping_mul(31i32.wrapping_add(string_hash("version")))
        .wrapping_add(string_hash(key.version.as_str()));
    31i32
        .wrapping_mul(string_hash(&key.name))
        .wrapping_add(version)
}

/// How many buckets a default `HashMap` has after `entries` insertions.
fn capacity(entries: usize) -> u32 {
    let mut capacity = 16u64;
    while entries as u64 * 4 > capacity * 3 {
        capacity *= 2;
    }
    capacity as u32
}

/// `keys` in the order a `HashMap` that holds `total` entries iterates them.
pub(crate) fn hash_map_order<'a>(
    keys: impl IntoIterator<Item = &'a ModuleKey>,
    total: usize,
) -> Vec<&'a ModuleKey> {
    let mask = capacity(total) - 1;
    let mut keys: Vec<(&ModuleKey, u32)> = keys
        .into_iter()
        .map(|key| (key, module_key_hash(key) as u32))
        .collect();
    keys.sort_by_key(|&(_, hash)| ((hash ^ (hash >> 16)) & mask, hash));
    keys.into_iter().map(|(key, _)| key).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::Version;

    fn key(name: &str) -> ModuleKey {
        ModuleKey::new(name, Version::parse("1.0").unwrap())
    }

    #[test]
    fn the_hash_is_what_bazels_classes_compute() {
        // ModuleKey.hashCode() of t@1.0, z@1.0 and k@1.0, printed by running Bazel's own
        // class (probes/jvm).
        assert_eq!(module_key_hash(&key("t")), -1985000024);
        assert_eq!(module_key_hash(&key("z")), -1984999838);
        assert_eq!(module_key_hash(&key("k")), -1985000303);
    }

    #[test]
    fn eight_modules_in_a_map_of_64_buckets_come_out_as_bazel_wrote_them() {
        let keys: Vec<ModuleKey> = ["a0", "z", "m", "q", "t", "r2", "b1", "k"]
            .iter()
            .map(|n| key(n))
            .collect();
        let order: Vec<&str> = hash_map_order(&keys, 35)
            .into_iter()
            .map(|k| k.name.as_str())
            .collect();
        assert_eq!(order, ["t", "z", "b1", "m", "q", "r2", "a0", "k"]);
        // The same keys in a map that has grown once more.
        let order: Vec<&str> = hash_map_order(&keys, 49)
            .into_iter()
            .map(|k| k.name.as_str())
            .collect();
        assert_ne!(order, ["t", "z", "b1", "m", "q", "r2", "a0", "k"]);
    }

    #[test]
    fn keys_in_one_bucket_go_by_their_unsigned_hash() {
        // e1013, e1035 (positive) and e41, e63 (negative) share a bucket of 64.
        let keys: Vec<ModuleKey> = ["e63", "e41", "e1035", "e1013"]
            .iter()
            .map(|n| key(n))
            .collect();
        let order: Vec<&str> = hash_map_order(&keys, 40)
            .into_iter()
            .map(|k| k.name.as_str())
            .collect();
        assert_eq!(order, ["e1013", "e1035", "e41", "e63"]);
    }

    #[test]
    fn capacity_doubles_when_a_map_is_three_quarters_full() {
        assert_eq!(capacity(12), 16);
        assert_eq!(capacity(13), 32);
        assert_eq!(capacity(48), 64);
        assert_eq!(capacity(49), 128);
    }
}
