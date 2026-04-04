use crate::openclaw::message_parsing::{
    extract_message_text, extract_message_timestamp_ms, extract_tool_calls,
    extract_tool_result_text, infer_openclaw_response_status,
};
use crate::openclaw::models::*;
use crate::{
    expand_home_path, resolve_openclaw_config_path, resolve_openclaw_home_path,
    resolve_workspace_agents_root, resolve_workspace_main_root, resolve_workspace_root_for_agent,
    suppress_windows_command_window, value_as_object, write_openclaw_config_value,
};
use base64::Engine;
use getrandom::getrandom;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
pub(crate) fn load_openclaw_message_logs() -> Result<OpenClawMessageLogResponse, String> {
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
    let id = format!(
        "{facet_key}-{}",
        relative_path.replace(['/', '\\', ' '], "-").to_lowercase()
    );

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

    let mut command = Command::new("sqlite3");
    suppress_windows_command_window(&mut command);
    let output = command
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
        resolve_openclaw_home_path()
            .join("workspace")
            .join("skills"),
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
    (
        "1password",
        "1Password 密码管理，设置并使用 1Password CLI 来管理密码",
    ),
    (
        "apple-notes",
        "通过 macOS 上的「备忘录」CLI 管理 Apple Notes",
    ),
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
pub(crate) fn load_openclaw_skills_list() -> Result<OpenClawSkillsListResponse, String> {
    let items = load_skill_file_items();
    let config_path = resolve_openclaw_config_path();
    let mut entries_enabled: std::collections::HashMap<String, bool> =
        std::collections::HashMap::new();
    if let Ok(raw) = std::fs::read_to_string(&config_path) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&raw) {
            if let Some(skills_root) = parsed.get("skills").and_then(Value::as_object) {
                if let Some(entries) = skills_root.get("entries").and_then(Value::as_object) {
                    for (key, val) in entries {
                        let enabled = val.get("enabled").and_then(Value::as_bool).unwrap_or(true);
                        entries_enabled.insert(key.trim().to_lowercase(), enabled);
                    }
                }
            }
        }
    }

    let source_path = resolve_openclaw_home_path()
        .join("skills")
        .display()
        .to_string();

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
                    let Ok(ft) = skill_entry.file_type() else {
                        continue;
                    };
                    if !ft.is_dir() {
                        continue;
                    }
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
        let display_name = if title_is_filename {
            skill_id.clone()
        } else {
            title
        };
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
            if truncated.is_empty() {
                "暂无描述。".to_string()
            } else {
                truncated
            }
        };
        OpenClawSkillListItem {
            id: item_id,
            name: display_name,
            description,
            enabled,
            relative_path,
            source_path,
        }
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
            make_skill_item(
                skill_id,
                item.id,
                item.title,
                item.content,
                item.relative_path,
                item.source_path,
                &entries_enabled,
            )
        })
        .chain(
            // 插件技能（openclaw-lark 等）
            plugin_items.into_iter().map(|item| {
                let skill_id = openclaw_skill_id_from_path(&item.relative_path);
                make_skill_item(
                    skill_id,
                    item.id,
                    item.title,
                    item.content,
                    item.relative_path,
                    item.source_path,
                    &entries_enabled,
                )
            }),
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
pub(crate) fn save_openclaw_skill_enabled(skill_id: String, enabled: bool) -> Result<(), String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut parsed: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
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
    write_openclaw_config_value(&config_path, &parsed)
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
        "sessions_list" | "sessions_history" | "sessions_send" | "sessions_spawn"
        | "session_status" => "Sessions",
        "message" => "Messaging",
        "browser" | "canvas" => "UI",
        "cron" | "gateway" => "Automation",
        "nodes" => "Nodes",
        _ => "Other",
    }
}

/// 解析 profile 得到基础允许的工具 id 集合；full = 全部
pub(crate) fn openclaw_profile_tool_ids(profile: &str) -> std::collections::HashSet<String> {
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
            for id in &[
                "read",
                "write",
                "edit",
                "apply_patch",
                "exec",
                "bash",
                "process",
                "sessions_list",
                "sessions_history",
                "sessions_send",
                "sessions_spawn",
                "session_status",
                "memory_search",
                "memory_get",
                "image",
            ] {
                set.insert((*id).to_string());
            }
            return set;
        }
        "messaging" => {
            for id in &[
                "message",
                "sessions_list",
                "sessions_history",
                "sessions_send",
                "session_status",
            ] {
                set.insert((*id).to_string());
            }
            return set;
        }
        _ => {}
    }
    set
}

/// 从 config 的 allow/deny 数组解析出允许的工具 id（* 表示全部）；deny 优先
pub(crate) fn openclaw_resolve_tools_from_config(
    root: &serde_json::Map<String, Value>,
    agent_obj: Option<&serde_json::Map<String, Value>>,
) -> (String, String, std::collections::HashSet<String>) {
    let tools_root = root.get("tools").and_then(value_as_object);
    let _agents_list = root
        .get("agents")
        .and_then(value_as_object)
        .and_then(|o| o.get("list"))
        .and_then(Value::as_array);
    let default_profile = tools_root
        .and_then(|t| t.get("profile").and_then(Value::as_str))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");
    let mut profile = default_profile.to_string();
    let mut allow: Option<Vec<String>> = tools_root
        .and_then(|t| t.get("allow"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                .collect()
        });
    let mut deny: Vec<String> = tools_root
        .and_then(|t| t.get("deny"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_ascii_lowercase()))
                .collect()
        })
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
                allow = Some(
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .collect(),
                );
            }
            if let Some(d) = tools.get("deny").and_then(Value::as_array) {
                deny = d
                    .iter()
                    .filter_map(|v| {
                        v.as_str()
                            .map(|s| s.trim().to_ascii_lowercase().to_string())
                    })
                    .collect();
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
pub(crate) fn load_openclaw_tools_list(
    agent_id: Option<String>,
) -> Result<OpenClawToolsListResponse, String> {
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
                        let aid = obj
                            .get("id")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .unwrap_or("");
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

/// 保存 OpenClaw 工具配置：
/// - scope = "agent"：写入 agents.list[].tools（需要 agent_id）
/// - scope = "global"：写入根 tools
/// enabled_tool_ids 为最终启用集合（会转换为 allow/deny）
#[tauri::command]
pub(crate) fn save_openclaw_tools_config(
    agent_id: Option<String>,
    scope: Option<String>,
    profile: Option<String>,
    enabled_tool_ids: Vec<String>,
) -> Result<(), String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut parsed: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let scope_mode = scope
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("agent")
        .to_ascii_lowercase();

    let canonical_tool_ids: std::collections::HashSet<String> = OPENCLAW_TOOL_INVENTORY
        .iter()
        .map(|(id, _, _)| (*id).to_string())
        .collect();

    let mut enabled_set = std::collections::HashSet::new();
    for raw_id in enabled_tool_ids {
        let trimmed = raw_id.trim();
        if trimmed.is_empty() {
            continue;
        }
        let canonical = OPENCLAW_TOOL_INVENTORY
            .iter()
            .find_map(|(id, _, _)| {
                if id.eq_ignore_ascii_case(trimmed) {
                    Some((*id).to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| trimmed.to_ascii_lowercase());
        enabled_set.insert(canonical);
    }

    // 按固定顺序输出 allow，便于 diff 稳定可读。
    let mut ordered_allow: Vec<String> = OPENCLAW_TOOL_INVENTORY
        .iter()
        .map(|(id, _, _)| (*id).to_string())
        .filter(|id| enabled_set.contains(id))
        .collect();
    let mut extras: Vec<String> = enabled_set
        .iter()
        .filter(|id| !canonical_tool_ids.contains((*id).as_str()))
        .cloned()
        .collect();
    extras.sort();
    let has_extras = !extras.is_empty();
    ordered_allow.extend(extras);

    let known_enabled_count = OPENCLAW_TOOL_INVENTORY
        .iter()
        .filter(|(id, _, _)| enabled_set.contains(*id))
        .count();

    let allow = if known_enabled_count == OPENCLAW_TOOL_INVENTORY.len() && !has_extras {
        vec![Value::String("*".to_string())]
    } else if ordered_allow.is_empty() {
        vec![Value::String("*".to_string())]
    } else {
        ordered_allow
            .iter()
            .map(|id| Value::String(id.to_string()))
            .collect()
    };

    let deny = if ordered_allow.is_empty() {
        Some(vec![Value::String("*".to_string())])
    } else {
        None
    };

    let profile_value = profile
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default")
        .to_string();

    let tools_obj = match scope_mode.as_str() {
        "global" => root
            .entry("tools")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .ok_or("tools 不是对象")?,
        "agent" => {
            let agent_key = agent_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or("scope=agent 时 agentId 不能为空")?;
            let agents = root
                .entry("agents")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or("agents 不是对象")?;
            let list = agents
                .entry("list")
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .ok_or("agents.list 不是数组")?;

            let target = list
                .iter_mut()
                .find_map(|item| {
                    let obj = item.as_object_mut()?;
                    let id = obj
                        .get("id")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .unwrap_or("");
                    if id.eq_ignore_ascii_case(agent_key) {
                        Some(obj)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| format!("未找到 id 为 {} 的员工。", agent_key))?;

            target
                .entry("tools")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or("agents.list[].tools 不是对象")?
        }
        _ => {
            return Err("scope 仅支持 agent 或 global".to_string());
        }
    };

    tools_obj.insert("profile".to_string(), Value::String(profile_value));
    tools_obj.insert("allow".to_string(), Value::Array(allow));
    if let Some(deny_arr) = deny {
        tools_obj.insert("deny".to_string(), Value::Array(deny_arr));
    } else {
        tools_obj.remove("deny");
    }

    write_openclaw_config_value(&config_path, &parsed)
}

#[tauri::command]
pub(crate) fn save_openclaw_agent_model(agent_id: String, model: String) -> Result<(), String> {
    let normalized_agent_id = agent_id.trim();
    if normalized_agent_id.is_empty() {
        return Err("agentId 不能为空".to_string());
    }
    let normalized_model = model.trim();
    if normalized_model.is_empty() {
        return Err("model 不能为空".to_string());
    }

    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let agents = root
        .entry("agents")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 agents 不是对象")?;

    let mut updated_in_list = false;
    let has_non_empty_list = agents
        .get("list")
        .and_then(Value::as_array)
        .map(|list| !list.is_empty())
        .unwrap_or(false);

    if let Some(list) = agents.get_mut("list").and_then(Value::as_array_mut) {
        if let Some(target) = list.iter_mut().find_map(|item| {
            let obj = item.as_object_mut()?;
            let id = obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            if id.eq_ignore_ascii_case(normalized_agent_id) {
                Some(obj)
            } else {
                None
            }
        }) {
            target.insert(
                "model".to_string(),
                Value::String(normalized_model.to_string()),
            );
            updated_in_list = true;
        }
    }

    if !updated_in_list {
        if has_non_empty_list {
            return Err(format!("未找到 id 为 {} 的员工。", normalized_agent_id));
        }

        if !matches!(agents.get("defaults"), Some(Value::Object(_))) {
            agents.insert("defaults".to_string(), serde_json::json!({}));
        }
        let defaults = agents
            .get_mut("defaults")
            .and_then(Value::as_object_mut)
            .ok_or("openclaw.json 的 agents.defaults 不是对象")?;
        let existing_fallbacks = defaults
            .get("model")
            .and_then(Value::as_object)
            .and_then(|obj| obj.get("fallbacks"))
            .cloned();
        let fallbacks = match existing_fallbacks {
            Some(Value::Array(items)) => Value::Array(items),
            _ => Value::Array(Vec::new()),
        };
        defaults.insert(
            "model".to_string(),
            serde_json::json!({
                "primary": normalized_model.to_string(),
                "fallbacks": fallbacks
            }),
        );
    }

    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
pub(crate) fn load_memory_file_snapshot() -> Result<SourceFileSnapshotResponse, String> {
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
pub(crate) fn load_document_file_snapshot() -> Result<SourceFileSnapshotResponse, String> {
    let items = load_document_file_items();
    let existing_count = items.iter().filter(|item| item.exists).count();
    let missing_count = items.len().saturating_sub(existing_count);
    Ok(SourceFileSnapshotResponse {
        source_path: resolve_workspace_main_root().display().to_string(),
        detail: format!(
            "已整理 {} 份核心文件（存在 {}，缺失 {}）。",
            items.len(),
            existing_count,
            missing_count
        ),
        items,
    })
}

#[tauri::command]
pub(crate) fn load_openclaw_resource_snapshot(
    kind: String,
    agent_id: Option<String>,
) -> Result<SourceFileSnapshotResponse, String> {
    let normalized_kind = kind.trim().to_ascii_lowercase();

    if normalized_kind == "skill" {
        let items = load_skill_file_items();
        let existing_count = items.iter().filter(|item| item.exists).count();
        let missing_count = items.len().saturating_sub(existing_count);
        return Ok(SourceFileSnapshotResponse {
            source_path: resolve_openclaw_home_path()
                .join("skills")
                .display()
                .to_string(),
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
    match guess_media_mime_type(path) {
        "application/octet-stream" => "application/octet-stream",
        resolved => resolved,
    }
}

fn guess_media_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("avif") => "image/avif",
        Some("heic") | Some("heif") => "image/heic",
        Some("tif") | Some("tiff") => "image/tiff",
        Some("ico") => "image/x-icon",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("m4a") => "audio/mp4",
        Some("aac") => "audio/aac",
        Some("ogg") => "audio/ogg",
        Some("flac") => "audio/flac",
        Some("opus") => "audio/opus",
        Some("wma") => "audio/x-ms-wma",
        Some("amr") => "audio/amr",
        Some("aif") | Some("aiff") => "audio/aiff",
        Some("caf") => "audio/x-caf",
        Some("alac") => "audio/alac",
        Some("mp4") | Some("m4v") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        Some("avi") => "video/x-msvideo",
        Some("wmv") => "video/x-ms-wmv",
        Some("flv") => "video/x-flv",
        Some("ogv") => "video/ogg",
        Some("3gp") => "video/3gpp",
        Some("mpeg") | Some("mpg") => "video/mpeg",
        Some("ts") => "video/mp2t",
        Some("m2ts") | Some("mts") => "video/mp2t",
        Some("html") | Some("htm") => "text/html",
        _ => "application/octet-stream",
    }
}

#[tauri::command]
pub(crate) fn read_local_audio_file(path: String) -> Result<AudioFilePayload, String> {
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

const LOCAL_MEDIA_PREVIEW_MAX_BYTES: usize = 36 * 1024 * 1024;
const CHAT_ATTACHMENT_PERSIST_MAX_BYTES: usize = 16 * 1024 * 1024;

#[tauri::command]
pub(crate) fn read_local_media_file(path: String) -> Result<LocalMediaFilePayload, String> {
    let normalized = path.trim();
    if normalized.is_empty() {
        return Err("本地文件路径不能为空。".to_string());
    }

    let expanded = expand_home_path(normalized);
    let resolved = if expanded.exists() {
        std::fs::canonicalize(&expanded).unwrap_or(expanded)
    } else {
        expanded
    };

    if !resolved.exists() || !resolved.is_file() {
        return Err(format!("本地文件不存在：{}", resolved.display()));
    }

    let metadata = std::fs::metadata(&resolved)
        .map_err(|error| format!("读取文件信息失败（{}）：{error}", resolved.display()))?;
    let byte_length = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    if byte_length > LOCAL_MEDIA_PREVIEW_MAX_BYTES {
        let max_mb = LOCAL_MEDIA_PREVIEW_MAX_BYTES / (1024 * 1024);
        return Err(format!(
            "文件过大（{} MB），暂不支持内存预览，请点击文件夹图标打开。",
            max_mb
        ));
    }

    let bytes = std::fs::read(&resolved)
        .map_err(|error| format!("读取本地文件失败（{}）：{error}", resolved.display()))?;
    let mime_type = guess_media_mime_type(&resolved).to_string();
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let data_url = format!("data:{mime_type};base64,{encoded}");
    let file_name = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("media")
        .to_string();

    Ok(LocalMediaFilePayload {
        data_url,
        mime_type,
        file_name,
        byte_length,
    })
}

fn guess_media_extension_from_mime_type(mime_type_raw: &str) -> &'static str {
    match mime_type_raw.trim().to_ascii_lowercase().as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/svg+xml" => "svg",
        "image/heic" => "heic",
        "image/heif" => "heif",
        "image/avif" => "avif",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "audio/mp4" => "m4a",
        "audio/aac" => "aac",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "video/x-msvideo" => "avi",
        "video/x-matroska" => "mkv",
        "text/html" => "html",
        "application/xhtml+xml" => "html",
        _ => "bin",
    }
}

fn sanitize_chat_attachment_file_stem(file_name: &str) -> String {
    let candidate = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("clipboard");
    let sanitized: String = candidate
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&ch) {
                ch
            } else if ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('_').trim();
    if trimmed.is_empty() {
        "clipboard".to_string()
    } else {
        trimmed.chars().take(48).collect()
    }
}

fn build_chat_attachment_output_root(workspace: Option<String>) -> PathBuf {
    if let Some(candidate) = workspace {
        let trimmed = candidate.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_absolute() {
                return path.join(".dragonclaw").join("chat-attachments");
            }
        }
    }
    resolve_openclaw_home_path().join("chat-attachments")
}

#[tauri::command]
pub(crate) fn persist_chat_attachment_data_url(
    file_name: String,
    data_url: String,
    workspace: Option<String>,
) -> Result<String, String> {
    let normalized = data_url.trim();
    if normalized.is_empty() {
        return Err("附件内容为空，无法保存。".to_string());
    }
    if !normalized.starts_with("data:") {
        return Err("仅支持 data URL 附件保存。".to_string());
    }

    let payload = &normalized["data:".len()..];
    let Some((meta, encoded)) = payload.split_once(',') else {
        return Err("附件 data URL 格式无效。".to_string());
    };

    if !meta.to_ascii_lowercase().contains(";base64") {
        return Err("附件 data URL 必须为 base64 编码。".to_string());
    }

    let mime_type = meta
        .split(';')
        .next()
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let extension = Path::new(&file_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| guess_media_extension_from_mime_type(&mime_type).to_string());

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .map_err(|error| format!("附件数据解码失败：{error}"))?;
    if bytes.is_empty() {
        return Err("附件内容为空，无法保存。".to_string());
    }
    if bytes.len() > CHAT_ATTACHMENT_PERSIST_MAX_BYTES {
        let max_mb = CHAT_ATTACHMENT_PERSIST_MAX_BYTES / (1024 * 1024);
        return Err(format!("附件过大，超过 {max_mb} MB 限制。"));
    }

    let output_root = build_chat_attachment_output_root(workspace);
    std::fs::create_dir_all(&output_root)
        .map_err(|error| format!("创建附件目录失败（{}）：{error}", output_root.display()))?;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let mut random_bytes = [0_u8; 4];
    if getrandom(&mut random_bytes).is_err() {
        random_bytes = [0x12, 0x34, 0x56, 0x78];
    }
    let random_hex = random_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let safe_stem = sanitize_chat_attachment_file_stem(&file_name);
    let output_name = format!("{safe_stem}-{now_ms}-{random_hex}.{extension}");
    let output_path = output_root.join(output_name);

    std::fs::write(&output_path, bytes)
        .map_err(|error| format!("保存附件失败（{}）：{error}", output_path.display()))?;

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub(crate) fn save_source_file(
    kind: String,
    source_path: String,
    content: String,
) -> Result<String, String> {
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
