//! Where `query`, `cquery` and `aquery` read their expression and write
//! their result: `--query_file` and `--output_file` (buildfiji-yvk).

use crate::CliError;

/// What those flags said.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Io {
    /// `--query_file`: read the expression from this file.
    pub query_file: Option<String>,
    /// `--output_file`: write the result to this file, not to standard output.
    pub output_file: Option<String>,
}

fn bad(message: impl Into<String>) -> CliError {
    CliError::CommandLine(anyhow::anyhow!(message.into()))
}

/// Pull the two flags out of `args`, returning the rest.
pub(crate) fn extract(args: &[String]) -> Result<(Io, Vec<String>), CliError> {
    let mut io = Io::default();
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let Some(body) = arg.strip_prefix("--") else {
            rest.push(arg.clone());
            continue;
        };
        let (name, value) = match body.split_once('=') {
            Some((n, v)) => (n, Some(v.to_owned())),
            None => (body, None),
        };
        let slot = match name {
            "query_file" => &mut io.query_file,
            "output_file" => &mut io.output_file,
            _ => {
                rest.push(arg.clone());
                continue;
            }
        };
        let value = value
            .or_else(|| iter.next().cloned())
            .ok_or_else(|| bad(format!("--{name} needs a value")))?;
        *slot = Some(value);
    }
    Ok((io, rest))
}

/// The expression: the words that are left, or the contents of the query file.
pub(crate) fn expression(io: &Io, rest: &[String]) -> Result<String, CliError> {
    let Some(file) = &io.query_file else {
        return Ok(rest.join(" "));
    };
    if !rest.is_empty() {
        return Err(bad(
            "Command-line query and --query_file cannot both be specified",
        ));
    }
    std::fs::read_to_string(file)
        .map(|text| text.trim_end().to_owned())
        .map_err(|e| bad(format!("Could not read query file {file}: {e}")))
}

/// Write `bytes` where `--output_file` says, or to standard output.
pub(crate) fn write(io: &Io, bytes: &[u8]) -> Result<(), CliError> {
    use std::io::Write;
    match &io.output_file {
        Some(file) => std::fs::write(file, bytes)
            .map_err(|e| bad(format!("Could not open query output file {file}: {e}"))),
        None => std::io::stdout()
            .write_all(bytes)
            .map_err(|e| CliError::Internal(anyhow::anyhow!("writing the result failed: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_flags_come_out_in_both_spellings() {
        let (io, rest) = extract(&args(&[
            "--query_file=q.txt",
            "--output_file",
            "o.txt",
            "//a",
        ]))
        .unwrap();
        assert_eq!(io.query_file.as_deref(), Some("q.txt"));
        assert_eq!(io.output_file.as_deref(), Some("o.txt"));
        assert_eq!(rest, ["//a"]);
    }

    #[test]
    fn the_expression_is_the_words_or_the_file_but_not_both() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("q.txt");
        std::fs::write(&file, "deps(//a)\n").unwrap();
        let io = Io {
            query_file: Some(file.display().to_string()),
            output_file: None,
        };
        assert_eq!(expression(&io, &[]).unwrap(), "deps(//a)");
        assert!(expression(&io, &args(&["//b"])).is_err());
        assert_eq!(
            expression(&Io::default(), &args(&["//a", "+", "//b"])).unwrap(),
            "//a + //b"
        );
    }

    #[test]
    fn the_result_goes_to_the_output_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("o.txt");
        let io = Io {
            query_file: None,
            output_file: Some(file.display().to_string()),
        };
        write(&io, b"//a\n").unwrap();
        assert_eq!(std::fs::read(&file).unwrap(), b"//a\n");
        let missing = Io {
            query_file: None,
            output_file: Some(dir.path().join("no/such/o.txt").display().to_string()),
        };
        assert!(write(&missing, b"x").is_err());
    }
}
