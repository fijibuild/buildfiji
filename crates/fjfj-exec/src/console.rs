//! Writes console UI updates to a real terminal. The rendering rules
//! (what a progress line looks like, whether it overwrites the last one,
//! ANSI color) live in `fjfj_bazel_compat::console` (pure, no I/O); this
//! module is the I/O half — the actual write — same split as
//! `fjfj_bazel_compat::workspace_status`/`workspace_status`.
//!
//! Tty detection is the caller's job (`std::io::IsTerminal::is_terminal`
//! on the real stream, e.g. `stdout()`), passed in as a plain `bool`:
//! `IsTerminal` is a sealed trait, so a test double can't implement it,
//! and there is no other reason for this module to know which concrete
//! stream type it's writing to.

use std::io::Write;

use fjfj_bazel_compat::console::{
    ConsoleConfig, InvalidEventKind, ProgressUpdate, render_progress,
};
use fjfj_bazel_compat::console_flags::ConsoleFlags;

/// A console UI bound to one output stream, tracking just enough state
/// (was the last thing written a progress line) to decide whether the
/// next progress update overwrites it or starts fresh.
pub struct ConsoleUi<W: Write> {
    out: W,
    config: ConsoleConfig,
    previous_was_progress: bool,
}

impl<W: Write> ConsoleUi<W> {
    /// Resolves `flags` against `is_tty` (Bazel decides `--color=auto`/
    /// `--curses=auto` per output stream, not globally, so this takes the
    /// tty-ness of `out` specifically — not e.g. always stdout's).
    pub fn new(
        out: W,
        flags: &ConsoleFlags,
        is_tty: bool,
    ) -> Result<ConsoleUi<W>, InvalidEventKind> {
        Ok(ConsoleUi {
            config: ConsoleConfig::resolve(flags, is_tty)?,
            out,
            previous_was_progress: false,
        })
    }

    #[cfg(test)]
    fn config(&self) -> &ConsoleConfig {
        &self.config
    }

    /// Writes one progress update, honoring `--show_progress` and
    /// `--curses` (a no-op, cleanly, when progress is suppressed).
    pub fn progress(&mut self, update: &ProgressUpdate) -> std::io::Result<()> {
        let rendered = render_progress(&self.config, update);
        if rendered.is_empty() {
            return Ok(());
        }
        self.out.write_all(rendered.as_bytes())?;
        self.out.flush()?;
        self.previous_was_progress = true;
        Ok(())
    }

    /// Writes a plain line (already formatted by the caller): if the
    /// previous write was an unfinished curses progress line, first move
    /// to a fresh one so the two don't run together.
    pub fn line(&mut self, text: &str) -> std::io::Result<()> {
        if self.previous_was_progress && self.config.curses {
            self.out.write_all(b"\n")?;
        }
        writeln!(self.out, "{text}")?;
        self.out.flush()?;
        self.previous_was_progress = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fjfj_bazel_compat::console::{ProgressUpdate, UiEventFilters};
    use fjfj_bazel_compat::console_flags::TriState;

    fn text(ui: &ConsoleUi<Vec<u8>>) -> String {
        String::from_utf8(ui.out.clone()).unwrap()
    }

    #[test]
    fn auto_color_and_curses_follow_the_given_tty_ness() {
        let ui = ConsoleUi::new(Vec::new(), &ConsoleFlags::default(), true).unwrap();
        assert!(ui.config().color);
        assert!(ui.config().curses);
        let ui = ConsoleUi::new(Vec::new(), &ConsoleFlags::default(), false).unwrap();
        assert!(!ui.config().color);
        assert!(!ui.config().curses);
    }

    #[test]
    fn explicit_color_wins_over_a_non_tty() {
        let flags = ConsoleFlags {
            color: TriState::Yes,
            ..ConsoleFlags::default()
        };
        let ui = ConsoleUi::new(Vec::new(), &flags, false).unwrap();
        assert!(ui.config().color);
    }

    #[test]
    fn successive_progress_updates_overwrite_under_curses() {
        let flags = ConsoleFlags {
            color: TriState::No,
            curses: TriState::Yes,
            ..ConsoleFlags::default()
        };
        let mut ui = ConsoleUi::new(Vec::new(), &flags, true).unwrap();
        ui.progress(&ProgressUpdate {
            done: 1,
            total: 2,
            message: "a".to_owned(),
        })
        .unwrap();
        ui.progress(&ProgressUpdate {
            done: 2,
            total: 2,
            message: "b".to_owned(),
        })
        .unwrap();
        assert_eq!(text(&ui), "\r\x1b[K[1 / 2] a\r\x1b[K[2 / 2] b");
    }

    #[test]
    fn a_plain_line_after_curses_progress_starts_fresh() {
        let flags = ConsoleFlags {
            color: TriState::No,
            curses: TriState::Yes,
            ..ConsoleFlags::default()
        };
        let mut ui = ConsoleUi::new(Vec::new(), &flags, true).unwrap();
        ui.progress(&ProgressUpdate {
            done: 1,
            total: 2,
            message: "a".to_owned(),
        })
        .unwrap();
        ui.line("done").unwrap();
        assert_eq!(text(&ui), "\r\x1b[K[1 / 2] a\ndone\n");
    }

    #[test]
    fn invalid_ui_event_filters_flag_is_rejected() {
        let flags = ConsoleFlags {
            ui_event_filters: vec!["NOTAKIND".to_owned()],
            ..ConsoleFlags::default()
        };
        assert!(ConsoleUi::new(Vec::new(), &flags, false).is_err());
    }

    #[test]
    fn default_filters_survive_into_the_resolved_config() {
        let ui = ConsoleUi::new(Vec::new(), &ConsoleFlags::default(), false).unwrap();
        assert_eq!(ui.config().filters, UiEventFilters::default_set());
    }
}
