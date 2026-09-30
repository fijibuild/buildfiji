//! Archives and patches, the way Bazel 9.2.0's repository rules handle them
//! (buildfiji-mum.8.3): [`extract`] unpacks an archive into a directory, with
//! a prefix stripped and some members renamed, and [`apply_patch`] applies a
//! unified diff, as `repository_ctx.extract()`, `download_and_extract()` and
//! `patch()` do.
//!
//! What Bazel does, read off probes of repository rules against a local HTTP
//! server:
//!
//! - **Formats**, by the archive's name (case-sensitive): `.zip`, `.jar`,
//!   `.war`, `.aar`, `.nupkg`, `.whl` (zip), `.tar`, `.tar.gz`/`.tgz`,
//!   `.tar.xz`/`.txz`, `.tar.zst`/`.tzst`, `.tar.bz2`/`.tbz`, the single-file
//!   `.gz`, `.xz`, `.zst` and `.bz2`, and `.ar` and `.deb` (an `ar` archive
//!   whose members are extracted as they are: it is the caller's job to
//!   unpack `data.tar.*`). `.7z` is named in the message for an unknown
//!   format but fails, with `null`.
//! - **Members.** A leading `/` is dropped (`/abs.txt` is `abs.txt`); `..` is
//!   not refused, so `../escape.txt` lands beside the output directory.
//!   Directories are made, files replace what is there (a directory in the way
//!   is `<path> (Is a directory)`), a file is executable if its mode says any
//!   execute bit, symbolic links are made as they are written, and a hard link
//!   needs its target to have been extracted already (`File "l" linked from
//!   "n" does not exist`).
//! - **Renames, then prefix.** `rename_files` maps a member's path in the
//!   archive to a path relative to the output directory; then `strip_prefix`
//!   drops members that do not start with it and the prefix from those that do.
//!   If no member has the prefix: `Prefix "x" was given, but not found in the
//!   archive. Here are possible prefixes for this archive: "top".`
//! - **Patches** are unified diffs, plain or `git diff`: `strip` leading path
//!   components are removed from the names, a file named `/dev/null` is one
//!   that is made or removed, a hunk applies wherever its lines are found,
//!   lines are read and written with `\n`, `rename from`/`rename to` and
//!   `new file mode` are honoured, and a patch that names no file is a no-op.
//!   A plain multi-file patch, one with no `diff --git` lines, applies only its
//!   last file: Bazel drops the others without a word.

mod extract;
mod patch;
pub mod testing;
#[cfg(test)]
mod tests;

pub use extract::{ExtractRequest, Format, SUFFIXES, extract, format_for};
pub use patch::apply_patch;
