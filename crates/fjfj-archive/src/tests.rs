use crate::testing::{Member, build};
use crate::{ExtractRequest, Format, apply_patch, extract, format_for};
use std::path::Path;

fn members() -> Vec<(&'static str, Member)> {
    vec![
        ("top/a.txt", Member::file("A")),
        ("top/sub/b.sh", Member::executable("B")),
        ("top/empty/", Member::Dir),
        ("top/link", Member::Symlink("a.txt".to_owned())),
    ]
}

fn unpack(
    name: &str,
    bytes: &[u8],
    strip: &str,
    rename: &[(&str, &str)],
) -> (tempfile::TempDir, Result<(), String>) {
    let dir = tempfile::tempdir().unwrap();
    let archive = dir.path().join(name);
    std::fs::write(&archive, bytes).unwrap();
    let out = dir.path().join("out");
    let rename: Vec<(String, String)> = rename
        .iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect();
    let result = extract(&ExtractRequest {
        archive: &archive,
        format: format_for(name).expect("a known suffix"),
        output: &out,
        strip_prefix: strip,
        strip_components: 0,
        rename: &rename,
    });
    (dir, result)
}

fn listing(root: &Path) -> Vec<String> {
    fn walk(root: &Path, at: &Path, out: &mut Vec<String>) {
        let mut entries: Vec<_> = std::fs::read_dir(at).unwrap().flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let rel = path.strip_prefix(root).unwrap().display().to_string();
            let meta = std::fs::symlink_metadata(&path).unwrap();
            if meta.file_type().is_symlink() {
                out.push(format!(
                    "{rel} -> {}",
                    std::fs::read_link(&path).unwrap().display()
                ));
            } else if meta.is_dir() {
                out.push(format!("{rel}/"));
                walk(root, &path, out);
            } else {
                use std::os::unix::fs::PermissionsExt;
                let exec = if meta.permissions().mode() & 0o111 != 0 {
                    "x "
                } else {
                    ""
                };
                out.push(format!(
                    "{rel} = {exec}{}",
                    std::fs::read_to_string(&path).unwrap()
                ));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}

const EXPECTED: [&str; 7] = [
    "top/",
    "top/a.txt = A",
    "top/empty/",
    "top/link -> a.txt",
    "top/sub/",
    "top/sub/b.sh = x B",
    "",
];

#[test]
fn every_format_unpacks_the_same_tree() {
    for kind in ["zip", "tar", "tar.gz", "tar.bz2", "tar.xz", "tar.zst"] {
        let bytes = build(kind, &members());
        let (dir, result) = unpack(&format!("a.{kind}"), &bytes, "", &[]);
        assert_eq!(result, Ok(()), "{kind}");
        let got = listing(&dir.path().join("out"));
        assert_eq!(got, &EXPECTED[..6], "{kind}");
    }
}

#[test]
fn format_is_the_longest_known_suffix_and_is_case_sensitive() {
    assert_eq!(format_for("x.tar.gz"), Some(Format::TarGz));
    assert_eq!(format_for("x.gz"), Some(Format::Gz));
    assert_eq!(format_for("x.TAR.GZ"), None);
    assert_eq!(format_for("x.deb"), Some(Format::Ar));
    assert_eq!(format_for("blob"), None);
}

#[test]
fn a_prefix_is_stripped_and_what_lacks_it_is_dropped() {
    let mut all = members();
    all.push(("other.txt", Member::file("O")));
    let (dir, result) = unpack("a.tar", &build("tar", &all), "top", &[]);
    assert_eq!(result, Ok(()));
    assert_eq!(
        listing(&dir.path().join("out")),
        [
            "a.txt = A",
            "empty/",
            "link -> a.txt",
            "sub/",
            "sub/b.sh = x B"
        ]
    );
}

#[test]
fn a_missing_prefix_lists_the_ones_there_are() {
    let (_dir, result) = unpack("a.zip", &build("zip", &members()), "nothere", &[]);
    assert_eq!(
        result.unwrap_err(),
        "Prefix \"nothere\" was given, but not found in the archive. Here are possible prefixes \
         for this archive: \"top\"."
    );
}

#[test]
fn renames_come_before_the_prefix_is_stripped() {
    let (dir, result) = unpack(
        "a.zip",
        &build("zip", &members()),
        "top",
        &[("top/a.txt", "top/renamed.txt"), ("a.txt", "never.txt")],
    );
    assert_eq!(result, Ok(()));
    let got = listing(&dir.path().join("out"));
    assert!(got.contains(&"renamed.txt = A".to_owned()), "{got:?}");
    assert!(!got.iter().any(|l| l.starts_with("never")), "{got:?}");
}

#[test]
fn a_member_is_not_stopped_from_leaving_the_output() {
    let (dir, result) = unpack(
        "a.tar",
        &build(
            "tar",
            &[
                ("../escape.txt", Member::file("x")),
                ("/abs.txt", Member::file("y")),
            ],
        ),
        "",
        &[],
    );
    assert_eq!(result, Ok(()));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("escape.txt")).unwrap(),
        "x"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("out/abs.txt")).unwrap(),
        "y"
    );
}

#[test]
fn hard_links_need_their_targets_first() {
    let (_dir, result) = unpack(
        "a.tar",
        &build("tar", &[("n", Member::Hardlink("l".to_owned()))]),
        "",
        &[],
    );
    assert_eq!(
        result.unwrap_err(),
        "File \"l\" linked from \"n\" does not exist"
    );
    let (dir, result) = unpack(
        "a.tar",
        &build(
            "tar",
            &[
                ("f", Member::file("x")),
                ("n", Member::Hardlink("f".to_owned())),
            ],
        ),
        "",
        &[],
    );
    assert_eq!(result, Ok(()));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("out/n")).unwrap(),
        "x"
    );
}

#[test]
fn bad_archives_say_what_bazel_says() {
    let (_d, r) = unpack("bad.zip", b"not a zip", "", &[]);
    assert_eq!(
        r.unwrap_err(),
        "Zip file 'bad.zip' is malformed. It does not contain an end of central directory record."
    );
    let (_d, r) = unpack("bad.tar.gz", b"not gzip", "", &[]);
    assert_eq!(r.unwrap_err(), "Input is not in the .gz format");
    let (_d, r) = unpack("a.7z", b"x", "", &[]);
    assert_eq!(r.unwrap_err(), "null");
}

#[test]
fn a_directory_in_the_way_of_a_file_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("out/x/y")).unwrap();
    let archive = dir.path().join("a.tar");
    std::fs::write(&archive, build("tar", &[("x", Member::file("new"))])).unwrap();
    let err = extract(&ExtractRequest {
        archive: &archive,
        format: Format::Tar,
        output: &dir.path().join("out"),
        strip_prefix: "",
        strip_components: 0,
        rename: &[],
    })
    .unwrap_err();
    assert_eq!(
        err,
        format!("{} (Is a directory)", dir.path().join("out/x").display())
    );
}

#[test]
fn a_single_compressed_file_is_named_for_the_archive() {
    let bytes = crate::testing::compress("gz", b"hello".to_vec());
    let (dir, result) = unpack("data.txt.gz", &bytes, "", &[]);
    assert_eq!(result, Ok(()));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("out/data.txt")).unwrap(),
        "hello"
    );
}

#[test]
fn a_deb_is_extracted_as_the_ar_archive_it_is() {
    let bytes = build(
        "ar",
        &[
            ("debian-binary", Member::file("2.0\n")),
            ("data.tar.xz", Member::file("zz")),
        ],
    );
    let (dir, result) = unpack("a.deb", &bytes, "", &[]);
    assert_eq!(result, Ok(()));
    assert_eq!(
        listing(&dir.path().join("out")),
        ["data.tar.xz = zz", "debian-binary = 2.0\n"]
    );
}

fn patched(
    files: &[(&str, &str)],
    patch: &str,
    strip: usize,
) -> (tempfile::TempDir, Result<(), String>) {
    let dir = tempfile::tempdir().unwrap();
    for (name, text) in files {
        let path = dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let result = apply_patch(patch, strip, dir.path());
    (dir, result)
}

const TEN: &str = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n";

#[test]
fn a_hunk_applies_where_its_lines_are() {
    let patch = "--- a/f.txt\n+++ b/f.txt\n@@ -4,3 +4,3 @@\n line2\n-line3\n+LINE3\n line4\n";
    let (dir, result) = patched(&[("f.txt", TEN)], patch, 1);
    assert_eq!(result, Ok(()));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("f.txt")).unwrap(),
        TEN.replace("line3", "LINE3")
    );
}

#[test]
fn strip_takes_leading_components_off_and_says_when_there_are_too_few() {
    let patch = "--- a/f.txt\n+++ b/f.txt\n@@ -2,3 +2,3 @@\n line2\n-line3\n+LINE3\n line4\n";
    let (_d, r) = patched(&[("f.txt", TEN)], patch, 0);
    assert_eq!(
        r.unwrap_err(),
        "Cannot find file to patch (near line 1), old file name (a/f.txt) doesn't exist, new \
         file name (b/f.txt) doesn't exist."
    );
    let (_d, r) = patched(&[("f.txt", TEN)], patch, 2);
    assert_eq!(
        r.unwrap_err(),
        "Cannot determine file name with strip = 2 at line 1:\n--- a/f.txt"
    );
}

#[test]
fn files_are_made_removed_renamed_and_moded() {
    let patch = "diff --git a/new.txt b/new.txt\n--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1,2 @@\n+hello\n+world\n\
                 diff --git a/old.txt b/old.txt\n--- a/old.txt\n+++ /dev/null\n@@ -1 +0,0 @@\n-gone\n\
                 diff --git a/a.txt b/b.txt\nsimilarity index 100%\nrename from a.txt\nrename to b.txt\n\
                 diff --git a/n.sh b/n.sh\nnew file mode 100755\n--- /dev/null\n+++ b/n.sh\n@@ -0,0 +1 @@\n+echo\n";
    let (dir, result) = patched(&[("old.txt", "gone\n"), ("a.txt", "kept\n")], patch, 1);
    assert_eq!(result, Ok(()));
    let read = |n: &str| std::fs::read_to_string(dir.path().join(n)).unwrap();
    assert_eq!(read("new.txt"), "hello\nworld\n");
    assert!(!dir.path().join("old.txt").exists());
    assert!(!dir.path().join("a.txt").exists());
    assert_eq!(read("b.txt"), "kept\n");
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(dir.path().join("n.sh"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111);
}

#[test]
fn a_patch_names_no_file_outside_the_repository() {
    let patch = "--- a/../a.txt\n+++ b/../a.txt\n@@ -1 +1 @@\n-1\n+one\n";
    let (dir, r) = patched(&[("a.txt", "1\n")], patch, 1);
    assert_eq!(
        r.unwrap_err(),
        format!(
            "Cannot patch file outside of external repository ({}), file path = \"../a.txt\" at line 1",
            dir.path().display()
        )
    );
}

#[test]
fn a_hunk_short_of_lines_and_a_patch_of_nothing() {
    let patch =
        "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n 1\n-2\n\\ No newline at end of file\n+two\n";
    let (_d, r) = patched(&[("a.txt", "1\n2")], patch, 1);
    assert_eq!(r.unwrap_err(), "Expecting more chunk line at line 6");
    for nothing in ["", "not a patch\n"] {
        let (_d, r) = patched(&[], nothing, 0);
        assert_eq!(r, Ok(()));
    }
}

#[test]
fn lines_are_read_and_written_with_a_plain_newline() {
    let patch = "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n-1\r\n+one\r\n 2\r\n";
    let (dir, r) = patched(&[("a.txt", "1\r\n2\r\n")], patch, 1);
    assert_eq!(r, Ok(()));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
        "one\n2\n"
    );
}

#[test]
fn a_plain_patch_of_several_files_applies_only_the_last() {
    let one = |c: &str, old: &str, new: &str| {
        format!("--- a/{c}.txt\n+++ b/{c}.txt\n@@ -1 +1 @@\n-{old}\n+{new}\n")
    };
    let plain = format!("{}{}", one("a", "1", "one"), one("b", "2", "two"));
    let (dir, r) = patched(&[("a.txt", "1\n"), ("b.txt", "2\n")], &plain, 1);
    assert_eq!(r, Ok(()));
    let read = |n: &str| std::fs::read_to_string(dir.path().join(n)).unwrap();
    assert_eq!(
        (read("a.txt").as_str(), read("b.txt").as_str()),
        ("1\n", "two\n")
    );
    // With `diff --git` lines between them, both apply.
    let git = format!(
        "diff --git a/a.txt b/a.txt\n{}diff --git a/b.txt b/b.txt\n{}",
        one("a", "1", "one"),
        one("b", "2", "two")
    );
    let (dir, r) = patched(&[("a.txt", "1\n"), ("b.txt", "2\n")], &git, 1);
    assert_eq!(r, Ok(()));
    let read = |n: &str| std::fs::read_to_string(dir.path().join(n)).unwrap();
    assert_eq!(
        (read("a.txt").as_str(), read("b.txt").as_str()),
        ("one\n", "two\n")
    );
}
