use serde::{Deserialize, Serialize};
use super::commit::CommitInfo;
use super::file::FileStatus;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    pub target_commit: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BranchDiffSummary {
    pub commits: Vec<CommitInfo>,
    pub files: Vec<FileStatus>,
}
