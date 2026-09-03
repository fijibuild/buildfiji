//! Stateright models of fjfj's concurrency protocols.
//!
//! Each model shares vocabulary with the implementation crate it constrains
//! and is checked exhaustively in `cargo test` / `bazel test`. Crash and
//! SIGKILL are explicit actions, so every property holds at every
//! interleaving of a kill with the protocol.

pub mod publish;
pub mod scheduler;
