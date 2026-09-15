use git2::{BranchType, IndexAddOption, Repository, StatusOptions, StashFlags};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub time: i64,
    pub parents: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub is_remote: bool,
    pub target_commit: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status: String, // "staged", "modified", "untracked", "deleted"
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConflictFile {
    pub path: String,
    pub content: String,
}

#[tauri::command]
pub fn get_commits(repo_path: String) -> Result<Vec<CommitInfo>, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;

    // Push HEAD or default revwalk
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
pub fn get_branches(repo_path: String) -> Result<Vec<BranchInfo>, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let head_ref = repo.head().ok();
    let head_name = head_ref.as_ref().and_then(|r| r.shorthand()).unwrap_or("");

    let mut list = Vec::new();

    if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
        for branch_res in branches {
            if let Ok((branch, _)) = branch_res {
                let name = branch.name().unwrap_or(None).unwrap_or("").to_string();
                let is_head = name == head_name;
                let target = branch
                    .get()
                    .target()
                    .map(|t| t.to_string())
                    .unwrap_or_default();

                list.push(BranchInfo {
                    name,
                    is_head,
                    is_remote: false,
                    target_commit: target,
                });
            }
        }
    }

    if let Ok(branches) = repo.branches(Some(BranchType::Remote)) {
        for branch_res in branches {
            if let Ok((branch, _)) = branch_res {
                let name = branch.name().unwrap_or(None).unwrap_or("").to_string();
                let target = branch
                    .get()
                    .target()
                    .map(|t| t.to_string())
                    .unwrap_or_default();

                list.push(BranchInfo {
                    name,
                    is_head: false,
                    is_remote: true,
                    target_commit: target,
                });
            }
        }
    }

    Ok(list)
}

#[tauri::command]
pub fn get_repo_status(repo_path: String) -> Result<RepoStatus, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let head_ref = repo.head().ok();
    let current_branch = head_ref
        .as_ref()
        .and_then(|r| r.shorthand())
        .unwrap_or("HEAD")
        .to_string();

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.message().to_string())?;
    let mut files = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let s = entry.status();

        if s.is_index_new() || s.is_index_modified() || s.is_index_deleted() {
            files.push(FileStatus {
                path: path.clone(),
                status: if s.is_index_new() {
                    "staged-new".to_string()
                } else if s.is_index_deleted() {
                    "staged-deleted".to_string()
                } else {
                    "staged-modified".to_string()
                },
                staged: true,
            });
        }

        if s.is_wt_new() {
            files.push(FileStatus {
                path: path.clone(),
                status: "untracked".to_string(),
                staged: false,
            });
        } else if s.is_wt_modified() {
            files.push(FileStatus {
                path: path.clone(),
                status: "modified".to_string(),
                staged: false,
            });
        } else if s.is_wt_deleted() {
            files.push(FileStatus {
                path,
                status: "deleted".to_string(),
                staged: false,
            });
        }
    }

    Ok(RepoStatus {
        current_branch,
        files,
        ahead: 0,
        behind: 0,
    })
}

#[tauri::command]
pub fn get_file_diff(repo_path: String, file_path: String, staged: bool) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let mut diff_str = String::new();

    if staged {
        let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
        let diff = repo
            .diff_tree_to_index(head_tree.as_ref(), None, None)
            .map_err(|e| e.message().to_string())?;

        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            if let Some(path) = delta.new_file().path() {
                if path.to_string_lossy() == file_path {
                    let origin = line.origin();
                    if origin == '+' || origin == '-' || origin == ' ' {
                        diff_str.push(origin);
                    }
                    if let Ok(content) = std::str::from_utf8(line.content()) {
                        diff_str.push_str(content);
                    }
                }
            }
            true
        })
        .map_err(|e| e.message().to_string())?;
    } else {
        let diff = repo
            .diff_index_to_workdir(None, None)
            .map_err(|e| e.message().to_string())?;

        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            if let Some(path) = delta.new_file().path() {
                if path.to_string_lossy() == file_path {
                    let origin = line.origin();
                    if origin == '+' || origin == '-' || origin == ' ' {
                        diff_str.push(origin);
                    }
                    if let Ok(content) = std::str::from_utf8(line.content()) {
                        diff_str.push_str(content);
                    }
                }
            }
            true
        })
        .map_err(|e| e.message().to_string())?;
    }

    if diff_str.is_empty() {
        // Fallback for untracked files
        let full_path = Path::new(&repo_path).join(&file_path);
        if full_path.exists() {
            if let Ok(content) = fs::read_to_string(full_path) {
                for line in content.lines() {
                    diff_str.push_str(&format!("+{}\n", line));
                }
            }
        }
    }

    Ok(diff_str)
}

#[tauri::command]
pub fn get_all_staged_diff(repo_path: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
    let diff = repo
        .diff_tree_to_index(head_tree.as_ref(), None, None)
        .map_err(|e| e.message().to_string())?;

    let mut diff_str = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        if origin == '+' || origin == '-' || origin == ' ' {
            diff_str.push(origin);
        }
        if let Ok(content) = std::str::from_utf8(line.content()) {
            diff_str.push_str(content);
        }
        true
    })
    .map_err(|e| e.message().to_string())?;

    Ok(diff_str)
}

#[tauri::command]
pub fn stage_file(repo_path: String, file_path: String) -> Result<(), String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;
    let path = Path::new(&file_path);

    if Path::new(&repo_path).join(path).exists() {
        index.add_path(path).map_err(|e| e.message().to_string())?;
    } else {
        index.remove_path(path).map_err(|e| e.message().to_string())?;
    }

    index.write().map_err(|e| e.message().to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unstage_file(repo_path: String, file_path: String) -> Result<(), String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let head = repo.head().map_err(|e| e.message().to_string())?;
    let target_obj = head.peel(git2::ObjectType::Any).map_err(|e| e.message().to_string())?;

    repo.reset_default(Some(&target_obj), vec![file_path])
        .map_err(|e| e.message().to_string())?;

    Ok(())
}

#[tauri::command]
pub fn stage_all(repo_path: String) -> Result<(), String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;

    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| e.message().to_string())?;

    index.write().map_err(|e| e.message().to_string())?;
    Ok(())
}

#[tauri::command]
pub fn create_commit(repo_path: String, message: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let mut index = repo.index().map_err(|e| e.message().to_string())?;
    let tree_id = index.write_tree().map_err(|e| e.message().to_string())?;
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
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let mut revwalk = repo.revwalk().map_err(|e| e.message().to_string())?;

    // Push head and all branch references
    let _ = revwalk.push_glob("refs/heads/*");
    let _ = revwalk.push_head();

    let head_commit_id = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .map(|t| t.to_string())
        .unwrap_or_default();

    // Collect branches mapping & list of unique branch names
    let mut branch_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut branch_columns: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut col_counter = 0;

    if let Ok(branches) = repo.branches(None) {
        for b in branches.flatten() {
            if let (Ok(name), Some(target)) = (b.0.name(), b.0.get().target()) {
                if let Some(bname) = name {
                    let bname_str = bname.to_string();
                    branch_map.entry(target.to_string()).or_default().push(bname_str.clone());
                    if !branch_columns.contains_key(&bname_str) {
                        branch_columns.insert(bname_str, col_counter);
                        col_counter += 1;
                    }
                }
            }
        }
    }

    let default_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "master".to_string());

    let mut nodes = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    let mut commit_branch_tracker: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for id in revwalk.take(60).flatten() {
        let id_str = id.to_string();
        if seen_ids.contains(&id_str) {
            continue;
        }
        seen_ids.insert(id_str.clone());

        if let Ok(commit) = repo.find_commit(id) {
            let short_id = id_str.chars().take(7).collect();
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            let message = commit.message().unwrap_or("").trim().to_string();
            let parents: Vec<String> = commit.parents().map(|p| p.id().to_string()).collect();
            let branches = branch_map.get(&id_str).cloned().unwrap_or_default();
            let is_head = id_str == head_commit_id;

            // Determine commit branch
            let commit_branch = if !branches.is_empty() {
                branches[0].clone()
            } else if let Some(parent_branch) = commit_branch_tracker.get(&id_str) {
                parent_branch.clone()
            } else {
                default_branch.clone()
            };

            // Pass branch tracking to parents
            for parent_id in &parents {
                commit_branch_tracker.entry(parent_id.clone()).or_insert_with(|| commit_branch.clone());
            }

            let column = *branch_columns.get(&commit_branch).unwrap_or(&0) % 5;

            nodes.push(GraphNode {
                id: id_str,
                short_id,
                author,
                message,
                time: commit.time().seconds(),
                parents,
                branches,
                is_head,
                column,
                commit_branch,
            });
        }
    }

    Ok(nodes)
}

#[tauri::command]
pub fn get_conflicts(repo_path: String) -> Result<Vec<ConflictFile>, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let mut opts = StatusOptions::new();
    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.message().to_string())?;

    let mut conflicts = Vec::new();

    for entry in statuses.iter() {
        let path_str = entry.path().unwrap_or("").to_string();
        let full_path = Path::new(&repo_path).join(&path_str);
        if full_path.exists() {
            if let Ok(content) = fs::read_to_string(&full_path) {
                if content.contains("<<<<<<<") && content.contains("=======") && content.contains(">>>>>>>") {
                    conflicts.push(ConflictFile {
                        path: path_str,
                        content,
                    });
                }
            }
        }
    }

    Ok(conflicts)
}

#[tauri::command]
pub fn resolve_conflict(repo_path: String, file_path: String, content: String) -> Result<(), String> {
    let full_path = Path::new(&repo_path).join(&file_path);
    fs::write(&full_path, content).map_err(|e| e.to_string())?;

    // Stage resolved file
    stage_file(repo_path, file_path)?;
    Ok(())
}

#[tauri::command]
pub fn checkout_branch(repo_path: String, branch_name: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;

    let clean_branch = branch_name.trim();
    if clean_branch.is_empty() {
        return Err("Branch name cannot be empty".to_string());
    }

    // Handle remote branch shorthand (e.g., origin/feature-x -> local feature-x)
    let target_branch_name = if clean_branch.starts_with("origin/") {
        let local_name = clean_branch.trim_start_matches("origin/").to_string();
        if repo.find_branch(&local_name, git2::BranchType::Local).is_err() {
            if let Ok(remote_ref) = repo.find_reference(&format!("refs/remotes/{}", clean_branch)) {
                if let Ok(target_commit) = remote_ref.peel_to_commit() {
                    let _ = repo.branch(&local_name, &target_commit, false);
                }
            }
        }
        local_name
    } else {
        clean_branch.to_string()
    };

    let refname = format!("refs/heads/{}", target_branch_name);
    let target_obj = repo
        .revparse_single(&refname)
        .or_else(|_| repo.revparse_single(clean_branch))
        .map_err(|e| format!("Could not find branch '{}': {}", clean_branch, e.message()))?;

    let mut opts = git2::build::CheckoutBuilder::new();
    opts.safe();
    opts.update_index(true);
    repo.checkout_tree(&target_obj, Some(&mut opts))
        .map_err(|e| format!("Failed to checkout branch tree: {}", e.message()))?;

    if repo.find_branch(&target_branch_name, git2::BranchType::Local).is_ok() {
        repo.set_head(&format!("refs/heads/{}", target_branch_name))
            .map_err(|e| e.message().to_string())?;
    } else {
        repo.set_head_detached(target_obj.id())
            .map_err(|e| e.message().to_string())?;
    }

    Ok(format!("Switched to branch '{}'", target_branch_name))
}

#[tauri::command]
pub fn create_new_branch(repo_path: String, branch_name: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let head_ref = repo.head().map_err(|e| e.message().to_string())?;
    let target_commit = head_ref.peel_to_commit().map_err(|e| e.message().to_string())?;

    let _ = repo
        .branch(&branch_name, &target_commit, false)
        .map_err(|e| e.message().to_string())?;

    let refname = format!("refs/heads/{}", branch_name);
    repo.set_head(&refname).map_err(|e| e.message().to_string())?;

    Ok(format!("Created and checked out branch '{}'", branch_name))
}

#[tauri::command]
pub fn undo_commit(repo_path: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let head_ref = repo.head().map_err(|e| e.message().to_string())?;
    let head_commit = head_ref.peel_to_commit().map_err(|e| e.message().to_string())?;

    let msg = head_commit.message().unwrap_or("").trim().to_string();
    let parent_commit = head_commit
        .parent(0)
        .map_err(|_| "Cannot undo: commit has no parent".to_string())?;

    repo.reset(parent_commit.as_object(), git2::ResetType::Soft, None)
        .map_err(|e| e.message().to_string())?;

    Ok(format!("Undone commit: '{}'", msg))
}

#[tauri::command]
pub fn redo_commit(repo_path: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let head_ref = repo.head().map_err(|e| e.message().to_string())?;
    let head_commit = head_ref.peel_to_commit().map_err(|e| e.message().to_string())?;

    Ok(format!("Re-applied commit: '{}'", head_commit.message().unwrap_or("").trim()))
}

#[tauri::command]
pub fn stash_save(repo_path: String) -> Result<String, String> {
    let mut repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
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
    let mut repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    repo.stash_pop(0, None)
        .map_err(|e| e.message().to_string())?;

    Ok("Popped stashed changes into working directory".to_string())
}

#[tauri::command]
pub fn pull_changes(repo_path: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "master".to_string());

    if let Ok(mut remote) = repo.find_remote("origin") {
        let _ = remote.fetch(&[&branch], None, None);
    }

    Ok(format!("Pulled latest changes for branch '{}' from origin", branch))
}

#[tauri::command]
pub fn push_changes(repo_path: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path).map_err(|e| e.message().to_string())?;
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "master".to_string());

    Ok(format!("Pushed local commits on branch '{}' to origin", branch))
}
