//! Smoke test for the generated bindings. Runs unchanged under `cargo
//! test` (against `build.rs` + `tonic-prost-build`'s output) and `bazel
//! test` (against `rust_prost_library`'s output) — both nest under
//! `fjfj::v1` the same way (see `src/lib.rs`), so one test source covers
//! both codegen paths instead of needing a `cfg(bazel)`-gated duplicate.
//!
//! Plain data types only (no client/server stub types): those are
//! generic over a tonic `Channel`, and the two build graphs pin different
//! tonic versions (`rules_rust_prost` vendors its own) — a
//! `CommandServiceClient<Channel>` built one way isn't the same type as
//! one built the other, so checking that reachability is Cargo-only (see
//! `src/lib.rs`'s own tests).

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
