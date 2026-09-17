use crate::models::{BranchDiffSummary, BranchInfo, CommitInfo, FileStatus};
use crate::services::git::utils::open_repo;
use std::process::Command;

#[tauri::command]
pub fn get_branches(repo_path: String) -> Result<Vec<BranchInfo>, String> {
    let repo = open_repo(&repo_path)?;
    let mut branch_list = Vec::new();

    let branches = repo
        .branches(Some(git2::BranchType::Local))
        .map_err(|e| e.message().to_string())?;

    for b in branches {
        let (branch, _type) = b.map_err(|e| e.message().to_string())?;
        let name = branch.name().map_err(|e| e.message().to_string())?.unwrap_or("").to_string();
        let is_head = branch.is_head();
        let target = branch.get().target().map(|t| t.to_string()).unwrap_or_default();

        branch_list.push(BranchInfo {
            name,
            is_head,
            is_remote: false,
            target_commit: target,
        });
    }

    Ok(branch_list)
}

#[tauri::command]
pub fn checkout_branch(repo_path: String, branch_name: String) -> Result<(), String> {
    let repo = open_repo(&repo_path)?;

    let (object, reference) = repo
        .revparse_ext(&branch_name)
        .map_err(|e| e.message().to_string())?;

    repo.checkout_tree(&object, None)
        .map_err(|e| e.message().to_string())?;

    if let Some(r) = reference {
        repo.set_head(r.name().unwrap_or("refs/heads/master"))
            .map_err(|e| e.message().to_string())?;
    } else {
        repo.set_head_detached(object.id())
            .map_err(|e| e.message().to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn create_new_branch(repo_path: String, branch_name: String) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
    let head_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    let target_commit = match head_commit {
        Some(c) => c,
        None => return Err("Repository has no commits to branch off from.".to_string()),
    };

    repo.branch(&branch_name, &target_commit, false)
        .map_err(|e| e.message().to_string())?;

    checkout_branch(repo_path, branch_name.clone())?;

    Ok(format!("Created and checked out new branch '{}'", branch_name))
}

#[tauri::command]
pub fn get_branch_diff_summary(
    repo_path: String,
    branch_name: String,
    target_branch: String,
) -> Result<BranchDiffSummary, String> {
    let repo = open_repo(&repo_path)?;

    let branch_ref = repo
        .find_branch(&branch_name, git2::BranchType::Local)
        .or_else(|_| repo.find_branch(&branch_name, git2::BranchType::Remote))
        .map_err(|e| e.message().to_string())?;

    let branch_commit = branch_ref.get().peel_to_commit().map_err(|e| e.message().to_string())?;

    let target_ref = repo
        .find_branch(&target_branch, git2::BranchType::Local)
        .or_else(|_| repo.find_branch(&target_branch, git2::BranchType::Remote))
        .ok();

    let target_commit = target_ref.and_then(|r| r.get().peel_to_commit().ok());

    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;
    let _ = revwalk.push(branch_commit.id());
    if let Some(ref tc) = target_commit {
        let _ = revwalk.hide(tc.id());
    }

    let mut commits = Vec::new();
    for id in revwalk.take(50).flatten() {
        if let Ok(c) = repo.find_commit(id) {
            commits.push(CommitInfo {
                id: id.to_string(),
                author: c.author().name().unwrap_or("Unknown").to_string(),
                email: c.author().email().unwrap_or("").to_string(),
                message: c.message().unwrap_or("").trim().to_string(),
                time: c.time().seconds(),
                parents: c.parents().map(|p| p.id().to_string()).collect(),
            });
        }
    }

    let branch_tree = branch_commit.tree().map_err(|e| e.message().to_string())?;
    let target_tree = target_commit.as_ref().and_then(|c| c.tree().ok());

    let diff = repo
        .diff_tree_to_tree(target_tree.as_ref(), Some(&branch_tree), None)
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

    Ok(BranchDiffSummary { commits, files })
}

#[tauri::command]
pub fn merge_branch(
    repo_path: String,
    source_branch: String,
    target_branch: String,
) -> Result<String, String> {
    checkout_branch(repo_path.clone(), target_branch.clone())?;

    let output = Command::new("git")
        .current_dir(&repo_path)
        .args([
            "merge",
            &source_branch,
            "-m",
            &format!("Merge branch '{}' into {}", source_branch, target_branch),
        ])
        .output()
        .map_err(|e| format!("Failed to execute git merge: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git merge failed: {}", err_msg));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}
