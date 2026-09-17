use crate::models::GitAccount;
use crate::services::git::utils::{expand_home_dir, open_repo};
use std::path::Path;
use std::process::Command;

#[tauri::command]
pub fn apply_repo_account(repo_path: String, account: GitAccount) -> Result<(), String> {
    let repo = open_repo(&repo_path)?;
    let mut config = repo.config().map_err(|e| e.message().to_string())?;

    if !account.username.is_empty() {
        config
            .set_str("user.name", &account.username)
            .map_err(|e| e.message().to_string())?;
    }
    if !account.email.is_empty() {
        config
            .set_str("user.email", &account.email)
            .map_err(|e| e.message().to_string())?;
    }

    if account.auth_type == "ssh" {
        if let Some(ref key_path) = account.ssh_key_path {
            if !key_path.trim().is_empty() {
                let clean_key_path = expand_home_dir(key_path);
                let ssh_cmd = format!("ssh -i \"{}\" -o IdentitiesOnly=yes", clean_key_path);
                config
                    .set_str("core.sshCommand", &ssh_cmd)
                    .map_err(|e| e.message().to_string())?;
            }
        }
    } else {
        let _ = config.remove("core.sshCommand");
    }

    Ok(())
}

#[tauri::command]
pub async fn test_account_connection(account: GitAccount) -> Result<String, String> {
    let host = match account.provider.to_lowercase().as_str() {
        "github" => "github.com",
        "gitlab" => "gitlab.com",
        "bitbucket" => "bitbucket.org",
        _ => "github.com",
    };

    if account.auth_type == "ssh" {
        let key_path = account.ssh_key_path.clone().unwrap_or_default();
        if key_path.trim().is_empty() {
            return Err("SSH Key file path is empty. Please select or enter your private SSH key path.".to_string());
        }

        let clean_path = expand_home_dir(&key_path);
        if !Path::new(&clean_path).exists() {
            return Err(format!("SSH Key file not found at: {}", clean_path));
        }

        let output = Command::new("ssh")
            .args([
                "-T",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "-o",
                "ConnectTimeout=10",
                "-i",
                &clean_path,
                &format!("git@{}", host),
            ])
            .output()
            .map_err(|e| format!("Failed to execute ssh command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);

        if combined.contains("successfully authenticated")
            || combined.contains("Welcome to GitLab")
            || combined.contains("logged in as")
            || combined.contains("authenticated")
            || output.status.success()
        {
            Ok(format!("SSH Connection Successful!\n{}", combined.trim()))
        } else {
            Err(format!("SSH Connection test output:\n{}", combined.trim()))
        }
    } else if account.auth_type == "pat" || account.auth_type == "https" {
        let token = account.personal_access_token.clone().unwrap_or_default();
        if token.trim().is_empty() {
            return Err("Personal Access Token (PAT) is empty.".to_string());
        }

        let url = match account.provider.to_lowercase().as_str() {
            "github" => "https://api.github.com/user",
            "gitlab" => "https://gitlab.com/api/v4/user",
            _ => "https://api.github.com/user",
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("BranchAI-Git-Client")
            .build()
            .map_err(|e| e.to_string())?;

        let req = client.get(url).header("Authorization", format!("token {}", token));
        let res = req
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if res.status().is_success() {
            Ok(format!("Token Authentication Successful! Server status: {}", res.status()))
        } else {
            Err(format!("Authentication failed with server status: {}", res.status()))
        }
    } else {
        Ok("Account settings configured successfully.".to_string())
    }
}
