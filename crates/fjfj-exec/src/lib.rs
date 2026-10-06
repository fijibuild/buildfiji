//! Execution phase: schedules `fjfj_graph::Action`s across local sandboxes
//! and remote executors, checking the action cache first.

pub mod cache;
pub mod console;
pub mod execroot;
pub mod run;
mod sandbox;
mod slots;
pub mod workspace_status;
