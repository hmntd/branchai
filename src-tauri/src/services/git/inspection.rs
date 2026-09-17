use crate::models::{ConflictFile, FileBlameLine, FileHistoryItem};
use crate::services::git::utils::open_repo;
use std::fs;
use std::path::Path;
use std::process::Command;

#[tauri::command]
pub fn get_file_history(
    repo_path: String,
    file_path: String,
) -> Result<Vec<FileHistoryItem>, String> {
    let repo = open_repo(&repo_path)?;
    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;

    if revwalk.push_head().is_err() {
        return Ok(Vec::new());
    }

    let mut history = Vec::new();
    for id in revwalk.take(200).flatten() {
        if let Ok(commit) = repo.find_commit(id) {
            let tree = commit.tree().ok();
            if let Some(t) = tree {
                if t.get_path(Path::new(&file_path)).is_ok() {
                    history.push(FileHistoryItem {
                        id: id.to_string(),
                        author: commit.author().name().unwrap_or("Unknown").to_string(),
                        message: commit.message().unwrap_or("").trim().to_string(),
                        time: commit.time().seconds(),
                    });
                }
            }
        }
    }

    Ok(history)
}

#[tauri::command]
pub fn get_file_blame(
    repo_path: String,
    file_path: String,
) -> Result<Vec<FileBlameLine>, String> {
    let repo = open_repo(&repo_path)?;
    let blame = repo
        .blame_file(Path::new(&file_path), None)
        .map_err(|e| e.message().to_string())?;

    let full_path = Path::new(&repo_path).join(&file_path);
    let file_content = fs::read_to_string(&full_path).unwrap_or_default();
    let lines: Vec<&str> = file_content.lines().collect();

    let mut result = Vec::new();
    let mut current_line = 1;

    for hunk in blame.iter() {
        let commit_id = hunk.final_commit_id().to_string();
        let sig = hunk.final_signature();
        let author = sig.name().unwrap_or("Unknown").to_string();

        let commit_time = if let Ok(c) = repo.find_commit(hunk.final_commit_id()) {
            c.time().seconds()
        } else {
            0
        };

        let num_lines = hunk.lines_in_hunk();
        for i in 0..num_lines {
            let line_idx = current_line - 1;
            let line_text = if line_idx < lines.len() {
                lines[line_idx].to_string()
            } else {
                String::new()
            };

            result.push(FileBlameLine {
                line_no: current_line,
                commit_id: commit_id.clone(),
                author: author.clone(),
                time: commit_time,
                content: line_text,
            });

            current_line += 1;
            if i + 1 == num_lines {
                // done hunk iteration
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn open_in_external_diff(repo_path: String, file_path: String) -> Result<(), String> {
    let full_path = Path::new(&repo_path).join(&file_path);
    let _ = Command::new("git")
        .current_dir(&repo_path)
        .args(["difftool", "-y", &full_path.to_string_lossy()])
        .spawn();
    Ok(())
}

#[tauri::command]
pub fn open_in_vscode(repo_path: String, file_path: String) -> Result<(), String> {
    let full_path = Path::new(&repo_path).join(&file_path);
    let _ = Command::new("code")
        .arg(full_path.to_string_lossy().to_string())
        .spawn()
        .or_else(|_| {
            Command::new("code-insiders")
                .arg(full_path.to_string_lossy().to_string())
                .spawn()
        });
    Ok(())
}

fn open_path_in_system(path: &Path) {
    #[cfg(target_os = "windows")]
    let _ = Command::new("explorer").arg(path).spawn();
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(path).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let _ = Command::new("xdg-open").arg(path).spawn();
}

#[tauri::command]
pub fn open_file_default(repo_path: String, file_path: String) -> Result<(), String> {
    let full_path = Path::new(&repo_path).join(&file_path);
    open_path_in_system(&full_path);
    Ok(())
}

#[tauri::command]
pub fn show_in_folder(repo_path: String, file_path: String) -> Result<(), String> {
    let full_path = Path::new(&repo_path).join(&file_path);
    let parent = full_path.parent().unwrap_or(Path::new(&repo_path));
    open_path_in_system(parent);
    Ok(())
}

#[tauri::command]
pub fn create_patch_from_file(repo_path: String, file_path: String) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["diff", "HEAD", "--", &file_path])
        .output()
        .map_err(|e| format!("Failed to generate patch: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Patch generation failed: {}", err_msg));
    }

    let patch = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(patch)
}

#[tauri::command]
pub fn delete_file(repo_path: String, file_path: String) -> Result<(), String> {
    let full_path = Path::new(&repo_path).join(&file_path);
    if full_path.exists() {
        if full_path.is_dir() {
            fs::remove_dir_all(&full_path).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(&full_path).map_err(|e| e.to_string())?;
        }
    }

    let repo = open_repo(&repo_path)?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;
    let _ = index.remove_path(Path::new(&file_path));
    let _ = index.write();

    Ok(())
}

#[tauri::command]
pub fn get_conflicts(repo_path: String) -> Result<Vec<ConflictFile>, String> {
    let repo = open_repo(&repo_path)?;
    let index = repo.index().map_err(|e| e.message().to_string())?;

    let mut conflicts = Vec::new();
    if index.has_conflicts() {
        let conflict_entries = index.conflicts().map_err(|e| e.message().to_string())?;
        for entry in conflict_entries {
            let entry = entry.map_err(|e| e.message().to_string())?;
            let path = if let Some(our) = entry.our {
                String::from_utf8_lossy(&our.path).to_string()
            } else if let Some(their) = entry.their {
                String::from_utf8_lossy(&their.path).to_string()
            } else {
                continue;
            };

            let full_path = Path::new(&repo_path).join(&path);
            let content = fs::read_to_string(full_path).unwrap_or_default();
            conflicts.push(ConflictFile { path, content });
        }
    }

    Ok(conflicts)
}

#[tauri::command]
pub fn resolve_conflict(
    repo_path: String,
    file_path: String,
    resolved_content: String,
) -> Result<(), String> {
    let full_path = Path::new(&repo_path).join(&file_path);
    fs::write(&full_path, resolved_content).map_err(|e| e.to_string())?;

    let repo = open_repo(&repo_path)?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;
    index
        .add_path(Path::new(&file_path))
        .map_err(|e| e.message().to_string())?;
    index.write().map_err(|e| e.message().to_string())?;

    Ok(())
}
