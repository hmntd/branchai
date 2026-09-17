use std::fs;
use std::path::Path;

#[tauri::command]
pub fn get_repo_prs(repo_path: String) -> Result<String, String> {
    let pr_file = Path::new(&repo_path).join(".git").join("branchai_prs.json");
    if pr_file.exists() {
        fs::read_to_string(pr_file).map_err(|e| e.to_string())
    } else {
        Ok("[]".to_string())
    }
}

#[tauri::command]
pub fn save_repo_prs(repo_path: String, prs_json: String) -> Result<(), String> {
    let git_dir = Path::new(&repo_path).join(".git");
    if !git_dir.exists() {
        return Err("Not a valid git repository".to_string());
    }
    let pr_file = git_dir.join("branchai_prs.json");
    fs::write(pr_file, prs_json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_repo_issues(repo_path: String) -> Result<String, String> {
    let issue_file = Path::new(&repo_path).join(".git").join("branchai_issues.json");
    if issue_file.exists() {
        fs::read_to_string(issue_file).map_err(|e| e.to_string())
    } else {
        Ok("[]".to_string())
    }
}

#[tauri::command]
pub fn save_repo_issues(repo_path: String, issues_json: String) -> Result<(), String> {
    let git_dir = Path::new(&repo_path).join(".git");
    if !git_dir.exists() {
        return Err("Not a valid git repository".to_string());
    }
    let issue_file = git_dir.join("branchai_issues.json");
    fs::write(issue_file, issues_json).map_err(|e| e.to_string())
}
