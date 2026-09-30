//! Building archives for tests: the bytes of an archive of the given members
//! in a format, so that a test can serve or unpack one without a fixture file.

use std::io::Write;

/// One member to put in an archive.
#[derive(Debug, Clone)]
pub enum Member {
    /// A file with this text; executable if the flag says so.
    File(String, bool),
    Dir,
    Symlink(String),
    Hardlink(String),
    /// A file of these bytes, which need not be text.
    Bytes(Vec<u8>),
}

impl Member {
    pub fn file(text: &str) -> Member {
        Member::File(text.to_owned(), false)
    }

    pub fn executable(text: &str) -> Member {
        Member::File(text.to_owned(), true)
    }
}

fn tar(members: &[(&str, Member)]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (path, member) in members {
        let mut header = tar::Header::new_gnu();
        header.set_mtime(1_700_000_000);
        header.set_size(0);
        // The name is written as it is: the library refuses a `..` or a
        // leading `/`, and a test wants to see what happens with one.
        let gnu = header.as_gnu_mut().expect("a gnu header");
        gnu.name[..path.len()].copy_from_slice(path.as_bytes());
        let mut data: &[u8] = &[];
        match member {
            Member::Dir => {
                header.set_entry_type(tar::EntryType::Directory);
                header.set_mode(0o755);
            }
            Member::File(text, executable) => {
                header.set_entry_type(tar::EntryType::Regular);
                header.set_mode(if *executable { 0o755 } else { 0o644 });
                header.set_size(text.len() as u64);
                data = text.as_bytes();
            }
            Member::Bytes(bytes) => {
                header.set_entry_type(tar::EntryType::Regular);
                header.set_mode(0o644);
                header.set_size(bytes.len() as u64);
                data = bytes;
            }
            Member::Symlink(target) | Member::Hardlink(target) => {
                header.set_entry_type(if matches!(member, Member::Symlink(_)) {
                    tar::EntryType::Symlink
                } else {
                    tar::EntryType::Link
                });
                header.set_mode(0o777);
                let gnu = header.as_gnu_mut().expect("a gnu header");
                gnu.linkname[..target.len()].copy_from_slice(target.as_bytes());
            }
        }
        header.set_cksum();
        builder.append(&header, data).expect("member");
    }
    builder.into_inner().expect("tar")
}

fn zip(members: &[(&str, Member)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (path, member) in members {
        let options = zip::write::SimpleFileOptions::default();
        match member {
            Member::Dir => writer
                .add_directory(path.trim_end_matches('/'), options.unix_permissions(0o755))
                .expect("dir"),
            Member::File(text, executable) => {
                writer
                    .start_file(
                        *path,
                        options.unix_permissions(if *executable { 0o755 } else { 0o644 }),
                    )
                    .expect("file");
                writer.write_all(text.as_bytes()).expect("write");
            }
            Member::Symlink(target) => writer
                .add_symlink(*path, target, options.unix_permissions(0o777))
                .expect("link"),
            Member::Hardlink(_) => panic!("a zip has no hard links"),
            Member::Bytes(bytes) => {
                writer
                    .start_file(*path, options.unix_permissions(0o644))
                    .expect("file");
                writer.write_all(bytes).expect("write");
            }
        }
    }
    writer.finish().expect("zip").into_inner()
}

fn ar(members: &[(&str, Member)]) -> Vec<u8> {
    let mut builder = ar::Builder::new(Vec::new());
    for (path, member) in members {
        let bytes: &[u8] = match member {
            Member::File(text, _) => text.as_bytes(),
            Member::Bytes(bytes) => bytes,
            _ => continue,
        };
        let header = ar::Header::new(path.as_bytes().to_vec(), bytes.len() as u64);
        builder.append(&header, bytes).expect("ar");
    }
    builder.into_inner().expect("ar")
}

/// The bytes of an archive of `members`, in the format `kind` names: `zip`,
/// `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `tar.zst` or `ar`.
pub fn build(kind: &str, members: &[(&str, Member)]) -> Vec<u8> {
    match kind {
        "zip" => zip(members),
        "ar" => ar(members),
        tarred => compress(tarred, tar(members)),
    }
}

/// `bytes` compressed the way the suffix of `kind` (`tar.gz`, `gz`, ...) says.
pub fn compress(kind: &str, bytes: Vec<u8>) -> Vec<u8> {
    match kind.rsplit('.').next().unwrap_or(kind) {
        "tar" => bytes,
        "gz" => {
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&bytes).expect("gz");
            encoder.finish().expect("gz")
        }
        "bz2" => {
            let mut encoder =
                bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::default());
            encoder.write_all(&bytes).expect("bz2");
            encoder.finish().expect("bz2")
        }
        "xz" => {
            let mut encoder = xz2::write::XzEncoder::new(Vec::new(), 1);
            encoder.write_all(&bytes).expect("xz");
            encoder.finish().expect("xz")
        }
        "zst" => zstd::stream::encode_all(&bytes[..], 1).expect("zst"),
        other => panic!("no such compression: {other}"),
    }
}

/// [`build`] for members written as (path, kind, text): kind `f` is a file, `x`
/// an executable file, `d` a directory, `l` a symlink and `h` a hard link (the
/// text is the target). `deb:<comp>` is an `ar` archive like a Debian package,
/// with its `data.tar.<comp>` holding the members.
pub fn build_spec(kind: &str, files: &[(&str, &str, &str)]) -> Vec<u8> {
    let members: Vec<(&str, Member)> = files
        .iter()
        .map(|(path, kind, text)| {
            let member = match *kind {
                "x" => Member::executable(text),
                "d" => Member::Dir,
                "l" => Member::Symlink((*text).to_owned()),
                "h" => Member::Hardlink((*text).to_owned()),
                _ => Member::file(text),
            };
            (*path, member)
        })
        .collect();
    match kind.strip_prefix("deb:") {
        Some(comp) => {
            let data = build(comp, &members);
            let control = build("tar.gz", &[("control", Member::file("x"))]);
            let data_name = format!("data.{comp}");
            build(
                "ar",
                &[
                    ("debian-binary", Member::file("2.0\n")),
                    ("control.tar.gz", Member::Bytes(control)),
                    (&data_name, Member::Bytes(data)),
                ],
            )
        }
        None => build(kind, &members),
    }
}
