use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::paths::resolve_openclaw_config_path;
use super::write::write_openclaw_config_value;

#[derive(Default, Debug, Clone)]
pub(crate) struct ChatCompletionsEndpointEnableOutcome {
    changed_paths: Vec<String>,
    unchanged_paths: Vec<String>,
    failures: Vec<String>,
}

impl ChatCompletionsEndpointEnableOutcome {
    pub(crate) fn any_success(&self) -> bool {
        !self.changed_paths.is_empty() || !self.unchanged_paths.is_empty()
    }

    pub(crate) fn changed(&self) -> bool {
        !self.changed_paths.is_empty()
    }

    pub(crate) fn detail(&self) -> String {
        let mut segments = Vec::new();
        if !self.changed_paths.is_empty() {
            segments.push(format!(
                "已写入 gateway.mode=local 且 gateway.http.endpoints.chatCompletions.enabled=true（{}）",
                self.changed_paths.join(", ")
            ));
        }
        if !self.unchanged_paths.is_empty() {
            segments.push(format!(
                "配置已就绪（gateway.mode=local 且 chatCompletions.enabled=true）（{}）",
                self.unchanged_paths.join(", ")
            ));
        }
        if !self.failures.is_empty() {
            segments.push(format!("写入失败（{}）", self.failures.join("；")));
        }
        if segments.is_empty() {
            "未找到可处理的 openclaw 配置路径。".to_string()
        } else {
            segments.join("；")
        }
    }
}

pub(crate) fn collect_openclaw_candidate_config_paths() -> Vec<PathBuf> {
    vec![resolve_openclaw_config_path()]
}

fn ensure_openclaw_chat_completions_endpoint_enabled_at_path(
    config_path: &Path,
) -> Result<bool, String> {
    let raw = match std::fs::read_to_string(config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }

    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
    let gateway_obj = root
        .entry("gateway")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway 字段不是对象")?;
    let mut changed = false;
    let gateway_mode_ready = gateway_obj
        .get("mode")
        .and_then(Value::as_str)
        .map(|value| value.trim().eq_ignore_ascii_case("local"))
        .unwrap_or(false);
    if !gateway_mode_ready {
        gateway_obj.insert("mode".to_string(), Value::String("local".to_string()));
        changed = true;
    }
    let http_obj = gateway_obj
        .entry("http")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway.http 字段不是对象")?;
    let endpoints_obj = http_obj
        .entry("endpoints")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway.http.endpoints 字段不是对象")?;
    let chat_obj = endpoints_obj
        .entry("chatCompletions")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway.http.endpoints.chatCompletions 字段不是对象")?;

    if chat_obj.get("enabled").and_then(Value::as_bool) != Some(true) {
        chat_obj.insert("enabled".to_string(), Value::Bool(true));
        changed = true;
    }
    if !changed {
        return Ok(false);
    }
    write_openclaw_config_value(config_path, &parsed)?;
    Ok(true)
}

pub(crate) fn ensure_openclaw_chat_completions_endpoint_enabled_outcome(
) -> ChatCompletionsEndpointEnableOutcome {
    let mut outcome = ChatCompletionsEndpointEnableOutcome::default();
    for path in collect_openclaw_candidate_config_paths() {
        let display = path.display().to_string();
        match ensure_openclaw_chat_completions_endpoint_enabled_at_path(&path) {
            Ok(true) => outcome.changed_paths.push(display),
            Ok(false) => outcome.unchanged_paths.push(display),
            Err(error) => outcome.failures.push(format!("{display}: {error}")),
        }
    }
    outcome
}

pub(crate) fn ensure_openclaw_chat_completions_endpoint_enabled() -> Result<bool, String> {
    let outcome = ensure_openclaw_chat_completions_endpoint_enabled_outcome();
    if outcome.any_success() {
        if !outcome.failures.is_empty() {
            eprintln!(
                "OpenClaw chatCompletions 端点自动启用存在部分失败：{}",
                outcome.detail()
            );
        }
        return Ok(outcome.changed());
    }

    Err(format!(
        "无法确保 gateway.mode=local 与 gateway.http.endpoints.chatCompletions.enabled=true：{}",
        outcome.detail()
    ))
}

fn remove_unknown_keys_from_object(
    object: &mut serde_json::Map<String, Value>,
    allowed_keys: &[&str],
) -> Vec<String> {
    let keys = object.keys().cloned().collect::<Vec<_>>();
    let mut removed = Vec::new();
    for key in keys {
        if !allowed_keys.iter().any(|allowed| key == *allowed) {
            object.remove(&key);
            removed.push(key);
        }
    }
    removed
}

fn sanitize_openclaw_models_provider_schema_at_path(
    config_path: &Path,
) -> Result<Option<String>, String> {
    let raw = match std::fs::read_to_string(config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let Some(root_obj) = parsed.as_object_mut() else {
        return Ok(None);
    };
    let Some(models_obj) = root_obj.get_mut("models").and_then(Value::as_object_mut) else {
        return Ok(None);
    };
    let Some(providers_obj) = models_obj
        .get_mut("providers")
        .and_then(Value::as_object_mut)
    else {
        return Ok(None);
    };

    let provider_allowed_keys = [
        "baseUrl",
        "apiKey",
        "auth",
        "api",
        "injectNumCtxForOpenAICompat",
        "headers",
        "authHeader",
        "models",
    ];
    let model_allowed_keys = [
        "id",
        "name",
        "api",
        "reasoning",
        "input",
        "cost",
        "contextWindow",
        "maxTokens",
        "headers",
        "compat",
    ];

    let mut changed = false;
    let mut notes = Vec::new();

    for (provider_id, provider_value) in providers_obj.iter_mut() {
        let Some(provider_obj) = provider_value.as_object_mut() else {
            continue;
        };

        let removed_provider_keys =
            remove_unknown_keys_from_object(provider_obj, &provider_allowed_keys);
        if !removed_provider_keys.is_empty() {
            changed = true;
            for key in removed_provider_keys {
                notes.push(format!("{provider_id}.{key}"));
            }
        }

        if !matches!(provider_obj.get("models"), Some(Value::Array(_))) {
            provider_obj.insert("models".to_string(), Value::Array(Vec::new()));
            changed = true;
            notes.push(format!("{provider_id}.models"));
        }

        let Some(models_array) = provider_obj.get_mut("models").and_then(Value::as_array_mut)
        else {
            continue;
        };

        let existing_models = std::mem::take(models_array);
        let mut next_models = Vec::with_capacity(existing_models.len());
        for (index, model_value) in existing_models.into_iter().enumerate() {
            let Some(mut model_obj) = model_value.as_object().cloned() else {
                changed = true;
                notes.push(format!("{provider_id}.models[{index}]"));
                continue;
            };

            let removed_model_keys =
                remove_unknown_keys_from_object(&mut model_obj, &model_allowed_keys);
            if !removed_model_keys.is_empty() {
                changed = true;
                for key in removed_model_keys {
                    notes.push(format!("{provider_id}.models[{index}].{key}"));
                }
            }

            let model_id = model_obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let model_name = model_obj
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            match (model_id, model_name) {
                (None, None) => {
                    changed = true;
                    notes.push(format!("{provider_id}.models[{index}].id/name"));
                    continue;
                }
                (None, Some(name)) => {
                    model_obj.insert("id".to_string(), Value::String(name.clone()));
                    changed = true;
                    notes.push(format!("{provider_id}.models[{index}].id"));
                    model_obj.insert("name".to_string(), Value::String(name));
                }
                (Some(id), None) => {
                    model_obj.insert("name".to_string(), Value::String(id));
                    changed = true;
                    notes.push(format!("{provider_id}.models[{index}].name"));
                }
                (Some(_), Some(_)) => {}
            }

            next_models.push(Value::Object(model_obj));
        }
        *models_array = next_models;
    }

    if !changed {
        return Ok(None);
    }

    write_openclaw_config_value(config_path, &parsed)?;
    let summary = if notes.is_empty() {
        "providers schema normalized".to_string()
    } else if notes.len() > 8 {
        format!(
            "{} 等 {} 项",
            notes.iter().take(8).cloned().collect::<Vec<_>>().join(", "),
            notes.len()
        )
    } else {
        notes.join(", ")
    };
    Ok(Some(format!("{} -> {}", config_path.display(), summary)))
}

pub(crate) fn sanitize_openclaw_models_provider_schema() -> Result<Option<String>, String> {
    let mut changed_paths = Vec::new();
    let mut failures = Vec::new();
    for path in collect_openclaw_candidate_config_paths() {
        match sanitize_openclaw_models_provider_schema_at_path(&path) {
            Ok(Some(detail)) => changed_paths.push(detail),
            Ok(None) => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    if changed_paths.is_empty() && failures.is_empty() {
        return Ok(None);
    }
    if !changed_paths.is_empty() && failures.is_empty() {
        return Ok(Some(format!(
            "已自动清理 OpenClaw 模型配置中的不兼容字段：{}",
            changed_paths.join("；")
        )));
    }
    if changed_paths.is_empty() && !failures.is_empty() {
        return Err(format!(
            "清理 OpenClaw 模型配置失败：{}",
            failures.join("；")
        ));
    }

    Ok(Some(format!(
        "已部分清理 OpenClaw 模型配置：{}；剩余失败：{}",
        changed_paths.join("；"),
        failures.join("；")
    )))
}

fn sanitize_openclaw_channel_schema_at_path(config_path: &Path) -> Result<Option<String>, String> {
    let raw = match std::fs::read_to_string(config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let Some(root_obj) = parsed.as_object_mut() else {
        return Ok(None);
    };

    let mut changed = false;
    let mut notes = Vec::new();

    if let Some(channels_obj) = root_obj.get_mut("channels").and_then(Value::as_object_mut) {
        let channel_keys = channels_obj.keys().cloned().collect::<Vec<_>>();
        let mut merge_targets = HashSet::new();
        for channel_key in channel_keys {
            let normalized = crate::normalize_channel_identifier(&channel_key);
            let canonical = crate::normalize_channel_identifier_for_openclaw_config(&channel_key);
            if canonical != normalized {
                merge_targets.insert(canonical.clone());
                notes.push(format!("channels.{channel_key}->channels.{canonical}"));
            }
        }
        if !merge_targets.is_empty() {
            changed = true;
            for target in merge_targets {
                let _ = crate::merge_channel_alias_sections(channels_obj, &target)?;
            }
        }
    }

    if let Some(bindings_arr) = root_obj.get_mut("bindings").and_then(Value::as_array_mut) {
        for (index, binding_value) in bindings_arr.iter_mut().enumerate() {
            let Some(binding_obj) = binding_value.as_object_mut() else {
                continue;
            };
            let Some(match_obj) = binding_obj.get_mut("match").and_then(Value::as_object_mut)
            else {
                continue;
            };
            let Some(channel_raw) = match_obj
                .get("channel")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };

            let canonical = crate::normalize_channel_identifier_for_openclaw_config(channel_raw);
            let normalized = crate::normalize_channel_identifier(channel_raw);
            if canonical != normalized {
                match_obj.insert("channel".to_string(), Value::String(canonical.clone()));
                changed = true;
                notes.push(format!(
                    "bindings[{index}].match.channel: {normalized}->{canonical}"
                ));
            }
        }
    }

    if !changed {
        return Ok(None);
    }

    write_openclaw_config_value(config_path, &parsed)?;
    let summary = if notes.is_empty() {
        "channel schema normalized".to_string()
    } else if notes.len() > 8 {
        format!(
            "{} 等 {} 项",
            notes.iter().take(8).cloned().collect::<Vec<_>>().join(", "),
            notes.len()
        )
    } else {
        notes.join(", ")
    };
    Ok(Some(format!("{} -> {}", config_path.display(), summary)))
}

pub(crate) fn sanitize_openclaw_channel_schema() -> Result<Option<String>, String> {
    let mut changed_paths = Vec::new();
    let mut failures = Vec::new();
    for path in collect_openclaw_candidate_config_paths() {
        match sanitize_openclaw_channel_schema_at_path(&path) {
            Ok(Some(detail)) => changed_paths.push(detail),
            Ok(None) => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    if changed_paths.is_empty() && failures.is_empty() {
        return Ok(None);
    }
    if !changed_paths.is_empty() && failures.is_empty() {
        return Ok(Some(format!(
            "已自动迁移 OpenClaw 频道配置中的旧 channel id：{}",
            changed_paths.join("；")
        )));
    }
    if changed_paths.is_empty() && !failures.is_empty() {
        return Err(format!(
            "迁移 OpenClaw 频道配置失败：{}",
            failures.join("；")
        ));
    }

    Ok(Some(format!(
        "已部分迁移 OpenClaw 频道配置：{}；剩余失败：{}",
        changed_paths.join("；"),
        failures.join("；")
    )))
}

fn normalize_path_for_plugin_compare(path: &Path) -> String {
    let display = path.display().to_string();
    #[cfg(target_os = "windows")]
    {
        display.replace('\\', "/").to_ascii_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        display.replace('\\', "/")
    }
}

fn sanitize_openclaw_plugin_load_paths_at_path(
    config_path: &Path,
) -> Result<Option<String>, String> {
    let raw = match std::fs::read_to_string(config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let Some(root_obj) = parsed.as_object_mut() else {
        return Ok(None);
    };
    let Some(plugins_obj) = root_obj.get_mut("plugins").and_then(Value::as_object_mut) else {
        return Ok(None);
    };
    let Some(load_obj) = plugins_obj.get_mut("load").and_then(Value::as_object_mut) else {
        return Ok(None);
    };
    let Some(paths_arr) = load_obj.get_mut("paths").and_then(Value::as_array_mut) else {
        return Ok(None);
    };

    let project_root = crate::resolve_project_root();
    let project_root_norm = normalize_path_for_plugin_compare(&project_root);
    let project_prefix = format!("{project_root_norm}/");

    let mut changed = false;
    let mut removed_project_paths = 0usize;
    let mut seen = HashSet::new();
    let mut next_paths = Vec::new();

    for value in paths_arr.iter() {
        let Some(path_raw) = value.as_str() else {
            changed = true;
            continue;
        };
        let trimmed = path_raw.trim();
        if trimmed.is_empty() {
            changed = true;
            continue;
        }

        let normalized = normalize_path_for_plugin_compare(Path::new(trimmed));
        if normalized == project_root_norm || normalized.starts_with(&project_prefix) {
            changed = true;
            removed_project_paths = removed_project_paths.saturating_add(1);
            continue;
        }

        if !seen.insert(normalized) {
            changed = true;
            continue;
        }

        next_paths.push(Value::String(trimmed.to_string()));
    }

    if !changed {
        return Ok(None);
    }

    *paths_arr = next_paths;
    write_openclaw_config_value(config_path, &parsed)?;
    Ok(Some(format!(
        "{} -> removed_project_paths={removed_project_paths}",
        config_path.display()
    )))
}

pub(crate) fn sanitize_openclaw_plugin_load_paths() -> Result<Option<String>, String> {
    let mut changed_paths = Vec::new();
    let mut failures = Vec::new();
    for path in collect_openclaw_candidate_config_paths() {
        match sanitize_openclaw_plugin_load_paths_at_path(&path) {
            Ok(Some(detail)) => changed_paths.push(detail),
            Ok(None) => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    if changed_paths.is_empty() && failures.is_empty() {
        return Ok(None);
    }
    if !changed_paths.is_empty() && failures.is_empty() {
        return Ok(Some(format!(
            "已清理 plugins.load.paths 中的项目内路径与重复项：{}",
            changed_paths.join("；")
        )));
    }
    if changed_paths.is_empty() && !failures.is_empty() {
        return Err(format!(
            "清理 plugins.load.paths 失败：{}",
            failures.join("；")
        ));
    }

    Ok(Some(format!(
        "已部分清理 plugins.load.paths：{}；剩余失败：{}",
        changed_paths.join("；"),
        failures.join("；")
    )))
}
