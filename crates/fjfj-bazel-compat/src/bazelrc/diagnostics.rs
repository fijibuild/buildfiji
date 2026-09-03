//! Ariadne rendering of `.bazelrc` parse errors, so a bad line points at
//! its exact source span instead of just a line number.

use std::path::Path;

use ariadne::{Color, Label, Report, ReportKind, Source};

use super::parse::ParseError;

/// Render `errors` (from [`super::parse::parse_rc_file`] on `source`, the
/// text of the file at `path`) as human-readable Ariadne reports,
/// concatenated in order.
pub fn render_parse_errors(path: &Path, source: &str, errors: &[ParseError]) -> String {
    let id = path.display().to_string();
    let mut buf = Vec::new();

    for err in errors {
        let span = (id.clone(), err.span.clone());
        let report = Report::build(ReportKind::Error, span.clone())
            .with_message(&err.message)
            .with_label(
                Label::new(span)
                    .with_message(&err.message)
                    .with_color(Color::Red),
            )
            .finish();
        // A `Source` per report is wasteful but these are cold-path
        // diagnostics, not a hot loop; revisit if that stops being true.
        let cache = (id.clone(), Source::from(source));
        let _ = report.write(cache, &mut buf);
    }

    String::from_utf8_lossy(&buf).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bazelrc::parse::parse_rc_file;
    use std::path::PathBuf;

    #[test]
    fn renders_something_for_a_bad_import() {
        let (_, errors) = parse_rc_file("import\n");
        assert!(!errors.is_empty());
        let rendered = render_parse_errors(&PathBuf::from(".bazelrc"), "import\n", &errors);
        assert!(rendered.contains("import"));
    }
}
