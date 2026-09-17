use branchai_lib::services::git::*;
use git2::{Repository, Signature};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper function to initialize a clean temporary Git repository with an initial commit on default branch.
fn create_test_repository() -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let repo = Repository::init(&repo_path).expect("Failed to initialize git repository");

    // Configure default signature
    let sig = Signature::now("Test User", "test@branchai.local").unwrap();

    // Create an initial file & commit
    let initial_file_path = temp_dir.path().join("README.md");
    fs::write(&initial_file_path, "# Test Repository\nInitial content\n").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(Path::new("README.md")).unwrap();
    let tree_id = index.write_tree().unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let _commit_id = repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    (temp_dir, repo_path)
}

#[test]
fn test_status_and_staging_workflow() {
    let (_temp, repo_path) = create_test_repository();

    // 1. Create a new untracked file
    let file_path = Path::new(&repo_path).join("src").join("main.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "fn main() { println!(\"Hello World\"); }\n").unwrap();

    // 2. Check repo status
    let status = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status.files.len(), 1);
    assert_eq!(status.files[0].path, "src/main.rs");
    assert_eq!(status.files[0].status, "untracked");
    assert!(!status.files[0].staged);

    // 3. Stage the file
    stage_file(repo_path.clone(), "src/main.rs".to_string()).unwrap();

    let status_after_stage = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_after_stage.files.len(), 1);
    assert!(status_after_stage.files[0].staged);

    // 4. Inspect file diff
    let diff = get_file_diff(repo_path.clone(), "src/main.rs".to_string(), true).unwrap();
    assert!(diff.contains("+fn main()"));

    // 5. Unstage the file
    unstage_file(repo_path.clone(), "src/main.rs".to_string()).unwrap();

    let status_after_unstage = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_after_unstage.files.len(), 1);
    assert!(!status_after_unstage.files[0].staged);

    // 6. Stage all & verify
    stage_all(repo_path.clone()).unwrap();
    let status_all_staged = get_repo_status(repo_path.clone()).unwrap();
    assert!(status_all_staged.files[0].staged);
}

#[test]
fn test_commit_creation_and_graph() {
    let (_temp, repo_path) = create_test_repository();

    // Create and stage a file
    let new_file = Path::new(&repo_path).join("app.js");
    fs::write(&new_file, "console.log('BranchAI');\n").unwrap();
    stage_file(repo_path.clone(), "app.js".to_string()).unwrap();

    // Create a commit via service
    let commit_id = create_commit(repo_path.clone(), "Add app.js script".to_string()).unwrap();
    assert!(!commit_id.is_empty());

    // Fetch commit history
    let commits = get_commits(repo_path.clone()).unwrap();
    assert!(commits.len() >= 2);
    assert_eq!(commits[0].message, "Add app.js script");

    // Check commit files
    let commit_files = get_commit_files(repo_path.clone(), commit_id.clone()).unwrap();
    assert_eq!(commit_files.len(), 1);
    assert_eq!(commit_files[0].path, "app.js");

    // Fetch graph data
    let graph_nodes = get_graph_data(repo_path.clone()).unwrap();
    assert!(graph_nodes.len() >= 2);
    assert_eq!(graph_nodes[0].message, "Add app.js script");

    // Test Undo Commit (Soft Reset)
    let undo_result = undo_commit(repo_path.clone()).unwrap();
    assert!(undo_result.contains("Undone commit"));

    let status_after_undo = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_after_undo.files.len(), 1);
    assert_eq!(status_after_undo.files[0].path, "app.js");
}

#[test]
fn test_branching_and_merging_services() {
    let (_temp, repo_path) = create_test_repository();
    let default_branch = get_repo_status(repo_path.clone()).unwrap().current_branch;

    // 1. Create a feature branch
    create_new_branch(repo_path.clone(), "feature/login".to_string()).unwrap();

    let branches = get_branches(repo_path.clone()).unwrap();
    assert!(branches.iter().any(|b| b.name == "feature/login"));

    // 2. Checkout feature branch
    checkout_branch(repo_path.clone(), "feature/login".to_string()).unwrap();

    let repo_status = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(repo_status.current_branch, "feature/login");

    // 3. Make a commit on feature branch
    let feature_file = Path::new(&repo_path).join("login.ts");
    fs::write(&feature_file, "export const login = () => true;\n").unwrap();
    stage_file(repo_path.clone(), "login.ts".to_string()).unwrap();
    create_commit(repo_path.clone(), "Implement login feature".to_string()).unwrap();

    // 4. Switch back to default branch
    checkout_branch(repo_path.clone(), default_branch.clone()).unwrap();

    // 5. Get branch diff summary (compare feature/login against default_branch)
    let diff_summary = get_branch_diff_summary(
        repo_path.clone(),
        "feature/login".to_string(),
        default_branch.clone(),
    )
    .unwrap();
    assert_eq!(diff_summary.commits.len(), 1);
    assert_eq!(diff_summary.files.len(), 1);

    // 6. Merge feature branch into default branch
    let merge_result = merge_branch(
        repo_path.clone(),
        "feature/login".to_string(),
        default_branch.clone(),
    )
    .unwrap();
    assert!(merge_result.contains("Merge branch") || merge_result.contains("Already up to date") || merge_result.contains("Fast-forward"));

    assert!(Path::new(&repo_path).join("login.ts").exists());
}

#[test]
fn test_stash_operations() {
    let (_temp, repo_path) = create_test_repository();

    // Modify existing file
    let readme_path = Path::new(&repo_path).join("README.md");
    fs::write(&readme_path, "# Test Repository\nModified for stash\n").unwrap();

    let status_before = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_before.files.len(), 1);

    // Stash save
    let stash_msg = stash_save(repo_path.clone()).unwrap();
    assert!(stash_msg.contains("Stashed all changes"));

    let status_after_stash = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_after_stash.files.len(), 0);

    // Stash pop
    let pop_msg = stash_pop(repo_path.clone()).unwrap();
    assert!(pop_msg.contains("Popped stashed changes"));

    let status_after_pop = get_repo_status(repo_path.clone()).unwrap();
    assert_eq!(status_after_pop.files.len(), 1);
}

#[test]
fn test_inspection_and_conflict_resolution() {
    let (_temp, repo_path) = create_test_repository();

    // Get file history for README.md
    let history = get_file_history(repo_path.clone(), "README.md".to_string()).unwrap();
    assert!(!history.is_empty());
    assert_eq!(history[0].message, "Initial commit");

    // Get file blame for README.md
    let blame = get_file_blame(repo_path.clone(), "README.md".to_string()).unwrap();
    assert!(!blame.is_empty());
    assert_eq!(blame[0].author, "Test User");

    // Test conflict resolution simulation
    let conflict_file = Path::new(&repo_path).join("conflict.txt");
    let conflict_content = "<<<<<<< HEAD\nLine A\n=======\nLine B\n>>>>>>> feature\n";
    fs::write(&conflict_file, conflict_content).unwrap();

    resolve_conflict(
        repo_path.clone(),
        "conflict.txt".to_string(),
        "Line A and Line B resolved\n".to_string(),
    )
    .unwrap();

    let resolved_disk_content = fs::read_to_string(&conflict_file).unwrap();
    assert_eq!(resolved_disk_content, "Line A and Line B resolved\n");
}

#[test]
fn test_prs_and_issues_persistence() {
    let (_temp, repo_path) = create_test_repository();

    let dummy_prs_json = r#"[{"id":101,"title":"Add feature X","status":"open"}]"#;

    // Save PRs
    save_repo_prs(repo_path.clone(), dummy_prs_json.to_string()).unwrap();

    // Fetch PRs
    let loaded_prs_json = get_repo_prs(repo_path.clone()).unwrap();
    assert!(loaded_prs_json.contains("Add feature X"));

    let dummy_issues_json = r#"[{"id":501,"title":"Bug in parser","status":"open"}]"#;

    // Save Issues
    save_repo_issues(repo_path.clone(), dummy_issues_json.to_string()).unwrap();

    // Fetch Issues
    let loaded_issues_json = get_repo_issues(repo_path.clone()).unwrap();
    assert!(loaded_issues_json.contains("Bug in parser"));
}
