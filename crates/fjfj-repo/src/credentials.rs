//! Credential helpers, `--credential_helper=[<scope>=]<program>`
//! (buildfiji-mum.12.1), as Bazel 9.2.0 runs them, read off probes.
//!
//! - A helper is run as `<program> get`, in the workspace directory, with
//!   `{"uri":"<url>"}` on its standard input, and answers
//!   `{"headers":{"<name>":["<value>",...]}}` on its standard output. The
//!   headers go on the request; an `Authorization` header from a helper wins
//!   over what `netrc` and `auth` gave.
//! - A scope is a host name, or `*.<domain>` for the domain and all under it; with none a helper is for every
//!   host. The helper for a host is the one with its scope, else one with a
//!   wildcard that matches, else the default; of two for one scope the last
//!   flag wins. A scope that is not a domain name (`127.0.0.1:1234`,
//!   `http://host`, `under_score`) is refused when the flag is parsed, as are an
//!   empty scope and an empty path.
//! - The program is an absolute path or a bare name looked up on `PATH`;
//!   anything else with a `/` in it is refused.
//! - A helper that cannot be run, exits with a code or answers nonsense does not
//!   stop the download: the request goes without, and `Error retrieving auth
//!   headers, continuing without: Failed to get credentials for '<url>' from
//!   helper '<program>': ...` is a warning.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// One `--credential_helper`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialHelper {
    /// A host name or `*.<suffix>`; `None` is every host.
    pub scope: Option<String>,
    /// As written: an absolute path or a bare name.
    pub program: String,
}

fn is_domain(text: &str) -> bool {
    !text.is_empty()
        && text.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}

impl CredentialHelper {
    /// Parse a flag's value, with Bazel's words for what is wrong with it.
    pub fn parse(value: &str) -> Result<CredentialHelper, String> {
        let (scope, program) = match value.split_once('=') {
            Some((scope, program)) => (Some(scope), program),
            None => (None, value),
        };
        if let Some(scope) = scope {
            if scope.is_empty() {
                return Err("Credential helper scope must not be empty".to_owned());
            }
            let host = scope.strip_prefix("*.").unwrap_or(scope);
            if !is_domain(host) {
                return Err(format!(
                    "Credential helper scope '{scope}' must be a valid domain name with an \
                     optional leading '*.' wildcard"
                ));
            }
        }
        if program.is_empty() {
            return Err("Credential helper path must not be empty".to_owned());
        }
        Ok(CredentialHelper {
            scope: scope.map(str::to_owned),
            program: program.to_owned(),
        })
    }

    /// The program to run: an absolute path, or a name found on `path_env`.
    fn resolve(&self, path_env: &str) -> Result<PathBuf, String> {
        if Path::new(&self.program).is_absolute() {
            return Ok(PathBuf::from(&self.program));
        }
        if self.program.contains('/') {
            return Err(format!(
                "Path '{}' must either be absolute or not contain any path separators",
                self.program
            ));
        }
        std::env::split_paths(path_env)
            .map(|dir| dir.join(&self.program))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| {
                format!(
                    "Could not find file with name '{}' on PATH '{path_env}'",
                    self.program
                )
            })
    }
}

/// The helpers of a run, ready to be asked for a URL.
#[derive(Debug, Clone)]
pub struct CredentialHelpers {
    helpers: Vec<(Option<String>, PathBuf)>,
    /// Where they run: the workspace.
    cwd: PathBuf,
    timeout: Duration,
}

/// The headers a helper gave: each name with its values.
pub type Headers = Vec<(String, Vec<String>)>;

fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    host.rsplit_once(':')
        .filter(|(_, port)| port.chars().all(|c| c.is_ascii_digit()))
        .map_or(host, |(h, _)| h)
}

impl CredentialHelpers {
    /// The helpers, in flag order, looked up on `path_env` where they are
    /// bare names; `Err` is the error that stops the run.
    pub fn new(
        helpers: &[CredentialHelper],
        cwd: PathBuf,
        path_env: &str,
        timeout: Duration,
    ) -> Result<CredentialHelpers, String> {
        let resolved = helpers
            .iter()
            .map(|h| Ok((h.scope.clone(), h.resolve(path_env)?)))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(CredentialHelpers {
            helpers: resolved,
            cwd,
            timeout,
        })
    }

    /// The program for `host`: its own scope, a wildcard (the longest suffix),
    /// or the default; the last flag of a kind wins.
    fn program_for(&self, host: &str) -> Option<&Path> {
        let last = |matches: &dyn Fn(&str) -> bool| {
            self.helpers
                .iter()
                .rev()
                .find(|(scope, _)| scope.as_deref().is_some_and(matches))
        };
        if let Some((_, program)) = last(&|scope| scope == host) {
            return Some(program);
        }
        let wildcard = self
            .helpers
            .iter()
            .rev()
            .filter_map(|(scope, program)| {
                // `*.a.com` is `a.com` and everything under it.
                let domain = scope.as_deref()?.strip_prefix("*.")?;
                (host == domain || host.ends_with(&format!(".{domain}")))
                    .then_some((domain.len(), program))
            })
            .max_by_key(|(len, _)| *len);
        if let Some((_, program)) = wildcard {
            return Some(program);
        }
        self.helpers
            .iter()
            .rev()
            .find(|(scope, _)| scope.is_none())
            .map(|(_, program)| program.as_path())
    }

    /// The headers for a request to `url`, `None` if no helper is for its
    /// host; `Err` is what to say as the warning's end.
    pub fn headers_for(&self, url: &str) -> Result<Option<Headers>, String> {
        let Some(program) = self.program_for(host_of(url)) else {
            return Ok(None);
        };
        let failed = |why: String| {
            format!(
                "Failed to get credentials for '{url}' from helper '{}': {why}",
                program.display()
            )
        };
        let mut child = Command::new(program)
            .arg("get")
            .current_dir(&self.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                failed(if e.kind() == std::io::ErrorKind::NotFound {
                    format!(
                        "Cannot run program \"{}\" (in directory \"{}\"): Exec failed, error: 2 \
                         (No such file or directory)",
                        program.display(),
                        self.cwd.display()
                    )
                } else {
                    e.to_string()
                })
            })?;
        let request = format!("{{\"uri\":{}}}\n", json_string(url));
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(request.as_bytes());
        }
        let mut stdout = child.stdout.take().expect("piped");
        let mut stderr = child.stderr.take().expect("piped");
        let out = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = stdout.read_to_string(&mut text);
            text
        });
        let err = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            text
        });
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() > self.timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(failed(format!(
                        "process timed out after {}s",
                        self.timeout.as_secs()
                    )));
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(e) => return Err(failed(e.to_string())),
            }
        };
        let out = out.join().unwrap_or_default();
        let err = err.join().unwrap_or_default();
        if !status.success() {
            return Err(failed(format!(
                "process exited with code {}. stderr: {err}",
                status.code().unwrap_or(-1)
            )));
        }
        parse_headers(&out)
            .map(Some)
            .ok_or_else(|| failed(format!("error parsing output. stderr: {err}")))
    }
}

fn json_string(text: &str) -> String {
    serde_json::Value::String(text.to_owned()).to_string()
}

/// `{"headers": {"name": ["value", ...]}}`, in the helper's order.
fn parse_headers(text: &str) -> Option<Headers> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    // `{}` is a helper with nothing to say.
    let Some(headers) = value.as_object()?.get("headers") else {
        return Some(Vec::new());
    };
    let headers = headers.as_object()?;
    headers
        .iter()
        .map(|(name, values)| {
            let values = values
                .as_array()?
                .iter()
                .map(|v| v.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()?;
            Some((name.clone(), values))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flag_is_a_scope_and_a_program_and_bazels_words_say_what_is_wrong() {
        assert_eq!(
            CredentialHelper::parse("*.example.com=/bin/h").unwrap(),
            CredentialHelper {
                scope: Some("*.example.com".to_owned()),
                program: "/bin/h".to_owned()
            }
        );
        assert_eq!(CredentialHelper::parse("h").unwrap().scope, None);
        for (value, error) in [
            ("", "Credential helper path must not be empty"),
            ("a=", "Credential helper path must not be empty"),
            ("=/h", "Credential helper scope must not be empty"),
            (
                "a:1=/h",
                "Credential helper scope 'a:1' must be a valid domain name with an optional \
                 leading '*.' wildcard",
            ),
            (
                "under_score=/h",
                "Credential helper scope 'under_score' must be a valid domain name with an \
                 optional leading '*.' wildcard",
            ),
        ] {
            assert_eq!(
                CredentialHelper::parse(value).unwrap_err(),
                error,
                "{value}"
            );
        }
    }

    #[test]
    fn a_program_is_absolute_or_found_on_the_path() {
        let h = |p: &str| CredentialHelper::parse(&format!("a.com={p}")).unwrap();
        assert!(h("/x/y").resolve("").is_ok());
        assert_eq!(
            h("rel/h").resolve("").unwrap_err(),
            "Path 'rel/h' must either be absolute or not contain any path separators"
        );
        assert_eq!(
            h("nope-h").resolve("/nonexistent").unwrap_err(),
            "Could not find file with name 'nope-h' on PATH '/nonexistent'"
        );
        assert!(h("sh").resolve("/bin:/usr/bin").is_ok());
    }

    fn helpers(list: &[&str]) -> CredentialHelpers {
        let parsed: Vec<_> = list
            .iter()
            .map(|v| CredentialHelper::parse(v).unwrap())
            .collect();
        CredentialHelpers::new(&parsed, PathBuf::from("/"), "", Duration::from_secs(5)).unwrap()
    }

    #[test]
    fn the_helper_for_a_host_is_its_own_then_a_wildcard_then_the_default() {
        let h = helpers(&[
            "/d",
            "a.com=/first",
            "a.com=/second",
            "*.com=/wild",
            "*.b.com=/longer",
        ]);
        let program = |url: &str| h.program_for(host_of(url)).map(|p| p.display().to_string());
        assert_eq!(program("https://a.com/x").as_deref(), Some("/second"));
        assert_eq!(
            program("https://x.b.com:8080/x").as_deref(),
            Some("/longer")
        );
        assert_eq!(program("https://c.com/x").as_deref(), Some("/wild"));
        assert_eq!(program("https://b.com/x").as_deref(), Some("/longer"));
        assert_eq!(program("https://xb.com/x").as_deref(), Some("/wild"));
        assert_eq!(program("http://u:p@other.org/x").as_deref(), Some("/d"));
        assert_eq!(helpers(&["a.com=/x"]).program_for("b.com"), None);
    }

    #[test]
    fn headers_are_read_from_what_a_helper_prints() {
        assert_eq!(
            parse_headers(r#"{"headers":{"A":["1","2"],"B":["3"]}}"#).unwrap(),
            [
                ("A".to_owned(), vec!["1".to_owned(), "2".to_owned()]),
                ("B".to_owned(), vec!["3".to_owned()])
            ]
        );
        assert_eq!(parse_headers("{}").unwrap(), []);
        assert!(parse_headers("not json").is_none());
        assert!(parse_headers(r#"{"headers":{"A":"x"}}"#).is_none());
    }
}
