use crate::models::{FileStatus, RepoStatus};
use crate::services::git::utils::open_repo;
use git2::{IndexAddOption, StatusOptions};
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn get_repo_status(repo_path: String) -> Result<RepoStatus, String> {
    let repo = open_repo(&repo_path)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.message().to_string())?;

    let mut files = Vec::new();
    for entry in statuses.iter() {
        let status_flags = entry.status();
        let path = entry.path().unwrap_or("").to_string();

        if status_flags.is_index_new() {
            files.push(FileStatus {
                path: path.clone(),
                status: "new".to_string(),
                staged: true,
            });
        } else if status_flags.is_index_modified() {
            files.push(FileStatus {
                path: path.clone(),
                status: "modified".to_string(),
                staged: true,
            });
        } else if status_flags.is_index_deleted() {
            files.push(FileStatus {
                path: path.clone(),
                status: "deleted".to_string(),
                staged: true,
            });
        }

        if status_flags.is_wt_modified() {
            files.push(FileStatus {
                path: path.clone(),
                status: "modified".to_string(),
                staged: false,
            });
        } else if status_flags.is_wt_deleted() {
            files.push(FileStatus {
                path: path.clone(),
                status: "deleted".to_string(),
                staged: false,
            });
        } else if status_flags.is_wt_new() {
            files.push(FileStatus {
                path,
                status: "untracked".to_string(),
                staged: false,
            });
        }
    }

    let current_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "HEAD".to_string());

    Ok(RepoStatus {
        current_branch,
        files,
        ahead: 0,
        behind: 0,
    })
}

#[tauri::command]
pub fn get_file_diff(
    repo_path: String,
    file_path: String,
    staged: bool,
) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
    let mut diff_output = String::new();

    let diff = if staged {
        let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
        let index = repo.index().map_err(|e| e.message().to_string())?;
        repo.diff_tree_to_index(head_tree.as_ref(), Some(&index), None)
            .map_err(|e| e.message().to_string())?
    } else {
        let full_file_path = Path::new(&repo_path).join(&file_path);
        if full_file_path.exists() {
            let index = repo.index().ok();
            let is_in_index = index.as_ref().map_or(false, |idx| idx.get_path(Path::new(&file_path), 0).is_some());

            if !is_in_index {
                if let Ok(content) = fs::read_to_string(&full_file_path) {
                    diff_output.push_str(&format!("@@ -0,0 +1,{} @@\n", content.lines().count()));
                    for line in content.lines() {
                        diff_output.push_str(&format!("+{}\n", line));
                    }
                    return Ok(diff_output);
                }
            }
        }

        repo.diff_index_to_workdir(None, None)
            .map_err(|e| e.message().to_string())?
    };

    let mut found = false;
    let _ = diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let path = _delta
            .new_file()
            .path()
            .or_else(|| _delta.old_file().path());

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

    if !found && diff_output.is_empty() {
        return Ok("No changes detected in file.".to_string());
    }

    Ok(diff_output)
}

#[tauri::command]
pub fn get_all_staged_diff(repo_path: String) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
    let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
    let index = repo.index().map_err(|e| e.message().to_string())?;

    let diff = repo
        .diff_tree_to_index(head_tree.as_ref(), Some(&index), None)
        .map_err(|e| e.message().to_string())?;

    let mut diff_output = String::new();
    let _ = diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let content = std::str::from_utf8(line.content()).unwrap_or("");
        let origin = line.origin();
        match origin {
            'F' => {
                diff_output.push_str(&format!("--- a/{}\n+++ b/{}\n", path, path));
            }
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
        true
    });

    Ok(diff_output)
}

#[tauri::command]
pub fn stage_file(repo_path: String, file_path: String) -> Result<(), String> {
    let repo = open_repo(&repo_path)?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;

    let full_path = Path::new(&repo_path).join(&file_path);
    if !full_path.exists() {
        index.remove_path(Path::new(&file_path)).map_err(|e| e.message().to_string())?;
    } else {
        index.add_path(Path::new(&file_path)).map_err(|e| e.message().to_string())?;
    }

    index.write().map_err(|e| e.message().to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unstage_file(repo_path: String, file_path: String) -> Result<(), String> {
    let repo = open_repo(&repo_path)?;
    let head_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    if let Some(commit) = head_commit {
        repo.reset_default(Some(commit.as_object()), [Path::new(&file_path)])
            .map_err(|e| e.message().to_string())?;
    } else {
        let mut index = repo.index().map_err(|e| e.message().to_string())?;
        index.remove_path(Path::new(&file_path)).map_err(|e| e.message().to_string())?;
        index.write().map_err(|e| e.message().to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn stage_all(repo_path: String) -> Result<(), String> {
    let repo = open_repo(&repo_path)?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;

    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| e.message().to_string())?;

    index.write().map_err(|e| e.message().to_string())?;
    Ok(())
}

#[tauri::command]
pub fn discard_file_changes(repo_path: String, file_path: String) -> Result<(), String> {
    let repo = open_repo(&repo_path)?;
    let full_path = Path::new(&repo_path).join(&file_path);

    let head = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
    if let Some(tree) = head {
        let entry = tree.get_path(Path::new(&file_path));
        if let Ok(entry) = entry {
            let obj = entry.to_object(&repo).map_err(|e| e.message().to_string())?;
            if let Some(blob) = obj.as_blob() {
                fs::write(&full_path, blob.content()).map_err(|e| e.to_string())?;
                let mut index = repo.index().map_err(|e| e.message().to_string())?;
                index.add_path(Path::new(&file_path)).map_err(|e| e.message().to_string())?;
                index.write().map_err(|e| e.message().to_string())?;
                return Ok(());
            }
        }
    }

    if full_path.exists() {
        fs::remove_file(&full_path).map_err(|e| e.to_string())?;
        let mut index = repo.index().map_err(|e| e.message().to_string())?;
        let _ = index.remove_path(Path::new(&file_path));
        let _ = index.write();
    }

    Ok(())
}
