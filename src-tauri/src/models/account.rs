use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GitAccount {
    pub id: String,
    pub name: String,
    pub provider: String, // "github", "gitlab", "bitbucket", "custom"
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub auth_type: String, // "ssh", "pat", "https"
    #[serde(default)]
    pub ssh_key_path: Option<String>,
    #[serde(default)]
    pub personal_access_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub openai_key: String,
    pub claude_key: String,
    pub gemini_key: String,
    pub grok_key: String,
    pub active_provider: String,
    pub active_model: String,
    pub custom_prompt_commit: String,
    pub custom_prompt_review: String,
    pub custom_prompt_conflict: String,
    #[serde(default)]
    pub last_opened_repo: Option<String>,
    #[serde(default)]
    pub recent_repos: Vec<String>,
    #[serde(default)]
    pub accounts: Vec<GitAccount>,
    #[serde(default)]
    pub repo_account_mappings: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            openai_key: String::new(),
            claude_key: String::new(),
            gemini_key: String::new(),
            grok_key: String::new(),
            active_provider: "openai".to_string(),
            active_model: "gpt-4o".to_string(),
            custom_prompt_commit: "Write a concise, high-quality git commit message following Conventional Commits format (e.g., feat:, fix:, refactor:). Include a short subject line (<50 chars) and a detailed body explaining 'why' if necessary. Return ONLY the commit message text.".to_string(),
            custom_prompt_review: "Perform a thorough code review on the provided Git diff. Identify bugs, security issues, performance bottlenecks, and style improvements. Group findings logically with clear line numbers or hunk context.".to_string(),
            custom_prompt_conflict: "Analyze the conflicting code blocks from Git merge conflict markers. Propose a clean, merged resolution that preserves functional intent from both sides without breaking syntax. Return the resolved code file.".to_string(),
            last_opened_repo: None,
            recent_repos: Vec::new(),
            accounts: Vec::new(),
            repo_account_mappings: HashMap::new(),
        }
    }
}
