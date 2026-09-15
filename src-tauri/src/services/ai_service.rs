use super::config_service::get_config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClaudeResponse {
    content: Vec<ClaudeContentBlock>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClaudeContentBlock {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPart>,
}

pub async fn call_llm(
    provider: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err(format!(
            "API Key for provider '{}' is missing. Please add your key in Settings.",
            provider
        ));
    }

    let client = reqwest::Client::new();

    match provider.to_lowercase().as_str() {
        "openai" => {
            let url = "https://api.openai.com/v1/chat/completions";
            let req = OpenAIRequest {
                model: if model.is_empty() {
                    "gpt-4o".to_string()
                } else {
                    model.to_string()
                },
                messages: vec![
                    OpenAIMessage {
                        role: "system".to_string(),
                        content: system_prompt.to_string(),
                    },
                    OpenAIMessage {
                        role: "user".to_string(),
                        content: user_prompt.to_string(),
                    },
                ],
            };

            let res = client
                .post(url)
                .bearer_auth(api_key)
                .json(&req)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !res.status().is_success() {
                let err_text = res.text().await.unwrap_or_default();
                return Err(format!("OpenAI API error: {}", err_text));
            }

            let resp: OpenAIResponse = res.json().await.map_err(|e| e.to_string())?;
            Ok(resp
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default())
        }

        "grok" => {
            let url = "https://api.x.ai/v1/chat/completions";
            let req = OpenAIRequest {
                model: if model.is_empty() {
                    "grok-2-latest".to_string()
                } else {
                    model.to_string()
                },
                messages: vec![
                    OpenAIMessage {
                        role: "system".to_string(),
                        content: system_prompt.to_string(),
                    },
                    OpenAIMessage {
                        role: "user".to_string(),
                        content: user_prompt.to_string(),
                    },
                ],
            };

            let res = client
                .post(url)
                .bearer_auth(api_key)
                .json(&req)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !res.status().is_success() {
                let err_text = res.text().await.unwrap_or_default();
                return Err(format!("xAI Grok API error: {}", err_text));
            }

            let resp: OpenAIResponse = res.json().await.map_err(|e| e.to_string())?;
            Ok(resp
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default())
        }

        "claude" => {
            let url = "https://api.anthropic.com/v1/messages";
            let req = ClaudeRequest {
                model: if model.is_empty() {
                    "claude-3-5-sonnet-20241022".to_string()
                } else {
                    model.to_string()
                },
                max_tokens: 2048,
                system: system_prompt.to_string(),
                messages: vec![ClaudeMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                }],
            };

            let res = client
                .post(url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&req)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !res.status().is_success() {
                let err_text = res.text().await.unwrap_or_default();
                return Err(format!("Anthropic Claude API error: {}", err_text));
            }

            let resp: ClaudeResponse = res.json().await.map_err(|e| e.to_string())?;
            Ok(resp
                .content
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_default())
        }

        "gemini" => {
            let selected_model = if model.is_empty() {
                "gemini-1.5-pro"
            } else {
                model
            };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                selected_model, api_key
            );

            let prompt_combined = format!("{}\n\n{}", system_prompt, user_prompt);
            let req = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart {
                        text: prompt_combined,
                    }],
                }],
            };

            let res = client
                .post(&url)
                .json(&req)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !res.status().is_success() {
                let err_text = res.text().await.unwrap_or_default();
                return Err(format!("Google Gemini API error: {}", err_text));
            }

            let resp: GeminiResponse = res.json().await.map_err(|e| e.to_string())?;
            let text = resp
                .candidates
                .and_then(|c| c.first().cloned())
                .and_then(|cand| cand.content.parts.first().map(|p| p.text.clone()))
                .unwrap_or_default();

            Ok(text)
        }

        _ => Err(format!("Unsupported AI Provider: {}", provider)),
    }
}

#[tauri::command]
pub async fn generate_commit_message(
    diff: String,
    custom_prompt: Option<String>,
) -> Result<String, String> {
    let cfg = get_config()?;
    let key = match cfg.active_provider.as_str() {
        "openai" => &cfg.openai_key,
        "claude" => &cfg.claude_key,
        "gemini" => &cfg.gemini_key,
        "grok" => &cfg.grok_key,
        _ => &cfg.openai_key,
    };

    let system_prompt = custom_prompt.unwrap_or(cfg.custom_prompt_commit);
    let user_prompt = format!(
        "Generate a commit message for the following diff:\n\n{}",
        diff
    );

    call_llm(
        &cfg.active_provider,
        key,
        &cfg.active_model,
        &system_prompt,
        &user_prompt,
    )
    .await
}

#[tauri::command]
pub async fn analyze_code_review(
    diff: String,
    custom_prompt: Option<String>,
) -> Result<String, String> {
    let cfg = get_config()?;
    let key = match cfg.active_provider.as_str() {
        "openai" => &cfg.openai_key,
        "claude" => &cfg.claude_key,
        "gemini" => &cfg.gemini_key,
        "grok" => &cfg.grok_key,
        _ => &cfg.openai_key,
    };

    let system_prompt = custom_prompt.unwrap_or(cfg.custom_prompt_review);
    let user_prompt = format!("Review the following code diff for potential bugs, security issues, style, and optimizations:\n\n{}", diff);

    call_llm(
        &cfg.active_provider,
        key,
        &cfg.active_model,
        &system_prompt,
        &user_prompt,
    )
    .await
}

#[tauri::command]
pub async fn assist_conflict_resolution(
    file_path: String,
    conflict_content: String,
    custom_prompt: Option<String>,
) -> Result<String, String> {
    let cfg = get_config()?;
    let key = match cfg.active_provider.as_str() {
        "openai" => &cfg.openai_key,
        "claude" => &cfg.claude_key,
        "gemini" => &cfg.gemini_key,
        "grok" => &cfg.grok_key,
        _ => &cfg.openai_key,
    };

    let system_prompt = custom_prompt.unwrap_or(cfg.custom_prompt_conflict);
    let user_prompt = format!(
        "File: {}\n\nResolve the conflict markers in the following code block:\n\n{}",
        file_path, conflict_content
    );

    call_llm(
        &cfg.active_provider,
        key,
        &cfg.active_model,
        &system_prompt,
        &user_prompt,
    )
    .await
}

#[tauri::command]
pub async fn test_ai_connection(
    provider: String,
    api_key: String,
    model: String,
) -> Result<String, String> {
    let sys = "You are a test ping agent. Reply with 'Connection successful!'.";
    let user = "Ping test.";
    call_llm(&provider, &api_key, &model, sys, user).await
}
