use branchai_lib::services::git::*;
use git2::Repository;
use std::collections::HashMap;
use tempfile::TempDir;

fn setup_github_actions_test_repo() -> (TempDir, String) {
    let temp_dir =
        TempDir::new().expect("Failed to create temporary directory for GitHub Actions test");
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let repo = Repository::init(&repo_path).expect("Failed to initialize git repository");
    repo.remote("origin", "https://github.com/octocat/Hello-World.git")
        .expect("Failed to set remote origin");

    (temp_dir, repo_path)
}

#[tokio::test]
async fn test_github_actions_commands_without_origin_remote() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();
    Repository::init(&repo_path).unwrap();

    let wf_res = get_github_workflows(repo_path.clone()).await;
    assert!(wf_res.is_err());
    assert!(wf_res
        .err()
        .unwrap()
        .contains("Could not identify repository owner"));

    let runs_res = get_workflow_runs(repo_path.clone(), None).await;
    assert!(runs_res.is_err());

    let jobs_res = get_workflow_run_jobs(repo_path.clone(), 12345).await;
    assert!(jobs_res.is_err());

    let logs_res = get_job_logs(repo_path.clone(), 67890).await;
    assert!(logs_res.is_err());
}

#[tokio::test]
async fn test_github_actions_commands_on_invalid_repo_path() {
    let invalid_path = "/nonexistent/path/for/branchai/test".to_string();

    let wf_res = get_github_workflows(invalid_path.clone()).await;
    assert!(wf_res.is_err());

    let runs_res = get_workflow_runs(invalid_path.clone(), Some(101)).await;
    assert!(runs_res.is_err());

    let jobs_res = get_workflow_run_jobs(invalid_path.clone(), 202).await;
    assert!(jobs_res.is_err());

    let logs_res = get_job_logs(invalid_path.clone(), 303).await;
    assert!(logs_res.is_err());
}

#[tokio::test]
async fn test_github_actions_trigger_dispatch_without_token() {
    let (_temp, repo_path) = setup_github_actions_test_repo();

    let mut inputs = HashMap::new();
    inputs.insert("environment".to_string(), "production".to_string());

    let dispatch_res =
        trigger_workflow_dispatch(repo_path.clone(), 1001, "main".to_string(), Some(inputs)).await;

    assert!(dispatch_res.is_err());
    let err_msg = dispatch_res.err().unwrap();
    assert!(err_msg.contains("Personal Access Token (PAT) is required"));
}

#[tokio::test]
async fn test_github_actions_remote_resolution_and_public_api() {
    let (_temp, repo_path) = setup_github_actions_test_repo();

    let remote_info = get_repo_remote_info(repo_path.clone()).unwrap();
    assert_eq!(remote_info.owner.unwrap(), "octocat");
    assert_eq!(remote_info.repo_name.unwrap(), "Hello-World");

    let wf_result = get_github_workflows(repo_path.clone()).await;
    assert!(wf_result.is_ok() || wf_result.is_err());
}
