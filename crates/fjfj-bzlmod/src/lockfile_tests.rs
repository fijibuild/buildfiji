use super::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Serves fixed bytes and counts what it was asked for.
struct Served {
    files: HashMap<String, Vec<u8>>,
    asked: Arc<AtomicUsize>,
}

impl Fetcher for Served {
    fn fetch(&self, url: &str) -> Result<Option<Vec<u8>>> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        Ok(self.files.get(url).cloned())
    }
}

const REG: &str = "https://r.example";

fn served(files: &[(&str, &str)]) -> (Box<dyn Fetcher>, Arc<AtomicUsize>) {
    let asked = Arc::new(AtomicUsize::new(0));
    let files = files
        .iter()
        .map(|(u, c)| (format!("{REG}/{u}"), c.as_bytes().to_vec()))
        .collect();
    (
        Box::new(Served {
            files,
            asked: asked.clone(),
        }),
        asked,
    )
}

fn url(path: &str) -> String {
    format!("{REG}/{path}")
}

fn sha(content: &str) -> String {
    sha256_hex(content.as_bytes())
}

fn lock_text(hashes: &[(&str, &str)]) -> String {
    let mut lock = Lockfile::default();
    for (path, hash) in hashes {
        lock.registry_file_hashes.insert(
            url(path),
            if *hash == "not found" {
                FileHash::NotFound
            } else {
                FileHash::Sha256((*hash).to_owned())
            },
        );
    }
    lock.to_text()
}

fn session(mode: LockfileMode, existing: Option<&str>) -> Arc<LockSession> {
    LockSession::new(mode, existing).expect("usable")
}

#[test]
fn an_empty_lockfile_is_written_the_way_bazel_writes_one() {
    assert_eq!(
        Lockfile::default().to_text(),
        "{\n  \"lockFileVersion\": 28,\n  \"registryFileHashes\": {},\n  \
         \"selectedYankedVersions\": {},\n  \"moduleExtensions\": {},\n  \"facts\": {},\n  \
         \"factsVersions\": {}\n}\n"
    );
}

#[test]
fn hashes_are_sorted_and_a_missing_file_says_not_found() {
    let mut lock = Lockfile::default();
    lock.registry_file_hashes
        .insert("https://b/x".into(), FileHash::NotFound);
    lock.registry_file_hashes
        .insert("https://a/y".into(), FileHash::Sha256("ab".into()));
    lock.selected_yanked_versions
        .push(("y@2.0".into(), "it's <bad> & \"worse\"".into()));
    let text = lock.to_text();
    assert!(text.contains(
        "\"registryFileHashes\": {\n    \"https://a/y\": \"ab\",\n    \"https://b/x\": \
         \"not found\"\n  }"
    ));
    // Gson with HTML escaping off leaves `'`, `<`, `>` and `&` alone.
    assert!(text.contains("\"y@2.0\": \"it's <bad> & \\\"worse\\\"\""));
    assert_eq!(Lockfile::parse(&text).unwrap(), lock);
}

#[test]
fn sections_fjfj_does_not_own_survive_in_their_original_order() {
    let text = "{\n  \"lockFileVersion\": 28,\n  \"registryFileHashes\": {},\n  \
        \"selectedYankedVersions\": {},\n  \"moduleExtensions\": {\n    \"@@e+//:x.bzl%e\": {\n      \
        \"general\": {\n        \"usagesDigest\": \"u\",\n        \"recordedInputs\": [\n          \
        \"REPO_MAPPING:e+,a b\"\n        ],\n        \"generatedRepoSpecs\": {},\n        \
        \"n\": 1,\n        \"ok\": true,\n        \"nothing\": null\n      }\n    }\n  },\n  \
        \"facts\": {},\n  \"factsVersions\": {}\n}\n";
    let lock = Lockfile::parse(text).unwrap();
    assert_eq!(lock.to_text(), text);
}

#[test]
fn a_lockfile_of_another_version_or_no_json_is_unusable() {
    assert_eq!(Lockfile::parse("{not json"), Err(Unusable::Malformed));
    assert_eq!(Lockfile::parse("[]"), Err(Unusable::Malformed));
    assert_eq!(Lockfile::parse("{}"), Err(Unusable::Version(0)));
    assert_eq!(
        Lockfile::parse("{\"lockFileVersion\": 27}"),
        Err(Unusable::Version(27))
    );
    assert_eq!(
        Lockfile::parse("{\"lockFileVersion\": 29, \"registryFileHashes\": 3}"),
        Err(Unusable::Version(29))
    );
    // The right version with a section of the wrong shape.
    assert_eq!(
        Lockfile::parse("{\"lockFileVersion\": 28, \"registryFileHashes\": []}"),
        Err(Unusable::Malformed)
    );
}

#[test]
fn error_mode_refuses_an_unusable_lockfile_and_the_others_start_over() {
    let Err(e) = LockSession::new(LockfileMode::Error, Some("{\"lockFileVersion\": 29}")) else {
        panic!("accepted");
    };
    assert_eq!(
        e.to_string(),
        "The version of MODULE.bazel.lock is not supported by this version of Bazel. Please \
         run `bazel mod deps --lockfile_mode=update` to update your lockfile."
    );
    for mode in [LockfileMode::Update, LockfileMode::Refresh] {
        let s = session(mode, Some("{not json"));
        assert_eq!(s.discarded_previous(), Some(&Unusable::Malformed));
    }
    // No lockfile at all is fine for every mode.
    session(LockfileMode::Error, None);
}

#[test]
fn modes_parse() {
    assert_eq!(LockfileMode::parse("off"), Ok(LockfileMode::Off));
    assert_eq!(LockfileMode::parse("refresh"), Ok(LockfileMode::Refresh));
    assert!(LockfileMode::parse("Update").is_err());
    assert!(LockfileMode::Update.writes() && LockfileMode::Refresh.writes());
    assert!(!LockfileMode::Error.writes() && !LockfileMode::Off.writes());
}

#[test]
fn what_a_run_reads_is_recorded_and_the_rest_is_dropped() {
    let old = lock_text(&[("modules/gone/1/MODULE.bazel", &sha("old"))]);
    let s = session(LockfileMode::Update, Some(&old));
    let (inner, asked) = served(&[("modules/a/1/MODULE.bazel", "module(name='a')")]);
    let f = s.fetcher(REG, inner);
    assert!(f.fetch(&url("modules/a/1/MODULE.bazel")).unwrap().is_some());
    assert!(f.fetch(&url("modules/b/1/MODULE.bazel")).unwrap().is_none());
    assert_eq!(asked.load(Ordering::SeqCst), 2);
    let lock = s.finish(&[]);
    assert_eq!(
        lock.registry_file_hashes,
        BTreeMap::from([
            (
                url("modules/a/1/MODULE.bazel"),
                FileHash::Sha256(sha("module(name='a')"))
            ),
            (url("modules/b/1/MODULE.bazel"), FileHash::NotFound),
        ])
    );
}

#[test]
fn keeping_unread_hashes_is_opt_in() {
    let old = lock_text(&[("modules/gone/1/MODULE.bazel", &sha("old"))]);
    let s = LockSession::keeping_unread(LockfileMode::Update, Some(&old)).unwrap();
    let (inner, _) = served(&[("modules/a/1/MODULE.bazel", "x")]);
    s.fetcher(REG, inner)
        .fetch(&url("modules/a/1/MODULE.bazel"))
        .unwrap();
    let kept = s.finish(&[]).registry_file_hashes;
    assert_eq!(kept.len(), 2, "{kept:?}");
    assert!(kept.contains_key(&url("modules/gone/1/MODULE.bazel")));
}

#[test]
fn a_recorded_hash_is_checked_and_a_mismatch_is_an_error() {
    let old = lock_text(&[("modules/a/1/MODULE.bazel", &"0".repeat(64))]);
    let s = session(LockfileMode::Update, Some(&old));
    let (inner, _) = served(&[("modules/a/1/MODULE.bazel", "x")]);
    let e = s
        .fetcher(REG, inner)
        .fetch(&url("modules/a/1/MODULE.bazel"))
        .unwrap_err();
    assert_eq!(
        e.to_string(),
        format!(
            "error accessing registry {REG}: Failed to fetch registry file \
             {REG}/modules/a/1/MODULE.bazel: Checksum was {} but wanted {}",
            sha("x"),
            "0".repeat(64)
        )
    );
}

#[test]
fn error_mode_wants_every_file_recorded() {
    let old = lock_text(&[("modules/a/1/MODULE.bazel", &sha("x"))]);
    let s = session(LockfileMode::Error, Some(&old));
    let (inner, _) = served(&[
        ("modules/a/1/MODULE.bazel", "x"),
        ("modules/b/1/MODULE.bazel", "y"),
    ]);
    let f = s.fetcher(REG, inner);
    assert!(f.fetch(&url("modules/a/1/MODULE.bazel")).unwrap().is_some());
    let e = f.fetch(&url("modules/b/1/MODULE.bazel")).unwrap_err();
    assert_eq!(
        e.to_string(),
        format!(
            "Missing checksum for registry file {REG}/modules/b/1/MODULE.bazel not permitted \
             with --lockfile_mode=error. Please run `bazel mod deps --lockfile_mode=update` to \
             update your lockfile."
        )
    );
}

#[test]
fn a_file_recorded_as_not_found_is_not_asked_for_until_refresh() {
    let old = lock_text(&[("modules/a/1/MODULE.bazel", "not found")]);
    let files = [("modules/a/1/MODULE.bazel", "now there")];
    let s = session(LockfileMode::Update, Some(&old));
    let (inner, asked) = served(&files);
    let f = s.fetcher(REG, inner);
    assert!(f.fetch(&url("modules/a/1/MODULE.bazel")).unwrap().is_none());
    assert_eq!(asked.load(Ordering::SeqCst), 0);
    assert!(s.recorded_missing(&url("modules/a/1/MODULE.bazel")));

    let s = session(LockfileMode::Refresh, Some(&old));
    let (inner, asked) = served(&files);
    let f = s.fetcher(REG, inner);
    assert!(f.fetch(&url("modules/a/1/MODULE.bazel")).unwrap().is_some());
    assert_eq!(asked.load(Ordering::SeqCst), 1);
    assert!(!s.recorded_missing(&url("modules/a/1/MODULE.bazel")));
}

#[test]
fn local_registries_and_metadata_are_not_recorded() {
    let s = session(LockfileMode::Error, None);
    let asked = Arc::new(AtomicUsize::new(0));
    let inner: Box<dyn Fetcher> = Box::new(Served {
        files: HashMap::from([
            (
                "file:///r/modules/a/1/MODULE.bazel".to_owned(),
                b"x".to_vec(),
            ),
            (url("modules/a/metadata.json"), b"{}".to_vec()),
        ]),
        asked,
    });
    let f = s.fetcher(REG, inner);
    // Even error mode, which refuses any unrecorded file, lets these by.
    assert!(
        f.fetch("file:///r/modules/a/1/MODULE.bazel")
            .unwrap()
            .is_some()
    );
    assert!(f.fetch(&url("modules/a/metadata.json")).unwrap().is_some());
    assert!(s.finish(&[]).registry_file_hashes.is_empty());
}

#[test]
fn yanked_versions_come_from_the_lockfile_unless_refreshing() {
    let mut lock = Lockfile::default();
    lock.selected_yanked_versions
        .push(("y@2.0".into(), "bad".into()));
    lock.selected_yanked_versions
        .push(("yy@1.0".into(), "other".into()));
    let text = lock.to_text();
    let s = session(LockfileMode::Update, Some(&text));
    assert_eq!(
        s.locked_yanked("y"),
        Some(vec![("2.0".to_owned(), "bad".to_owned())])
    );
    assert_eq!(s.locked_yanked("z"), None);
    assert_eq!(
        session(LockfileMode::Refresh, Some(&text)).locked_yanked("y"),
        None
    );
}

#[test]
fn the_lockfile_is_written_only_when_it_changes_and_the_mode_writes() {
    let s = session(LockfileMode::Update, None);
    let text = s.to_write(&[], None).expect("a new lockfile is written");
    assert_eq!(s.to_write(&[], Some(&text)), None);
    let key = ModuleKey::new("y", crate::version::Version::parse("2.0").unwrap());
    let with_yanked = s.to_write(&[(key.clone(), "bad".into())], Some(&text));
    assert!(with_yanked.unwrap().contains("\"y@2.0\": \"bad\""));
    for mode in [LockfileMode::Error, LockfileMode::Off] {
        assert_eq!(
            session(mode, None).to_write(&[(key.clone(), "x".into())], None),
            None
        );
    }
}
