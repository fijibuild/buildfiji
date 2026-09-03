//! Refresh `src/bazel_flags/generated.rs` from `bazel help flags-as-proto`
//! for whatever `bazel` is on `PATH` (normally the version pinned in
//! `.bazelversion`, via bazelisk). Run with:
//! `cargo run -p fjfj-bazel-compat --bin refresh_bazel_flags`.
//!
//! `bazel help flags-as-proto` prints a base64-encoded, serialized
//! `bazel_flags.FlagCollection` message (see `../../proto/bazel_flags.proto`,
//! vendored from bazel.build/bazelbuild/bazel for reference). Decoding it
//! properly would mean either shelling out to `protoc` or pulling in
//! `prost-build`'s protoc dependency — heavyweight machinery for a script
//! that runs rarely (only when bumping the pinned Bazel version) against
//! one small, stable, all-scalar/repeated-string proto2 message. So this
//! is a minimal hand-rolled protobuf wire-format reader scoped to exactly
//! that message; see `read_flag_info` for the field-number mapping, which
//! must be kept in sync with the vendored `.proto` by hand.

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bazel_version = bazel_version()?;
    let encoded = run_bazel(&["help", "flags-as-proto"])?;
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded.trim())?;

    let mut flags = decode_flag_collection(&bytes)?;
    flags.sort_by(|a, b| a.name.cmp(&b.name));

    let out_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/bazel_flags/generated.rs");
    let mut out = std::fs::File::create(out_path)?;
    write_generated_file(&mut out, &bazel_version, &flags)?;
    println!(
        "wrote {} flags from bazel {bazel_version} to {out_path}",
        flags.len()
    );
    Ok(())
}

fn bazel_version() -> Result<String, Box<dyn std::error::Error>> {
    let output = run_bazel(&["--version"])?;
    // "bazel 9.2.0\n"
    Ok(output
        .trim()
        .strip_prefix("bazel ")
        .unwrap_or(output.trim())
        .to_string())
}

fn run_bazel(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("bazel").args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "`bazel {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

#[derive(Debug, Default)]
struct OwnedFlagInfo {
    name: String,
    has_negative_flag: bool,
    documentation: String,
    commands: Vec<String>,
    abbreviation: Option<String>,
    allows_multiple: bool,
    effect_tags: Vec<String>,
    metadata_tags: Vec<String>,
    documentation_category: String,
    requires_value: bool,
    default_value: Option<String>,
    old_name: Option<String>,
    deprecation_warning: Option<String>,
    option_expansions: Vec<String>,
    type_converter: Option<String>,
    enum_values: Vec<String>,
}

/// A cursor over a proto2 wire-format byte slice.
struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }

    fn eof(&self) -> bool {
        self.pos >= self.buf.len()
    }

    fn read_varint(&mut self) -> u64 {
        let mut result = 0u64;
        let mut shift = 0;
        loop {
            let b = self.buf[self.pos];
            self.pos += 1;
            result |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        result
    }

    fn read_bytes(&mut self, len: usize) -> &'a [u8] {
        let s = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        s
    }

    fn read_len_delimited(&mut self) -> &'a [u8] {
        let len = self.read_varint() as usize;
        self.read_bytes(len)
    }

    fn read_string(&mut self) -> String {
        String::from_utf8_lossy(self.read_len_delimited()).into_owned()
    }

    /// (field number, wire type) per the proto2 wire format.
    fn read_tag(&mut self) -> (u32, u8) {
        let tag = self.read_varint();
        ((tag >> 3) as u32, (tag & 0x7) as u8)
    }

    fn skip(&mut self, wire_type: u8) {
        match wire_type {
            0 => {
                self.read_varint();
            }
            1 => {
                self.read_bytes(8);
            }
            2 => {
                self.read_len_delimited();
            }
            5 => {
                self.read_bytes(4);
            }
            other => panic!("unsupported wire type {other}"),
        }
    }
}

fn decode_flag_collection(bytes: &[u8]) -> Result<Vec<OwnedFlagInfo>, Box<dyn std::error::Error>> {
    let mut r = Reader::new(bytes);
    let mut flags = Vec::new();
    while !r.eof() {
        let (field, wire) = r.read_tag();
        match field {
            1 if wire == 2 => flags.push(read_flag_info(r.read_len_delimited())),
            _ => r.skip(wire),
        }
    }
    Ok(flags)
}

/// Field numbers here must match `proto/bazel_flags.proto`'s `FlagInfo`.
fn read_flag_info(bytes: &[u8]) -> OwnedFlagInfo {
    let mut r = Reader::new(bytes);
    let mut info = OwnedFlagInfo::default();
    while !r.eof() {
        let (field, wire) = r.read_tag();
        match field {
            1 => info.name = r.read_string(),
            2 => info.has_negative_flag = r.read_varint() != 0,
            3 => info.documentation = r.read_string(),
            4 => info.commands.push(r.read_string()),
            5 => info.abbreviation = Some(r.read_string()),
            6 => info.allows_multiple = r.read_varint() != 0,
            7 => info.effect_tags.push(r.read_string()),
            8 => info.metadata_tags.push(r.read_string()),
            9 => info.documentation_category = r.read_string(),
            10 => info.requires_value = r.read_varint() != 0,
            11 => info.old_name = Some(r.read_string()),
            12 => info.deprecation_warning = Some(r.read_string()),
            13 => info.default_value = Some(r.read_string()),
            14 => info.option_expansions.push(r.read_string()),
            15 => info.type_converter = Some(r.read_string()),
            16 => info.enum_values.push(r.read_string()),
            _ => r.skip(wire),
        }
    }
    info
}

fn write_generated_file(
    out: &mut impl std::io::Write,
    bazel_version: &str,
    flags: &[OwnedFlagInfo],
) -> std::io::Result<()> {
    writeln!(
        out,
        "//! @generated by `refresh_bazel_flags` — do not edit by hand."
    )?;
    writeln!(
        out,
        "//! Run `cargo run -p fjfj-bazel-compat --bin refresh_bazel_flags` to refresh."
    )?;
    writeln!(out)?;
    writeln!(out, "use super::FlagInfo;")?;
    writeln!(out)?;
    writeln!(out, "pub const BAZEL_VERSION: &str = {bazel_version:?};")?;
    writeln!(out)?;
    writeln!(out, "pub static FLAGS: &[FlagInfo] = &[")?;
    for f in flags {
        write!(out, "    FlagInfo {{ name: {:?}, ", f.name)?;
        write!(out, "has_negative_flag: {:?}, ", f.has_negative_flag)?;
        write!(out, "documentation: {:?}, ", f.documentation)?;
        write!(out, "commands: &{:?}, ", f.commands)?;
        write!(out, "abbreviation: {:?}, ", f.abbreviation)?;
        write!(out, "allows_multiple: {:?}, ", f.allows_multiple)?;
        write!(out, "effect_tags: &{:?}, ", f.effect_tags)?;
        write!(out, "metadata_tags: &{:?}, ", f.metadata_tags)?;
        write!(
            out,
            "documentation_category: {:?}, ",
            f.documentation_category
        )?;
        write!(out, "requires_value: {:?}, ", f.requires_value)?;
        write!(out, "default_value: {:?}, ", f.default_value)?;
        write!(out, "old_name: {:?}, ", f.old_name)?;
        write!(out, "deprecation_warning: {:?}, ", f.deprecation_warning)?;
        write!(out, "option_expansions: &{:?}, ", f.option_expansions)?;
        write!(out, "type_converter: {:?}, ", f.type_converter)?;
        writeln!(out, "enum_values: &{:?} }},", f.enum_values)?;
    }
    writeln!(out, "];")?;
    out.flush()
}
