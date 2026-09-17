use git2::Repository;

/// Opens a Git repository at the given path, returning a user-friendly error message if failing.
pub fn open_repo(repo_path: &str) -> Result<Repository, String> {
    Repository::open(repo_path).map_err(|e| format!("Failed to open repository at '{}': {}", repo_path, e.message()))
}

/// Resolves path expanding `~` to the user's home directory.
pub fn expand_home_dir(path_str: &str) -> String {
    if path_str.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_default();
        path_str.replacen('~', &home, 1)
    } else {
        path_str.to_string()
    }
}

/// Helper to format git2 errors consistently.
pub fn fmt_err<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}
