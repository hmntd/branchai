use crate::services::git::utils::open_repo;
use git2::StashFlags;
use std::process::Command;

#[tauri::command]
pub fn stash_save(repo_path: String) -> Result<String, String> {
    let mut repo = open_repo(&repo_path)?;
    let signature = repo.signature().unwrap_or_else(|_| {
        git2::Signature::now("BranchAI", "user@branchai.local").unwrap()
    });

    let oid = repo
        .stash_save(&signature, "WIP Stash by BranchAI", Some(StashFlags::INCLUDE_UNTRACKED))
        .map_err(|e| e.message().to_string())?;

    Ok(format!("Stashed all changes including untracked files ({})", oid))
}

#[tauri::command]
pub fn stash_pop(repo_path: String) -> Result<String, String> {
    let mut repo = open_repo(&repo_path)?;
    repo.stash_pop(0, None)
        .map_err(|e| e.message().to_string())?;

    Ok("Popped stashed changes into working directory".to_string())
}

#[tauri::command]
pub fn stash_file(repo_path: String, file_path: String) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["stash", "push", "-m", &format!("Stashed {}", file_path), "--", &file_path])
        .output()
        .map_err(|e| format!("Failed to execute git stash push: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git stash file failed: {}", err_msg));
    }

    Ok(format!("Stashed changes for file: {}", file_path))
}
