use getrandom::getrandom;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(target_os = "windows")]
pub(crate) fn suppress_windows_command_window(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn suppress_windows_command_window(command: &mut Command) -> &mut Command {
    command
}

#[cfg(target_os = "windows")]
pub(crate) fn normalize_windows_path_for_child_process(path: &Path) -> PathBuf {
    let raw = path.as_os_str().to_string_lossy();
    if let Some(stripped) = raw.strip_prefix("\\\\?\\UNC\\") {
        return PathBuf::from(format!("\\\\{stripped}"));
    }
    if let Some(stripped) = raw.strip_prefix("\\\\?\\") {
        return PathBuf::from(stripped);
    }
    path.to_path_buf()
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn normalize_windows_path_for_child_process(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub(crate) fn load_openclaw_gateway_token_from_config() -> Option<String> {
    let config_path = crate::resolve_openclaw_config_path();
    let config_text = std::fs::read_to_string(config_path).ok()?;
    let config = serde_json::from_str::<serde_json::Value>(&config_text).ok()?;
    let auth = config.get("gateway")?.get("auth")?;
    let mode = auth.get("mode")?.as_str()?;
    if mode != "token" {
        return None;
    }

    auth.get("token")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

pub(crate) fn generate_ephemeral_openclaw_gateway_token() -> Result<String, String> {
    let mut random_bytes = [0u8; 24];
    getrandom(&mut random_bytes).map_err(|error| format!("生成网关令牌失败: {error}"))?;
    Ok(bytes_to_lower_hex(&random_bytes))
}

pub(crate) fn resolve_openclaw_gateway_token() -> Option<String> {
    std::env::var("OPENCLAW_GATEWAY_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(load_openclaw_gateway_token_from_config)
}

pub(crate) fn resolve_openclaw_gateway_token_for_onboard() -> Result<String, String> {
    if let Some(token) = resolve_openclaw_gateway_token() {
        return Ok(token);
    }
    generate_ephemeral_openclaw_gateway_token()
}

pub(crate) fn official_openclaw_install_hint_for_platform() -> String {
    if cfg!(target_os = "windows") {
        return "请先按官方安装脚本安装 OpenClaw（Windows PowerShell）：`iwr -useb https://openclaw.ai/install.ps1 | iex`；安装后执行 `openclaw onboard --install-daemon`。".to_string();
    }
    "请先按官方安装脚本安装 OpenClaw（macOS/Linux/WSL2）：`curl -fsSL https://openclaw.ai/install.sh | bash`；安装后执行 `openclaw onboard --install-daemon`。".to_string()
}

pub(crate) fn load_staff_mission_statement() -> String {
    let fallback = "构建可持续自治的 AI 员工体系，持续完成高价值任务。".to_string();
    let current_dir = match std::env::current_dir() {
        Ok(value) => value,
        Err(_) => return fallback,
    };
    let agent_path = current_dir.join("AGENTS.md");
    let raw = match std::fs::read_to_string(agent_path) {
        Ok(value) => value,
        Err(_) => return fallback,
    };

    raw.lines()
        .map(str::trim)
        .find(|line| line.starts_with("- ") && line.to_ascii_lowercase().contains("objective"))
        .map(|line| line.trim_start_matches('-').trim().to_string())
        .filter(|line| line.len() > 8)
        .unwrap_or(fallback)
}

pub(crate) fn value_as_object(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    value.as_object()
}

pub(crate) fn read_string_or_primary<'a>(value: &'a Value) -> Option<&'a str> {
    value
        .as_str()
        .or_else(|| {
            value_as_object(value)
                .and_then(|obj| obj.get("primary"))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn normalize_provider_id(raw: &str) -> String {
    let mut cleaned = String::new();
    for ch in raw.trim().chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            cleaned.push(ch.to_ascii_lowercase());
        } else if ch == '/' || ch == ':' || ch == '.' || ch.is_whitespace() {
            cleaned.push('-');
        }
    }

    let normalized = cleaned
        .trim_matches('-')
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if normalized.is_empty() {
        "platform".to_string()
    } else {
        normalized
    }
}

pub(crate) fn humanize_provider_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "未命名平台".to_string();
    }

    let parts = trimmed
        .split(|ch: char| ch == '-' || ch == '_' || ch == '/' || ch == ':' || ch == '.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return trimmed.to_string();
    }

    let words = parts
        .into_iter()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            match lower.as_str() {
                "openai" => "OpenAI".to_string(),
                "api" => "API".to_string(),
                "ai" => "AI".to_string(),
                "aws" => "AWS".to_string(),
                _ => {
                    if !part.is_ascii() {
                        return part.to_string();
                    }
                    let mut chars = lower.chars();
                    match chars.next() {
                        Some(first) => {
                            let mut word = first.to_ascii_uppercase().to_string();
                            word.push_str(chars.as_str());
                            word
                        }
                        None => String::new(),
                    }
                }
            }
        })
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    if words.is_empty() {
        trimmed.to_string()
    } else {
        words.join(" ")
    }
}

#[allow(dead_code)]
pub(crate) fn infer_provider_name_from_base_url(base_url: &str) -> Option<String> {
    let raw = base_url.trim();
    if raw.is_empty() {
        return None;
    }

    let candidate = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };

    let mut hostname = reqwest::Url::parse(&candidate)
        .ok()
        .and_then(|url| {
            url.host_str()
                .map(|value| value.trim().to_ascii_lowercase())
        })
        .unwrap_or_default();

    if hostname.is_empty() {
        let without_scheme = raw
            .strip_prefix("http://")
            .or_else(|| raw.strip_prefix("https://"))
            .unwrap_or(raw);
        hostname = without_scheme
            .split(['/', '?', '#'])
            .next()
            .unwrap_or_default()
            .split(':')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
    }

    if hostname.is_empty() {
        return None;
    }

    let parts = hostname
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }

    let mut index = parts.len().saturating_sub(2);
    if parts.len() >= 3 {
        let top_level = parts[parts.len() - 1];
        let second_level = parts[parts.len() - 2];
        if top_level.len() == 2 && second_level.len() <= 3 {
            index = parts.len().saturating_sub(3);
        }
    }

    let token = parts
        .get(index)
        .copied()
        .unwrap_or_default()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>();
    let token = token.trim_matches('-').trim_matches('_').to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

pub(crate) fn infer_platform_protocol(api_value: Option<&str>) -> String {
    let api = api_value
        .unwrap_or("openai-completions")
        .to_ascii_lowercase();
    if api.contains("anthropic") {
        "anthropic".to_string()
    } else {
        "openai".to_string()
    }
}

pub(crate) fn infer_platform_api_path(
    protocol: &str,
    api_value: Option<&str>,
    base_url: &str,
) -> String {
    let api = api_value.unwrap_or("").trim().to_ascii_lowercase();
    if api.contains("openai-responses") {
        if base_url
            .trim_end_matches('/')
            .to_ascii_lowercase()
            .ends_with("/v1")
        {
            return "/responses".to_string();
        }
        return "/v1/responses".to_string();
    }

    let normalized_base = base_url.trim_end_matches('/').to_ascii_lowercase();
    if protocol.eq_ignore_ascii_case("anthropic") {
        if normalized_base.ends_with("/v1") {
            "/messages".to_string()
        } else {
            "/v1/messages".to_string()
        }
    } else if normalized_base.ends_with("/v1") {
        "/chat/completions".to_string()
    } else {
        "/v1/chat/completions".to_string()
    }
}

pub(crate) fn is_local_proxy_host(url: &reqwest::Url) -> bool {
    matches!(url.host_str(), Some("localhost") | Some("127.0.0.1"))
}

pub(crate) fn normalize_local_proxy_base_url_for_persist(base_url: &str) -> String {
    let normalized = base_url.trim().trim_end_matches('/').to_string();
    if normalized.is_empty() {
        return normalized;
    }

    let Ok(mut url) = reqwest::Url::parse(&normalized) else {
        return normalized;
    };
    if !is_local_proxy_host(&url) {
        return normalized;
    }

    let mut path = url.path().trim_end_matches('/').to_string();
    let lower = path.to_ascii_lowercase();
    if lower.ends_with("/v1/chat/completions") {
        path = path[..path.len() - "/v1/chat/completions".len()].to_string();
    } else if lower.ends_with("/chat/completions") {
        path = path[..path.len() - "/chat/completions".len()].to_string();
    } else if lower.ends_with("/v1/responses") {
        path = path[..path.len() - "/v1/responses".len()].to_string();
    } else if lower.ends_with("/responses") {
        path = path[..path.len() - "/responses".len()].to_string();
    } else if lower.ends_with("/v1/messages") {
        path = path[..path.len() - "/v1/messages".len()].to_string();
    } else if lower.ends_with("/messages") {
        path = path[..path.len() - "/messages".len()].to_string();
    }

    if path.to_ascii_lowercase().ends_with("/v1") {
        path = path[..path.len() - "/v1".len()].to_string();
    }

    url.set_path(if path.is_empty() { "/" } else { &path });
    url.set_query(None);
    url.set_fragment(None);
    url.to_string().trim_end_matches('/').to_string()
}
