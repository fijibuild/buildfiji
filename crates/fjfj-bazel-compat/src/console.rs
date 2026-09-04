//! Console UI rendering rules — pure, no I/O (no terminal, no clock): given
//! whatever [`crate::console_flags::ConsoleFlags`] said and whether stdout
//! turned out to be a tty, decide what a progress update should look like.
//! The actual writing (and tty detection) is `fjfj-exec::console`'s job,
//! same split as `workspace_status`/`fjfj-exec::workspace_status`.
//!
//! Modeled on Bazel's console `UiEventHandler`: with curses, later updates
//! overwrite the last progress line instead of scrolling; without it,
//! every update is its own line, matching `--curses`'s own doc ("minimize
//! scrolling output").

use crate::console_flags::{ConsoleFlags, TriState};

impl TriState {
    /// Resolves `auto` against whether stdout is actually a tty — the one
    /// place `--color`/`--curses`'s "auto" grammar becomes a plain bool.
    pub fn resolve(self, is_tty: bool) -> bool {
        match self {
            TriState::Yes => true,
            TriState::No => false,
            TriState::Auto => is_tty,
        }
    }
}

/// One `[<done> / <total>] <message>` progress update, Bazel's own shape
/// (`[123 / 456] Compiling foo.rs`). `total` of `0` means "unknown yet" —
/// Bazel shows a bare count until the action graph is fully counted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressUpdate {
    pub done: u64,
    pub total: u64,
    pub message: String,
}

impl ProgressUpdate {
    /// The line's own text, with no cursor control or color — what a
    /// `--nocurses --nocolor` run prints verbatim.
    pub fn line(&self) -> String {
        if self.total == 0 {
            format!("[{}] {}", self.done, self.message)
        } else {
            format!("[{} / {}] {}", self.done, self.total, self.message)
        }
    }
}

/// Bazel's own `EventKind` values `--ui_event_filters` selects among
/// (`UiEventFilters`'s documented set: "INFO, DEBUG, ERROR and more").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    Progress,
    Start,
    Finish,
    Pass,
    Fail,
    Timeout,
    Cancelled,
    Subcommand,
    Stdout,
    Stderr,
    Info,
    Debug,
    Warning,
    Error,
}

impl EventKind {
    fn parse(name: &str) -> Option<EventKind> {
        match name.to_ascii_uppercase().as_str() {
            "PROGRESS" => Some(EventKind::Progress),
            "START" => Some(EventKind::Start),
            "FINISH" => Some(EventKind::Finish),
            "PASS" => Some(EventKind::Pass),
            "FAIL" => Some(EventKind::Fail),
            "TIMEOUT" => Some(EventKind::Timeout),
            "CANCELLED" => Some(EventKind::Cancelled),
            "SUBCOMMAND" => Some(EventKind::Subcommand),
            "STDOUT" => Some(EventKind::Stdout),
            "STDERR" => Some(EventKind::Stderr),
            "INFO" => Some(EventKind::Info),
            "DEBUG" => Some(EventKind::Debug),
            "WARNING" => Some(EventKind::Warning),
            "ERROR" => Some(EventKind::Error),
            _ => None,
        }
    }

    /// Bazel shows everything except `DEBUG` by default — the flag's own
    /// examples treat `+DEBUG` as the thing you have to opt into.
    const DEFAULT: &'static [EventKind] = &[
        EventKind::Progress,
        EventKind::Start,
        EventKind::Finish,
        EventKind::Pass,
        EventKind::Fail,
        EventKind::Timeout,
        EventKind::Cancelled,
        EventKind::Subcommand,
        EventKind::Stdout,
        EventKind::Stderr,
        EventKind::Info,
        EventKind::Warning,
        EventKind::Error,
    ];
}

/// Which [`EventKind`]s the console shows, per `--ui_event_filters`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiEventFilters(std::collections::BTreeSet<EventKind>);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid --ui_event_filters entry '{0}': not one of PROGRESS, START, FINISH, PASS, FAIL, TIMEOUT, CANCELLED, SUBCOMMAND, STDOUT, STDERR, INFO, DEBUG, WARNING, ERROR"
)]
pub struct InvalidEventKind(String);

impl UiEventFilters {
    /// Bazel's own default set (every kind but `DEBUG`).
    pub fn default_set() -> UiEventFilters {
        UiEventFilters(EventKind::DEFAULT.iter().copied().collect())
    }

    /// Parses `--ui_event_filters`' repeated values in order. Each
    /// comma-separated entry either adjusts the running set (every name
    /// prefixed `+`/`-`) or, the moment one name has neither prefix,
    /// replaces the set outright with just the entries from that point
    /// forward that also lack a prefix — matching the flag's own doc:
    /// "add or remove... using leading +/-, or override the default set
    /// completely with direct assignment."
    pub fn parse<'a>(
        values: impl IntoIterator<Item = &'a str>,
    ) -> Result<UiEventFilters, InvalidEventKind> {
        let mut set = Self::default_set().0;
        let mut overriding = false;
        for entry in values {
            for name in entry.split(',') {
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                let (op, rest) = match name.as_bytes()[0] {
                    b'+' => (Some(true), &name[1..]),
                    b'-' => (Some(false), &name[1..]),
                    _ => (None, name),
                };
                let kind =
                    EventKind::parse(rest).ok_or_else(|| InvalidEventKind(name.to_owned()))?;
                match op {
                    Some(true) => {
                        set.insert(kind);
                    }
                    Some(false) => {
                        set.remove(&kind);
                    }
                    None => {
                        if !overriding {
                            set.clear();
                            overriding = true;
                        }
                        set.insert(kind);
                    }
                }
            }
        }
        Ok(UiEventFilters(set))
    }

    pub fn allows(&self, kind: EventKind) -> bool {
        self.0.contains(&kind)
    }
}

/// Resolved console behavior, computed once per invocation from
/// [`ConsoleFlags`] and the real terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleConfig {
    pub color: bool,
    pub curses: bool,
    pub show_progress: bool,
    pub filters: UiEventFilters,
}

impl ConsoleConfig {
    pub fn resolve(flags: &ConsoleFlags, is_tty: bool) -> Result<ConsoleConfig, InvalidEventKind> {
        Ok(ConsoleConfig {
            color: flags.color.resolve(is_tty),
            curses: flags.curses.resolve(is_tty),
            show_progress: flags.show_progress,
            filters: UiEventFilters::parse(flags.ui_event_filters.iter().map(String::as_str))?,
        })
    }
}

/// One ANSI SGR color Bazel's own UI uses for a line — not every
/// `EventKind` has one; the rest print uncolored.
pub fn color_code(kind: EventKind) -> Option<&'static str> {
    match kind {
        EventKind::Error | EventKind::Fail | EventKind::Timeout => Some("31"), // red
        EventKind::Warning => Some("33"),                                      // yellow
        EventKind::Pass => Some("32"),                                         // green
        EventKind::Info | EventKind::Progress => Some("36"),                   // cyan
        _ => None,
    }
}

/// Wraps `text` in `code`'s SGR escape when `color` is on; passes it
/// through unchanged otherwise. Kept as a pure string transform so the
/// terminal-writing half never has to special-case "color is off".
pub fn colorize(text: &str, code: Option<&str>, color: bool) -> String {
    match (color, code) {
        (true, Some(code)) => format!("\x1b[{code}m{text}\x1b[0m"),
        _ => text.to_owned(),
    }
}

/// What to write for a progress update.
///
/// Under curses, a progress line is *never* terminated with `\n` — that's
/// what makes the next one overwrite it instead of scrolling: `\r` returns
/// to the line's start (a no-op the very first time, since nothing has
/// been written yet) and `\x1b[K` clears whatever was there before the new
/// text goes down. The line only actually becomes permanent (gets its
/// `\n`) when something else needs to be written after it — that's
/// `ConsoleUi::line`'s job, since it's the one write that isn't another
/// progress update.
///
/// Without curses, every update is its own permanent line, matching
/// `--curses`'s own doc ("minimize scrolling output" is what curses
/// buys you; without it, Bazel just prints each update and moves on).
pub fn render_progress(config: &ConsoleConfig, update: &ProgressUpdate) -> String {
    if !config.show_progress {
        return String::new();
    }
    let line = colorize(
        &update.line(),
        color_code(EventKind::Progress),
        config.color,
    );
    if config.curses {
        format!("\r\x1b[K{line}")
    } else {
        format!("{line}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tristate_auto_follows_the_tty() {
        assert!(TriState::Auto.resolve(true));
        assert!(!TriState::Auto.resolve(false));
        assert!(TriState::Yes.resolve(false));
        assert!(!TriState::No.resolve(true));
    }

    #[test]
    fn progress_line_shape() {
        let update = ProgressUpdate {
            done: 12,
            total: 34,
            message: "Compiling foo.rs".to_owned(),
        };
        assert_eq!(update.line(), "[12 / 34] Compiling foo.rs");
        let unknown_total = ProgressUpdate {
            done: 5,
            total: 0,
            message: "Analyzing".to_owned(),
        };
        assert_eq!(unknown_total.line(), "[5] Analyzing");
    }

    #[test]
    fn default_filters_exclude_debug_only() {
        let filters = UiEventFilters::default_set();
        assert!(filters.allows(EventKind::Info));
        assert!(filters.allows(EventKind::Error));
        assert!(!filters.allows(EventKind::Debug));
    }

    #[test]
    fn plus_minus_adjust_the_default_set() {
        let filters = UiEventFilters::parse(["+DEBUG", "-INFO"]).unwrap();
        assert!(filters.allows(EventKind::Debug));
        assert!(!filters.allows(EventKind::Info));
        // Untouched entries keep their default.
        assert!(filters.allows(EventKind::Error));
    }

    #[test]
    fn bare_names_override_the_set_completely() {
        let filters = UiEventFilters::parse(["INFO,ERROR"]).unwrap();
        assert!(filters.allows(EventKind::Info));
        assert!(filters.allows(EventKind::Error));
        assert!(!filters.allows(EventKind::Warning));
        assert!(!filters.allows(EventKind::Debug));
    }

    #[test]
    fn unknown_kind_is_rejected() {
        assert!(UiEventFilters::parse(["NOTAKIND"]).is_err());
    }

    #[test]
    fn invalid_ui_event_filters_value_fails_resolve() {
        let flags = crate::console_flags::ConsoleFlags {
            ui_event_filters: vec!["NOTAKIND".to_owned()],
            ..crate::console_flags::ConsoleFlags::default()
        };
        assert!(ConsoleConfig::resolve(&flags, true).is_err());
    }

    #[test]
    fn color_wraps_only_when_enabled() {
        assert_eq!(colorize("x", Some("31"), true), "\x1b[31mx\x1b[0m");
        assert_eq!(colorize("x", Some("31"), false), "x");
        assert_eq!(colorize("x", None, true), "x");
    }

    #[test]
    fn curses_progress_never_ends_with_a_newline() {
        let config = ConsoleConfig {
            color: false,
            curses: true,
            show_progress: true,
            filters: UiEventFilters::default_set(),
        };
        let update = ProgressUpdate {
            done: 1,
            total: 2,
            message: "a".to_owned(),
        };
        assert_eq!(render_progress(&config, &update), "\r\x1b[K[1 / 2] a");
    }

    #[test]
    fn no_curses_always_ends_with_a_newline() {
        let config = ConsoleConfig {
            color: false,
            curses: false,
            show_progress: true,
            filters: UiEventFilters::default_set(),
        };
        let update = ProgressUpdate {
            done: 1,
            total: 2,
            message: "a".to_owned(),
        };
        assert_eq!(render_progress(&config, &update), "[1 / 2] a\n");
    }

    #[test]
    fn show_progress_false_suppresses_output() {
        let config = ConsoleConfig {
            color: false,
            curses: false,
            show_progress: false,
            filters: UiEventFilters::default_set(),
        };
        let update = ProgressUpdate {
            done: 1,
            total: 2,
            message: "a".to_owned(),
        };
        assert_eq!(render_progress(&config, &update), "");
    }
}
