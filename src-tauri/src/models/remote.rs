use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RepoRemoteInfo {
    pub remote_url: Option<String>,
    pub owner: Option<String>,
    pub repo_name: Option<String>,
    pub default_branch: String,
}
