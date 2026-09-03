//! Execution phase: schedules `fjfj_graph::Action`s across local sandboxes
//! and remote executors, checking the action cache first.

pub mod workspace_status;

use fjfj_graph::Action;

/// Result of running an action.
#[derive(Debug)]
pub struct ActionResult {
    pub exit_code: i32,
    pub cached: bool,
}

/// Placeholder scheduler. Real implementation: a work-stealing executor with
/// per-strategy concurrency limits (`--jobs`, `--local_cpu_resources`).
pub async fn execute(_action: &Action) -> anyhow::Result<ActionResult> {
    Ok(ActionResult {
        exit_code: 0,
        cached: false,
    })
}
