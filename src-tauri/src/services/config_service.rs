use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut dir = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    dir.push(".branchai");
    let _ = fs::create_dir_all(&dir);
    dir.push("config.json");
    dir
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[tauri::command]
pub fn get_config() -> Result<AppConfig, String> {
    let path = get_config_path();
    if !path.exists() {
        let default_cfg = AppConfig::default();
        let _ = save_config(default_cfg.clone());
        return Ok(default_cfg);
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).unwrap_or_else(|_| AppConfig::default());
    Ok(config)
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}
