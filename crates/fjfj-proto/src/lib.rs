//! Generated client/daemon command service types (see
//! `proto/fjfj/v1/command.proto`) plus the tonic client and server stubs.
//!
//! Code is generated at build time from the same `.proto` sources under
//! both Cargo (`build.rs`, via `tonic-build` + vendored `protoc`) and
//! Bazel (`rust_prost_library`, via `rust_prost_toolchain`) so the two
//! build graphs stay in sync by construction rather than by convention.
//!
//! Types live under [`fjfj::v1`] (i.e. `fjfj_proto::fjfj::v1::RunCommandRequest`,
//! not a flat `fjfj_proto::RunCommandRequest`) on purpose: `rust_prost_library`
//! nests its generated module by proto package and doesn't offer a flat
//! mode, so this crate nests its own `include!` to match rather than
//! leaving Cargo and Bazel callers with two different paths for the same
//! type (buildfiji-23d.23). `tests/smoke.rs` runs unmodified under both
//! `cargo test` and `bazel test` as a result. The two build graphs still
//! pin different prost/tonic runtime versions (`rules_rust_prost` vendors
//! its own); that's fine as long as nothing in one build graph mixes
//! generated types from both.

#![allow(clippy::all)]

pub mod fjfj {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/fjfj.v1.rs"));
    }
}

#[cfg(test)]
mod tests {
    use super::fjfj::v1::{InfoResponse, PingResponse, command_service_client};

    // Exercises that the client stub actually builds and links against
    // this crate's own prost/tonic versions; a version skew between
    // fjfj-proto's runtime deps and a caller's would fail here first.
    // Cargo-only (see `tests/smoke.rs`'s doc comment for why): the
    // generated client type is generic over a tonic `Channel`, and the
    // Bazel build graph pins a different tonic version.
    #[test]
    fn client_type_names_are_reachable() {
        fn _assert_type<T>() {}
        _assert_type::<command_service_client::CommandServiceClient<tonic::transport::Channel>>();
        _assert_type::<PingResponse>();
        _assert_type::<InfoResponse>();
    }
}
