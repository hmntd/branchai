use crate::models::{CommitInfo, FileStatus, GraphNode};
use crate::services::git::utils::open_repo;
use git2::{Oid, ResetType};
use std::collections::HashMap;

#[tauri::command]
pub fn get_commits(repo_path: String) -> Result<Vec<CommitInfo>, String> {
    let repo = open_repo(&repo_path)?;
    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;

    if revwalk.push_head().is_err() {
        return Ok(Vec::new());
    }

    let mut commits = Vec::new();
    for id in revwalk.take(100) {
        let id = id.map_err(|e| e.message().to_string())?;
        let commit = repo.find_commit(id).map_err(|e| e.message().to_string())?;

        let author = commit.author();
        let name = author.name().unwrap_or("Unknown").to_string();
        let email = author.email().unwrap_or("").to_string();
        let message = commit.message().unwrap_or("").to_string();
        let parents: Vec<String> = commit.parents().map(|p| p.id().to_string()).collect();

        commits.push(CommitInfo {
            id: id.to_string(),
            author: name,
            email,
            message: message.trim().to_string(),
            time: commit.time().seconds(),
            parents,
        });
    }

    Ok(commits)
}

#[tauri::command]
pub fn create_commit(repo_path: String, message: String) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;
    let tree_id = index.write_tree().map_err(|e| e.message().to_string())?;
    index.write().map_err(|e| e.message().to_string())?;
    let tree = repo.find_tree(tree_id).map_err(|e| e.message().to_string())?;

    let signature = repo.signature().unwrap_or_else(|_| {
        git2::Signature::now("BranchAI User", "user@branchai.local").unwrap()
    });

    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let mut parents = Vec::new();
    if let Some(ref p) = parent_commit {
        parents.push(p);
    }

    let commit_id = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &parents,
        )
        .map_err(|e| e.message().to_string())?;

    Ok(commit_id.to_string())
}

#[tauri::command]
pub fn get_graph_data(repo_path: String) -> Result<Vec<GraphNode>, String> {
    let repo = open_repo(&repo_path)?;
    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;

    let _ = revwalk.push_glob("refs/heads/*");
    let _ = revwalk.push_head();

    let head_commit_id = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .map(|oid| oid.to_string())
        .unwrap_or_default();

    let mut branch_map: HashMap<String, Vec<String>> = HashMap::new();
    if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
        for b in branches.flatten() {
            if let (Ok(Some(name)), Some(target_oid)) = (b.0.name(), b.0.get().target()) {
                branch_map
                    .entry(target_oid.to_string())
                    .or_default()
                    .push(name.to_string());
            }
        }
    }

    let mut nodes = Vec::new();
    let mut active_columns: Vec<String> = Vec::new();

    for id in revwalk.take(150).flatten() {
        if let Ok(commit) = repo.find_commit(id) {
            let commit_id = id.to_string();
            let short_id = commit_id.chars().take(7).collect::<String>();
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            let message = commit.message().unwrap_or("").trim().to_string();
            let time = commit.time().seconds();
            let parents: Vec<String> = commit.parents().map(|p| p.id().to_string()).collect();

            let branches_at_commit = branch_map.get(&commit_id).cloned().unwrap_or_default();
            let is_head = commit_id == head_commit_id;

            let commit_branch = if !branches_at_commit.is_empty() {
                branches_at_commit[0].clone()
            } else if is_head {
                "main".to_string()
            } else {
                "master".to_string()
            };

            let col = if let Some(pos) = active_columns.iter().position(|x| x == &commit_id) {
                let p = pos;
                active_columns.remove(pos);
                p
            } else {
                active_columns.len()
            };

            for parent_id in &parents {
                if !active_columns.contains(parent_id) {
                    active_columns.push(parent_id.clone());
                }
            }

            nodes.push(GraphNode {
                id: commit_id,
                short_id,
                author,
                message,
                time,
                parents,
                branches: branches_at_commit,
                is_head,
                column: col,
                commit_branch,
            });
        }
    }

    Ok(nodes)
}

#[tauri::command]
pub fn undo_commit(repo_path: String) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
    let head = repo.head().map_err(|e| e.message().to_string())?;
    let head_commit = head.peel_to_commit().map_err(|e| e.message().to_string())?;

    let parent = head_commit
        .parent(0)
        .map_err(|_| "Cannot undo: initial commit has no parent".to_string())?;

    let commit_msg = head_commit.message().unwrap_or("").trim().to_string();
    repo.reset(parent.as_object(), ResetType::Soft, None)
        .map_err(|e| e.message().to_string())?;

    Ok(format!("Undone commit: \"{}\"", commit_msg))
}

#[tauri::command]
pub fn redo_commit(repo_path: String) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
    let reflog = repo.reflog("HEAD").map_err(|e| e.message().to_string())?;

    if let Some(entry) = reflog.iter().next() {
        let target_oid = entry.id_new();
        let target_obj = repo
            .find_object(target_oid, None)
            .map_err(|e| e.message().to_string())?;

        repo.reset(&target_obj, ResetType::Soft, None)
            .map_err(|e| e.message().to_string())?;

        return Ok(format!("Redone commit ({})", &target_oid.to_string()[..7]));
    }

    Err("No commit found in reflog to redo".to_string())
}

#[tauri::command]
pub fn get_commit_files(
    repo_path: String,
    commit_id: String,
) -> Result<Vec<FileStatus>, String> {
    let repo = open_repo(&repo_path)?;
    let oid = Oid::from_str(&commit_id).map_err(|e| e.message().to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.message().to_string())?;
    let commit_tree = commit.tree().map_err(|e| e.message().to_string())?;

    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)
        .map_err(|e| e.message().to_string())?;

    let mut files = Vec::new();
    for delta in diff.deltas() {
        let status = match delta.status() {
            git2::Delta::Added => "new".to_string(),
            git2::Delta::Deleted => "deleted".to_string(),
            git2::Delta::Modified => "modified".to_string(),
            git2::Delta::Renamed => "renamed".to_string(),
            _ => "modified".to_string(),
        };

        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if !path.is_empty() {
            files.push(FileStatus {
                path,
                status,
                staged: false,
            });
        }
    }

    Ok(files)
}

#[tauri::command]
pub fn get_commit_file_diff(
    repo_path: String,
    commit_id: String,
    file_path: String,
) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
    let oid = Oid::from_str(&commit_id).map_err(|e| e.message().to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.message().to_string())?;
    let commit_tree = commit.tree().map_err(|e| e.message().to_string())?;

    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)
        .map_err(|e| e.message().to_string())?;

    let mut diff_output = String::new();
    let mut found = false;

    let _ = diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path());

        if let Some(p) = path {
            if p.to_string_lossy() == file_path {
                found = true;
                let content = std::str::from_utf8(line.content()).unwrap_or("");
                let origin = line.origin();
                match origin {
                    '+' | '-' | ' ' => {
                        diff_output.push(origin);
                        diff_output.push_str(content);
                    }
                    'H' => {
                        diff_output.push_str(content);
                    }
                    _ => {
                        diff_output.push_str(content);
                    }
                }
            }
        }
        true
    });

    if !found || diff_output.is_empty() {
        return Ok("No diff changes found for this file in commit.".to_string());
    }

    Ok(diff_output)
}
