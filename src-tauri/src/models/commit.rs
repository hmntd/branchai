use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub time: i64,
    pub parents: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub short_id: String,
    pub author: String,
    pub message: String,
    pub time: i64,
    pub parents: Vec<String>,
    pub branches: Vec<String>,
    pub is_head: bool,
    pub column: usize,
    pub commit_branch: String,
}
