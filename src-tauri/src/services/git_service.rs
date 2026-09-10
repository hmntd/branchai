use git2::Repository;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub message: String,
    pub time: i64,
}

#[tauri::command]
pub fn get_commits(repo_path: String) -> Result<Vec<CommitInfo>, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;

    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;
    revwalk.push_head().map_err(|e| e.message().to_string())?;

    let mut commits = Vec::new();

    for id in revwalk.take(50) {
        let id = id.map_err(|e| e.message().to_string())?;
        let commit = repo.find_commit(id).map_err(|e| e.message().to_string())?;

        let author = commit.author();
        let message = commit.message().unwrap_or("").to_string();

        commits.push(CommitInfo {
            id: id.to_string(),
            author: author.name().unwrap_or("Unknown").to_string(),
            message: message.trim().to_string(),
            time: commit.time().seconds(),
        });
    }

    Ok(commits)
}
