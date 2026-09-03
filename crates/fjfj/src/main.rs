//! `fjfj` binary entry point. All logic lives in `fjfj-cli` so it can be
//! reused from tests and from a future `fjfj server` mode.

fn main() -> std::process::ExitCode {
    fjfj_cli::main()
}
