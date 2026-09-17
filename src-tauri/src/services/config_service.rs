use crate::models::AppConfig;
use std::fs;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.active_provider, "openai");
        assert_eq!(cfg.active_model, "gpt-4o");
        assert!(cfg.accounts.is_empty());
    }

    #[test]
    fn test_save_and_load_custom_config() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.json");

        let mut custom_cfg = AppConfig::default();
        custom_cfg.active_provider = "claude".to_string();
        custom_cfg.active_model = "claude-3-5-sonnet".to_string();

        let json = serde_json::to_string_pretty(&custom_cfg).unwrap();
        fs::write(&config_path, &json).unwrap();

        let read_content = fs::read_to_string(&config_path).unwrap();
        let loaded_cfg: AppConfig = serde_json::from_str(&read_content).unwrap();

        assert_eq!(loaded_cfg.active_provider, "claude");
        assert_eq!(loaded_cfg.active_model, "claude-3-5-sonnet");
    }
}
