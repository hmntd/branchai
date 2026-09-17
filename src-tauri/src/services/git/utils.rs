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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home_dir() {
        let path = "~/my_repo";
        let expanded = expand_home_dir(path);
        let home = std::env::var("HOME").unwrap_or_default();
        assert_eq!(expanded, format!("{}/my_repo", home));

        let absolute_path = "/var/log/git";
        assert_eq!(expand_home_dir(absolute_path), absolute_path);
    }

    #[test]
fn test_open_repo_invalid_path() {
        match open_repo("/non_existent_folder_path_12345") {
            Err(err) => assert!(err.contains("Failed to open repository")),
            Ok(_) => panic!("Expected error when opening invalid repo path"),
        }
    }

    #[test]
    fn test_fmt_err() {
        let err_msg = "test error";
        assert_eq!(fmt_err(err_msg), "test error");
    }
}
