pub mod account;
pub mod branch;
pub mod commit;
pub mod file;
pub mod remote;

pub use account::*;
pub use branch::*;
pub use commit::*;
pub use file::*;
pub use remote::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_account_serde() {
        let account = GitAccount {
            id: "acc_1".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            provider: "GitHub".to_string(),
            username: "johndoe".to_string(),
            auth_type: "ssh".to_string(),
            ssh_key_path: Some("/home/user/.ssh/id_ed25519".to_string()),
            personal_access_token: None,
            avatar_url: None,
        };

        let json = serde_json::to_string(&account).expect("Failed to serialize GitAccount");
        let deserialized: GitAccount = serde_json::from_str(&json).expect("Failed to deserialize GitAccount");
        assert_eq!(account, deserialized);
    }

    #[test]
    fn test_commit_info_serde() {
        let commit = CommitInfo {
            id: "a1b2c3d4e5f6".to_string(),
            author: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            message: "Initial commit".to_string(),
            time: 1700000000,
            parents: vec!["parent1".to_string()],
        };

        let json = serde_json::to_string(&commit).expect("Failed to serialize CommitInfo");
        let deserialized: CommitInfo = serde_json::from_str(&json).expect("Failed to deserialize CommitInfo");
        assert_eq!(commit.id, deserialized.id);
        assert_eq!(commit.author, deserialized.author);
        assert_eq!(commit.parents, deserialized.parents);
    }

    #[test]
    fn test_repo_status_serde() {
        let status = RepoStatus {
            current_branch: "main".to_string(),
            files: vec![
                FileStatus {
                    path: "src/main.rs".to_string(),
                    status: "modified".to_string(),
                    staged: true,
                },
            ],
            ahead: 2,
            behind: 1,
        };

        let json = serde_json::to_string(&status).expect("Failed to serialize RepoStatus");
        let deserialized: RepoStatus = serde_json::from_str(&json).expect("Failed to deserialize RepoStatus");
        assert_eq!(status.current_branch, deserialized.current_branch);
        assert_eq!(status.files.len(), 1);
        assert_eq!(status.ahead, 2);
    }

    #[test]
    fn test_branch_info_serde() {
        let branch = BranchInfo {
            name: "feature/ai".to_string(),
            is_head: true,
            is_remote: false,
            target_commit: "deadbeef".to_string(),
        };

        let json = serde_json::to_string(&branch).expect("Failed to serialize BranchInfo");
        let deserialized: BranchInfo = serde_json::from_str(&json).expect("Failed to deserialize BranchInfo");
        assert_eq!(branch, deserialized);
    }

    #[test]
    fn test_remote_info_serde() {
        let remote = RepoRemoteInfo {
            remote_url: Some("git@github.com:org/repo.git".to_string()),
            owner: Some("org".to_string()),
            repo_name: Some("repo".to_string()),
            default_branch: "main".to_string(),
        };

        let json = serde_json::to_string(&remote).expect("Failed to serialize RepoRemoteInfo");
        let deserialized: RepoRemoteInfo = serde_json::from_str(&json).expect("Failed to deserialize RepoRemoteInfo");
        assert_eq!(remote.owner, deserialized.owner);
        assert_eq!(remote.repo_name, deserialized.repo_name);
        assert_eq!(remote.default_branch, "main");
    }
}
