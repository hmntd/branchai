use crate::models::GitAccount;
use crate::services::config_service::get_config;
use crate::services::git::remote::get_repo_remote_info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GithubWorkflow {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub state: String,
    pub html_url: Option<String>,
}

#[derive(Deserialize)]
struct WorkflowsResponse {
    workflows: Vec<GithubWorkflowApi>,
}

#[derive(Deserialize)]
struct GithubWorkflowApi {
    id: u64,
    name: String,
    path: String,
    state: String,
    html_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GithubWorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub head_branch: Option<String>,
    pub head_sha: Option<String>,
    pub event: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub run_number: u64,
    pub actor_login: Option<String>,
    pub actor_avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct WorkflowRunsResponse {
    workflow_runs: Vec<GithubWorkflowRunApi>,
}

#[derive(Deserialize)]
struct GithubWorkflowRunApi {
    id: u64,
    name: Option<String>,
    head_branch: Option<String>,
    head_sha: Option<String>,
    event: String,
    status: String,
    conclusion: Option<String>,
    html_url: String,
    created_at: String,
    updated_at: String,
    run_number: u64,
    actor: Option<GithubActorApi>,
}

#[derive(Deserialize)]
struct GithubActorApi {
    login: String,
    avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GithubStep {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub number: u64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GithubJob {
    pub id: u64,
    pub run_id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub steps: Vec<GithubStep>,
}

#[derive(Deserialize)]
struct JobsResponse {
    jobs: Vec<GithubJobApi>,
}

#[derive(Deserialize)]
struct GithubJobApi {
    id: u64,
    run_id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
    steps: Vec<GithubStep>,
}

#[derive(Serialize)]
struct WorkflowDispatchRequest {
    #[serde(rename = "ref")]
    ref_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    inputs: Option<HashMap<String, String>>,
}

struct RepoContext {
    owner: String,
    repo_name: String,
    token: Option<String>,
}

fn resolve_repo_context(repo_path: &str) -> Result<RepoContext, String> {
    let remote_info = get_repo_remote_info(repo_path.to_string())?;
    let owner = remote_info.owner.ok_or_else(|| {
        "Could not identify repository owner from Git remote origin URL.".to_string()
    })?;
    let repo_name = remote_info.repo_name.ok_or_else(|| {
        "Could not identify repository name from Git remote origin URL.".to_string()
    })?;

    let mut token = None;
    if let Ok(config) = get_config() {
        let mapped_acc_id = config.repo_account_mappings.get(repo_path);
        let active_acc: Option<&GitAccount> = if let Some(acc_id) = mapped_acc_id {
            config.accounts.iter().find(|a| &a.id == acc_id)
        } else {
            config.accounts.first()
        };

        if let Some(acc) = active_acc {
            if let Some(ref pat) = acc.personal_access_token {
                if !pat.trim().is_empty() {
                    token = Some(pat.trim().to_string());
                }
            }
        }
    }

    Ok(RepoContext {
        owner,
        repo_name,
        token,
    })
}

fn build_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("BranchAI-Git-Client")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

fn apply_headers(builder: reqwest::RequestBuilder, ctx: &RepoContext) -> reqwest::RequestBuilder {
    let mut req = builder.header("Accept", "application/vnd.github.v3+json");
    if let Some(ref token) = ctx.token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    req
}

#[tauri::command]
pub async fn get_github_workflows(repo_path: String) -> Result<Vec<GithubWorkflow>, String> {
    let ctx = resolve_repo_context(&repo_path)?;
    let client = build_http_client()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/workflows",
        ctx.owner, ctx.repo_name
    );

    let req = apply_headers(client.get(&url), &ctx);
    let res = req
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;
    let status = res.status();

    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, err_text));
    }

    let parsed: WorkflowsResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse workflows JSON: {}", e))?;

    let workflows = parsed
        .workflows
        .into_iter()
        .map(|w| GithubWorkflow {
            id: w.id,
            name: w.name,
            path: w.path,
            state: w.state,
            html_url: w.html_url,
        })
        .collect();

    Ok(workflows)
}

#[tauri::command]
pub async fn get_workflow_runs(
    repo_path: String,
    workflow_id: Option<u64>,
) -> Result<Vec<GithubWorkflowRun>, String> {
    let ctx = resolve_repo_context(&repo_path)?;
    let client = build_http_client()?;

    let url = if let Some(wid) = workflow_id {
        format!(
            "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs?per_page=30",
            ctx.owner, ctx.repo_name, wid
        )
    } else {
        format!(
            "https://api.github.com/repos/{}/{}/actions/runs?per_page=30",
            ctx.owner, ctx.repo_name
        )
    };

    let req = apply_headers(client.get(&url), &ctx);
    let res = req
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;
    let status = res.status();

    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, err_text));
    }

    let parsed: WorkflowRunsResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse workflow runs JSON: {}", e))?;

    let runs = parsed
        .workflow_runs
        .into_iter()
        .map(|r| {
            let (actor_login, actor_avatar_url) = if let Some(a) = r.actor {
                (Some(a.login), a.avatar_url)
            } else {
                (None, None)
            };

            GithubWorkflowRun {
                id: r.id,
                name: r.name,
                head_branch: r.head_branch,
                head_sha: r.head_sha,
                event: r.event,
                status: r.status,
                conclusion: r.conclusion,
                html_url: r.html_url,
                created_at: r.created_at,
                updated_at: r.updated_at,
                run_number: r.run_number,
                actor_login,
                actor_avatar_url,
            }
        })
        .collect();

    Ok(runs)
}

#[tauri::command]
pub async fn get_workflow_run_jobs(
    repo_path: String,
    run_id: u64,
) -> Result<Vec<GithubJob>, String> {
    let ctx = resolve_repo_context(&repo_path)?;
    let client = build_http_client()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
        ctx.owner, ctx.repo_name, run_id
    );

    let req = apply_headers(client.get(&url), &ctx);
    let res = req
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;
    let status = res.status();

    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, err_text));
    }

    let parsed: JobsResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse jobs JSON: {}", e))?;

    let jobs = parsed
        .jobs
        .into_iter()
        .map(|j| GithubJob {
            id: j.id,
            run_id: j.run_id,
            name: j.name,
            status: j.status,
            conclusion: j.conclusion,
            started_at: j.started_at,
            completed_at: j.completed_at,
            steps: j.steps,
        })
        .collect();

    Ok(jobs)
}

#[tauri::command]
pub async fn get_job_logs(repo_path: String, job_id: u64) -> Result<String, String> {
    let ctx = resolve_repo_context(&repo_path)?;
    let client = build_http_client()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/jobs/{}/logs",
        ctx.owner, ctx.repo_name, job_id
    );

    let req = apply_headers(client.get(&url), &ctx);
    let res = req
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;
    let status = res.status();

    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!(
            "Failed to fetch job logs ({}): {}",
            status, err_text
        ));
    }

    let logs = res
        .text()
        .await
        .map_err(|e| format!("Failed to read log stream: {}", e))?;
    Ok(logs)
}

#[tauri::command]
pub async fn trigger_workflow_dispatch(
    repo_path: String,
    workflow_id: u64,
    ref_name: String,
    inputs: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let ctx = resolve_repo_context(&repo_path)?;
    if ctx.token.is_none() {
        return Err("GitHub Personal Access Token (PAT) is required to trigger workflows. Please add a PAT in Settings -> Accounts.".to_string());
    }

    let client = build_http_client()?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches",
        ctx.owner, ctx.repo_name, workflow_id
    );

    let payload = WorkflowDispatchRequest {
        ref_name: if ref_name.trim().is_empty() {
            "main".to_string()
        } else {
            ref_name.trim().to_string()
        },
        inputs,
    };

    let req = apply_headers(client.post(&url), &ctx).json(&payload);
    let res = req
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;
    let status = res.status();

    if status.is_success() || status == reqwest::StatusCode::NO_CONTENT {
        Ok("Workflow triggered successfully!".to_string())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!(
            "Failed to trigger workflow ({}): {}",
            status, err_text
        ))
    }
}
