use crate::openclaw::models::*;
use crate::{
    ensure_openclaw_chat_completions_endpoint_enabled, load_agent_context_messages,
    normalize_local_openclaw_chat_endpoint, resolve_agent_model_from_config,
    resolve_default_openclaw_api_url, resolve_openclaw_gateway_token,
    should_try_enable_chat_completions_endpoint,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

fn is_openai_compatible_endpoint(endpoint: &str) -> bool {
    let normalized = endpoint.trim_end_matches('/').to_ascii_lowercase();
    normalized.ends_with("/v1/chat/completions") || normalized.ends_with("/chat/completions")
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
pub(crate) async fn openclaw_chat(
    messages: Vec<OpenClawMessage>,
    endpoint: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    protocol: Option<String>,
    agent_id: Option<String>,
    session_key: Option<String>,
) -> Result<OpenClawResponse, String> {
    let effective_agent_id = agent_id.as_deref().map(str::trim).filter(|v| !v.is_empty());

    let mut final_messages = Vec::new();
    if let Some(aid) = effective_agent_id {
        final_messages.extend(load_agent_context_messages(aid));
    }
    final_messages.extend(messages);

    let endpoint = endpoint
        .filter(|value| !value.trim().is_empty())
        .or_else(resolve_default_openclaw_api_url)
        .ok_or_else(|| "未设置可用的聊天接口地址。".to_string())?;
    let request_protocol = protocol
        .unwrap_or_else(|| "openai".to_string())
        .to_lowercase();
    let endpoint = if request_protocol == "openai" {
        normalize_local_openclaw_chat_endpoint(&endpoint)
    } else {
        endpoint
    };
    let should_try_enable_chat_completions =
        request_protocol == "openai" && should_try_enable_chat_completions_endpoint(&endpoint);
    let mut chat_completions_enable_error: Option<String> = None;
    if should_try_enable_chat_completions {
        if let Err(error) = ensure_openclaw_chat_completions_endpoint_enabled() {
            chat_completions_enable_error = Some(error);
        }
    }
    let is_openai_compatible = is_openai_compatible_endpoint(&endpoint);
    let gateway_token = resolve_openclaw_gateway_token();
    let api_key = api_key.filter(|value| !value.trim().is_empty());
    let session_key = session_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let agent_id_owned = effective_agent_id.map(|s| s.to_string());

    let mut model = model
        .filter(|value| !value.trim().is_empty())
        .or_else(|| effective_agent_id.and_then(resolve_agent_model_from_config))
        .or_else(|| std::env::var("OPENCLAW_MODEL").ok());
    if request_protocol == "openai" && should_try_enable_chat_completions {
        let has_openclaw_model = model
            .as_deref()
            .map(str::trim)
            .map(|value| {
                let lower = value.to_ascii_lowercase();
                lower == "openclaw" || lower.starts_with("openclaw/")
            })
            .unwrap_or(false);
        if !has_openclaw_model {
            model = Some(
                agent_id_owned
                    .as_deref()
                    .map(|aid| format!("openclaw/{aid}"))
                    .unwrap_or_else(|| "openclaw".to_string()),
            );
        }
    }

    let client = reqwest::Client::new();
    let mut request = client
        .post(&endpoint)
        .header(CONTENT_TYPE, "application/json");

    if let Some(aid) = agent_id_owned.as_deref() {
        request = request.header("X-OpenClaw-Agent-Id", aid);
    }
    if request_protocol == "openai" && should_try_enable_chat_completions {
        // OpenClaw gateway HTTP API (2026.3.x) requires explicit operator scopes
        // via x-openclaw-scopes for guarded methods such as chat.send.
        request = request.header("x-openclaw-scopes", "operator.write");
        if let Some(session_key) = session_key.as_deref() {
            request = request.header("x-openclaw-session-key", session_key);
        }
    }

    if request_protocol == "anthropic" {
        if let Some(api_key) = api_key.as_deref().filter(|value| !value.trim().is_empty()) {
            request = request.header("x-api-key", api_key);
        }
        request = request.header("anthropic-version", "2023-06-01");
    } else if let Some(api_key) = api_key
        .as_deref()
        .filter(|api_key| !api_key.trim().is_empty())
    {
        request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
    } else if let Some(token) = gateway_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    request = if request_protocol == "anthropic" {
        let model = model.ok_or_else(|| "Anthropic 协议需要模型配置。".to_string())?;
        let system = final_messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        let anthropic_messages = final_messages
            .into_iter()
            .filter(|message| message.role != "system")
            .map(|message| AnthropicMessage {
                role: if message.role == "assistant" {
                    "assistant".to_string()
                } else {
                    "user".to_string()
                },
                content: message.content,
            })
            .collect::<Vec<_>>();

        request.json(&AnthropicRequest {
            model,
            max_tokens: 1024,
            system: if system.is_empty() {
                None
            } else {
                Some(system)
            },
            messages: anthropic_messages,
        })
    } else if is_openai_compatible {
        request.json(&OpenAiChatRequest {
            model,
            messages: final_messages,
            agent_id: agent_id_owned.clone(),
        })
    } else {
        request.json(&OpenClawRequest {
            messages: final_messages,
            agent_id: agent_id_owned.clone(),
        })
    };

    let response = request
        .send()
        .await
        .map_err(|error| format!("请求 OpenClaw 失败: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let mut body = response
            .text()
            .await
            .unwrap_or_else(|_| "未返回错误详情".to_string());
        if status.as_u16() == 404
            && request_protocol == "openai"
            && should_try_enable_chat_completions
        {
            let hint = if let Some(error) = chat_completions_enable_error.as_deref() {
                format!(
                    "提示：已尝试自动启用 gateway.http.endpoints.chatCompletions.enabled，但失败：{error}"
                )
            } else {
                "提示：请确认 openclaw.json 中 gateway.http.endpoints.chatCompletions.enabled=true。"
                    .to_string()
            };
            if body.trim().is_empty() {
                body = hint;
            } else {
                body = format!("{body}\n{hint}");
            }
        }
        return Err(format!(
            "OpenClaw 返回错误状态 {status}（endpoint: {endpoint}）: {body}"
        ));
    }

    let (text, raw, usage) = if request_protocol == "anthropic" {
        let payload = response
            .json::<AnthropicResponse>()
            .await
            .map_err(|error| format!("解析 Anthropic 响应失败: {error}"))?;

        let text = payload
            .content
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| item.block_type.as_deref() == Some("text"))
            .filter_map(|item| item.text)
            .collect::<Vec<_>>()
            .join("\n");
        let raw = serde_json::to_string_pretty(&payload).ok();
        let usage = payload
            .usage
            .as_ref()
            .and_then(|value| serde_json::to_value(value).ok());

        (text, raw, usage)
    } else if is_openai_compatible {
        let payload = response
            .json::<OpenAiChatResponse>()
            .await
            .map_err(|error| format!("解析 OpenClaw Gateway 响应失败: {error}"))?;

        let text = payload
            .choices
            .clone()
            .and_then(|choices| choices.into_iter().next())
            .and_then(|choice| choice.message)
            .and_then(|message| message.content)
            .and_then(|content| extract_openai_content(&content))
            .unwrap_or_else(|| "OpenClaw Gateway 返回了空内容。".to_string());
        let raw = serde_json::to_string_pretty(&payload).ok();
        let usage = payload
            .usage
            .as_ref()
            .and_then(|value| serde_json::to_value(value).ok());

        (text, raw, usage)
    } else {
        let payload = response
            .json::<OpenClawHttpResponse>()
            .await
            .map_err(|error| format!("解析 OpenClaw 响应失败: {error}"))?;

        let text = payload
            .text
            .clone()
            .or(payload.content.clone())
            .unwrap_or_else(|| "OpenClaw 返回了空内容。".to_string());
        let raw = serde_json::to_string_pretty(&payload).ok();

        (text, raw, None)
    };

    Ok(OpenClawResponse {
        text: if text.is_empty() {
            "接口返回了空内容。".to_string()
        } else {
            text
        },
        raw,
        usage,
    })
}
