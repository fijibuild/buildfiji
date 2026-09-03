fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Vendored protoc so `cargo build` doesn't depend on a system install.
    // The Bazel build instead points rust_prost_toolchain at
    // @protobuf//:protoc (see MODULE.bazel); both paths must be fed the
    // same .proto sources under proto/.
    unsafe {
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/fjfj/v1/command.proto"], &["proto"])?;

    Ok(())
}
