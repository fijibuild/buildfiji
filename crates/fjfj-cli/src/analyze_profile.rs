//! `fjfj analyze-profile <trace>` (buildfiji-k62.12): a summary of a trace
//! file written with `FJFJ_TRACE_FILE`, for a run without a collector.
//!
//! The file holds one `action` span per action and a `step` span for each
//! part of it (see `fjfj_exec::run`): `deps` and `slot` are waits, `key`,
//! `prepare`, `spawn`, `collect`, `record` and `make` are fjfj's own work, and
//! `command` is the program. An action's own time is its span less its wait on
//! dependencies; its `blocker` names the dependency it waited for last, which
//! is how the critical path is followed.

use fjfj_bazel_compat::AnalyzeProfileArgs;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

const STEPS: &[&str] = &[
    "deps", "slot", "key", "prepare", "spawn", "command", "collect", "record", "make",
];
/// Time fjfj spends on an action that is not waiting and not the program.
const OVERHEAD: &[&str] = &["key", "prepare", "spawn", "collect", "record", "make"];

#[derive(Default)]
pub(crate) struct Action {
    mnemonic: String,
    target: String,
    output: String,
    blocker: String,
    cached: bool,
    start: u64,
    dur: u64,
    steps: BTreeMap<String, u64>,
}

impl Action {
    fn step(&self, name: &str) -> u64 {
        self.steps.get(name).copied().unwrap_or(0)
    }
    fn own(&self) -> u64 {
        self.dur.saturating_sub(self.step("deps"))
    }
    fn overhead(&self) -> u64 {
        OVERHEAD.iter().map(|s| self.step(s)).sum()
    }
}

pub(crate) fn parse(text: &str) -> Vec<Action> {
    let mut by_row: HashMap<u64, Action> = HashMap::new();
    let mut steps: Vec<(u64, String, u64)> = Vec::new();
    for line in text.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line.trim().trim_end_matches(',')) else {
            continue;
        };
        let (Some(cat), Some(row)) = (event["cat"].as_str(), event["tid"].as_u64()) else {
            continue;
        };
        let dur = event["dur"].as_u64().unwrap_or(0);
        let arg = |k: &str| event["args"][k].as_str().unwrap_or("").to_owned();
        match cat {
            "action" => {
                by_row.insert(
                    row,
                    Action {
                        mnemonic: arg("mnemonic"),
                        target: arg("target"),
                        output: arg("output"),
                        blocker: arg("blocker"),
                        cached: event["args"]["cached"].as_bool().unwrap_or(false),
                        start: event["ts"].as_u64().unwrap_or(0),
                        dur,
                        ..Action::default()
                    },
                );
            }
            "step" => steps.push((row, event["name"].as_str().unwrap_or("").to_owned(), dur)),
            _ => {}
        }
    }
    for (row, name, dur) in steps {
        if let Some(action) = by_row.get_mut(&row) {
            *action.steps.entry(name).or_default() += dur;
        }
    }
    let mut actions: Vec<Action> = by_row.into_values().collect();
    actions.sort_by_key(|a| a.start);
    actions
}

fn secs(micros: u64) -> String {
    format!("{:.1}s", micros as f64 / 1e6)
}

fn millis(micros: u64) -> String {
    format!("{:.0}ms", micros as f64 / 1e3)
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    sorted
        .get(((sorted.len() as f64 * p) as usize).min(sorted.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0)
}

pub(crate) fn report(actions: &[Action], top: usize) -> String {
    let mut out = String::new();
    if actions.is_empty() {
        return "no actions in the trace\n".to_owned();
    }
    let end = actions.iter().map(|a| a.start + a.dur).max().unwrap_or(0);
    let begin = actions.iter().map(|a| a.start).min().unwrap_or(0);
    let wall = end - begin;
    let ran: Vec<&Action> = actions.iter().filter(|a| !a.cached).collect();
    let _ = writeln!(
        out,
        "{} actions: {} ran, {} up to date, over {}",
        actions.len(),
        ran.len(),
        actions.len() - ran.len(),
        secs(wall)
    );

    let _ = writeln!(out, "\nTime in each step of the actions that ran");
    let _ = writeln!(
        out,
        "  {:<9} {:>8} {:>9} {:>9} {:>9} {:>9}  per second of build",
        "step", "total", "mean", "p95", "max", "count"
    );
    for step in STEPS {
        let mut times: Vec<u64> = ran
            .iter()
            .map(|a| a.step(step))
            .filter(|&t| t > 0)
            .collect();
        if times.is_empty() {
            continue;
        }
        times.sort_unstable();
        let total: u64 = times.iter().sum();
        let _ = writeln!(
            out,
            "  {:<9} {:>8} {:>9} {:>9} {:>9} {:>9}  {:.1} actions",
            step,
            secs(total),
            millis(total / times.len() as u64),
            millis(percentile(&times, 0.95)),
            millis(*times.last().unwrap()),
            times.len(),
            total as f64 / wall.max(1) as f64
        );
    }
    let command: u64 = ran.iter().map(|a| a.step("command")).sum();
    let overhead: u64 = ran.iter().map(|a| a.overhead()).sum();
    let _ = writeln!(
        out,
        "  fjfj's own work is {:.0}% of the time a command and its setup take",
        100.0 * overhead as f64 / (command + overhead).max(1) as f64
    );

    // How many commands ran at once, over the build.
    let mut edges: Vec<(u64, i64)> = Vec::new();
    for a in &ran {
        if let Some(c) = a.steps.get("command") {
            // The command step ends where the action's collect starts; its
            // start is not recorded apart from the action's, so place it by
            // the work before it.
            let before = a.step("deps")
                + a.step("slot")
                + a.step("key")
                + a.step("prepare")
                + a.step("spawn");
            edges.push((a.start + before, 1));
            edges.push((a.start + before + c, -1));
        }
    }
    edges.sort_unstable();
    let (mut level, mut last) = (0i64, begin);
    let mut at_level: BTreeMap<i64, u64> = BTreeMap::new();
    for (t, d) in edges {
        *at_level.entry(level).or_default() += t.saturating_sub(last);
        level += d;
        last = t;
    }
    let _ = writeln!(out, "\nCommands running at once, share of the build");
    let mut buckets: BTreeMap<&str, u64> = BTreeMap::new();
    for (level, t) in at_level {
        let b = match level {
            0 => "0",
            1..=3 => "1-3",
            4..=7 => "4-7",
            8..=15 => "8-15",
            _ => "16+",
        };
        *buckets.entry(b).or_default() += t;
    }
    for b in ["0", "1-3", "4-7", "8-15", "16+"] {
        if let Some(t) = buckets.get(b) {
            let _ = writeln!(
                out,
                "  {:>5}  {:>5.1}%  {}",
                b,
                100.0 * *t as f64 / wall.max(1) as f64,
                secs(*t)
            );
        }
    }

    let _ = writeln!(out, "\nBy mnemonic (actions that ran)");
    let mut by: BTreeMap<&str, (usize, u64, u64, u64, u64)> = BTreeMap::new();
    for a in &ran {
        let e = by.entry(&a.mnemonic).or_default();
        e.0 += 1;
        e.1 += a.step("command");
        e.2 += a.overhead();
        e.3 += a.step("slot");
        e.4 = e.4.max(a.own());
    }
    let mut rows: Vec<_> = by.into_iter().collect();
    rows.sort_by_key(|(_, e)| std::cmp::Reverse(e.1 + e.2));
    let _ = writeln!(
        out,
        "  {:<22} {:>6} {:>9} {:>9} {:>9} {:>9}",
        "mnemonic", "count", "command", "overhead", "slot", "longest"
    );
    for (name, e) in rows.iter().take(top) {
        let _ = writeln!(
            out,
            "  {:<22} {:>6} {:>9} {:>9} {:>9} {:>9}",
            name,
            e.0,
            secs(e.1),
            secs(e.2),
            secs(e.3),
            secs(e.4)
        );
    }

    // The critical path: from the last action to finish, back through the
    // dependency each one waited for last.
    let by_output: HashMap<&str, &Action> = actions
        .iter()
        .filter(|a| !a.output.is_empty())
        .map(|a| (a.output.as_str(), a))
        .collect();
    let mut chain: Vec<&Action> = Vec::new();
    let mut at = actions.iter().max_by_key(|a| a.start + a.dur);
    while let Some(a) = at {
        chain.push(a);
        at = by_output.get(a.blocker.as_str()).copied();
        if chain.len() > actions.len() {
            break;
        }
    }
    chain.reverse();
    let _ = writeln!(
        out,
        "\nCritical path: {} actions, {} of the {} build",
        chain.len(),
        secs(chain.iter().map(|a| a.own()).sum()),
        secs(wall)
    );
    for step in STEPS.iter().filter(|s| **s != "deps") {
        let t: u64 = chain.iter().map(|a| a.step(step)).sum();
        if t > 0 {
            let _ = writeln!(out, "  {:<9} {:>8}", step, secs(t));
        }
    }
    let mut links = chain.clone();
    links.sort_by_key(|a| std::cmp::Reverse(a.own()));
    let _ = writeln!(out, "  longest links:");
    for a in links.iter().take(top) {
        let _ = writeln!(
            out,
            "    {:>8}  {:<14} {}  (command {}, slot {}, overhead {})",
            secs(a.own()),
            a.mnemonic,
            a.target,
            secs(a.step("command")),
            secs(a.step("slot")),
            secs(a.overhead())
        );
    }

    let mut slow: Vec<&Action> = ran.clone();
    slow.sort_by_key(|a| std::cmp::Reverse(a.step("command")));
    let _ = writeln!(out, "\nSlowest commands");
    for a in slow.iter().take(top) {
        let _ = writeln!(
            out,
            "  {:>8}  {:<14} {}",
            secs(a.step("command")),
            a.mnemonic,
            a.target
        );
    }
    out
}

pub(crate) fn run(args: &AnalyzeProfileArgs) -> std::io::Result<()> {
    let text = std::fs::read_to_string(&args.path)?;
    print!("{}", report(&parse(&text), args.top));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(cat: &str, name: &str, tid: u64, ts: u64, dur: u64, args: &str) -> String {
        format!(
            r#"{{"name":"{name}","cat":"{cat}","ph":"X","ts":{ts},"dur":{dur},"pid":1,"tid":{tid},"args":{args}}},"#
        ) + "\n"
    }

    #[test]
    fn the_critical_path_follows_the_last_dependency_each_action_waited_for() {
        let mut text = String::from("[\n");
        // a (row 1) then b (row 2, waits for a) then c (row 3, waits for b);
        // d (row 4) is quicker and nothing waits for it.
        for (row, out, blocker, ts, dur, deps, cmd) in [
            (1, "a", "", 0, 1_000_000, 0, 900_000),
            (2, "b", "a", 0, 3_000_000, 1_000_000, 1_900_000),
            (3, "c", "b", 0, 6_000_000, 3_000_000, 2_900_000),
            (4, "d", "", 0, 500_000, 0, 400_000),
        ] {
            let args = format!(
                r#"{{"mnemonic":"M","target":"//{out}","output":"{out}","blocker":"{blocker}","cached":false}}"#
            );
            text += &event("action", "action", row, ts, dur, &args);
            text += &event("step", "deps", row, ts, deps, r#"{"step":"deps"}"#);
            text += &event("step", "command", row, ts, cmd, r#"{"step":"command"}"#);
        }
        text += "{}\n]\n";
        let report = report(&parse(&text), 5);
        assert!(report.contains("4 actions: 4 ran"), "{report}");
        assert!(
            report.contains("Critical path: 3 actions, 6.0s of the 6.0s build"),
            "{report}"
        );
    }

    #[test]
    fn a_file_cut_short_still_reads() {
        let text = event("action", "action", 1, 0, 10, r#"{"output":"a"}"#);
        assert_eq!(parse(&text).len(), 1);
    }
}
