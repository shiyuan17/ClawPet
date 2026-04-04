use crate::openclaw::message_parsing::{
    extract_message_text, extract_message_timestamp_ms, extract_text_from_message_content,
    sanitize_staff_output,
};
use crate::openclaw::models::*;
use crate::{read_string_or_primary, resolve_channels_from_bindings, value_as_object};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) fn load_staff_from_runtime_dirs(
    scheduled_agents: &std::collections::HashSet<String>,
    channels_by_agent: &std::collections::HashMap<String, String>,
    default_model: &str,
    default_tools_profile: &str,
) -> Result<Vec<StaffMemberSnapshot>, String> {
    let agents_path = crate::resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents");

    let entries = match std::fs::read_dir(&agents_path) {
        Ok(value) => value,
        Err(_) => return Ok(Vec::new()),
    };

    let mut members = Vec::new();
    let preferred_model = default_model.trim();
    let preferred_model = if preferred_model.is_empty() || preferred_model == "未标注" {
        None
    } else {
        Some(preferred_model.to_string())
    };
    let preferred_tools_profile = {
        let normalized = default_tools_profile.trim();
        if normalized.is_empty() {
            "default".to_string()
        } else {
            normalized.to_string()
        }
    };
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
        let channel = channels_by_agent
            .get(&agent_id)
            .cloned()
            .unwrap_or_default();
        members.push(StaffMemberSnapshot {
            agent_id: agent_id.clone(),
            display_name: if agent_id.eq_ignore_ascii_case("main") {
                MAIN_STAFF_DISPLAY_NAME.to_string()
            } else {
                agent_id.clone()
            },
            role_label: humanize_agent_role(&agent_id),
            channel,
            model: preferred_model
                .clone()
                .or(runtime_summary.latest_model)
                .unwrap_or_else(|| "未标注".to_string()),
            workspace: "未标注".to_string(),
            tools_profile: preferred_tools_profile.clone(),
            tools_enabled_count: crate::openclaw_profile_tool_ids(&preferred_tools_profile).len(),
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

pub(crate) fn humanize_agent_role(agent_id: &str) -> String {
    match agent_id.trim().to_lowercase().as_str() {
        "main" => "主控员工".to_string(),
        "gateway" => "网关员工".to_string(),
        other => format!("{other} 员工"),
    }
}

pub(crate) const MAIN_STAFF_DISPLAY_NAME: &str = "超级管理者";

pub(crate) fn ensure_agents_list_has_main(list: &mut Vec<Value>) {
    let mut has_main = false;
    for item in list.iter_mut() {
        let Some(obj) = item.as_object_mut() else {
            continue;
        };
        let current_id = obj
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if !current_id.eq_ignore_ascii_case("main") {
            continue;
        }
        has_main = true;
        obj.insert("id".to_string(), Value::String("main".to_string()));
        obj.insert(
            "name".to_string(),
            Value::String(MAIN_STAFF_DISPLAY_NAME.to_string()),
        );
        let has_workspace = obj
            .get("workspace")
            .and_then(Value::as_str)
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
        if !has_workspace {
            obj.insert(
                "workspace".to_string(),
                Value::String("~/.openclaw/workspace-main".to_string()),
            );
        }
        break;
    }

    if !has_main {
        list.push(serde_json::json!({
            "id": "main",
            "name": MAIN_STAFF_DISPLAY_NAME,
            "workspace": "~/.openclaw/workspace-main"
        }));
    }
}

pub(crate) fn build_main_staff_snapshot(
    scheduled_agents: &std::collections::HashSet<String>,
    channels_by_agent: &std::collections::HashMap<String, String>,
    default_model: &str,
    default_tools_profile: &str,
) -> StaffMemberSnapshot {
    let runtime_summary = load_runtime_session_summary("main");
    let status_label = runtime_summary
        .latest_updated_at_ms
        .map(derive_status_label)
        .unwrap_or_else(|| "待命".to_string());
    let (current_work, recent_output) = runtime_summary
        .latest_session_file
        .as_deref()
        .map(load_recent_activity_from_session_file)
        .unwrap_or((None, None));

    StaffMemberSnapshot {
        agent_id: "main".to_string(),
        display_name: MAIN_STAFF_DISPLAY_NAME.to_string(),
        role_label: humanize_agent_role("main"),
        channel: channels_by_agent.get("main").cloned().unwrap_or_default(),
        model: runtime_summary
            .latest_model
            .unwrap_or_else(|| default_model.trim().to_string()),
        workspace: "~/.openclaw/workspace-main".to_string(),
        tools_profile: default_tools_profile.trim().to_string(),
        tools_enabled_count: crate::openclaw_profile_tool_ids(default_tools_profile).len(),
        status_label,
        current_work_label: "正在处理什么".to_string(),
        current_work: current_work.unwrap_or_else(|| "当前无实时任务".to_string()),
        recent_output: recent_output.unwrap_or_else(|| "最近暂无产出。".to_string()),
        scheduled_label: if scheduled_agents.contains("main") {
            "已排班".to_string()
        } else {
            "未排班".to_string()
        },
    }
}
pub(crate) fn load_recent_activity_from_session_file(
    session_file: &str,
) -> (Option<String>, Option<String>) {
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

pub(crate) fn derive_status_label(updated_at_ms: i64) -> String {
    let now_ms = crate::current_timestamp_millis() as i64;
    let delta = now_ms.saturating_sub(updated_at_ms);
    if delta <= 45 * 60 * 1000 {
        "工作中".to_string()
    } else {
        "待命".to_string()
    }
}

pub(crate) fn load_scheduled_agents() -> std::collections::HashSet<String> {
    let cron_path = crate::resolve_openclaw_config_path()
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
    let sessions_path = crate::resolve_openclaw_config_path()
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

pub(crate) fn derive_session_target_from_session_key(
    session_key: &str,
    agent_id: &str,
) -> Option<String> {
    let normalized_agent_id = agent_id.trim().to_lowercase();
    if normalized_agent_id.is_empty() {
        return None;
    }
    let normalized_session_key = session_key.trim().to_lowercase();
    let prefix = format!("agent:{normalized_agent_id}:");
    if !normalized_session_key.starts_with(&prefix) {
        return None;
    }
    let target = normalized_session_key[prefix.len()..].trim();
    if target.is_empty() {
        return Some(normalized_agent_id);
    }
    Some(target.to_string())
}

pub(crate) fn count_session_messages_from_file(session_file: &str) -> usize {
    let raw = match std::fs::read_to_string(session_file) {
        Ok(value) => value,
        Err(_) => return 0,
    };

    let mut count = 0usize;
    for line in raw.lines() {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if parsed.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }
        let Some(message) = parsed.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if role == "user" || role == "assistant" {
            count = count.saturating_add(1);
        }
    }
    count
}

pub(crate) fn build_session_preview_text(session_file: &str) -> String {
    let (current_work, recent_output) = load_recent_activity_from_session_file(session_file);
    if let Some(output) = recent_output {
        if !output.trim().is_empty() {
            return output;
        }
    }
    if let Some(work) = current_work {
        if !work.trim().is_empty() {
            return work;
        }
    }
    "暂无消息".to_string()
}

#[tauri::command]
pub(crate) fn load_openclaw_agent_sessions_snapshot(
    agent_id: String,
) -> Result<OpenClawAgentSessionsSnapshotResponse, String> {
    let normalized_agent_id = agent_id.trim().to_lowercase();
    if normalized_agent_id.is_empty() {
        return Ok(OpenClawAgentSessionsSnapshotResponse {
            detail: "agentId 为空，无法读取会话列表。".to_string(),
            sessions: Vec::new(),
        });
    }

    let sessions_path = crate::resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents")
        .join(&normalized_agent_id)
        .join("sessions")
        .join("sessions.json");

    let raw = match std::fs::read_to_string(&sessions_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawAgentSessionsSnapshotResponse {
                detail: format!("未找到 {} 的 OpenClaw 会话文件。", normalized_agent_id),
                sessions: Vec::new(),
            })
        }
    };
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return Err(format!("会话文件解析失败: {error}"));
        }
    };

    let Some(entries) = parsed.as_object() else {
        return Ok(OpenClawAgentSessionsSnapshotResponse {
            detail: "会话文件格式异常。".to_string(),
            sessions: Vec::new(),
        });
    };

    let mut sessions = Vec::new();
    for (session_key, value) in entries {
        let Some(session_target) =
            derive_session_target_from_session_key(session_key, &normalized_agent_id)
        else {
            continue;
        };
        let Some(session_obj) = value.as_object() else {
            continue;
        };

        let session_id = session_obj
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_default();
        let updated_at_ms = value_as_i64(session_obj.get("updatedAt"))
            .or_else(|| value_as_i64(session_obj.get("lastActivityAt")))
            .unwrap_or(0);
        let session_file = session_obj
            .get("sessionFile")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned);
        let fallback_session_file = if session_id.is_empty() {
            None
        } else {
            let fallback_path = sessions_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("{session_id}.jsonl"));
            if fallback_path.exists() {
                Some(fallback_path.display().to_string())
            } else {
                None
            }
        };
        let resolved_session_file = session_file.or(fallback_session_file);
        let message_count = resolved_session_file
            .as_deref()
            .map(count_session_messages_from_file)
            .unwrap_or(0);
        let preview = resolved_session_file
            .as_deref()
            .map(build_session_preview_text)
            .unwrap_or_else(|| "暂无消息".to_string());
        let last_channel = session_obj
            .get("lastChannel")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned);
        let chat_type = session_obj
            .get("chatType")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned);

        sessions.push(OpenClawAgentSessionSnapshotItem {
            session_key: session_key.to_string(),
            session_target,
            session_id,
            updated_at_ms,
            message_count,
            preview,
            last_channel,
            chat_type,
        });
    }

    sessions.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| left.session_target.cmp(&right.session_target))
    });

    Ok(OpenClawAgentSessionsSnapshotResponse {
        detail: format!(
            "已读取 {} 的 {} 条 OpenClaw 会话。",
            normalized_agent_id,
            sessions.len()
        ),
        sessions,
    })
}

#[tauri::command]
pub(crate) fn load_openclaw_agent_session_history(
    agent_id: String,
    session_key: String,
) -> Result<OpenClawAgentSessionHistoryResponse, String> {
    let normalized_agent_id = agent_id.trim().to_lowercase();
    let normalized_session_key = session_key.trim().to_lowercase();
    if normalized_agent_id.is_empty() || normalized_session_key.is_empty() {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "agentId 或 sessionKey 为空。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    }

    if derive_session_target_from_session_key(&normalized_session_key, &normalized_agent_id)
        .is_none()
    {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "会话不属于当前 Agent。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    }

    let sessions_path = crate::resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents")
        .join(&normalized_agent_id)
        .join("sessions")
        .join("sessions.json");

    let raw = match std::fs::read_to_string(&sessions_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawAgentSessionHistoryResponse {
                detail: "未找到 OpenClaw 会话文件。".to_string(),
                session_key: normalized_session_key,
                messages: Vec::new(),
            })
        }
    };
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("会话文件解析失败: {error}"))?;
    let Some(entries) = parsed.as_object() else {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "会话文件格式异常。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    };

    let mut matched_session: Option<&serde_json::Map<String, Value>> = None;
    for (key, value) in entries {
        if key.trim().to_lowercase() != normalized_session_key {
            continue;
        }
        if let Some(session_obj) = value.as_object() {
            matched_session = Some(session_obj);
            break;
        }
    }

    let Some(session_obj) = matched_session else {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "未找到该会话。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    };

    let session_id = session_obj
        .get("sessionId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    let session_file = session_obj
        .get("sessionFile")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            if session_id.is_empty() {
                return None;
            }
            let fallback = sessions_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("{session_id}.jsonl"));
            if fallback.exists() {
                Some(fallback.display().to_string())
            } else {
                None
            }
        });

    let Some(session_file) = session_file else {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "该会话缺少可读的 sessionFile。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    };

    let session_raw = std::fs::read_to_string(&session_file)
        .map_err(|error| format!("读取会话消息失败: {error}"))?;
    let mut messages = Vec::new();
    for (index, line) in session_raw.lines().enumerate() {
        let parsed_line: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if parsed_line.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }
        let Some(message) = parsed_line.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if role != "user" && role != "assistant" {
            continue;
        }
        let Some(text) = extract_message_text(message) else {
            continue;
        };
        let fallback_created_at = i64::try_from(index).unwrap_or(0);
        let created_at_ms = extract_message_timestamp_ms(message, fallback_created_at);
        let message_id = message
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("msg-{index}"));
        messages.push(OpenClawAgentSessionHistoryMessage {
            id: message_id,
            role: role.to_string(),
            text,
            created_at_ms,
        });
    }

    Ok(OpenClawAgentSessionHistoryResponse {
        detail: format!(
            "已读取会话 {} 的 {} 条消息。",
            normalized_session_key,
            messages.len()
        ),
        session_key: normalized_session_key,
        messages,
    })
}

pub(crate) fn value_as_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(Value::as_i64).or_else(|| {
        value
            .and_then(Value::as_u64)
            .and_then(|value| i64::try_from(value).ok())
    })
}

pub(crate) fn extract_agent_id_from_session_key(value: Option<&Value>) -> Option<String> {
    let session_key = value.and_then(Value::as_str).map(str::trim)?;
    if session_key.is_empty() {
        return None;
    }

    let rest = session_key.strip_prefix("agent:")?;
    let candidate = rest.split(':').next().map(str::trim).unwrap_or("");
    if candidate.is_empty() {
        return None;
    }

    Some(candidate.to_string())
}

pub(crate) fn extract_task_payload_summary(payload: Option<&Value>) -> String {
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

pub(crate) fn derive_task_status(
    enabled: bool,
    next_run_at_ms: Option<i64>,
    now_ms: i64,
) -> (String, String) {
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
pub(crate) fn load_task_snapshot(agent_id: Option<String>) -> Result<TaskSnapshotResponse, String> {
    let requested_agent_id = agent_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let requested_agent_lower = requested_agent_id
        .as_deref()
        .map(|value| value.to_ascii_lowercase());
    let source_path = crate::resolve_openclaw_config_path()
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

    let now_ms = crate::current_timestamp_millis() as i64;
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
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                obj.get("ownerAgentId")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            })
            .or_else(|| extract_agent_id_from_session_key(obj.get("sessionKey")))
            .unwrap_or_else(|| "未标注".to_string());
        if let Some(expected_agent) = requested_agent_lower.as_deref() {
            if agent_id.to_ascii_lowercase() != expected_agent {
                continue;
            }
        }
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

    let detail = if let Some(requested_agent_id) = requested_agent_id {
        format!(
            "已从 cron/jobs.json 读取 {} 条任务（角色：{}）。",
            jobs.len(),
            requested_agent_id
        )
    } else {
        format!("已从 cron/jobs.json 读取 {} 条任务。", jobs.len())
    };

    Ok(TaskSnapshotResponse {
        source_path: source_path.display().to_string(),
        detail,
        jobs,
    })
}

#[tauri::command]
pub(crate) fn set_task_enabled(task_id: String, enabled: bool) -> Result<(), String> {
    let source_path = crate::resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");

    let raw = std::fs::read_to_string(&source_path)
        .map_err(|err| format!("无法读取 cron/jobs.json: {}", err))?;

    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|err| format!("cron/jobs.json 解析失败: {}", err))?;

    let jobs = parsed
        .get_mut("jobs")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "cron/jobs.json 中没有 jobs 数组。".to_string())?;

    let mut found = false;
    for job in jobs.iter_mut() {
        if let Some(obj) = job.as_object_mut() {
            let id = obj.get("id").and_then(Value::as_str).unwrap_or("").trim();
            if id == task_id.trim() {
                obj.insert("enabled".to_string(), Value::Bool(enabled));
                obj.insert(
                    "updatedAtMs".to_string(),
                    Value::Number(serde_json::Number::from(
                        crate::current_timestamp_millis() as i64
                    )),
                );
                found = true;
                break;
            }
        }
    }

    if !found {
        return Err(format!("未找到 id 为 {} 的任务。", task_id));
    }

    let output =
        serde_json::to_string_pretty(&parsed).map_err(|err| format!("序列化失败: {}", err))?;
    std::fs::write(&source_path, output)
        .map_err(|err| format!("写入 cron/jobs.json 失败: {}", err))?;

    Ok(())
}

#[tauri::command]
pub(crate) fn delete_task(task_id: String) -> Result<(), String> {
    let source_path = crate::resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");

    let raw = std::fs::read_to_string(&source_path)
        .map_err(|err| format!("无法读取 cron/jobs.json: {}", err))?;

    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|err| format!("cron/jobs.json 解析失败: {}", err))?;

    let jobs = parsed
        .get_mut("jobs")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "cron/jobs.json 中没有 jobs 数组。".to_string())?;

    let target_id = task_id.trim();
    if target_id.is_empty() {
        return Err("任务 id 不能为空。".to_string());
    }

    let before_len = jobs.len();
    jobs.retain(|job| {
        job.as_object()
            .and_then(|obj| obj.get("id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            != target_id
    });

    if jobs.len() == before_len {
        return Err(format!("未找到 id 为 {} 的任务。", task_id));
    }

    let output =
        serde_json::to_string_pretty(&parsed).map_err(|err| format!("序列化失败: {}", err))?;
    std::fs::write(&source_path, output)
        .map_err(|err| format!("写入 cron/jobs.json 失败: {}", err))?;

    Ok(())
}
pub(crate) fn load_staff_snapshot_blocking() -> Result<StaffSnapshotResponse, String> {
    let source_path = crate::resolve_openclaw_config_path();
    let mission_statement = crate::load_staff_mission_statement();
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

    let root = crate::value_as_object(&parsed);
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

    let channels_by_agent = root.map(resolve_channels_from_bindings).unwrap_or_default();

    let mut members = Vec::new();
    if let Some(list) = agents_root
        .and_then(|obj| obj.get("list"))
        .and_then(Value::as_array)
    {
        for item in list {
            let Some(obj) = crate::value_as_object(item) else {
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
                .unwrap_or(if agent_id.eq_ignore_ascii_case("main") {
                    MAIN_STAFF_DISPLAY_NAME
                } else {
                    agent_id
                });
            let display_name = if agent_id.eq_ignore_ascii_case("main") {
                MAIN_STAFF_DISPLAY_NAME
            } else {
                display_name
            };
            let channel = channels_by_agent.get(agent_id).cloned().unwrap_or_default();
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
            let tools_enabled_count = root
                .map(|root_obj| {
                    let (_, _, allowed_ids) =
                        crate::openclaw_resolve_tools_from_config(root_obj, Some(obj));
                    allowed_ids.len()
                })
                .unwrap_or_else(|| crate::openclaw_profile_tool_ids(tools_profile).len());
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
                channel,
                model: effective_model.to_string(),
                workspace: workspace.to_string(),
                tools_profile: tools_profile.to_string(),
                tools_enabled_count,
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
    if !members.is_empty()
        && !members
            .iter()
            .any(|member| member.agent_id.eq_ignore_ascii_case("main"))
    {
        members.push(build_main_staff_snapshot(
            &scheduled_agents,
            &channels_by_agent,
            default_model,
            default_tools_profile,
        ));
        members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    }

    if members.is_empty() {
        let mut runtime_members = load_staff_from_runtime_dirs(
            &scheduled_agents,
            &channels_by_agent,
            default_model,
            default_tools_profile,
        )?;
        if !runtime_members
            .iter()
            .any(|member| member.agent_id.eq_ignore_ascii_case("main"))
        {
            runtime_members.push(build_main_staff_snapshot(
                &scheduled_agents,
                &channels_by_agent,
                default_model,
                default_tools_profile,
            ));
            runtime_members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
        }
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

#[tauri::command]
pub(crate) async fn load_staff_snapshot() -> Result<StaffSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_staff_snapshot_blocking)
        .await
        .map_err(|error| format!("读取员工快照任务失败：{error}"))?
}
