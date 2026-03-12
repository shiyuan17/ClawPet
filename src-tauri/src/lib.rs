use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenClawMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenClawRequest {
    messages: Vec<OpenClawMessage>,
}

#[derive(Debug, Deserialize)]
struct OpenClawHttpResponse {
    text: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    content: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    choices: Option<Vec<OpenAiChoice>>,
}

#[derive(Debug, Serialize)]
struct OpenClawResponse {
    text: String,
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenClawMessage>,
}

fn load_env_file(path: &Path) {
    if path.exists() {
        let _ = dotenvy::from_path(path);
    }
}

fn load_openclaw_env() {
    if let Ok(current_dir) = std::env::current_dir() {
        load_env_file(&current_dir.join(".env"));
        load_env_file(&current_dir.join("../.env"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    load_env_file(&manifest_dir.join(".env"));
    if let Some(workspace_dir) = manifest_dir.parent() {
        load_env_file(&workspace_dir.join(".env"));
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            load_env_file(&exe_dir.join(".env"));
            load_env_file(&exe_dir.join("../.env"));
        }
    }
}

fn is_openai_compatible_endpoint(endpoint: &str) -> bool {
    endpoint
        .trim_end_matches('/')
        .ends_with("/v1/chat/completions")
}

fn extract_openai_content(content: &serde_json::Value) -> Option<String> {
    match content {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(|item| item.get("text").and_then(|value| value.as_str()))
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
async fn openclaw_chat(messages: Vec<OpenClawMessage>) -> Result<OpenClawResponse, String> {
    let endpoint = std::env::var("OPENCLAW_API_URL")
        .map_err(|_| "未设置 OPENCLAW_API_URL，无法从桌宠原生命令调用 OpenClaw。".to_string())?;
    let is_openai_compatible = is_openai_compatible_endpoint(&endpoint);
    let gateway_token = std::env::var("OPENCLAW_GATEWAY_TOKEN").ok();
    let api_key = std::env::var("OPENCLAW_API_KEY").ok();
    let model = std::env::var("OPENCLAW_MODEL").unwrap_or_else(|_| "openclaw".to_string());

    let client = reqwest::Client::new();
    let mut request = client
        .post(endpoint)
        .header(CONTENT_TYPE, "application/json");

    if let Some(token) = gateway_token.as_deref().filter(|token| !token.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    } else if let Some(api_key) = api_key.as_deref().filter(|api_key| !api_key.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
    }

    request = if is_openai_compatible {
        request.json(&OpenAiChatRequest { model, messages })
    } else {
        request.json(&OpenClawRequest { messages })
    };

    let response = request
        .send()
        .await
        .map_err(|error| format!("请求 OpenClaw 失败: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "未返回错误详情".to_string());
        return Err(format!("OpenClaw 返回错误状态 {status}: {body}"));
    }

    let text = if is_openai_compatible {
        let payload = response
            .json::<OpenAiChatResponse>()
            .await
            .map_err(|error| format!("解析 OpenClaw Gateway 响应失败: {error}"))?;

        payload
            .choices
            .and_then(|choices| choices.into_iter().next())
            .and_then(|choice| choice.message)
            .and_then(|message| message.content)
            .and_then(|content| extract_openai_content(&content))
            .unwrap_or_else(|| "OpenClaw Gateway 返回了空内容。".to_string())
    } else {
        let payload = response
            .json::<OpenClawHttpResponse>()
            .await
            .map_err(|error| format!("解析 OpenClaw 响应失败: {error}"))?;

        payload
            .text
            .or(payload.content)
            .unwrap_or_else(|| "OpenClaw 返回了空内容。".to_string())
    };

    Ok(OpenClawResponse { text })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_openclaw_env();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![quit_app, openclaw_chat])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_always_on_top(true);
                let _ = window.set_shadow(false);
                let _ = window.set_skip_taskbar(false);
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let size = monitor.size();
                    let position = monitor.position();
                    let _ = window.set_position(tauri::Position::Physical(*position));
                    let _ = window.set_size(tauri::Size::Physical(*size));
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
