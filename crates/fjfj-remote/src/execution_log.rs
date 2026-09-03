//! `--execution_log_compact_file`: a zstd-compressed stream of
//! length-delimited `ExecLogEntry` protos (Bazel's
//! `src/main/protobuf/spawn.proto`), one entry per executed spawn or
//! referenced input, kept for post-hoc cache-miss debugging (`bazel-differ`,
//! ad hoc `zstdcat | protoc --decode` inspection, or a future fjfj
//! equivalent). The wire types here are hand-transcribed from that proto
//! (same approach as [`crate::action_key::CacheSalt`]) rather than generated
//! from a vendored `.proto` file, since the message set is small and stable
//! and this crate has no protoc/prost-build step yet.
//!
//! Entries reference each other by a caller-assigned `u32` id (e.g. a
//! `Spawn`'s `input_set_id` points at an earlier `InputSet` entry); Bazel
//! requires that every entry be written only after everything it references,
//! but does not require increasing id order. This module only encodes what
//! it is given — assigning ids and honoring that ordering is the caller's
//! job, once there is a real action graph to walk.

use std::io::{self, Write};

use prost::Message;

/// Digest of a file or action cache entry (`spawn.proto`'s `Digest`).
#[derive(Clone, PartialEq, Message)]
pub struct Digest {
    #[prost(string, tag = "1")]
    pub hash: String,
    #[prost(int64, tag = "2")]
    pub size_bytes: i64,
    #[prost(string, tag = "3")]
    pub hash_function_name: String,
}

/// `ExecLogEntry.Invocation`: metadata for the whole log, written once in
/// the initial position so every other entry can omit it (e.g. the hash
/// function name is not repeated per-file).
#[derive(Clone, PartialEq, Message)]
pub struct Invocation {
    #[prost(string, tag = "1")]
    pub hash_function_name: String,
    #[prost(string, tag = "2")]
    pub workspace_runfiles_directory: String,
    #[prost(bool, tag = "3")]
    pub sibling_repository_layout: bool,
    #[prost(string, tag = "4")]
    pub id: String,
}

/// `ExecLogEntry.File`: an input or output file. The hash function name is
/// omitted (see [`Invocation`]); digest may be omitted for empty files.
#[derive(Clone, PartialEq, Message)]
pub struct File {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(message, optional, tag = "2")]
    pub digest: Option<Digest>,
}

/// `ExecLogEntry.Directory`: a source directory, fileset tree, or tree
/// artifact, as its contained files (paths relative to the directory).
#[derive(Clone, PartialEq, Message)]
pub struct Directory {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(message, repeated, tag = "2")]
    pub files: Vec<File>,
}

/// `ExecLogEntry.UnresolvedSymlink`.
#[derive(Clone, PartialEq, Message)]
pub struct UnresolvedSymlink {
    #[prost(string, tag = "1")]
    pub path: String,
    #[prost(string, tag = "2")]
    pub target_path: String,
}

/// `ExecLogEntry.InputSet`: entries are the union of `input_ids` (files,
/// directories, unresolved symlinks or runfiles trees) and everything in
/// `transitive_set_ids`. Not canonical — the same contents may be encoded by
/// different set structures.
#[derive(Clone, PartialEq, Message)]
pub struct InputSet {
    #[prost(uint32, repeated, tag = "4")]
    pub transitive_set_ids: Vec<u32>,
    #[prost(uint32, repeated, tag = "5")]
    pub input_ids: Vec<u32>,
}

/// `ExecLogEntry.Output`: either a declared output that was produced
/// (`output_id`, pointing at a `File`/`Directory`/`UnresolvedSymlink` entry)
/// or one that is missing / has the wrong type (`invalid_output_path`).
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum OutputType {
    #[prost(string, tag = "4")]
    InvalidOutputPath(String),
    #[prost(uint32, tag = "5")]
    OutputId(u32),
}

#[derive(Clone, PartialEq, Message)]
pub struct Output {
    #[prost(oneof = "OutputType", tags = "4, 5")]
    pub r#type: Option<OutputType>,
}

/// `ExecLogEntry.Spawn`: an executed spawn, the compact analogue of
/// `SpawnExec`. Field meanings mirror `SpawnExec`'s (see that message's
/// doc comments in `spawn.proto`) with inputs/tools/outputs replaced by
/// entry-id references so shared input sets aren't repeated per spawn.
#[derive(Clone, PartialEq, Message)]
pub struct Spawn {
    #[prost(string, repeated, tag = "1")]
    pub args: Vec<String>,
    #[prost(message, repeated, tag = "2")]
    pub env_vars: Vec<EnvironmentVariable>,
    #[prost(message, optional, tag = "3")]
    pub platform: Option<Platform>,
    #[prost(uint32, tag = "4")]
    pub input_set_id: u32,
    #[prost(uint32, tag = "5")]
    pub tool_set_id: u32,
    #[prost(message, repeated, tag = "6")]
    pub outputs: Vec<Output>,
    #[prost(string, tag = "7")]
    pub target_label: String,
    #[prost(string, tag = "8")]
    pub mnemonic: String,
    #[prost(int32, tag = "9")]
    pub exit_code: i32,
    #[prost(string, tag = "10")]
    pub status: String,
    #[prost(string, tag = "11")]
    pub runner: String,
    #[prost(bool, tag = "12")]
    pub cache_hit: bool,
    #[prost(bool, tag = "13")]
    pub remotable: bool,
    #[prost(bool, tag = "14")]
    pub cacheable: bool,
    #[prost(bool, tag = "15")]
    pub remote_cacheable: bool,
    #[prost(message, optional, tag = "16")]
    pub digest: Option<Digest>,
    #[prost(int64, tag = "17")]
    pub timeout_millis: i64,
}

/// `spawn.proto`'s `EnvironmentVariable`.
#[derive(Clone, PartialEq, Message)]
pub struct EnvironmentVariable {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub value: String,
}

/// `spawn.proto`'s `Platform`.
#[derive(Clone, PartialEq, Message)]
pub struct Platform {
    #[prost(message, repeated, tag = "1")]
    pub properties: Vec<PlatformProperty>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PlatformProperty {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub value: String,
}

/// `ExecLogEntry.SymlinkAction`: a symlink created directly by the action
/// graph, with no backing spawn to log.
#[derive(Clone, PartialEq, Message)]
pub struct SymlinkAction {
    #[prost(string, tag = "1")]
    pub input_path: String,
    #[prost(string, tag = "2")]
    pub output_path: String,
    #[prost(string, tag = "3")]
    pub target_label: String,
    #[prost(string, tag = "4")]
    pub mnemonic: String,
}

/// The payload of one [`ExecLogEntry`] — Bazel's `ExecLogEntry.type` oneof.
/// Only `Invocation`, `File`, `InputSet` and `Spawn` are transcribed for
/// now; `Directory`, `UnresolvedSymlink`, `SymlinkAction`,
/// `SymlinkEntrySet` and `RunfilesTree` (runfiles-tree reconstruction) are
/// left for whichever bead first needs to log a runfiles-bearing spawn.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum EntryType {
    #[prost(message, tag = "2")]
    Invocation(Invocation),
    #[prost(message, tag = "3")]
    File(File),
    #[prost(message, tag = "6")]
    InputSet(InputSet),
    #[prost(message, tag = "7")]
    Spawn(Spawn),
}

/// One `ExecLogEntry`: an optional id (nonzero to be referenced by later
/// entries) plus its payload.
#[derive(Clone, PartialEq, Message)]
pub struct ExecLogEntry {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(oneof = "EntryType", tags = "2, 3, 6, 7")]
    pub r#type: Option<EntryType>,
}

/// Writes `ExecLogEntry` messages to `--execution_log_compact_file`'s
/// format: length-delimited protos, the whole stream zstd-compressed, one
/// continuous frame (matching Bazel's `CompactSpawnLogContext`, minus its
/// off-thread compression — fjfj has no spawns to log yet, so there is
/// nothing to keep off a hot path).
pub struct CompactExecutionLogWriter<W: Write> {
    encoder: zstd::Encoder<'static, W>,
}

impl<W: Write> CompactExecutionLogWriter<W> {
    /// Opens a writer over `sink`, at zstd's default compression level.
    pub fn new(sink: W) -> io::Result<Self> {
        Ok(CompactExecutionLogWriter {
            encoder: zstd::Encoder::new(sink, 0)?,
        })
    }

    /// Appends one entry: its length as a varint, then the serialized proto.
    pub fn write_entry(&mut self, entry: &ExecLogEntry) -> io::Result<()> {
        let mut buf = Vec::with_capacity(entry.encoded_len());
        entry
            .encode_length_delimited(&mut buf)
            .expect("Vec<u8> writes are infallible");
        self.encoder.write_all(&buf)
    }

    /// Flushes the zstd frame and returns the underlying sink.
    pub fn finish(self) -> io::Result<W> {
        self.encoder.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_back(compressed: &[u8]) -> Vec<ExecLogEntry> {
        let decompressed = zstd::decode_all(compressed).unwrap();
        let mut cursor = decompressed.as_slice();
        let mut entries = Vec::new();
        while !cursor.is_empty() {
            entries.push(ExecLogEntry::decode_length_delimited(&mut cursor).unwrap());
        }
        entries
    }

    #[test]
    fn round_trips_an_invocation_and_a_spawn() {
        let mut writer = CompactExecutionLogWriter::new(Vec::new()).unwrap();
        let invocation = ExecLogEntry {
            id: 0,
            r#type: Some(EntryType::Invocation(Invocation {
                hash_function_name: "SHA-256".into(),
                workspace_runfiles_directory: "_main".into(),
                sibling_repository_layout: true,
                id: "abc-123".into(),
            })),
        };
        let spawn = ExecLogEntry {
            id: 1,
            r#type: Some(EntryType::Spawn(Spawn {
                args: vec!["/bin/echo".into(), "hi".into()],
                mnemonic: "Genrule".into(),
                exit_code: 0,
                ..Default::default()
            })),
        };
        writer.write_entry(&invocation).unwrap();
        writer.write_entry(&spawn).unwrap();
        let compressed = writer.finish().unwrap();

        // Really is zstd: a plain concatenation of the two messages
        // wouldn't start with zstd's magic number.
        assert_eq!(&compressed[0..4], &0xFD2FB528u32.to_le_bytes());

        let entries = read_back(&compressed);
        assert_eq!(entries, vec![invocation, spawn]);
    }

    #[test]
    fn empty_log_is_still_a_valid_zstd_frame() {
        let writer = CompactExecutionLogWriter::new(Vec::new()).unwrap();
        let compressed = writer.finish().unwrap();
        assert!(read_back(&compressed).is_empty());
    }

    #[test]
    fn output_oneof_round_trips_both_variants() {
        let by_id = Output {
            r#type: Some(OutputType::OutputId(7)),
        };
        let invalid = Output {
            r#type: Some(OutputType::InvalidOutputPath("bazel-out/missing".into())),
        };
        for output in [by_id, invalid] {
            let mut buf = Vec::new();
            output.encode(&mut buf).unwrap();
            assert_eq!(Output::decode(buf.as_slice()).unwrap(), output);
        }
    }
}
