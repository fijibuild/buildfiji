//! Smoke test for the Bazel-generated bindings (rust_prost_library), run
//! only under `bazel test`. The Cargo-generated bindings (build.rs +
//! tonic-prost-build) have the equivalent test in src/lib.rs; the two
//! codegen paths nest their modules differently (see BUILD.bazel), so the
//! two tests can't share source. `cfg(bazel)` (set by this crate's
//! `rust_test` via `rustc_flags`, absent under `cargo test`) keeps this
//! file a no-op under Cargo instead of failing to find `fjfj_proto::fjfj`.
#![cfg(bazel)]

use fjfj_proto::fjfj::v1::{CommandEvent, CommandResult, RunCommandRequest, command_event};

#[test]
fn generated_types_round_trip_defaults() {
    let request = RunCommandRequest {
        invocation_id: "abc123".to_string(),
        args: vec!["build".to_string(), "//...".to_string()],
        working_directory: "/repo".to_string(),
        env: Default::default(),
    };
    assert_eq!(request.args.len(), 2);

    let event = CommandEvent {
        event: Some(command_event::Event::Result(CommandResult { exit_code: 0 })),
    };
    match event.event {
        Some(command_event::Event::Result(result)) => assert_eq!(result.exit_code, 0),
        _ => panic!("expected a CommandResult event"),
    }
}
