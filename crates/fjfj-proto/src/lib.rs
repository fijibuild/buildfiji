//! Generated client/daemon command service types (see
//! `proto/fjfj/v1/command.proto`) plus the tonic client and server stubs.
//!
//! Code is generated at build time from the same `.proto` sources under
//! both Cargo (`build.rs`, via `tonic-build` + vendored `protoc`) and
//! Bazel (`rust_prost_library`, via `rust_prost_toolchain`) so the two
//! build graphs stay in sync by construction rather than by convention.

#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/fjfj.v1.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    // Exercises that the generated request/response types and the client
    // stub actually build and link against the crate's prost/tonic
    // versions; a version skew between fjfj-proto's runtime deps and a
    // caller's would fail here first.
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

    #[test]
    fn client_type_names_are_reachable() {
        fn _assert_type<T>() {}
        _assert_type::<command_service_client::CommandServiceClient<tonic::transport::Channel>>();
        _assert_type::<PingResponse>();
        _assert_type::<InfoResponse>();
    }
}
