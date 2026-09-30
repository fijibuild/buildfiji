//! Unpacking an archive into a directory.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

/// What an archive is, from its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZst,
    Gz,
    Bz2,
    Xz,
    Zst,
    Ar,
    SevenZ,
}

/// The suffixes [`format_for`] knows, as Bazel lists them when a name has none
/// of them.
pub const SUFFIXES: &str = ".zip, .jar, .war, .aar, .nupkg, .whl, .tar, .tar.gz, .tgz, .gz, \
                            .tar.xz, .txz, .xz, .tar.zst, .tzst, .zst, .tar.bz2, .tbz, .bz2, .ar, \
                            .deb or .7z";

const BY_SUFFIX: &[(&str, Format)] = &[
    (".zip", Format::Zip),
    (".jar", Format::Zip),
    (".war", Format::Zip),
    (".aar", Format::Zip),
    (".nupkg", Format::Zip),
    (".whl", Format::Zip),
    (".tar", Format::Tar),
    (".tar.gz", Format::TarGz),
    (".tgz", Format::TarGz),
    (".gz", Format::Gz),
    (".tar.xz", Format::TarXz),
    (".txz", Format::TarXz),
    (".xz", Format::Xz),
    (".tar.zst", Format::TarZst),
    (".tzst", Format::TarZst),
    (".zst", Format::Zst),
    (".tar.bz2", Format::TarBz2),
    (".tbz", Format::TarBz2),
    (".bz2", Format::Bz2),
    (".ar", Format::Ar),
    (".deb", Format::Ar),
    (".7z", Format::SevenZ),
];

/// The format of a file called `name`: the longest suffix it has.
pub fn format_for(name: &str) -> Option<Format> {
    BY_SUFFIX
        .iter()
        .filter(|(suffix, _)| name.ends_with(suffix))
        .max_by_key(|(suffix, _)| suffix.len())
        .map(|(_, format)| *format)
}

/// What to unpack and where.
pub struct ExtractRequest<'a> {
    pub archive: &'a Path,
    pub format: Format,
    pub output: &'a Path,
    pub strip_prefix: &'a str,
    /// Member paths in the archive, and the paths (relative to `output`) they
    /// are given instead.
    pub rename: &'a [(String, String)],
}

enum Kind {
    Dir,
    File { executable: bool, data: Vec<u8> },
    Symlink(String),
    Hardlink(String),
}

struct Member {
    name: String,
    kind: Kind,
}

/// `path` with `.` and `..` resolved by its text alone.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn io_message(error: &io::Error) -> String {
    let text = error.to_string();
    // What Bazel says of a gzip stream whose checksum is wrong.
    if text.contains("matching checksum") {
        return "Gzip-compressed data is corrupt (CRC32 error)".to_owned();
    }
    text
}

fn read_members(
    request: &ExtractRequest<'_>,
    each: &mut dyn FnMut(Member) -> Result<(), String>,
) -> Result<(), String> {
    let open = || File::open(request.archive).map_err(|e| io_message(&e));
    match request.format {
        Format::Zip => read_zip(request.archive, each),
        Format::Tar => read_tar(open()?, each),
        Format::TarGz => read_tar(flate2::read::GzDecoder::new(gzip_checked(open()?)?), each),
        Format::TarBz2 => read_tar(bzip2::read::BzDecoder::new(open()?), each),
        Format::TarXz => read_tar(xz2::read::XzDecoder::new(open()?), each),
        Format::TarZst => read_tar(
            zstd::stream::read::Decoder::new(open()?).map_err(|e| io_message(&e))?,
            each,
        ),
        Format::Gz => single(
            flate2::read::GzDecoder::new(gzip_checked(open()?)?),
            request,
            each,
        ),
        Format::Bz2 => single(bzip2::read::BzDecoder::new(open()?), request, each),
        Format::Xz => single(xz2::read::XzDecoder::new(open()?), request, each),
        Format::Zst => single(
            zstd::stream::read::Decoder::new(open()?).map_err(|e| io_message(&e))?,
            request,
            each,
        ),
        Format::Ar => read_ar(open()?, each),
        Format::SevenZ => Err("null".to_owned()),
    }
}

/// A gzip stream must start with the gzip magic, which Bazel says so about.
fn gzip_checked(mut file: File) -> Result<impl Read, String> {
    use std::io::{Seek, SeekFrom};
    let mut magic = [0u8; 2];
    let n = file.read(&mut magic).map_err(|e| io_message(&e))?;
    if n < 2 || magic != [0x1f, 0x8b] {
        return Err("Input is not in the .gz format".to_owned());
    }
    file.seek(SeekFrom::Start(0)).map_err(|e| io_message(&e))?;
    Ok(file)
}

fn read_zip(path: &Path, each: &mut dyn FnMut(Member) -> Result<(), String>) -> Result<(), String> {
    let file = File::open(path).map_err(|e| io_message(&e))?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut zip = zip::ZipArchive::new(file).map_err(|_| {
        format!(
            "Zip file '{name}' is malformed. It does not contain an end of central directory \
             record."
        )
    })?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let member_name = entry.name().to_owned();
        let mode = entry.unix_mode().unwrap_or(0o644);
        if entry.is_dir() || member_name.ends_with('/') {
            each(Member {
                name: member_name,
                kind: Kind::Dir,
            })?;
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| io_message(&e))?;
        let kind = if mode & 0o170000 == 0o120000 {
            Kind::Symlink(String::from_utf8_lossy(&data).into_owned())
        } else {
            Kind::File {
                executable: mode & 0o111 != 0,
                data,
            }
        };
        each(Member {
            name: member_name,
            kind,
        })?;
    }
    Ok(())
}

fn read_tar(
    reader: impl Read,
    each: &mut dyn FnMut(Member) -> Result<(), String>,
) -> Result<(), String> {
    let mut archive = tar::Archive::new(reader);
    let entries = archive.entries().map_err(|e| io_message(&e))?;
    for entry in entries {
        let mut entry = entry.map_err(|e| io_message(&e))?;
        let name = String::from_utf8_lossy(&entry.path_bytes()).into_owned();
        let kind = match entry.header().entry_type() {
            tar::EntryType::Directory => Kind::Dir,
            tar::EntryType::Symlink => Kind::Symlink(
                entry
                    .link_name_bytes()
                    .map(|b| String::from_utf8_lossy(&b).into_owned())
                    .unwrap_or_default(),
            ),
            tar::EntryType::Link => Kind::Hardlink(
                entry
                    .link_name_bytes()
                    .map(|b| String::from_utf8_lossy(&b).into_owned())
                    .unwrap_or_default(),
            ),
            tar::EntryType::Regular | tar::EntryType::Continuous => {
                let executable = entry.header().mode().unwrap_or(0o644) & 0o111 != 0;
                let mut data = Vec::new();
                entry.read_to_end(&mut data).map_err(|e| io_message(&e))?;
                Kind::File { executable, data }
            }
            // Extended headers, devices and the like carry nothing to write.
            _ => continue,
        };
        each(Member { name, kind })?;
    }
    Ok(())
}

fn read_ar(
    reader: impl Read,
    each: &mut dyn FnMut(Member) -> Result<(), String>,
) -> Result<(), String> {
    let mut archive = ar::Archive::new(reader);
    while let Some(entry) = archive.next_entry() {
        let mut entry = entry.map_err(|e| io_message(&e))?;
        let name = String::from_utf8_lossy(entry.header().identifier()).into_owned();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(|e| io_message(&e))?;
        each(Member {
            name,
            kind: Kind::File {
                executable: false,
                data,
            },
        })?;
    }
    Ok(())
}

/// A compressed file of one member, named for the archive without its suffix.
fn single(
    mut reader: impl Read,
    request: &ExtractRequest<'_>,
    each: &mut dyn FnMut(Member) -> Result<(), String>,
) -> Result<(), String> {
    let file = request
        .archive
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = BY_SUFFIX
        .iter()
        .filter(|(suffix, _)| file.ends_with(suffix))
        .max_by_key(|(suffix, _)| suffix.len())
        .map_or(file.clone(), |(suffix, _)| {
            file[..file.len() - suffix.len()].to_owned()
        });
    let mut data = Vec::new();
    reader.read_to_end(&mut data).map_err(|e| io_message(&e))?;
    each(Member {
        name,
        kind: Kind::File {
            executable: false,
            data,
        },
    })
}

/// Unpack the archive. `Err` is the message Bazel gives after `Error
/// extracting <archive> to <dir>: `.
pub fn extract(request: &ExtractRequest<'_>) -> Result<(), String> {
    let prefix: Vec<&str> = request
        .strip_prefix
        .split('/')
        .filter(|c| !c.is_empty())
        .collect();
    let mut seen_names: Vec<String> = Vec::new();
    let mut matched = 0usize;
    let mut extracted: BTreeMap<String, PathBuf> = BTreeMap::new();
    std::fs::create_dir_all(request.output).map_err(|e| io_message(&e))?;
    read_members(request, &mut |member| {
        let original = member.name.trim_start_matches('/').to_owned();
        seen_names.push(original.clone());
        // Renames first, on the member's name in the archive; the prefix is
        // taken off what they leave.
        let name = request
            .rename
            .iter()
            .find(|(from, _)| from.trim_end_matches('/') == original.trim_end_matches('/'))
            .map_or(original.clone(), |(_, to)| to.clone());
        let components: Vec<&str> = name.split('/').filter(|c| !c.is_empty()).collect();
        if components.len() < prefix.len() || components[..prefix.len()] != prefix[..] {
            return Ok(());
        }
        matched += 1;
        let rest = &components[prefix.len()..];
        if rest.is_empty() {
            return Ok(());
        }
        let destination = lexical(&request.output.join(rest.join("/")));
        match member.kind {
            Kind::Dir => std::fs::create_dir_all(&destination).map_err(|e| io_message(&e)),
            Kind::File { executable, data } => {
                write_file(&destination, &data, executable)?;
                extracted.insert(original, destination);
                Ok(())
            }
            Kind::Symlink(target) => {
                make_parent(&destination)?;
                let _ = std::fs::remove_file(&destination);
                #[cfg(unix)]
                std::os::unix::fs::symlink(&target, &destination).map_err(|e| io_message(&e))?;
                #[cfg(not(unix))]
                let _ = target;
                Ok(())
            }
            Kind::Hardlink(target) => {
                let Some(source) = extracted.get(target.trim_start_matches('/')) else {
                    return Err(format!(
                        "File \"{target}\" linked from \"{original}\" does not exist"
                    ));
                };
                let data = std::fs::read(source).map_err(|e| io_message(&e))?;
                let executable = is_executable(source);
                write_file(&destination, &data, executable)?;
                extracted.insert(original, destination);
                Ok(())
            }
        }
    })?;
    if !prefix.is_empty() && matched == 0 && !seen_names.is_empty() {
        let mut firsts: Vec<String> = Vec::new();
        for name in &seen_names {
            let parts: Vec<&str> = name.split('/').filter(|c| !c.is_empty()).collect();
            if (parts.len() > 1 || name.ends_with('/')) && !firsts.contains(&parts[0].to_owned()) {
                firsts.push(parts[0].to_owned());
            }
        }
        let list = firsts
            .iter()
            .map(|p| format!("\"{p}\""))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Prefix \"{}\" was given, but not found in the archive. Here are possible prefixes \
             for this archive: {list}.",
            request.strip_prefix
        ));
    }
    Ok(())
}

fn make_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_message(&e))?;
    }
    Ok(())
}

fn write_file(path: &Path, data: &[u8], executable: bool) -> Result<(), String> {
    if path.is_dir() {
        return Err(format!("{} (Is a directory)", path.display()));
    }
    make_parent(path)?;
    let _ = std::fs::remove_file(path);
    std::fs::write(path, data).map_err(|e| io_message(&e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if executable { 0o755 } else { 0o644 };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(|e| io_message(&e))?;
    }
    #[cfg(not(unix))]
    let _ = executable;
    Ok(())
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}
