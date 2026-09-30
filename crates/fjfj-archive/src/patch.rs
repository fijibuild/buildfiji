//! Applying a unified diff, plain or from `git diff`.

use std::path::{Component, Path, PathBuf};

struct Hunk {
    old_start: usize,
    old: Vec<String>,
    new: Vec<String>,
}

#[derive(Default)]
struct FilePatch {
    /// The line of the first header, 1-based: where Bazel says it is "near".
    line: usize,
    /// The text of the `---` line, for the error about its name.
    old_header: String,
    old: Option<String>,
    new: Option<String>,
    /// Started by a `diff --git` line.
    from_git: bool,
    rename_from: Option<String>,
    rename_to: Option<String>,
    new_mode: Option<u32>,
    hunks: Vec<Hunk>,
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

fn header_name(text: &str) -> String {
    text.split('\t')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_owned()
}

fn parse_range(text: &str) -> Option<(usize, usize)> {
    let text = text.trim_start_matches(['-', '+']);
    let (start, count) = match text.split_once(',') {
        Some((s, c)) => (s.parse().ok()?, c.parse().ok()?),
        None => (text.parse().ok()?, 1),
    };
    Some((start, count))
}

fn parse(patch: &str) -> Result<Vec<FilePatch>, String> {
    let lines: Vec<&str> = patch.lines().collect();
    let mut files: Vec<FilePatch> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(rest) = line.strip_prefix("diff --git ") {
            let mut file = FilePatch {
                line: i + 1,
                from_git: true,
                ..FilePatch::default()
            };
            // `a/x b/y`: the two names, which cannot have spaces to be told apart.
            if let Some((a, b)) = rest.split_once(' ') {
                file.old = Some(a.to_owned());
                file.new = Some(b.to_owned());
            }
            files.push(file);
        } else if let Some(name) = line.strip_prefix("rename from ") {
            if let Some(file) = files.last_mut() {
                file.rename_from = Some(name.to_owned());
            }
        } else if let Some(name) = line.strip_prefix("rename to ") {
            if let Some(file) = files.last_mut() {
                file.rename_to = Some(name.to_owned());
            }
        } else if let Some(mode) = line
            .strip_prefix("new file mode ")
            .or_else(|| line.strip_prefix("new mode "))
        {
            if let Some(file) = files.last_mut() {
                file.new_mode = u32::from_str_radix(mode.trim(), 8).ok();
            }
        } else if let Some(old) = line.strip_prefix("--- ") {
            if i + 1 < lines.len() && lines[i + 1].starts_with("+++ ") {
                let new = header_name(&lines[i + 1][4..]);
                // A `git diff` header already made this file's entry.
                let in_git = files.last().is_some_and(|f| {
                    f.hunks.is_empty() && f.old_header.is_empty() && f.old.is_some()
                });
                if !in_git {
                    // Bazel 9.2.0 keeps only the last file of a patch that has
                    // no `diff --git` headers: a `---` line starts over.
                    if files.last().is_some_and(|f| !f.from_git) {
                        files.pop();
                    }
                    files.push(FilePatch {
                        line: i + 1,
                        ..FilePatch::default()
                    });
                }
                let file = files.last_mut().expect("just pushed or existing");
                file.old_header = line.to_owned();
                file.old = Some(header_name(old));
                file.new = Some(new);
                i += 1;
            }
        } else if line.starts_with("@@ ") {
            let Some(file) = files.last_mut() else {
                i += 1;
                continue;
            };
            let mut parts = line.split_whitespace();
            parts.next();
            let (old_range, new_range) = (parts.next(), parts.next());
            let (Some((old_start, old_count)), Some((_, new_count))) = (
                old_range.and_then(parse_range),
                new_range.and_then(parse_range),
            ) else {
                i += 1;
                continue;
            };
            let mut hunk = Hunk {
                old_start,
                old: Vec::new(),
                new: Vec::new(),
            };
            let (mut old_left, mut new_left) = (old_count, new_count);
            i += 1;
            while old_left > 0 || new_left > 0 {
                let Some(body) = lines.get(i) else {
                    return Err(format!("Expecting more chunk line at line {}", i + 1));
                };
                let (kind, text) = match body.chars().next() {
                    Some(c @ (' ' | '-' | '+')) => (c, &body[1..]),
                    None => (' ', ""),
                    _ => return Err(format!("Expecting more chunk line at line {}", i + 1)),
                };
                match kind {
                    ' ' if old_left > 0 && new_left > 0 => {
                        hunk.old.push(text.to_owned());
                        hunk.new.push(text.to_owned());
                        old_left -= 1;
                        new_left -= 1;
                    }
                    '-' if old_left > 0 => {
                        hunk.old.push(text.to_owned());
                        old_left -= 1;
                    }
                    '+' if new_left > 0 => {
                        hunk.new.push(text.to_owned());
                        new_left -= 1;
                    }
                    _ => return Err(format!("Expecting more chunk line at line {}", i + 1)),
                }
                i += 1;
            }
            file.hunks.push(hunk);
            continue;
        }
        i += 1;
    }
    Ok(files)
}

/// `name` with `strip` leading components removed, or `None` if there are
/// not enough.
fn stripped(name: &str, strip: usize) -> Option<String> {
    if name == "/dev/null" {
        return Some(name.to_owned());
    }
    let components: Vec<&str> = name.split('/').filter(|c| !c.is_empty()).collect();
    (components.len() > strip).then(|| components[strip..].join("/"))
}

fn lines_of(text: &str) -> Vec<String> {
    text.split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).to_owned())
        .collect()
}

fn apply_hunks(mut lines: Vec<String>, hunks: &[Hunk]) -> Result<Vec<String>, String> {
    let mut drift: isize = 0;
    for hunk in hunks {
        let expected = (hunk.old_start.saturating_sub(1) as isize + drift).max(0) as usize;
        let fits = |at: usize| {
            at + hunk.old.len() <= lines.len()
                && lines[at..at + hunk.old.len()]
                    .iter()
                    .zip(&hunk.old)
                    .all(|(a, b)| a == b)
        };
        let found = (0..=lines.len())
            .flat_map(|d| [expected.checked_add(d), expected.checked_sub(d)])
            .flatten()
            .find(|&at| fits(at))
            .ok_or_else(|| {
                format!(
                    "Cannot apply the hunk at line {}: its context does not match the file",
                    hunk.old_start
                )
            })?;
        lines.splice(found..found + hunk.old.len(), hunk.new.iter().cloned());
        drift += (found as isize - expected as isize) + hunk.new.len() as isize
            - hunk.old.len() as isize;
    }
    Ok(lines)
}

/// Apply `patch` to the files under `root`, after taking `strip` leading
/// components off the names in it. `Err` is what Bazel says after `Error
/// applying patch <file>: `.
pub fn apply_patch(patch: &str, strip: usize, root: &Path) -> Result<(), String> {
    let files = parse(patch)?;
    for file in files {
        let name = |name: &Option<String>| -> Result<Option<String>, String> {
            match name {
                None => Ok(None),
                Some(n) => stripped(n, strip).map(Some).ok_or_else(|| {
                    format!(
                        "Cannot determine file name with strip = {strip} at line {}:\n{}",
                        file.line,
                        if file.old_header.is_empty() {
                            n.clone()
                        } else {
                            file.old_header.clone()
                        }
                    )
                }),
            }
        };
        let old = name(&file.old)?;
        let new = name(&file.new)?;
        let inside = |relative: &str| -> Result<PathBuf, String> {
            let path = lexical(&root.join(relative));
            if path.starts_with(root) {
                Ok(path)
            } else {
                Err(format!(
                    "Cannot patch file outside of external repository ({}), file path = \"{}\" at \
                     line {}",
                    root.display(),
                    relative,
                    file.line
                ))
            }
        };
        let creating = old.as_deref() == Some("/dev/null");
        let deleting = new.as_deref() == Some("/dev/null");
        if let (Some(from), Some(to)) = (&file.rename_from, &file.rename_to) {
            let from = inside(from)?;
            let to = inside(to)?;
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::rename(&from, &to).map_err(|e| e.to_string())?;
            if file.hunks.is_empty() {
                continue;
            }
        }
        if creating {
            let target = inside(new.as_deref().unwrap_or_default())?;
            let mut content: Vec<String> = Vec::new();
            for hunk in &file.hunks {
                content.extend(hunk.new.iter().cloned());
            }
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut text = content.join("\n");
            if !content.is_empty() {
                text.push('\n');
            }
            std::fs::write(&target, text).map_err(|e| e.to_string())?;
            set_mode(&target, file.new_mode.is_some_and(|m| m & 0o111 != 0))?;
            continue;
        }
        if deleting {
            let target = inside(old.as_deref().unwrap_or_default())?;
            std::fs::remove_file(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if file.hunks.is_empty() {
            if let (Some(mode), Some(n)) = (file.new_mode, &new) {
                set_mode(&inside(n)?, mode & 0o111 != 0)?;
            }
            continue;
        }
        let (old_name, new_name) = (old.unwrap_or_default(), new.unwrap_or_default());
        let target = if file.rename_to.is_some() || root.join(&new_name).is_file() {
            inside(&new_name)?
        } else if root.join(&old_name).is_file() {
            inside(&old_name)?
        } else {
            // A name outside the repository is refused before it is looked for.
            inside(&new_name)?;
            return Err(format!(
                "Cannot find file to patch (near line {}), old file name ({old_name}) doesn't \
                 exist, new file name ({new_name}) doesn't exist.",
                file.line
            ));
        };
        let text = std::fs::read_to_string(&target).map_err(|e| e.to_string())?;
        let ends_with_newline = text.ends_with('\n');
        let mut lines = lines_of(&text);
        if ends_with_newline {
            lines.pop();
        }
        let lines = apply_hunks(lines, &file.hunks)?;
        let mut out = lines.join("\n");
        if !lines.is_empty() {
            out.push('\n');
        }
        std::fs::write(&target, out).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn set_mode(path: &Path, executable: bool) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if executable { 0o755 } else { 0o644 };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(unix))]
    let _ = (path, executable);
    Ok(())
}
