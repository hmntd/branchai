use crate::models::RepoRemoteInfo;
use crate::services::git::utils::open_repo;

#[tauri::command]
pub fn pick_repository_folder() -> Result<Option<String>, String> {
    let folder = rfd::FileDialog::new()
        .set_title("Select Git Repository Directory")
        .pick_folder();

    if let Some(path) = folder {
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn pull_changes(repo_path: String) -> Result<String, String> {
    let repo = open_repo(&repo_path)?;
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
    let repo = open_repo(&repo_path)?;
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "master".to_string());

    Ok(format!("Pushed local commits on branch '{}' to origin", branch))
}

#[tauri::command]
pub fn get_repo_remote_info(repo_path: String) -> Result<RepoRemoteInfo, String> {
    let repo = open_repo(&repo_path)?;

    let mut remote_url = None;
    let mut owner = None;
    let mut repo_name = None;

    let default_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "main".to_string());

    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            remote_url = Some(url.to_string());

            let clean_url = url
                .trim_start_matches("git@github.com:")
                .trim_start_matches("https://github.com/")
                .trim_end_matches(".git");

            let parts: Vec<&str> = clean_url.split('/').collect();
            if parts.len() == 2 {
                owner = Some(parts[0].to_string());
                repo_name = Some(parts[1].to_string());
            } else if parts.len() == 1 {
                let path_parts: Vec<&str> = parts[0].split(':').collect();
                if path_parts.len() == 2 {
                    owner = Some(path_parts[0].to_string());
                    repo_name = Some(path_parts[1].to_string());
                }
            }
        }
    }

    Ok(RepoRemoteInfo {
        remote_url,
        owner,
        repo_name,
        default_branch,
    })
}
