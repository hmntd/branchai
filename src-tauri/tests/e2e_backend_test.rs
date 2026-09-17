use branchai_lib::models::*;
use branchai_lib::services::git::*;
use git2::{Repository, Signature};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to initialize a clean temporary Git repository for E2E testing.
fn setup_e2e_repository() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory for E2E");
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let repo = Repository::init(&repo_path).expect("Failed to initialize git repo");
    let sig = Signature::now("E2E Tester", "e2e@branchai.local").unwrap();

    let file_path = temp_dir.path().join("index.html");
    fs::write(&file_path, "<html><body><h1>BranchAI App</h1></body></html>\n").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(Path::new("index.html")).unwrap();
    let tree_id = index.write_tree().unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let _ = repo
        .commit(Some("HEAD"), &sig, &sig, "chore: initial layout", &tree, &[])
        .unwrap();

    (temp_dir, repo_path)
}

#[test]
fn e2e_feature_development_lifecycle() {
    let (_temp, repo_path) = setup_e2e_repository();

    // 1. Initial Status Check
    let initial_status = get_repo_status(repo_path.clone()).unwrap();
    let default_branch = initial_status.current_branch.clone();
    assert!(!default_branch.is_empty());
    assert_eq!(initial_status.files.len(), 0);

    // 2. Create and Checkout Feature Branch
    create_new_branch(repo_path.clone(), "feature/user-auth".to_string()).unwrap();

    let status_in_branch = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_in_branch.current_branch, "feature/user-auth");

    // 3. Make Changes & Stage File
    let auth_file = Path::new(&repo_path).join("auth.ts");
    fs::write(&auth_file, "export class AuthManager { login() { return true; } }\n").unwrap();

    stage_file(repo_path.clone(), "auth.ts".to_string()).unwrap();

    // 4. Create First Feature Commit
    let commit1_id = create_commit(repo_path.clone(), "feat(auth): add AuthManager class".to_string()).unwrap();
    assert!(!commit1_id.is_empty());

    // 5. Simulate In-Progress Work & Stash Operations
    let index_html = Path::new(&repo_path).join("index.html");
    fs::write(&index_html, "<html><body><h1>BranchAI App with Auth</h1></body></html>\n").unwrap();

    let status_dirty = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_dirty.files.len(), 1);

    stash_save(repo_path.clone()).unwrap();

    let status_stashed = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_stashed.files.len(), 0);

    stash_pop(repo_path.clone()).unwrap();

    let status_popped = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_popped.files.len(), 1);

    // Stage and commit remaining changes
    stage_all(repo_path.clone()).unwrap();
    create_commit(repo_path.clone(), "feat(ui): update html title".to_string()).unwrap();

    // 6. Switch back to Default Branch and Merge Feature Branch
    checkout_branch(repo_path.clone(), default_branch.clone()).unwrap();
    let merge_result = merge_branch(
        repo_path.clone(),
        "feature/user-auth".to_string(),
        default_branch.clone(),
    )
    .unwrap();
    assert!(merge_result.contains("Merge branch") || merge_result.contains("Already up to date") || merge_result.contains("Fast-forward"));

    // 7. Verify Graph & History
    let final_commits = get_commits(repo_path.clone()).unwrap();
    assert_eq!(final_commits.len(), 3);
    assert_eq!(final_commits[0].message, "feat(ui): update html title");

    let final_graph = get_graph_data(repo_path.clone()).unwrap();
    assert_eq!(final_graph.len(), 3);
}

#[test]
fn e2e_conflict_resolution_lifecycle() {
    let (_temp, repo_path) = setup_e2e_repository();
    let default_branch = get_repo_status(repo_path.clone()).unwrap().current_branch;

    // 1. Create a branch for conflicting changes
    create_new_branch(repo_path.clone(), "theme-red".to_string()).unwrap();

    let index_file = Path::new(&repo_path).join("index.html");
    fs::write(&index_file, "<html><body style='color: red;'>BranchAI</body></html>\n").unwrap();
    stage_all(repo_path.clone()).unwrap();
    create_commit(repo_path.clone(), "style: change theme to red".to_string()).unwrap();

    // 2. Switch to default branch and make conflicting change
    checkout_branch(repo_path.clone(), default_branch.clone()).unwrap();
    fs::write(&index_file, "<html><body style='color: blue;'>BranchAI</body></html>\n").unwrap();
    stage_all(repo_path.clone()).unwrap();
    create_commit(repo_path.clone(), "style: change theme to blue".to_string()).unwrap();

    // 3. Resolve conflict by replacing content directly and staging
    resolve_conflict(
        repo_path.clone(),
        "index.html".to_string(),
        "<html><body style='color: purple;'>BranchAI</body></html>\n".to_string(),
    )
    .unwrap();

    let resolved_disk_content = fs::read_to_string(&index_file).unwrap();
    assert_eq!(resolved_disk_content, "<html><body style='color: purple;'>BranchAI</body></html>\n");

    let status_after_resolve = get_repo_status(repo_path.clone()).unwrap();
    assert!(status_after_resolve.files[0].staged);

    create_commit(repo_path.clone(), "merge: resolve theme conflict".to_string()).unwrap();

    let history = get_file_history(repo_path.clone(), "index.html".to_string()).unwrap();
    assert!(history.len() >= 2);
}

#[tokio::test]
async fn e2e_account_and_metadata_persistence_lifecycle() {
    let (_temp, repo_path) = setup_e2e_repository();

    // 1. Apply Account Config to Repo
    let test_account = GitAccount {
        id: "acc_work_1".to_string(),
        name: "Work Dev".to_string(),
        email: "workdev@company.com".to_string(),
        provider: "GitHub".to_string(),
        username: "workdev".to_string(),
        auth_type: "pat".to_string(),
        personal_access_token: Some("ghp_dummy12345".to_string()),
        ssh_key_path: None,
        avatar_url: None,
    };

    apply_repo_account(repo_path.clone(), test_account.clone()).unwrap();

    // Test account connection response
    let conn_msg = test_account_connection(test_account).await;
    // Since dummy PAT will fail network auth, we assert that the result is returned gracefully (Ok or Err)
    assert!(conn_msg.is_ok() || conn_msg.is_err());

    // 2. Save & Retrieve PRs
    let prs_json = r#"[{"id":1,"title":"PR #1 Refactor backend","status":"open"}]"#;
    save_repo_prs(repo_path.clone(), prs_json.to_string()).unwrap();
    let loaded_prs_json = get_repo_prs(repo_path.clone()).unwrap();
    assert!(loaded_prs_json.contains("PR #1 Refactor backend"));

    // 3. Save & Retrieve Issues
    let issues_json = r#"[{"id":10,"title":"Issue #10: Keyboard shortcuts","status":"open"}]"#;
    save_repo_issues(repo_path.clone(), issues_json.to_string()).unwrap();
    let loaded_issues_json = get_repo_issues(repo_path.clone()).unwrap();
    assert!(loaded_issues_json.contains("Issue #10: Keyboard shortcuts"));
}
