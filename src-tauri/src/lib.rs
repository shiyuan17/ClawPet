use base64::Engine;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenClawMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenClawRequest {
    messages: Vec<OpenClawMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenClawHttpResponse {
    text: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiMessage {
    content: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiChatResponse {
    choices: Option<Vec<OpenAiChoice>>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct OpenClawResponse {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GatewayHealthResponse {
    status: String,
    checked_url: Option<String>,
    detail: Option<String>,
    latency_ms: Option<u128>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffMemberSnapshot {
    agent_id: String,
    display_name: String,
    role_label: String,
    model: String,
    workspace: String,
    tools_profile: String,
    status_label: String,
    current_work_label: String,
    current_work: String,
    recent_output: String,
    scheduled_label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffSnapshotResponse {
    mission_statement: String,
    source_path: String,
    detail: String,
    members: Vec<StaffMemberSnapshot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskSnapshotItem {
    id: String,
    name: String,
    agent_id: String,
    session_target: String,
    enabled: bool,
    delete_after_run: bool,
    status_kind: String,
    status_label: String,
    summary: String,
    next_run_at_ms: Option<i64>,
    created_at_ms: Option<i64>,
    updated_at_ms: Option<i64>,
    schedule_kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskSnapshotResponse {
    source_path: String,
    detail: String,
    jobs: Vec<TaskSnapshotItem>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SourceFileSnapshotItem {
    id: String,
    title: String,
    summary: String,
    content: String,
    source_path: String,
    relative_path: String,
    facet_key: String,
    facet_label: String,
    category: String,
    updated_at_ms: i64,
    exists: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceFileSnapshotResponse {
    source_path: String,
    detail: String,
    items: Vec<SourceFileSnapshotItem>,
}

/// 已安装技能项：来自 ~/.openclaw/skills 与 workspace/skills，对应 openclaw 技能信息（非文档编辑）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenClawSkillListItem {
    id: String,
    name: String,
    description: String,
    enabled: bool,
    relative_path: String,
    source_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawSkillsListResponse {
    source_path: String,
    /// 内置技能（来自 openclaw.json skills.allowBundled + entries）
    built_in: Vec<OpenClawSkillListItem>,
    /// 安装技能（来自 ~/.openclaw/skills 与 workspace/skills 下的 SKILL.md）
    installed: Vec<OpenClawSkillListItem>,
}

/// 已配置工具项：来自 openclaw tools.profile / allow/deny，对应已安装工具信息（非 TOOLS.md 编辑）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenClawToolListItem {
    id: String,
    name: String,
    description: String,
    category: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawToolsListResponse {
    profile: String,
    profile_label: String,
    tools: Vec<OpenClawToolListItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawMessageLogItem {
    id: String,
    session_id: String,
    platform_id: String,
    platform_name: String,
    protocol: String,
    method: String,
    endpoint: String,
    base_url: Option<String>,
    path: Option<String>,
    request_body: String,
    response_status: u16,
    response_body: String,
    stream_summary: Option<String>,
    duration: u32,
    first_token_time: Option<u32>,
    error: Option<String>,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
    created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawMessageLogResponse {
    detail: String,
    logs: Vec<OpenClawMessageLogItem>,
}

#[derive(Debug, Clone)]
struct PendingToolCall {
    tool_name: String,
    arguments: String,
    created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioFilePayload {
    data_url: String,
    mime_type: String,
    file_name: String,
}

#[derive(Debug, Clone)]
struct EditableScope {
    facet_key: String,
    facet_label: String,
    workspace_root: PathBuf,
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    messages: Vec<OpenClawMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AnthropicResponse {
    content: Option<Vec<AnthropicContentBlock>>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LocalProxyPlatform {
    protocol: String,
    base_url: String,
    path_prefix: String,
    api_key: String,
}

#[derive(Default)]
struct LocalProxyState {
    stop_signal: Option<Arc<AtomicBool>>,
    handle: Option<JoinHandle<()>>,
}

static LOCAL_PROXY_STATE: OnceLock<Mutex<LocalProxyState>> = OnceLock::new();

fn local_proxy_state() -> &'static Mutex<LocalProxyState> {
    LOCAL_PROXY_STATE.get_or_init(|| Mutex::new(LocalProxyState::default()))
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

fn load_openclaw_gateway_token_from_config() -> Option<String> {
    let home_dir = std::env::var_os("HOME")?;
    let config_path = PathBuf::from(home_dir)
        .join(".openclaw")
        .join("openclaw.json");
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

fn resolve_openclaw_config_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("OPENCLAW_CONFIG_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(home_dir) = std::env::var("HOME") {
        return PathBuf::from(home_dir)
            .join(".openclaw")
            .join("openclaw.json");
    }

    PathBuf::from(".openclaw").join("openclaw.json")
}

fn resolve_openclaw_home_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("OPENCLAW_HOME") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(home_dir) = std::env::var("HOME") {
        return PathBuf::from(home_dir).join(".openclaw");
    }

    PathBuf::from(".openclaw")
}

fn resolve_workspace_main_root() -> PathBuf {
    resolve_openclaw_home_path().join("workspace-main")
}

fn resolve_workspace_agents_root() -> PathBuf {
    resolve_openclaw_home_path().join("workspaces")
}

fn expand_home_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return resolve_openclaw_home_path();
    }

    if trimmed == "~" {
        return std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(trimmed));
    }

    if let Some(suffix) = trimmed.strip_prefix("~/") {
        return std::env::var("HOME")
            .map(|home| PathBuf::from(home).join(suffix))
            .unwrap_or_else(|_| PathBuf::from(trimmed));
    }

    PathBuf::from(trimmed)
}

fn resolve_workspace_root_for_agent(agent_id: &str, configured_workspace: Option<&str>) -> PathBuf {
    if let Some(workspace) = configured_workspace {
        let expanded = expand_home_path(workspace);
        if expanded.is_absolute() {
            return expanded;
        }
        return resolve_openclaw_home_path().join(expanded);
    }

    let preferred = resolve_openclaw_home_path().join(format!("workspace-{agent_id}"));
    if preferred.exists() {
        return preferred;
    }

    let legacy = resolve_workspace_agents_root().join(agent_id);
    if legacy.exists() {
        return legacy;
    }

    preferred
}

fn load_staff_mission_statement() -> String {
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

fn value_as_object(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    value.as_object()
}

fn read_string_or_primary<'a>(value: &'a Value) -> Option<&'a str> {
    value
        .as_str()
        .or_else(|| value_as_object(value).and_then(|obj| obj.get("primary")).and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn load_staff_from_runtime_dirs(
    scheduled_agents: &std::collections::HashSet<String>,
) -> Result<Vec<StaffMemberSnapshot>, String> {
    let agents_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents");

    let entries = match std::fs::read_dir(&agents_path) {
        Ok(value) => value,
        Err(_) => return Ok(Vec::new()),
    };

    let mut members = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let agent_id = entry.file_name().to_string_lossy().trim().to_string();
        if agent_id.is_empty() {
            continue;
        }
        let runtime_summary = load_runtime_session_summary(&agent_id);
        let status_label = runtime_summary
            .latest_updated_at_ms
            .map(derive_status_label)
            .unwrap_or_else(|| "待命".to_string());
        let (current_work, recent_output) = runtime_summary
            .latest_session_file
            .as_deref()
            .map(load_recent_activity_from_session_file)
            .unwrap_or((None, None));
        members.push(StaffMemberSnapshot {
            agent_id: agent_id.clone(),
            display_name: agent_id.clone(),
            role_label: humanize_agent_role(&agent_id),
            model: runtime_summary
                .latest_model
                .unwrap_or_else(|| "未标注".to_string()),
            workspace: "未标注".to_string(),
            tools_profile: "default".to_string(),
            status_label,
            current_work_label: "正在处理什么".to_string(),
            current_work: current_work.unwrap_or_else(|| "当前无实时任务".to_string()),
            recent_output: recent_output.unwrap_or_else(|| "最近暂无产出。".to_string()),
            scheduled_label: if scheduled_agents.contains(&agent_id) {
                "已排班".to_string()
            } else {
                "未排班".to_string()
            },
        });
    }

    members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    Ok(members)
}

fn humanize_agent_role(agent_id: &str) -> String {
    match agent_id.trim().to_lowercase().as_str() {
        "main" => "主控员工".to_string(),
        "gateway" => "网关员工".to_string(),
        other => format!("{other} 员工"),
    }
}

fn extract_text_from_message_content(content: &Value) -> Option<String> {
    if let Some(text) = content.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(items) = content.as_array() {
        let mut texts = Vec::new();
        for item in items {
            let Some(obj) = item.as_object() else {
                continue;
            };
            if let Some(text) = obj.get("text").and_then(Value::as_str) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    texts.push(trimmed.to_string());
                }
            }
        }
        if !texts.is_empty() {
            return Some(texts.join("\n"));
        }
    }

    None
}

fn sanitize_staff_output(content: &str) -> String {
    let mut normalized = content
        .replace("[[reply_to_current]]", "")
        .trim()
        .to_string();
    if normalized.starts_with('[') {
        if let Some(pos) = normalized.find(']') {
            normalized = normalized[(pos + 1)..].trim().to_string();
        }
    }
    if normalized.is_empty() {
        "最近暂无产出。".to_string()
    } else {
        normalized
    }
}

fn extract_message_timestamp_ms(message: &serde_json::Map<String, Value>, fallback: i64) -> i64 {
    value_as_i64(message.get("timestamp")).unwrap_or(fallback)
}

fn extract_message_text(message: &serde_json::Map<String, Value>) -> Option<String> {
    message
        .get("content")
        .and_then(extract_text_from_message_content)
        .map(|text| sanitize_staff_output(&text))
}

fn extract_tool_calls(message: &serde_json::Map<String, Value>) -> Vec<(String, PendingToolCall)> {
    let Some(items) = message.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };

    let created_at = extract_message_timestamp_ms(message, 0);
    let mut output = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };
        if obj.get("type").and_then(Value::as_str) != Some("toolCall") {
            continue;
        }

        let Some(tool_call_id) = obj.get("id").and_then(Value::as_str) else {
            continue;
        };
        let tool_name = obj
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("tool")
            .to_string();
        let arguments = obj
            .get("arguments")
            .map(|value| serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()))
            .unwrap_or_else(|| "{}".to_string());

        output.push((
            tool_call_id.to_string(),
            PendingToolCall {
                tool_name,
                arguments,
                created_at,
            },
        ));
    }

    output
}

fn extract_tool_result_text(message: &serde_json::Map<String, Value>) -> Option<String> {
    let text = extract_message_text(message)?;
    let details = message.get("details").and_then(Value::as_object);
    let audio_path = details
        .and_then(|value| value.get("audioPath"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(path) = audio_path {
        if text.contains(path) {
            return Some(text);
        }
        return Some(format!("{text}\n{path}"));
    }

    Some(text)
}

fn infer_openclaw_response_status(text: &str) -> (u16, Option<String>) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (200, None);
    }

    if let Some(start) = trimmed.find("（") {
        if let Some(end_offset) = trimmed[start + 3..].find('）') {
            let code = &trimmed[start + 3..start + 3 + end_offset];
            if let Ok(status) = code.parse::<u16>() {
                return (status, (status >= 400).then(|| trimmed.to_string()));
            }
        }
    }

    if let Some(index) = trimmed.find("status code ") {
        let digits = trimmed[index + "status code ".len()..]
            .chars()
            .take_while(|char| char.is_ascii_digit())
            .collect::<String>();
        if let Ok(status) = digits.parse::<u16>() {
            return (status, (status >= 400).then(|| trimmed.to_string()));
        }
    }

    if let Some(index) = trimmed.find("HTTP ") {
        let digits = trimmed[index + "HTTP ".len()..]
            .chars()
            .take_while(|char| char.is_ascii_digit())
            .collect::<String>();
        if let Ok(status) = digits.parse::<u16>() {
            return (status, (status >= 400).then(|| trimmed.to_string()));
        }
    }

    if trimmed.contains("请求失败")
        || trimmed.contains("返回错误状态")
        || trimmed.contains("invalid_api_key")
        || trimmed.contains("unauthorized")
        || trimmed.contains("rate limit")
    {
        return (500, Some(trimmed.to_string()));
    }

    (200, None)
}

fn extract_usage_numbers(
    message: &serde_json::Map<String, Value>,
) -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    let usage = message.get("usage").and_then(Value::as_object);
    let prompt_tokens = usage
        .and_then(|value| value.get("input"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let completion_tokens = usage
        .and_then(|value| value.get("output"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let total_tokens = usage
        .and_then(|value| value.get("totalTokens"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| match (prompt_tokens, completion_tokens) {
            (Some(input), Some(output)) => Some(input.saturating_add(output)),
            _ => None,
        });
    let cache_read_input_tokens = usage
        .and_then(|value| value.get("cacheRead"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());

    (
        prompt_tokens,
        completion_tokens,
        total_tokens,
        cache_read_input_tokens,
    )
}

fn load_openclaw_message_logs_from_session_file(
    agent_id: &str,
    session_file: &Path,
) -> Vec<OpenClawMessageLogItem> {
    let raw = match std::fs::read_to_string(session_file) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let session_file_name = session_file
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown-session");
    let session_id = session_file
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(session_file_name);
    let endpoint = format!("openclaw://runtime/{agent_id}/{session_id}");
    let base_url = format!("openclaw://runtime/{agent_id}");
    let path = format!("/{session_id}");
    let platform_id = format!("openclaw-runtime-{agent_id}");
    let platform_name = format!("OpenClaw 运行时 / {agent_id}");

    let mut pending_user: Option<(String, i64)> = None;
    let mut pending_tool_calls: std::collections::HashMap<String, PendingToolCall> =
        std::collections::HashMap::new();
    let mut output = Vec::new();

    for (line_index, line) in raw.lines().enumerate() {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let Some(obj) = parsed.as_object() else {
            continue;
        };
        if obj.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }
        let Some(message) = obj.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let fallback_created_at = i64::try_from(line_index).unwrap_or(0);
        let created_at = extract_message_timestamp_ms(message, fallback_created_at);

        if role == "user" {
            let Some(text) = extract_message_text(message) else {
                continue;
            };
            pending_user = Some((text, created_at));
            continue;
        }

        if role == "assistant" {
            for (tool_call_id, tool_call) in extract_tool_calls(message) {
                pending_tool_calls.insert(tool_call_id, tool_call);
            }

            let Some(text) = extract_message_text(message) else {
                continue;
            };

            let (request_body, request_at) = pending_user
                .take()
                .unwrap_or_else(|| ("".to_string(), created_at));
            let duration = created_at.saturating_sub(request_at);
            let duration = u32::try_from(duration).unwrap_or(u32::MAX);
            let (response_status, error) = infer_openclaw_response_status(&text);
            let (prompt_tokens, completion_tokens, total_tokens, cache_read_input_tokens) =
                extract_usage_numbers(message);

            output.push(OpenClawMessageLogItem {
                id: format!("runtime-{agent_id}-{session_id}-{created_at}"),
                session_id: format!("runtime-{agent_id}-{session_id}"),
                platform_id: platform_id.clone(),
                platform_name: platform_name.clone(),
                protocol: "openai".to_string(),
                method: "MESSAGE".to_string(),
                endpoint: endpoint.clone(),
                base_url: Some(base_url.clone()),
                path: Some(path.clone()),
                request_body,
                response_status,
                response_body: text.clone(),
                stream_summary: Some(text),
                duration,
                first_token_time: Some(duration),
                error,
                prompt_tokens,
                completion_tokens,
                total_tokens,
                cache_read_input_tokens,
                created_at,
            });
            continue;
        }

        if role != "toolResult" {
            continue;
        }

        let tool_call_id = message
            .get("toolCallId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let tool_name = message
            .get("toolName")
            .and_then(Value::as_str)
            .unwrap_or("tool");
        let Some(text) = extract_tool_result_text(message) else {
            continue;
        };
        let pending_tool_call = pending_tool_calls.remove(&tool_call_id);
        let request_at = pending_tool_call
            .as_ref()
            .map(|value| value.created_at)
            .unwrap_or(created_at);
        let duration = created_at.saturating_sub(request_at);
        let duration = u32::try_from(duration).unwrap_or(u32::MAX);
        let (response_status, error) = infer_openclaw_response_status(&text);
        let resolved_tool_name = pending_tool_call
            .as_ref()
            .map(|value| value.tool_name.clone())
            .unwrap_or_else(|| tool_name.to_string());
        let request_body = pending_tool_call
            .map(|value| value.arguments)
            .unwrap_or_else(|| "{}".to_string());

        output.push(OpenClawMessageLogItem {
            id: format!("runtime-{agent_id}-{session_id}-tool-{created_at}"),
            session_id: format!("runtime-{agent_id}-{session_id}"),
            platform_id: platform_id.clone(),
            platform_name: format!("{platform_name} / {resolved_tool_name}"),
            protocol: "openai".to_string(),
            method: format!("TOOL:{resolved_tool_name}"),
            endpoint: format!("{endpoint}/tool/{resolved_tool_name}"),
            base_url: Some(base_url.clone()),
            path: Some(format!("{path}/tool/{resolved_tool_name}")),
            request_body,
            response_status,
            response_body: text.clone(),
            stream_summary: Some(text),
            duration,
            first_token_time: Some(duration),
            error,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            cache_read_input_tokens: None,
            created_at,
        });
    }

    output
}

#[tauri::command]
fn load_openclaw_message_logs() -> Result<OpenClawMessageLogResponse, String> {
    let agents_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents");

    let entries = match std::fs::read_dir(&agents_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawMessageLogResponse {
                detail: "未找到 OpenClaw 运行时会话目录。".to_string(),
                logs: Vec::new(),
            })
        }
    };

    let mut session_files = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let agent_id = entry.file_name().to_string_lossy().trim().to_string();
        if agent_id.is_empty() {
            continue;
        }

        let sessions_dir = entry.path().join("sessions");
        let Ok(files) = std::fs::read_dir(&sessions_dir) else {
            continue;
        };

        for file in files {
            let Ok(file) = file else {
                continue;
            };
            let path = file.path();
            if path.extension().and_then(|value| value.to_str()) != Some("jsonl") {
                continue;
            }

            let modified_at = file
                .metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_millis())
                .and_then(|value| i64::try_from(value).ok())
                .unwrap_or_default();

            session_files.push((modified_at, agent_id.clone(), path));
        }
    }

    session_files.sort_by(|left, right| right.0.cmp(&left.0));

    let mut logs = Vec::new();
    for (_, agent_id, path) in session_files.into_iter().take(12) {
        logs.extend(load_openclaw_message_logs_from_session_file(
            &agent_id, &path,
        ));
    }

    logs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    logs.truncate(180);

    Ok(OpenClawMessageLogResponse {
        detail: format!("已从 OpenClaw 运行时会话读取 {} 条消息日志。", logs.len()),
        logs,
    })
}

fn humanize_scope_label(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("main") {
        return "Main".to_string();
    }
    if trimmed.is_empty() {
        return "未标注".to_string();
    }
    trimmed.to_string()
}

fn load_editable_scopes() -> Vec<EditableScope> {
    let mut scopes = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let main_root = resolve_workspace_main_root();
    scopes.push(EditableScope {
        facet_key: "main".to_string(),
        facet_label: "Main".to_string(),
        workspace_root: main_root.clone(),
    });
    seen.insert("main".to_string());

    let config_path = resolve_openclaw_config_path();
    if let Ok(raw) = std::fs::read_to_string(config_path) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&raw) {
            if let Some(list) = parsed
                .get("agents")
                .and_then(Value::as_object)
                .and_then(|value| value.get("list"))
                .and_then(Value::as_array)
            {
                for row in list {
                    let Some(obj) = row.as_object() else {
                        continue;
                    };
                    let agent_id = obj
                        .get("id")
                        .and_then(Value::as_str)
                        .or_else(|| obj.get("name").and_then(Value::as_str))
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .unwrap_or("");
                    if agent_id.is_empty() {
                        continue;
                    }
                    let facet_key = agent_id.to_lowercase();
                    if seen.contains(&facet_key) {
                        continue;
                    }
                    let configured_workspace = obj
                        .get("workspace")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty());
                    let workspace_root = if facet_key == "main" {
                        main_root.clone()
                    } else {
                        resolve_workspace_root_for_agent(agent_id, configured_workspace)
                    };
                    scopes.push(EditableScope {
                        facet_key: facet_key.clone(),
                        facet_label: humanize_scope_label(agent_id),
                        workspace_root,
                    });
                    seen.insert(facet_key);
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(resolve_workspace_agents_root()) {
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let agent_id = entry.file_name().to_string_lossy().trim().to_string();
            if agent_id.is_empty() {
                continue;
            }
            let facet_key = agent_id.to_lowercase();
            if seen.contains(&facet_key) {
                continue;
            }
            scopes.push(EditableScope {
                facet_key: facet_key.clone(),
                facet_label: humanize_scope_label(&agent_id),
                workspace_root: entry.path(),
            });
            seen.insert(facet_key);
        }
    }

    scopes.sort_by(|left, right| {
        if left.facet_key == "main" {
            return std::cmp::Ordering::Less;
        }
        if right.facet_key == "main" {
            return std::cmp::Ordering::Greater;
        }
        left.facet_label.cmp(&right.facet_label)
    });
    scopes
}

fn safe_read_source_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Some(String::new());
    }
    Some(trimmed.to_string())
}

fn build_source_file_title(path: &Path, content: &str) -> String {
    for line in content.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("# ") {
            let heading = value.trim();
            if !heading.is_empty() {
                return heading.to_string();
            }
        }
    }

    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("未命名文件")
        .to_string()
}

fn build_source_file_summary(content: &str) -> String {
    let normalized = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(4)
        .collect::<Vec<_>>()
        .join(" ");
    if normalized.is_empty() {
        "文件内容为空。".to_string()
    } else if normalized.chars().count() > 180 {
        normalized.chars().take(180).collect::<String>()
    } else {
        normalized
    }
}

fn build_source_file_item(
    path: &Path,
    relative_base: &Path,
    facet_key: &str,
    facet_label: &str,
    category: &str,
) -> Option<SourceFileSnapshotItem> {
    let meta = std::fs::metadata(path).ok();
    let exists = meta.as_ref().map(|value| value.is_file()).unwrap_or(false);
    let content = if exists {
        safe_read_source_file(path)?
    } else {
        String::new()
    };
    let updated_at_ms = meta
        .as_ref()
        .and_then(|value| value.modified().ok())
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis() as i64)
        .unwrap_or(0);
    let title = build_source_file_title(path, &content);
    let summary = if exists {
        build_source_file_summary(&content)
    } else {
        "文件缺失，保存时会自动创建。".to_string()
    };
    let source_path = path.display().to_string();
    let relative_path = path
        .strip_prefix(relative_base)
        .ok()
        .map(|value| value.display().to_string())
        .unwrap_or_else(|| source_path.clone());
    let id = format!("{facet_key}-{}", relative_path.replace(['/', '\\', ' '], "-").to_lowercase());

    Some(SourceFileSnapshotItem {
        id,
        title,
        summary,
        content,
        source_path,
        relative_path,
        facet_key: facet_key.to_string(),
        facet_label: facet_label.to_string(),
        category: category.to_string(),
        updated_at_ms,
        exists,
    })
}

fn load_memory_file_items() -> Vec<SourceFileSnapshotItem> {
    let mut output = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let main_root = resolve_workspace_main_root();
    let scopes = load_editable_scopes();
    let memory_candidates = [
        "MEMORY.md",
        "SOUL.md",
        "TOOLS.md",
        "IDENTITY.md",
        "USER.md",
        "HEARTBEAT.md",
        "BOOTSTRAP.md",
        "AGENTS.md",
    ];

    let append = |output: &mut Vec<SourceFileSnapshotItem>,
                  seen: &mut std::collections::HashSet<String>,
                  item: Option<SourceFileSnapshotItem>| {
        if let Some(row) = item {
            if seen.insert(row.source_path.clone()) {
                output.push(row);
            }
        }
    };

    for file_name in memory_candidates {
        append(
            &mut output,
            &mut seen,
            build_source_file_item(
                &main_root.join(file_name),
                &main_root,
                "main",
                "Main",
                "Main",
            ),
        );
    }

    if let Ok(entries) = std::fs::read_dir(main_root.join("memory")) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !["md", "markdown", "txt"].contains(&ext.as_str()) {
                continue;
            }
            append(
                &mut output,
                &mut seen,
                build_source_file_item(&path, &main_root, "main", "Main", "Main 记忆记录"),
            );
        }
    }

    for scope in scopes.iter().filter(|scope| scope.facet_key != "main") {
        for file_name in memory_candidates {
            append(
                &mut output,
                &mut seen,
                build_source_file_item(
                    &scope.workspace_root.join(file_name),
                    &scope.workspace_root,
                    &scope.facet_key,
                    &scope.facet_label,
                    &scope.facet_label,
                ),
            );
        }

        if let Ok(entries) = std::fs::read_dir(scope.workspace_root.join("memory")) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !["md", "markdown", "txt"].contains(&ext.as_str()) {
                    continue;
                }
                append(
                    &mut output,
                    &mut seen,
                    build_source_file_item(
                        &path,
                        &scope.workspace_root,
                        &scope.facet_key,
                        &scope.facet_label,
                        &format!("{} 记忆记录", scope.facet_label),
                    ),
                );
            }
        }
    }

    output.sort_by(|left, right| {
        (left.facet_key != "main")
            .cmp(&(right.facet_key != "main"))
            .then_with(|| left.facet_label.cmp(&right.facet_label))
            .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    output
}

fn resolve_memory_db_path(scope: &EditableScope) -> PathBuf {
    let file_name = if scope.facet_key == "main" {
        "main.sqlite".to_string()
    } else {
        format!("{}.sqlite", scope.facet_key)
    };
    resolve_openclaw_home_path().join("memory").join(file_name)
}

fn query_sqlite_count(db_path: &Path, table_name: &str) -> Option<usize> {
    if !db_path.exists() {
        return None;
    }

    let output = Command::new("sqlite3")
        .arg(db_path)
        .arg(format!("select count(*) from {table_name};"))
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()?
        .trim()
        .parse::<usize>()
        .ok()
}

fn summarize_memory_store(scopes: &[EditableScope]) -> (usize, usize, usize) {
    let mut db_count = 0;
    let mut file_count = 0;
    let mut chunk_count = 0;

    for scope in scopes {
        let db_path = resolve_memory_db_path(scope);
        if !db_path.exists() {
            continue;
        }
        db_count += 1;
        file_count += query_sqlite_count(&db_path, "files").unwrap_or(0);
        chunk_count += query_sqlite_count(&db_path, "chunks").unwrap_or(0);
    }

    (db_count, file_count, chunk_count)
}

fn load_document_file_items() -> Vec<SourceFileSnapshotItem> {
    let mut output = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let main_root = resolve_workspace_main_root();
    let scopes = load_editable_scopes();
    let main_candidates = [
        "AGENTS.md",
        "IDENTITY.md",
        "SOUL.md",
        "BOOTSTRAP.md",
        "HEARTBEAT.md",
        "TOOLS.md",
        "README.md",
        "NOTEBOOK.md",
        "focus.md",
        "inbox.md",
        "routines.md",
    ];

    let append = |output: &mut Vec<SourceFileSnapshotItem>,
                  seen: &mut std::collections::HashSet<String>,
                  item: Option<SourceFileSnapshotItem>| {
        if let Some(row) = item {
            if seen.insert(row.source_path.clone()) {
                output.push(row);
            }
        }
    };

    for file_name in main_candidates {
        append(
            &mut output,
            &mut seen,
            build_source_file_item(
                &main_root.join(file_name),
                &main_root,
                "main",
                "Main",
                "Main 核心文档",
            ),
        );
    }

    for scope in scopes.iter().filter(|scope| scope.facet_key != "main") {
        for file_name in [
            "AGENTS.md",
            "IDENTITY.md",
            "SOUL.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
            "TOOLS.md",
            "README.md",
            "NOTEBOOK.md",
            "focus.md",
            "inbox.md",
            "routines.md",
        ] {
            append(
                &mut output,
                &mut seen,
                build_source_file_item(
                    &scope.workspace_root.join(file_name),
                    &scope.workspace_root,
                    &scope.facet_key,
                    &scope.facet_label,
                    &format!("{} 核心文档", scope.facet_label),
                ),
            );
        }
    }

    output.sort_by(|left, right| {
        (left.facet_key != "main")
            .cmp(&(right.facet_key != "main"))
            .then_with(|| left.facet_label.cmp(&right.facet_label))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    output
}

fn load_skill_file_items() -> Vec<SourceFileSnapshotItem> {
    let mut output = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let candidates = [
        resolve_openclaw_home_path().join("workspace").join("skills"),
        resolve_openclaw_home_path().join("skills"),
    ];

    for root in candidates {
        let entries = match std::fs::read_dir(&root) {
            Ok(value) => value,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }

            let skill_file = path.join("SKILL.md");
            let key = skill_file.display().to_string();
            if seen.contains(&key) {
                continue;
            }

            if let Some(item) = build_source_file_item(
                &skill_file,
                &root,
                "openclaw-skills",
                "OpenClaw",
                "OpenClaw Skills",
            ) {
                seen.insert(key);
                output.push(item);
            }
        }
    }

    output.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    output
}

fn load_tool_file_items(agent_id: Option<&str>) -> Vec<SourceFileSnapshotItem> {
    let scopes = load_editable_scopes();
    let filtered_scopes = scopes.into_iter().filter(|scope| {
        if scope.facet_key == "main" {
            return false;
        }

        match agent_id {
            Some(value) => scope.facet_key.eq_ignore_ascii_case(value.trim()),
            None => true,
        }
    });

    let mut output = Vec::new();
    for scope in filtered_scopes {
        if let Some(item) = build_source_file_item(
            &scope.workspace_root.join("TOOLS.md"),
            &scope.workspace_root,
            &scope.facet_key,
            &scope.facet_label,
            &format!("{} Tools", scope.facet_label),
        ) {
            output.push(item);
        }
    }

    output.sort_by(|left, right| left.facet_label.cmp(&right.facet_label));
    output
}

/// 从 relative_path（如 "transcribe/SKILL.md"）得到技能 id（如 "transcribe"），用于匹配 openclaw.json skills.entries
fn openclaw_skill_id_from_path(relative_path: &str) -> String {
    let path = relative_path.trim().replace('\\', "/");
    path.split('/')
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string()
}

/// OpenClaw 完整内置技能清单 (id, 描述)
static OPENCLAW_BUILTIN_SKILLS: &[(&str, &str)] = &[
    ("1password", "1Password 密码管理，设置并使用 1Password CLI 来管理密码"),
    ("apple-notes", "通过 macOS 上的「备忘录」CLI 管理 Apple Notes"),
    ("apple-reminders", "通过 remindctl 的 CLI 管理苹果提醒事项"),
    ("bear-notes", "通过 grizzly CLI 搜索和管理 Bear 笔记"),
    ("blogwatcher", "博客监控，定期监听博客更新"),
    ("blucli", "BlueBubbles CLI 操作"),
    ("bluebubbles", "BlueBubbles iMessage 收发消息"),
    ("camsnap", "RTSP/ONVIF 摄像头截图"),
    ("canvas", "Canvas 画布操作"),
    ("clawhub", "ClawHub CLI - 技能市场，安装/管理技能"),
    ("coding-agent", "编程代理，辅助代码编写与调试"),
    ("discord", "Discord 消息收发与频道管理"),
    ("eightctl", "Eight Sleep 智能床控制"),
    ("gemini", "Gemini CLI 问答与推理"),
    ("gh-issues", "GitHub Issues 自动处理"),
    ("gifgrep", "GIF 搜索与下载"),
    ("github", "GitHub CLI 仓库与 PR 操作"),
    ("gog", "Google Workspace（Gmail / Calendar / Drive）"),
    ("goplaces", "地点搜索与导航"),
    ("healthcheck", "安全审计与系统健康检查"),
    ("himalaya", "邮件客户端，收发管理邮件"),
    ("imsg", "iMessage / SMS 收发消息"),
    ("mcporter", "Minecraft 相关操作"),
    ("model-usage", "模型使用统计与费用追踪"),
    ("nano-banana-pro", "Banana Pro 设备管理"),
    ("nano-pdf", "PDF 生成与读取操作"),
    ("notion", "Notion 笔记与数据库操作"),
    ("obsidian", "Obsidian 笔记管理"),
    ("openai-image-gen", "OpenAI DALL·E 图像生成"),
    ("openai-whisper", "本地 Whisper 语音识别"),
    ("openai-whisper-api", "OpenAI Whisper API 语音识别"),
    ("openhue", "Philips Hue 智能灯控制"),
    ("oracle", "Oracle 数据库查询与管理"),
    ("ordercli", "订单管理 CLI"),
    ("peekaboo", "macOS UI 自动化截图工具"),
    ("sag", "ElevenLabs TTS 语音合成"),
    ("session-logs", "会话日志查看与管理"),
    ("sherpa-onnx-tts", "本地 Sherpa-ONNX TTS 语音合成"),
    ("skill-creator", "创建和编辑 Skills"),
    ("slack", "Slack 消息与频道操作"),
    ("songsee", "歌曲识别"),
    ("sonoscli", "Sonos 音响控制"),
    ("spotify-player", "Spotify 播放器控制"),
    ("summarize", "内容摘要与转录"),
    ("things-mac", "Things 3 任务管理"),
    ("tmux", "Tmux 会话管理"),
    ("trello", "Trello 看板任务管理"),
    ("video-frames", "视频帧提取与分析"),
    ("voice-call", "语音通话"),
    ("wacli", "WhatsApp CLI 消息收发"),
    ("weather", "天气查询"),
    ("xurl", "URL 处理与内容抓取"),
];

#[tauri::command]
fn load_openclaw_skills_list() -> Result<OpenClawSkillsListResponse, String> {
    let items = load_skill_file_items();
    let config_path = resolve_openclaw_config_path();
    let mut entries_enabled: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    if let Ok(raw) = std::fs::read_to_string(&config_path) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&raw) {
            if let Some(skills_root) = parsed.get("skills").and_then(Value::as_object) {
                if let Some(entries) = skills_root.get("entries").and_then(Value::as_object) {
                    for (key, val) in entries {
                        let enabled = val
                            .get("enabled")
                            .and_then(Value::as_bool)
                            .unwrap_or(true);
                        entries_enabled.insert(key.trim().to_lowercase(), enabled);
                    }
                }
            }
        }
    }

    let source_path = resolve_openclaw_home_path().join("skills").display().to_string();

    // 内置技能：使用完整硬编码清单，从 entries_enabled 读取启用状态
    let built_in: Vec<OpenClawSkillListItem> = OPENCLAW_BUILTIN_SKILLS
        .iter()
        .map(|(id, desc)| {
            let key_lower = id.to_lowercase();
            let enabled = entries_enabled.get(&key_lower).copied().unwrap_or(true);
            OpenClawSkillListItem {
                id: id.to_string(),
                name: id.to_string(),
                description: desc.to_string(),
                enabled,
                relative_path: String::new(),
                source_path: String::new(),
            }
        })
        .collect();

    // 内置技能 ID 集合，用于排除
    let builtin_ids: std::collections::HashSet<String> = OPENCLAW_BUILTIN_SKILLS
        .iter()
        .map(|(id, _)| id.to_lowercase())
        .collect();

    // 扫描插件目录里的额外技能（~/.openclaw/openclaw-*/skills/）
    let openclaw_home = resolve_openclaw_home_path();
    let mut plugin_items: Vec<SourceFileSnapshotItem> = Vec::new();
    if let Ok(home_entries) = std::fs::read_dir(&openclaw_home) {
        for entry in home_entries.flatten() {
            let dir_name = entry.file_name();
            let dir_str = dir_name.to_string_lossy();
            if !dir_str.starts_with("openclaw-") {
                continue;
            }
            let plugin_skills_dir = entry.path().join("skills");
            if let Ok(skill_dirs) = std::fs::read_dir(&plugin_skills_dir) {
                for skill_entry in skill_dirs.flatten() {
                    let skill_path = skill_entry.path();
                    let Ok(ft) = skill_entry.file_type() else { continue };
                    if !ft.is_dir() { continue }
                    let skill_file = skill_path.join("SKILL.md");
                    if let Some(item) = build_source_file_item(
                        &skill_file,
                        &plugin_skills_dir,
                        "openclaw-skills",
                        "OpenClaw",
                        "OpenClaw Skills",
                    ) {
                        plugin_items.push(item);
                    }
                }
            }
        }
    }

    fn make_skill_item(
        skill_id: String,
        item_id: String,
        title: String,
        content: String,
        relative_path: String,
        source_path: String,
        entries_enabled: &std::collections::HashMap<String, bool>,
    ) -> OpenClawSkillListItem {
        let key_lower = skill_id.to_lowercase();
        let enabled = entries_enabled.get(&key_lower).copied().unwrap_or(true);
        let title_is_filename = title.eq_ignore_ascii_case("SKILL.md") || title.trim().is_empty();
        let display_name = if title_is_filename { skill_id.clone() } else { title };
        let description = {
            let clean: String = content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .take(3)
                .collect::<Vec<_>>()
                .join(" ");
            let truncated = if clean.chars().count() > 180 {
                clean.chars().take(180).collect::<String>()
            } else {
                clean
            };
            if truncated.is_empty() { "暂无描述。".to_string() } else { truncated }
        };
        OpenClawSkillListItem { id: item_id, name: display_name, description, enabled, relative_path, source_path }
    }

    // 用户安装技能：~/.openclaw/skills/ 下不在内置清单里的技能
    let installed: Vec<OpenClawSkillListItem> = items
        .into_iter()
        .filter(|item| {
            let skill_id = openclaw_skill_id_from_path(&item.relative_path);
            !builtin_ids.contains(&skill_id.to_lowercase())
        })
        .map(|item| {
            let skill_id = openclaw_skill_id_from_path(&item.relative_path);
            make_skill_item(skill_id, item.id, item.title, item.content, item.relative_path, item.source_path, &entries_enabled)
        })
        .chain(
            // 插件技能（openclaw-lark 等）
            plugin_items.into_iter().map(|item| {
                let skill_id = openclaw_skill_id_from_path(&item.relative_path);
                make_skill_item(skill_id, item.id, item.title, item.content, item.relative_path, item.source_path, &entries_enabled)
            })
        )
        .collect();

    Ok(OpenClawSkillsListResponse {
        source_path,
        built_in,
        installed,
    })
}

/// 更新 openclaw.json 中某技能的启用状态（skills.entries[skill_id].enabled）
#[tauri::command]
fn save_openclaw_skill_enabled(skill_id: String, enabled: bool) -> Result<(), String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut parsed: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let root = parsed.as_object_mut().ok_or("openclaw.json 根节点不是对象")?;
    let skills = root
        .entry("skills")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("skills 不是对象")?;
    let entries = skills
        .entry("entries")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("skills.entries 不是对象")?;
    let skill_key = skill_id.trim();
    if skill_key.is_empty() {
        return Err("技能 id 不能为空".to_string());
    }
    let entry = entries
        .entry(skill_key.to_string())
        .or_insert_with(|| serde_json::json!({}));
    let entry_obj = entry.as_object_mut().ok_or("skills.entries 项不是对象")?;
    entry_obj.insert("enabled".to_string(), Value::Bool(enabled));
    let new_raw = serde_json::to_string_pretty(&parsed).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, new_raw).map_err(|e| e.to_string())?;
    Ok(())
}

/// OpenClaw 内置工具清单（与 docs 一致）：id, 名称, 描述, 分类
const OPENCLAW_TOOL_INVENTORY: &[(&str, &str, &str)] = &[
    // group:fs
    ("read", "read", "读取文件内容"),
    ("write", "write", "创建或覆盖文件"),
    ("edit", "edit", "精确编辑文件"),
    ("apply_patch", "apply_patch", "应用补丁（多块编辑）"),
    // group:runtime
    ("exec", "exec", "执行 shell 命令"),
    ("bash", "bash", "Bash 执行"),
    ("process", "process", "管理后台进程"),
    // group:web
    ("web_search", "web_search", "网页搜索"),
    ("web_fetch", "web_fetch", "抓取网页内容"),
    // group:memory
    ("memory_search", "memory_search", "记忆语义搜索"),
    ("memory_get", "memory_get", "读取记忆文件"),
    // group:sessions
    ("sessions_list", "sessions_list", "会话列表"),
    ("sessions_history", "sessions_history", "会话历史"),
    ("sessions_send", "sessions_send", "发送会话消息"),
    ("sessions_spawn", "sessions_spawn", "创建会话"),
    ("session_status", "session_status", "会话状态"),
    // group:messaging
    ("message", "message", "消息发送"),
    // group:ui
    ("browser", "browser", "浏览器控制"),
    ("canvas", "canvas", "Canvas 节点"),
    // group:automation
    ("cron", "cron", "定时任务"),
    ("gateway", "gateway", "网关"),
    // group:nodes
    ("nodes", "nodes", "节点发现与配对"),
    // other
    ("image", "image", "图像生成/处理"),
];

fn openclaw_tool_category(tool_id: &str) -> &'static str {
    match tool_id {
        "read" | "write" | "edit" | "apply_patch" => "Files",
        "exec" | "bash" | "process" => "Runtime",
        "web_search" | "web_fetch" => "Web",
        "memory_search" | "memory_get" => "Memory",
        "sessions_list" | "sessions_history" | "sessions_send" | "sessions_spawn" | "session_status" => "Sessions",
        "message" => "Messaging",
        "browser" | "canvas" => "UI",
        "cron" | "gateway" => "Automation",
        "nodes" => "Nodes",
        _ => "Other",
    }
}

/// 解析 profile 得到基础允许的工具 id 集合；full = 全部
fn openclaw_profile_tool_ids(profile: &str) -> std::collections::HashSet<String> {
    let profile = profile.trim().to_ascii_lowercase();
    let mut set = std::collections::HashSet::new();
    match profile.as_str() {
        "full" | "default" | "" => {
            for (id, _, _) in OPENCLAW_TOOL_INVENTORY {
                set.insert((*id).to_string());
            }
            return set;
        }
        "minimal" => {
            set.insert("session_status".to_string());
            return set;
        }
        "coding" => {
            for id in &["read", "write", "edit", "apply_patch", "exec", "bash", "process", "sessions_list", "sessions_history", "sessions_send", "sessions_spawn", "session_status", "memory_search", "memory_get", "image"] {
                set.insert((*id).to_string());
            }
            return set;
        }
        "messaging" => {
            for id in &["message", "sessions_list", "sessions_history", "sessions_send", "session_status"] {
                set.insert((*id).to_string());
            }
            return set;
        }
        _ => {}
    }
    set
}

/// 从 config 的 allow/deny 数组解析出允许的工具 id（* 表示全部）；deny 优先
fn openclaw_resolve_tools_from_config(
    root: &serde_json::Map<String, Value>,
    agent_obj: Option<&serde_json::Map<String, Value>>,
) -> (String, String, std::collections::HashSet<String>) {
    let tools_root = root.get("tools").and_then(value_as_object);
    let _agents_list = root.get("agents").and_then(value_as_object).and_then(|o| o.get("list")).and_then(Value::as_array);
    let default_profile = tools_root
        .and_then(|t| t.get("profile").and_then(Value::as_str))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");
    let mut profile = default_profile.to_string();
    let mut allow: Option<Vec<String>> = tools_root.and_then(|t| t.get("allow")).and_then(Value::as_array).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
            .collect()
    });
    let mut deny: Vec<String> = tools_root
        .and_then(|t| t.get("deny"))
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.trim().to_ascii_lowercase())).collect())
        .unwrap_or_default();

    if let Some(agent) = agent_obj {
        if let Some(tools) = agent.get("tools").and_then(value_as_object) {
            if let Some(p) = tools.get("profile").and_then(Value::as_str) {
                let p = p.trim();
                if !p.is_empty() {
                    profile = p.to_string();
                }
            }
            if let Some(a) = tools.get("allow").and_then(Value::as_array) {
                allow = Some(a.iter().filter_map(|v| v.as_str().map(|s| s.trim().to_string())).collect());
            }
            if let Some(d) = tools.get("deny").and_then(Value::as_array) {
                deny = d.iter().filter_map(|v| v.as_str().map(|s| s.trim().to_ascii_lowercase().to_string())).collect();
            }
        }
    }

    let profile_label = match profile.to_ascii_lowercase().as_str() {
        "full" | "default" | "" => "Full",
        "minimal" => "Minimal",
        "coding" => "Coding",
        "messaging" => "Messaging",
        _ => profile.as_str(),
    }
    .to_string();

    let mut allowed_ids = openclaw_profile_tool_ids(&profile);
    if let Some(ref allow_list) = allow {
        if !allow_list.is_empty() && !allow_list.iter().any(|s| s.eq_ignore_ascii_case("*")) {
            let mut from_allow = std::collections::HashSet::new();
            for entry in allow_list {
                let e = entry.trim().to_ascii_lowercase();
                if e == "*" {
                    for (id, _, _) in OPENCLAW_TOOL_INVENTORY {
                        from_allow.insert((*id).to_string());
                    }
                    break;
                }
                if e.starts_with("group:") {
                    let group = e.strip_prefix("group:").unwrap_or("").trim();
                    for (id, _, _) in OPENCLAW_TOOL_INVENTORY {
                        let in_group = match group {
                            "fs" => matches!(*id, "read" | "write" | "edit" | "apply_patch"),
                            "runtime" => matches!(*id, "exec" | "bash" | "process"),
                            "web" => matches!(*id, "web_search" | "web_fetch"),
                            "memory" => matches!(*id, "memory_search" | "memory_get"),
                            "sessions" => id.starts_with("session"),
                            "messaging" => *id == "message",
                            "ui" => matches!(*id, "browser" | "canvas"),
                            "automation" => matches!(*id, "cron" | "gateway"),
                            "nodes" => *id == "nodes",
                            "openclaw" => true,
                            _ => false,
                        };
                        if in_group {
                            from_allow.insert((*id).to_string());
                        }
                    }
                } else {
                    from_allow.insert(entry.trim().to_string());
                }
            }
            if !from_allow.is_empty() {
                allowed_ids = from_allow;
            }
        }
    }
    for d in &deny {
        if d == "*" {
            allowed_ids.clear();
            break;
        }
        if d.starts_with("group:") {
            let group = d.strip_prefix("group:").unwrap_or("").trim();
            let in_deny_group = |id: &str| -> bool {
                matches!(
                    (group, id),
                    ("fs", "read" | "write" | "edit" | "apply_patch")
                        | ("runtime", "exec" | "bash" | "process")
                        | ("web", "web_search" | "web_fetch")
                        | ("memory", "memory_search" | "memory_get")
                        | ("messaging", "message")
                        | ("ui", "browser" | "canvas")
                        | ("automation", "cron" | "gateway")
                        | ("nodes", "nodes")
                ) || (group == "sessions" && id.starts_with("session"))
                  || (group == "openclaw")
            };
            allowed_ids.retain(|id| !in_deny_group(id.as_str()));
        } else {
            allowed_ids.retain(|id| !id.eq_ignore_ascii_case(d));
        }
    }

    (profile, profile_label, allowed_ids)
}

#[tauri::command]
fn load_openclaw_tools_list(agent_id: Option<String>) -> Result<OpenClawToolsListResponse, String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let parsed: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let root = value_as_object(&parsed).ok_or("openclaw.json 根节点不是对象")?;

    let agent_obj = if let Some(ref id) = agent_id {
        let id_trim = id.trim();
        if id_trim.is_empty() {
            None
        } else {
            root.get("agents")
                .and_then(Value::as_object)
                .and_then(|a| a.get("list"))
                .and_then(Value::as_array)
                .and_then(|list| {
                    list.iter().find_map(|item| {
                        let obj = value_as_object(item)?;
                        let aid = obj.get("id").and_then(Value::as_str).map(str::trim).unwrap_or("");
                        if aid.eq_ignore_ascii_case(id_trim) {
                            Some(obj)
                        } else {
                            None
                        }
                    })
                })
        }
    } else {
        None
    };

    let (profile, profile_label, allowed_ids) = openclaw_resolve_tools_from_config(root, agent_obj);

    let tools: Vec<OpenClawToolListItem> = OPENCLAW_TOOL_INVENTORY
        .iter()
        .map(|(id, name, desc)| {
            let enabled = allowed_ids.contains(&id.to_ascii_lowercase());
            OpenClawToolListItem {
                id: (*id).to_string(),
                name: (*name).to_string(),
                description: (*desc).to_string(),
                category: openclaw_tool_category(id).to_string(),
                enabled,
            }
        })
        .collect();

    Ok(OpenClawToolsListResponse {
        profile,
        profile_label,
        tools,
    })
}

#[tauri::command]
fn load_memory_file_snapshot() -> Result<SourceFileSnapshotResponse, String> {
    let scopes = load_editable_scopes();
    let items = load_memory_file_items();
    let (db_count, db_file_count, db_chunk_count) = summarize_memory_store(&scopes);
    let scope_count = scopes.len();
    let source_path = format!(
        "{} | {}",
        resolve_workspace_main_root().display(),
        resolve_openclaw_home_path().join("memory").display()
    );
    let existing_count = items.iter().filter(|item| item.exists).count();
    let missing_count = items.len().saturating_sub(existing_count);
    let detail = if items.is_empty() {
        format!(
            "已扫描 {} 个工作区与 {} 个记忆库，当前未发现可展示的文件型记忆。memory-core 状态：{} files / {} chunks。",
            scope_count, db_count, db_file_count, db_chunk_count
        )
    } else {
        format!(
            "已整理 {} 个记忆文件槽位（存在 {}，缺失 {}），并检查了 {} 个记忆库（{} files / {} chunks）。",
            items.len(), existing_count, missing_count, db_count, db_file_count, db_chunk_count
        )
    };
    Ok(SourceFileSnapshotResponse {
        source_path,
        detail,
        items,
    })
}

#[tauri::command]
fn load_document_file_snapshot() -> Result<SourceFileSnapshotResponse, String> {
    let items = load_document_file_items();
    let existing_count = items.iter().filter(|item| item.exists).count();
    let missing_count = items.len().saturating_sub(existing_count);
    Ok(SourceFileSnapshotResponse {
        source_path: resolve_workspace_main_root().display().to_string(),
        detail: format!("已整理 {} 份核心文件（存在 {}，缺失 {}）。", items.len(), existing_count, missing_count),
        items,
    })
}

#[tauri::command]
fn load_openclaw_resource_snapshot(
    kind: String,
    agent_id: Option<String>,
) -> Result<SourceFileSnapshotResponse, String> {
    let normalized_kind = kind.trim().to_ascii_lowercase();

    if normalized_kind == "skill" {
        let items = load_skill_file_items();
        let existing_count = items.iter().filter(|item| item.exists).count();
        let missing_count = items.len().saturating_sub(existing_count);
        return Ok(SourceFileSnapshotResponse {
            source_path: resolve_openclaw_home_path().join("skills").display().to_string(),
            detail: format!(
                "已整理 {} 份 OpenClaw skills（存在 {}，缺失 {}）。",
                items.len(),
                existing_count,
                missing_count
            ),
            items,
        });
    }

    if normalized_kind == "tool" {
        let items = load_tool_file_items(agent_id.as_deref());
        let existing_count = items.iter().filter(|item| item.exists).count();
        let missing_count = items.len().saturating_sub(existing_count);
        let label = agent_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("全部员工");
        return Ok(SourceFileSnapshotResponse {
            source_path: resolve_openclaw_home_path().display().to_string(),
            detail: format!(
                "已整理 {} 的 {} 份工具文件（存在 {}，缺失 {}）。",
                label,
                items.len(),
                existing_count,
                missing_count
            ),
            items,
        });
    }

    Err("不支持的资源类型。".to_string())
}

fn guess_audio_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("m4a") => "audio/mp4",
        Some("aac") => "audio/aac",
        Some("ogg") => "audio/ogg",
        Some("flac") => "audio/flac",
        _ => "application/octet-stream",
    }
}

#[tauri::command]
fn read_local_audio_file(path: String) -> Result<AudioFilePayload, String> {
    let resolved = PathBuf::from(path.trim());
    if resolved.as_os_str().is_empty() {
        return Err("音频路径不能为空。".to_string());
    }

    if !resolved.is_file() {
        return Err(format!("音频文件不存在：{}", resolved.display()));
    }

    let bytes = std::fs::read(&resolved)
        .map_err(|error| format!("读取音频文件失败（{}）：{error}", resolved.display()))?;
    let mime_type = guess_audio_mime_type(&resolved).to_string();
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let data_url = format!("data:{mime_type};base64,{encoded}");
    let file_name = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("audio")
        .to_string();

    Ok(AudioFilePayload {
        data_url,
        mime_type,
        file_name,
    })
}

#[tauri::command]
fn save_source_file(kind: String, source_path: String, content: String) -> Result<String, String> {
    let allowed = if kind == "memory" {
        load_memory_file_items()
    } else if kind == "document" {
        load_document_file_items()
    } else if kind == "skill" {
        load_skill_file_items()
    } else if kind == "tool" {
        load_tool_file_items(None)
    } else {
        return Err("不支持的文件类型。".to_string());
    };

    let Some(target) = allowed
        .into_iter()
        .find(|item| std::path::Path::new(&item.source_path) == std::path::Path::new(&source_path))
    else {
        return Err("目标文件不在允许编辑范围内。".to_string());
    };

    if let Some(parent) = Path::new(&target.source_path).parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(&target.source_path, content).map_err(|error| error.to_string())?;
    Ok(target.source_path)
}

fn load_recent_activity_from_session_file(session_file: &str) -> (Option<String>, Option<String>) {
    let raw = match std::fs::read_to_string(session_file) {
        Ok(value) => value,
        Err(_) => return (None, None),
    };

    let mut current_work = None;
    let mut recent_output = None;

    for line in raw.lines().rev() {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let Some(message) = parsed.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let content = message.get("content");

        if recent_output.is_none() && role == "assistant" {
            if let Some(content) = content.and_then(extract_text_from_message_content) {
                recent_output = Some(sanitize_staff_output(&content));
            }
        }

        if current_work.is_none() && role == "user" {
            if let Some(content) = content.and_then(extract_text_from_message_content) {
                current_work = Some(sanitize_staff_output(&content));
            }
        }

        if current_work.is_some() && recent_output.is_some() {
            break;
        }
    }

    (current_work, recent_output)
}

fn derive_status_label(updated_at_ms: i64) -> String {
    let now_ms = current_timestamp_millis() as i64;
    let delta = now_ms.saturating_sub(updated_at_ms);
    if delta <= 45 * 60 * 1000 {
        "工作中".to_string()
    } else {
        "待命".to_string()
    }
}

fn load_scheduled_agents() -> std::collections::HashSet<String> {
    let cron_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");
    let raw = match std::fs::read_to_string(cron_path) {
        Ok(value) => value,
        Err(_) => return std::collections::HashSet::new(),
    };
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => return std::collections::HashSet::new(),
    };

    let mut agents = std::collections::HashSet::new();
    if let Some(jobs) = parsed.get("jobs").and_then(Value::as_array) {
        for job in jobs {
            let Some(obj) = job.as_object() else {
                continue;
            };
            let enabled = obj.get("enabled").and_then(Value::as_bool).unwrap_or(true);
            if !enabled {
                continue;
            }
            for key in ["agentId", "ownerAgentId", "sessionTarget"] {
                if let Some(agent_id) = obj.get(key).and_then(Value::as_str) {
                    let trimmed = agent_id.trim();
                    if !trimmed.is_empty() && trimmed != "isolated" {
                        agents.insert(trimmed.to_string());
                    }
                }
            }
        }
    }
    agents
}

#[derive(Debug, Default)]
struct RuntimeSessionSummary {
    latest_updated_at_ms: Option<i64>,
    latest_model: Option<String>,
    latest_session_file: Option<String>,
}

fn load_runtime_session_summary(agent_id: &str) -> RuntimeSessionSummary {
    let sessions_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents")
        .join(agent_id)
        .join("sessions")
        .join("sessions.json");

    let raw = match std::fs::read_to_string(sessions_path) {
        Ok(value) => value,
        Err(_) => return RuntimeSessionSummary::default(),
    };
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => return RuntimeSessionSummary::default(),
    };

    let mut best = RuntimeSessionSummary::default();
    let mut best_updated_at = i64::MIN;
    let Some(entries) = parsed.as_object() else {
        return best;
    };

    for value in entries.values() {
        let Some(session) = value.as_object() else {
            continue;
        };
        let updated_at = session
            .get("updatedAt")
            .and_then(Value::as_i64)
            .or_else(|| session.get("lastActivityAt").and_then(Value::as_i64))
            .unwrap_or(0);
        if updated_at < best_updated_at {
            continue;
        }

        let latest_model = session
            .get("model")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let latest_session_file = session
            .get("sessionFile")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        best_updated_at = updated_at;
        best = RuntimeSessionSummary {
            latest_updated_at_ms: if updated_at > 0 {
                Some(updated_at)
            } else {
                None
            },
            latest_model,
            latest_session_file,
        };
    }

    best
}

fn value_as_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(Value::as_i64).or_else(|| {
        value
            .and_then(Value::as_u64)
            .and_then(|value| i64::try_from(value).ok())
    })
}

fn extract_task_payload_summary(payload: Option<&Value>) -> String {
    let Some(payload) = payload.and_then(Value::as_object) else {
        return "未提供任务说明。".to_string();
    };

    for key in ["text", "prompt", "message", "summary", "description"] {
        if let Some(value) = payload.get(key).and_then(Value::as_str) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    if let Some(kind) = payload.get("kind").and_then(Value::as_str) {
        let trimmed = kind.trim();
        if !trimmed.is_empty() {
            return format!("任务载荷类型：{trimmed}");
        }
    }

    "未提供任务说明。".to_string()
}

fn derive_task_status(enabled: bool, next_run_at_ms: Option<i64>, now_ms: i64) -> (String, String) {
    if !enabled {
        return ("disabled".to_string(), "已停用".to_string());
    }

    if let Some(next_run_at_ms) = next_run_at_ms {
        if next_run_at_ms <= now_ms {
            return ("late".to_string(), "待执行".to_string());
        }
    }

    ("scheduled".to_string(), "已启用".to_string())
}

#[tauri::command]
fn load_task_snapshot() -> Result<TaskSnapshotResponse, String> {
    let source_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");

    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(TaskSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "cron/jobs.json 未找到，当前没有可读取的任务调度数据。".to_string(),
                jobs: Vec::new(),
            })
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => {
            return Ok(TaskSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "cron/jobs.json 解析失败，当前无法读取任务调度数据。".to_string(),
                jobs: Vec::new(),
            })
        }
    };

    let Some(items) = parsed.get("jobs").and_then(Value::as_array) else {
        return Ok(TaskSnapshotResponse {
            source_path: source_path.display().to_string(),
            detail: "cron/jobs.json 中没有 jobs 数组。".to_string(),
            jobs: Vec::new(),
        });
    };

    let now_ms = current_timestamp_millis() as i64;
    let mut jobs = Vec::new();

    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };

        let id = obj
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("unknown-job")
            .to_string();
        let name = obj
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&id)
            .to_string();
        let agent_id = obj
            .get("agentId")
            .and_then(Value::as_str)
            .or_else(|| obj.get("ownerAgentId").and_then(Value::as_str))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("未标注")
            .to_string();
        let session_target = obj
            .get("sessionTarget")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&agent_id)
            .to_string();
        let enabled = obj.get("enabled").and_then(Value::as_bool).unwrap_or(true);
        let delete_after_run = obj
            .get("deleteAfterRun")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let created_at_ms = value_as_i64(obj.get("createdAtMs"));
        let updated_at_ms = value_as_i64(obj.get("updatedAtMs"));
        let schedule = obj.get("schedule").and_then(Value::as_object);
        let schedule_kind = schedule
            .and_then(|value| value.get("kind"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("unknown")
            .to_string();
        let next_run_at_ms = obj
            .get("state")
            .and_then(Value::as_object)
            .and_then(|value| value.get("nextRunAtMs"))
            .and_then(Value::as_i64)
            .or_else(|| {
                obj.get("state")
                    .and_then(Value::as_object)
                    .and_then(|value| value.get("nextRunAtMs"))
                    .and_then(Value::as_u64)
                    .and_then(|value| i64::try_from(value).ok())
            });
        let summary = extract_task_payload_summary(obj.get("payload"));
        let (status_kind, status_label) = derive_task_status(enabled, next_run_at_ms, now_ms);

        jobs.push(TaskSnapshotItem {
            id,
            name,
            agent_id,
            session_target,
            enabled,
            delete_after_run,
            status_kind,
            status_label,
            summary,
            next_run_at_ms,
            created_at_ms,
            updated_at_ms,
            schedule_kind,
        });
    }

    jobs.sort_by(|left, right| {
        let left_rank = match left.status_kind.as_str() {
            "late" => 0,
            "scheduled" => 1,
            "disabled" => 2,
            _ => 3,
        };
        let right_rank = match right.status_kind.as_str() {
            "late" => 0,
            "scheduled" => 1,
            "disabled" => 2,
            _ => 3,
        };

        left_rank
            .cmp(&right_rank)
            .then_with(|| {
                left.next_run_at_ms
                    .unwrap_or(i64::MAX)
                    .cmp(&right.next_run_at_ms.unwrap_or(i64::MAX))
            })
            .then_with(|| {
                right
                    .updated_at_ms
                    .unwrap_or(0)
                    .cmp(&left.updated_at_ms.unwrap_or(0))
            })
    });

    Ok(TaskSnapshotResponse {
        source_path: source_path.display().to_string(),
        detail: format!("已从 cron/jobs.json 读取 {} 条任务。", jobs.len()),
        jobs,
    })
}

#[tauri::command]
fn load_staff_snapshot() -> Result<StaffSnapshotResponse, String> {
    let source_path = resolve_openclaw_config_path();
    let mission_statement = load_staff_mission_statement();
    let scheduled_agents = load_scheduled_agents();

    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(StaffSnapshotResponse {
                mission_statement,
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 未找到，当前没有可读取的员工配置。".to_string(),
                members: Vec::new(),
            })
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => {
            return Ok(StaffSnapshotResponse {
                mission_statement,
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 解析失败，当前无法读取员工配置。".to_string(),
                members: Vec::new(),
            })
        }
    };

    let root = value_as_object(&parsed);
    let agents_root = root
        .and_then(|obj| obj.get("agents"))
        .and_then(value_as_object);
    let defaults = agents_root
        .and_then(|obj| obj.get("defaults"))
        .and_then(value_as_object);
    let default_model = defaults
        .and_then(|obj| obj.get("model"))
        .and_then(read_string_or_primary)
        .unwrap_or("未标注");
    let default_tools_profile = root
        .and_then(|obj| obj.get("tools"))
        .and_then(value_as_object)
        .and_then(|tools| tools.get("profile"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default");

    let mut members = Vec::new();
    if let Some(list) = agents_root
        .and_then(|obj| obj.get("list"))
        .and_then(Value::as_array)
    {
        for item in list {
            let Some(obj) = value_as_object(item) else {
                continue;
            };
            let agent_id = obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("");
            if agent_id.is_empty() {
                continue;
            }

            let display_name = obj
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(agent_id);
            let model = obj
                .get("model")
                .and_then(read_string_or_primary)
                .unwrap_or(default_model);
            let workspace = obj
                .get("workspace")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("未标注");
            let tools_profile = obj
                .get("tools")
                .and_then(value_as_object)
                .and_then(|tools| tools.get("profile"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(default_tools_profile);
            let runtime_summary = load_runtime_session_summary(agent_id);
            let status_label = runtime_summary
                .latest_updated_at_ms
                .map(derive_status_label)
                .unwrap_or_else(|| "待命".to_string());
            let (current_work, recent_output) = runtime_summary
                .latest_session_file
                .as_deref()
                .map(load_recent_activity_from_session_file)
                .unwrap_or((None, None));
            let effective_model = if model == "未标注" {
                runtime_summary.latest_model.as_deref().unwrap_or(model)
            } else {
                model
            };

            members.push(StaffMemberSnapshot {
                agent_id: agent_id.to_string(),
                display_name: display_name.to_string(),
                role_label: humanize_agent_role(agent_id),
                model: effective_model.to_string(),
                workspace: workspace.to_string(),
                tools_profile: tools_profile.to_string(),
                status_label,
                current_work_label: "正在处理什么".to_string(),
                current_work: current_work.unwrap_or_else(|| "当前无实时任务".to_string()),
                recent_output: recent_output.unwrap_or_else(|| "最近暂无产出。".to_string()),
                scheduled_label: if scheduled_agents.contains(agent_id) {
                    "已排班".to_string()
                } else {
                    "未排班".to_string()
                },
            });
        }
    }

    members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));

    if members.is_empty() {
        let runtime_members = load_staff_from_runtime_dirs(&scheduled_agents)?;
        if !runtime_members.is_empty() {
            return Ok(StaffSnapshotResponse {
                mission_statement,
                source_path: source_path.display().to_string(),
                detail: format!(
                    "openclaw.json 中 agents.list 为空，已回退为运行时员工目录，共读取 {} 名员工。",
                    runtime_members.len()
                ),
                members: runtime_members,
            });
        }
    }

    Ok(StaffSnapshotResponse {
        mission_statement,
        source_path: source_path.display().to_string(),
        detail: format!("已从 openclaw.json 读取 {} 名员工。", members.len()),
        members,
    })
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

fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        return "/platform".to_string();
    }

    let normalized = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    normalized.trim_end_matches('/').to_string()
}

fn parse_request_path(first_line: &str) -> Option<(String, String)> {
    let mut parts = first_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    Some((method, path))
}

fn find_header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.clone())
}

fn has_header(headers: &[(String, String)], name: &str) -> bool {
    headers
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}

fn build_cors_headers(request_headers: &[(String, String)]) -> Vec<(String, String)> {
    let origin = find_header_value(request_headers, "origin").unwrap_or_else(|| "*".to_string());
    let allow_headers = find_header_value(request_headers, "access-control-request-headers")
        .unwrap_or_else(|| {
            "content-type, authorization, x-api-key, anthropic-version, accept".to_string()
        });

    vec![
        ("Access-Control-Allow-Origin".to_string(), origin),
        ("Access-Control-Allow-Headers".to_string(), allow_headers),
        (
            "Access-Control-Allow-Methods".to_string(),
            "GET, POST, PUT, PATCH, DELETE, OPTIONS".to_string(),
        ),
        (
            "Vary".to_string(),
            "Origin, Access-Control-Request-Headers".to_string(),
        ),
    ]
}

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn read_http_request(
    stream: &mut std::net::TcpStream,
) -> Result<(String, String, Vec<(String, String)>, Vec<u8>), String> {
    let mut buffer = Vec::new();
    let mut header_end = None;
    let mut chunk = [0_u8; 4096];

    while header_end.is_none() {
        let bytes = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes]);
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            header_end = Some(position + 4);
        }
        if buffer.len() > 1024 * 1024 {
            return Err("请求头过大".to_string());
        }
    }

    let header_end = header_end.ok_or_else(|| "无法解析请求头".to_string())?;
    let header_text = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = header_text.split("\r\n").filter(|line| !line.is_empty());
    let first_line = lines.next().ok_or_else(|| "缺少请求行".to_string())?;
    let (method, path) = parse_request_path(first_line).ok_or_else(|| "无效请求行".to_string())?;

    let mut headers = Vec::new();
    let mut content_length = 0_usize;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            let key = name.trim().to_string();
            let normalized = value.trim().to_string();
            if key.eq_ignore_ascii_case("content-length") {
                content_length = normalized.parse::<usize>().unwrap_or(0);
            }
            headers.push((key, normalized));
        }
    }

    let mut body = buffer[header_end..].to_vec();
    while body.len() < content_length {
        let bytes = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..bytes]);
    }

    body.truncate(content_length);
    Ok((method, path, headers, body))
}

fn write_http_response(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: Option<&str>,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        _ => "OK",
    };

    let mut header_lines = format!("HTTP/1.1 {status} {reason}\r\n");
    let mut has_content_type = false;

    for (key, value) in headers {
        if key.eq_ignore_ascii_case("content-length") || key.eq_ignore_ascii_case("connection") {
            continue;
        }
        if key.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        header_lines.push_str(&format!("{key}: {value}\r\n"));
    }

    if !has_content_type {
        header_lines.push_str(&format!(
            "Content-Type: {}\r\n",
            content_type.unwrap_or("application/json")
        ));
    }

    header_lines.push_str(&format!("Content-Length: {}\r\n", body.len()));
    header_lines.push_str("Connection: close\r\n\r\n");

    stream
        .write_all(header_lines.as_bytes())
        .map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn find_platform_by_path<'a>(
    platforms: &'a [LocalProxyPlatform],
    path: &str,
) -> Option<&'a LocalProxyPlatform> {
    platforms
        .iter()
        .filter(|platform| path.starts_with(&normalize_prefix(&platform.path_prefix)))
        .max_by_key(|platform| platform.path_prefix.len())
}

fn proxy_single_request(
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    platforms: Arc<Vec<LocalProxyPlatform>>,
) -> Result<(u16, String, Vec<(String, String)>, Vec<u8>), String> {
    if method.eq_ignore_ascii_case("OPTIONS") {
        return Ok((204, "application/json".to_string(), Vec::new(), Vec::new()));
    }

    let platform = match find_platform_by_path(&platforms, &path) {
        Some(platform) => platform,
        None => {
            let available_prefixes = platforms
                .iter()
                .map(|platform| platform.path_prefix.clone())
                .collect::<Vec<_>>();
            let payload = serde_json::json!({
                "error": "Platform not found",
                "path": path,
                "availablePrefixes": available_prefixes
            });
            return Ok((
                404,
                "application/json".to_string(),
                Vec::new(),
                serde_json::to_vec(&payload).map_err(|error| error.to_string())?,
            ));
        }
    };
    let prefix = normalize_prefix(&platform.path_prefix);
    let actual_path = path.strip_prefix(&prefix).unwrap_or("/");
    let target_url = format!(
        "{}{}",
        platform.base_url.trim_end_matches('/'),
        if actual_path.is_empty() {
            "/"
        } else {
            actual_path
        }
    );
    let protocol = platform.protocol.to_lowercase();
    let api_key = platform.api_key.clone();
    let has_authorization = has_header(&headers, "authorization");
    let has_x_api_key = has_header(&headers, "x-api-key");
    let has_anthropic_version = has_header(&headers, "anthropic-version");

    tauri::async_runtime::block_on(async move {
        let client = reqwest::Client::new();
        let method_value = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|error| format!("无效请求方法: {error}"))?;
        let mut request = client.request(method_value, target_url);

        for (key, value) in headers {
            let lower = key.to_ascii_lowercase();
            if matches!(
                lower.as_str(),
                "host" | "content-length" | "connection" | "origin"
            ) {
                continue;
            }
            request = request.header(&key, value);
        }

        if protocol == "anthropic" {
            if !has_x_api_key && !api_key.trim().is_empty() {
                request = request.header("x-api-key", api_key);
            }
            if !has_anthropic_version {
                request = request.header("anthropic-version", "2023-06-01");
            }
        } else if !has_authorization && !api_key.trim().is_empty() {
            request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
        }

        let response = request
            .body(body)
            .send()
            .await
            .map_err(|error| format!("代理请求失败: {error}"))?;

        let status = response.status().as_u16();
        let response_headers = response
            .headers()
            .iter()
            .filter_map(|(key, value)| {
                if matches!(
                    key.as_str().to_ascii_lowercase().as_str(),
                    "content-length" | "connection" | "transfer-encoding"
                ) {
                    return None;
                }
                value
                    .to_str()
                    .ok()
                    .map(|parsed| (key.to_string(), parsed.to_string()))
            })
            .collect::<Vec<_>>();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/json")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("读取代理响应失败: {error}"))?
            .to_vec();

        Ok((status, content_type, response_headers, bytes))
    })
}

fn run_local_proxy(
    listener: TcpListener,
    stop: Arc<AtomicBool>,
    platforms: Arc<Vec<LocalProxyPlatform>>,
) {
    let _ = listener.set_nonblocking(true);

    while !stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let platforms = Arc::clone(&platforms);
                thread::spawn(move || {
                    let result = (|| -> Result<(), String> {
                        let (method, path, headers, body) = read_http_request(&mut stream)?;
                        let cors_headers = build_cors_headers(&headers);

                        if method.eq_ignore_ascii_case("GET") && path == "/health" {
                            let payload = serde_json::json!({
                                "status": "ok",
                                "timestamp": current_timestamp_millis(),
                                "platforms": platforms
                                    .iter()
                                    .map(|platform| serde_json::json!({
                                        "pathPrefix": platform.path_prefix,
                                        "baseUrl": platform.base_url,
                                        "protocol": platform.protocol
                                    }))
                                    .collect::<Vec<_>>()
                            });
                            let body =
                                serde_json::to_vec(&payload).map_err(|error| error.to_string())?;
                            write_http_response(
                                &mut stream,
                                200,
                                Some("application/json"),
                                &cors_headers,
                                &body,
                            )?;
                            return Ok(());
                        }

                        let (status, content_type, mut response_headers, response_body) =
                            proxy_single_request(method, path, headers, body, platforms)?;
                        response_headers.extend(cors_headers);
                        write_http_response(
                            &mut stream,
                            status,
                            Some(&content_type),
                            &response_headers,
                            &response_body,
                        )?;
                        Ok(())
                    })();

                    if let Err(error) = result {
                        let fallback = format!(r#"{{"error":"{error}"}}"#);
                        let _ = write_http_response(
                            &mut stream,
                            502,
                            Some("application/json"),
                            &[("Access-Control-Allow-Origin".to_string(), "*".to_string())],
                            fallback.as_bytes(),
                        );
                    }
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(40));
            }
            Err(_) => break,
        }
    }
}

#[tauri::command]
fn sync_local_proxy(port: u16, platforms: Vec<LocalProxyPlatform>) -> Result<(), String> {
    let state_mutex = local_proxy_state();
    let mut state = state_mutex
        .lock()
        .map_err(|_| "无法获取本地代理状态锁".to_string())?;

    if let Some(stop) = state.stop_signal.take() {
        stop.store(true, Ordering::SeqCst);
    }
    if let Some(handle) = state.handle.take() {
        let _ = handle.join();
    }

    if platforms.is_empty() {
        return Ok(());
    }

    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| format!("无法启动本地代理，端口 {port} 可能已被占用: {error}"))?;
    let stop_signal = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop_signal);
    let thread_platforms = Arc::new(platforms);
    let handle = thread::spawn(move || run_local_proxy(listener, thread_stop, thread_platforms));

    state.stop_signal = Some(stop_signal);
    state.handle = Some(handle);
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[derive(Debug, Serialize)]
struct MonitorInfo {
    /// Logical position (physical / scale_factor) for comparison with screenX/screenY
    position: (f64, f64),
    /// Logical size (physical / scale_factor)
    size: (f64, f64),
    #[serde(rename = "scaleFactor")]
    scale_factor: f64,
}

#[tauri::command]
fn get_available_monitors(app: tauri::AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let monitors = window
        .available_monitors()
        .map_err(|e| e.to_string())?;
    let list: Vec<MonitorInfo> = monitors
        .into_iter()
        .map(|m| {
            let pos = m.position();
            let size = m.size();
            let scale = m.scale_factor();
            MonitorInfo {
                position: (pos.x as f64 / scale, pos.y as f64 / scale),
                size: (size.width as f64 / scale, size.height as f64 / scale),
                scale_factor: scale,
            }
        })
        .collect();
    Ok(list)
}

#[tauri::command]
fn move_window_to_monitor(app: tauri::AppHandle, index: usize) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let monitors = window
        .available_monitors()
        .map_err(|e| e.to_string())?;
    let monitor = monitors
        .into_iter()
        .nth(index)
        .ok_or_else(|| format!("monitor index out of range: {}", index))?;
    let position = monitor.position();
    let size = monitor.size();
    window
        .set_position(tauri::Position::Physical(*position))
        .map_err(|e| e.to_string())?;
    window
        .set_size(tauri::Size::Physical(*size))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct WindowInnerPosition {
    x: f64,
    y: f64,
}

#[tauri::command]
fn get_window_inner_position(app: tauri::AppHandle) -> Result<WindowInnerPosition, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let pos = window
        .inner_position()
        .map_err(|e| e.to_string())?;
    Ok(WindowInnerPosition {
        x: pos.x as f64,
        y: pos.y as f64,
    })
}

fn toggle_main_window_visibility(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let is_visible = window.is_visible().unwrap_or(true);
    if is_visible {
        let _ = window.hide();
        return;
    }

    let _ = window.show();
    let _ = window.set_always_on_top(true);
}

#[tauri::command]
async fn openclaw_chat(
    messages: Vec<OpenClawMessage>,
    endpoint: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    protocol: Option<String>,
) -> Result<OpenClawResponse, String> {
    let endpoint = endpoint
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("OPENCLAW_API_URL").ok())
        .ok_or_else(|| "未设置可用的聊天接口地址。".to_string())?;
    let request_protocol = protocol
        .unwrap_or_else(|| "openai".to_string())
        .to_lowercase();
    let is_openai_compatible = is_openai_compatible_endpoint(&endpoint);
    let gateway_token = load_openclaw_gateway_token_from_config()
        .or_else(|| std::env::var("OPENCLAW_GATEWAY_TOKEN").ok());
    let api_key = api_key.filter(|value| !value.trim().is_empty());
    let model = model
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("OPENCLAW_MODEL").ok());

    let client = reqwest::Client::new();
    let mut request = client
        .post(endpoint)
        .header(CONTENT_TYPE, "application/json");

    if request_protocol == "anthropic" {
        if let Some(api_key) = api_key.as_deref().filter(|value| !value.trim().is_empty()) {
            request = request.header("x-api-key", api_key);
        }
        request = request.header("anthropic-version", "2023-06-01");
    } else if let Some(token) = gateway_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    } else if let Some(api_key) = api_key
        .as_deref()
        .filter(|api_key| !api_key.trim().is_empty())
    {
        request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
    }

    request = if request_protocol == "anthropic" {
        let model = model.ok_or_else(|| "Anthropic 协议需要模型配置。".to_string())?;
        let system = messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        let anthropic_messages = messages
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

#[tauri::command]
async fn check_openclaw_gateway(endpoint: Option<String>) -> Result<GatewayHealthResponse, String> {
    let endpoint = endpoint
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("OPENCLAW_API_URL").ok());

    let Some(endpoint) = endpoint else {
        return Ok(GatewayHealthResponse {
            status: "unconfigured".to_string(),
            checked_url: None,
            detail: Some("未设置 OPENCLAW_API_URL。".to_string()),
            latency_ms: None,
        });
    };

    let endpoint = endpoint.trim().to_string();
    let mut candidates = Vec::new();

    if let Ok(mut url) = reqwest::Url::parse(&endpoint) {
        url.set_path("/health");
        url.set_query(None);
        url.set_fragment(None);
        candidates.push(url.to_string());
    }
    candidates.push(endpoint.clone());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|error| format!("创建网关检查客户端失败: {error}"))?;

    let mut last_error = None;

    for candidate in candidates {
        let started_at = std::time::Instant::now();
        match client.get(&candidate).send().await {
            Ok(response) => {
                let latency_ms = started_at.elapsed().as_millis();
                let status = response.status();
                let detail = if status.is_success() {
                    Some(format!("HTTP {status}"))
                } else {
                    Some(format!("HTTP {status}，服务可达"))
                };

                return Ok(GatewayHealthResponse {
                    status: "online".to_string(),
                    checked_url: Some(candidate),
                    detail,
                    latency_ms: Some(latency_ms),
                });
            }
            Err(error) => {
                last_error = Some(format!("{candidate}: {error}"));
            }
        }
    }

    Ok(GatewayHealthResponse {
        status: "offline".to_string(),
        checked_url: Some(endpoint),
        detail: last_error,
        latency_ms: None,
    })
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("仅支持打开 http 或 https 链接。".to_string());
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(trimmed);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", trimmed]);
        command
    };

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(trimmed);
        command
    };

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("打开外部浏览器失败：{error}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_openclaw_env();

    tauri::Builder::default()
        .plugin(
            GlobalShortcutBuilder::new()
                .with_shortcuts(["Ctrl+`", "Alt+`"])
                .expect("failed to configure global shortcuts")
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_main_window_visibility(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            quit_app,
            get_available_monitors,
            get_window_inner_position,
            move_window_to_monitor,
            openclaw_chat,
            sync_local_proxy,
            check_openclaw_gateway,
            read_local_audio_file,
            load_openclaw_message_logs,
            load_staff_snapshot,
            load_task_snapshot,
            load_memory_file_snapshot,
            load_document_file_snapshot,
            load_openclaw_resource_snapshot,
            load_openclaw_skills_list,
            save_openclaw_skill_enabled,
            load_openclaw_tools_list,
            save_source_file,
            open_external_url
        ])
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
