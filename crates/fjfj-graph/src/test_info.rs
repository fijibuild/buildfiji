//! What `bazel test` needs to know of a test target (buildfiji-fyz.9).

use crate::action::Action;
use crate::artifact::Artifact;

/// A test target's runner action and where it leaves its results.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TestInfo {
    /// The action that runs the test through `test-setup.sh`.
    pub action: Action,
    /// `bazel-out/k8-fastbuild/testlogs/pkg/name/test.log`.
    pub log: Artifact,
    /// `test.xml`.
    pub xml: Artifact,
    /// `small`, `medium`, `large` or `enormous`.
    pub size: String,
    /// Seconds before the test is killed.
    pub timeout_seconds: u64,
}
