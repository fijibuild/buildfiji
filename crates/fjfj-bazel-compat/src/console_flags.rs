//! `--color`, `--curses`, `--show_progress`, `--ui_event_filters`: the
//! console UI flags. Pulled out of a command's raw argv slice the same
//! way as [`crate::diagnostics_flags::extract`].
//!
//! This module only produces raw values, same split as
//! `workspace_status_flags`/`workspace_status`: turning them into actual
//! terminal behavior (auto-detecting a tty, deciding what a progress line
//! looks like) is [`crate::console`]'s job, kept separate so that logic
//! stays pure and testable without a real terminal.

use crate::flag_registry::FlagRegistry;

/// Flag names this module reads, for `clap_flags::validate`'s
/// unimplemented-flag gate.
pub const IMPLEMENTED: &[&str] = &["color", "curses", "show_progress", "ui_event_filters"];

/// `--color`/`--curses`'s three-way value: Bazel's `UseColor`/`UseCurses`
/// converters both parse to this same `YES`/`NO`/`AUTO` shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TriState {
    Yes,
    No,
    #[default]
    Auto,
}

impl TriState {
    fn parse(value: &str) -> Option<TriState> {
        match value.to_ascii_lowercase().as_str() {
            "yes" => Some(TriState::Yes),
            "no" => Some(TriState::No),
            "auto" => Some(TriState::Auto),
            _ => None,
        }
    }
}

/// Console UI flag values, defaulting to Bazel's own: color and curses
/// auto-detected, progress shown, no `--ui_event_filters` override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleFlags {
    pub color: TriState,
    pub curses: TriState,
    /// `--show_progress`/`--noshow_progress`.
    pub show_progress: bool,
    /// `--ui_event_filters`, repeatable — each occurrence is one
    /// comma-separated entry, e.g. `+DEBUG` or `INFO,ERROR`. Left as raw
    /// strings; parsing the leading-+/- grammar is `console`'s job.
    pub ui_event_filters: Vec<String>,
}

impl Default for ConsoleFlags {
    fn default() -> Self {
        ConsoleFlags {
            color: TriState::Auto,
            curses: TriState::Auto,
            show_progress: true,
            ui_event_filters: Vec::new(),
        }
    }
}

/// Pull [`ConsoleFlags`] for `command` out of `args`, returning the flags
/// found and every argument *not* consumed, in their original relative
/// order.
pub fn extract(args: &[String], command: &str) -> (ConsoleFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = ConsoleFlags::default();
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if !arg.starts_with('-') {
            rest.push(arg.clone());
            continue;
        }
        let Ok(m) = registry.resolve(arg, command) else {
            rest.push(arg.clone());
            continue;
        };
        match m.flag.name {
            "color" => match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => match TriState::parse(&value) {
                    Some(state) => flags.color = state,
                    None => rest.push(arg.clone()),
                },
                None => rest.push(arg.clone()),
            },
            "curses" => match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => match TriState::parse(&value) {
                    Some(state) => flags.curses = state,
                    None => rest.push(arg.clone()),
                },
                None => rest.push(arg.clone()),
            },
            "show_progress" => flags.show_progress = !m.negated,
            "ui_event_filters" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => flags.ui_event_filters.push(value),
                    None => rest.push(arg.clone()),
                }
            }
            _ => rest.push(arg.clone()),
        }
    }

    (flags, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_are_bazels_defaults() {
        let (flags, rest) = extract(&args(&[]), "build");
        assert_eq!(flags, ConsoleFlags::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn color_and_curses_parse_case_insensitively() {
        let (flags, rest) = extract(&args(&["--color=YES", "--curses=No"]), "build");
        assert_eq!(flags.color, TriState::Yes);
        assert_eq!(flags.curses, TriState::No);
        assert!(rest.is_empty());
    }

    #[test]
    fn bad_tristate_value_is_left_unconsumed() {
        let (flags, rest) = extract(&args(&["--color=maybe"]), "build");
        assert_eq!(flags.color, TriState::Auto);
        assert_eq!(rest, args(&["--color=maybe"]));
    }

    #[test]
    fn show_progress_and_negation() {
        let (flags, _) = extract(&args(&["--noshow_progress"]), "build");
        assert!(!flags.show_progress);
        let (flags, _) = extract(&args(&["--noshow_progress", "--show_progress"]), "build");
        assert!(flags.show_progress);
    }

    #[test]
    fn ui_event_filters_repeatable_and_ordered() {
        let (flags, rest) = extract(
            &args(&["--ui_event_filters=+DEBUG", "--ui_event_filters=-INFO"]),
            "build",
        );
        assert_eq!(flags.ui_event_filters, ["+DEBUG", "-INFO"]);
        assert!(rest.is_empty());
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, ConsoleFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }
}
