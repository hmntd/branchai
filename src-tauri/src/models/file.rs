use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: String, // "staged", "modified", "untracked", "deleted", "new"
    pub staged: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoStatus {
    pub current_branch: String,
    pub files: Vec<FileStatus>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConflictFile {
    pub path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileHistoryItem {
    pub id: String,
    pub author: String,
    pub message: String,
    pub time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileBlameLine {
    pub line_no: usize,
    pub commit_id: String,
    pub author: String,
    pub time: i64,
    pub content: String,
}
