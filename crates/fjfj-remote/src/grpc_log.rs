//! `--remote_grpc_log=<path>`: a log of every gRPC call made to the remote
//! cache/executor, for diagnosing what actually went over the wire on a
//! given invocation (a stuck build, an unexpected cache miss, a server
//! returning malformed responses).
//!
//! Wire format is Bazel's `remote_execution_log.proto`, hand-transcribed as
//! `prost::Message` structs the same way as [`crate::execution_log`] and
//! [`crate::action_key::CacheSalt`] — reusing the REAPI/bytestream/longrunning
//! types this crate already depends on via `bazel-remote-apis` (`crate::reapi`,
//! [`bazel_remote_apis::google`]) rather than vendoring a second proto for
//! just the handful of `remote_logging`-only messages (`LogEntry`,
//! `RetrySummary`, and the per-method `*Details` wrappers).
//!
//! Unlike [`crate::execution_log`]'s compact format, this log is *not*
//! zstd-compressed: Bazel documents it as a plain sequence of
//! length-delimited `LogEntry` protos (`LogEntry.writeDelimitedTo`), so
//! [`GrpcLogWriter`] only adds the varint length prefix.

use std::io::{self, Write};

use bazel_remote_apis::google;
use prost::Message;

use crate::reapi;

/// One logged gRPC call (`remote_execution_log.proto`'s `LogEntry`).
#[derive(Clone, PartialEq, Message)]
pub struct LogEntry {
    #[prost(message, optional, tag = "1")]
    pub metadata: Option<reapi::RequestMetadata>,
    #[prost(message, optional, tag = "2")]
    pub status: Option<google::rpc::Status>,
    /// `$FULL_SERVICE_NAME/$METHOD_NAME`, e.g.
    /// `build.bazel.remote.execution.v2.ActionCache/GetActionResult`.
    #[prost(string, tag = "3")]
    pub method_name: String,
    #[prost(message, optional, tag = "4")]
    pub details: Option<RpcCallDetails>,
    #[prost(message, optional, tag = "5")]
    pub start_time: Option<google::protobuf::Timestamp>,
    #[prost(message, optional, tag = "6")]
    pub end_time: Option<google::protobuf::Timestamp>,
    /// Groups every attempt (initial call plus retries) of one logical RPC.
    #[prost(string, tag = "7")]
    pub rpc_id: String,
    /// 1-based: the initial attempt is 1, the first retry is 2, and so on.
    #[prost(int32, tag = "8")]
    pub attempt_number: i32,
    /// Only set on the synthetic terminal entry emitted once a logical RPC
    /// fails after exhausting retries.
    #[prost(message, optional, tag = "9")]
    pub retry_summary: Option<RetrySummary>,
}

/// `remote_execution_log.proto`'s `RetrySummary`.
#[derive(Clone, PartialEq, Message)]
pub struct RetrySummary {
    /// Does not count the initial attempt: total attempts = this + 1.
    #[prost(int32, tag = "1")]
    pub retry_attempts: i32,
    #[prost(bool, tag = "2")]
    pub retries_exhausted: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetCapabilitiesDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::GetCapabilitiesRequest>,
    #[prost(message, optional, tag = "2")]
    pub response: Option<reapi::ServerCapabilities>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ExecuteDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::ExecuteRequest>,
    #[prost(message, repeated, tag = "2")]
    pub responses: Vec<google::longrunning::Operation>,
}

#[derive(Clone, PartialEq, Message)]
pub struct GetActionResultDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::GetActionResultRequest>,
    #[prost(message, optional, tag = "2")]
    pub response: Option<reapi::ActionResult>,
}

#[derive(Clone, PartialEq, Message)]
pub struct UpdateActionResultDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::UpdateActionResultRequest>,
    #[prost(message, optional, tag = "2")]
    pub response: Option<reapi::ActionResult>,
}

#[derive(Clone, PartialEq, Message)]
pub struct WaitExecutionDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::WaitExecutionRequest>,
    #[prost(message, repeated, tag = "2")]
    pub responses: Vec<google::longrunning::Operation>,
}

#[derive(Clone, PartialEq, Message)]
pub struct FindMissingBlobsDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::FindMissingBlobsRequest>,
    #[prost(message, optional, tag = "2")]
    pub response: Option<reapi::FindMissingBlobsResponse>,
}

#[derive(Clone, PartialEq, Message)]
pub struct SplitBlobDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::SplitBlobRequest>,
    #[prost(message, optional, tag = "2")]
    pub response: Option<reapi::SplitBlobResponse>,
}

#[derive(Clone, PartialEq, Message)]
pub struct SpliceBlobDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<reapi::SpliceBlobRequest>,
    #[prost(message, optional, tag = "2")]
    pub response: Option<reapi::SpliceBlobResponse>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ReadDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<google::bytestream::ReadRequest>,
    #[prost(int64, tag = "2")]
    pub num_reads: i64,
    #[prost(int64, tag = "3")]
    pub bytes_read: i64,
}

#[derive(Clone, PartialEq, Message)]
pub struct WriteDetails {
    #[prost(string, repeated, tag = "1")]
    pub resource_names: Vec<String>,
    #[prost(int64, tag = "2")]
    pub num_writes: i64,
    #[prost(int64, tag = "3")]
    pub bytes_sent: i64,
    #[prost(message, optional, tag = "4")]
    pub response: Option<google::bytestream::WriteResponse>,
    #[prost(int64, repeated, tag = "5")]
    pub offsets: Vec<i64>,
    #[prost(int64, repeated, tag = "6")]
    pub finish_writes: Vec<i64>,
}

#[derive(Clone, PartialEq, Message)]
pub struct QueryWriteStatusDetails {
    #[prost(message, optional, tag = "1")]
    pub request: Option<google::bytestream::QueryWriteStatusRequest>,
    #[prost(message, optional, tag = "2")]
    pub response: Option<google::bytestream::QueryWriteStatusResponse>,
}

/// The payload of one [`LogEntry`] — `RpcCallDetails.details`. Tags 1-4 and
/// 11 are reserved in Bazel's proto (an earlier, non-oneof layout); the
/// remaining tags are kept exactly as Bazel numbers them so a decoder reading
/// both fjfj's and Bazel's logs sees the same wire representation.
///
/// Every variant is boxed: several of these details messages embed whole
/// REAPI request/response types (e.g. `ActionResult`), so an unboxed oneof
/// would size every `LogEntry` to its single largest variant.
#[derive(Clone, PartialEq, ::prost::Oneof)]
pub enum DetailsType {
    #[prost(message, boxed, tag = "5")]
    Read(Box<ReadDetails>),
    #[prost(message, boxed, tag = "6")]
    Write(Box<WriteDetails>),
    #[prost(message, boxed, tag = "7")]
    Execute(Box<ExecuteDetails>),
    #[prost(message, boxed, tag = "8")]
    GetActionResult(Box<GetActionResultDetails>),
    #[prost(message, boxed, tag = "9")]
    WaitExecution(Box<WaitExecutionDetails>),
    #[prost(message, boxed, tag = "10")]
    FindMissingBlobs(Box<FindMissingBlobsDetails>),
    #[prost(message, boxed, tag = "12")]
    GetCapabilities(Box<GetCapabilitiesDetails>),
    #[prost(message, boxed, tag = "13")]
    UpdateActionResult(Box<UpdateActionResultDetails>),
    #[prost(message, boxed, tag = "14")]
    QueryWriteStatus(Box<QueryWriteStatusDetails>),
    #[prost(message, boxed, tag = "15")]
    SplitBlob(Box<SplitBlobDetails>),
    #[prost(message, boxed, tag = "16")]
    SpliceBlob(Box<SpliceBlobDetails>),
}

#[derive(Clone, PartialEq, Message)]
pub struct RpcCallDetails {
    #[prost(oneof = "DetailsType", tags = "5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16")]
    pub details: Option<DetailsType>,
}

/// Writes [`LogEntry`] messages to `--remote_grpc_log`'s format: each
/// message prefixed by a varint of its encoded length, uncompressed
/// (`LogEntry.writeDelimitedTo`) — no outer framing beyond that.
pub struct GrpcLogWriter<W: Write> {
    sink: W,
}

impl<W: Write> GrpcLogWriter<W> {
    pub fn new(sink: W) -> Self {
        GrpcLogWriter { sink }
    }

    /// Appends one entry: its length as a varint, then the serialized proto.
    pub fn write_entry(&mut self, entry: &LogEntry) -> io::Result<()> {
        let mut buf = Vec::with_capacity(entry.encoded_len());
        entry
            .encode_length_delimited(&mut buf)
            .expect("Vec<u8> writes are infallible");
        self.sink.write_all(&buf)
    }

    /// Flushes and returns the underlying sink.
    pub fn finish(mut self) -> io::Result<W> {
        self.sink.flush()?;
        Ok(self.sink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_back(bytes: &[u8]) -> Vec<LogEntry> {
        let mut cursor = bytes;
        let mut entries = Vec::new();
        while !cursor.is_empty() {
            entries.push(LogEntry::decode_length_delimited(&mut cursor).unwrap());
        }
        entries
    }

    #[test]
    fn round_trips_a_read_and_an_execute_entry() {
        let mut writer = GrpcLogWriter::new(Vec::new());
        let read_entry = LogEntry {
            method_name: "google.bytestream.ByteStream/Read".into(),
            rpc_id: "rpc-1".into(),
            attempt_number: 1,
            details: Some(RpcCallDetails {
                details: Some(DetailsType::Read(Box::new(ReadDetails {
                    num_reads: 3,
                    bytes_read: 4096,
                    ..Default::default()
                }))),
            }),
            ..Default::default()
        };
        let execute_entry = LogEntry {
            method_name: "build.bazel.remote.execution.v2.Execution/Execute".into(),
            rpc_id: "rpc-2".into(),
            attempt_number: 1,
            details: Some(RpcCallDetails {
                details: Some(DetailsType::Execute(Box::default())),
            }),
            retry_summary: None,
            ..Default::default()
        };
        writer.write_entry(&read_entry).unwrap();
        writer.write_entry(&execute_entry).unwrap();
        let bytes = writer.finish().unwrap();

        assert_eq!(read_back(&bytes), vec![read_entry, execute_entry]);
    }

    #[test]
    fn retry_summary_round_trips() {
        let mut writer = GrpcLogWriter::new(Vec::new());
        let entry = LogEntry {
            method_name: "build.bazel.remote.execution.v2.ActionCache/GetActionResult".into(),
            rpc_id: "rpc-3".into(),
            attempt_number: 3,
            retry_summary: Some(RetrySummary {
                retry_attempts: 2,
                retries_exhausted: true,
            }),
            ..Default::default()
        };
        writer.write_entry(&entry).unwrap();
        let bytes = writer.finish().unwrap();
        assert_eq!(read_back(&bytes), vec![entry]);
    }

    #[test]
    fn empty_log_round_trips_to_nothing() {
        let writer = GrpcLogWriter::new(Vec::new());
        let bytes = writer.finish().unwrap();
        assert!(read_back(&bytes).is_empty());
    }
}
