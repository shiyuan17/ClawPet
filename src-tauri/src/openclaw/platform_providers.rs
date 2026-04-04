use crate::openclaw::config::paths::resolve_openclaw_config_path;
use crate::openclaw::config::write::write_openclaw_config_value;
use crate::openclaw::models::*;
use crate::openclaw::utils::{
    humanize_provider_name, infer_platform_api_path, infer_platform_protocol,
    normalize_local_proxy_base_url_for_persist, normalize_provider_id, value_as_object,
};
use serde_json::Value;

fn load_openclaw_platforms_snapshot_blocking() -> Result<OpenClawPlatformSnapshotResponse, String> {
    let source_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&source_path)
        .map_err(|error| format!("无法读取 openclaw.json: {error}"))?;
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = value_as_object(&parsed).ok_or("openclaw.json 根节点不是对象")?;

    let providers = root
        .get("models")
        .and_then(value_as_object)
        .and_then(|models| models.get("providers"))
        .and_then(value_as_object)
        .ok_or("openclaw.json 中未找到 models.providers 配置")?;

    let mut platforms = Vec::new();
    for (provider_id, item) in providers {
        let Some(provider) = value_as_object(item) else {
            continue;
        };

        let base_url = provider
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("")
            .to_string();
        if base_url.is_empty() {
            continue;
        }

        let normalized_provider_id = normalize_provider_id(provider_id);
        let api_kind = provider.get("api").and_then(Value::as_str);
        let protocol = infer_platform_protocol(api_kind);
        let api_path = infer_platform_api_path(&protocol, api_kind, &base_url);
        let model = provider
            .get("models")
            .and_then(Value::as_array)
            .and_then(|list| {
                list.iter().find_map(|model| {
                    value_as_object(model)
                        .and_then(|obj| obj.get("id").and_then(Value::as_str))
                        .or_else(|| {
                            value_as_object(model)
                                .and_then(|obj| obj.get("name").and_then(Value::as_str))
                        })
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(|value| value.to_string())
                })
            })
            .unwrap_or_default();

        let name = provider
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .or_else(|| {
                provider
                    .get("alias")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| value.to_string())
            })
            .unwrap_or_else(|| humanize_provider_name(provider_id));
        let api_key = provider
            .get("apiKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string();

        platforms.push(OpenClawPlatformSnapshotItem {
            id: format!("openclaw-provider-{normalized_provider_id}"),
            provider_id: provider_id.to_string(),
            name,
            protocol,
            base_url,
            path_prefix: format!("/{normalized_provider_id}"),
            api_path,
            api_key,
            model,
        });
    }

    platforms.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(OpenClawPlatformSnapshotResponse {
        source_path: source_path.display().to_string(),
        detail: format!("已从 openclaw.json 读取 {} 个平台。", platforms.len()),
        platforms,
    })
}

#[tauri::command]
pub(crate) async fn load_openclaw_platforms_snapshot(
) -> Result<OpenClawPlatformSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_openclaw_platforms_snapshot_blocking)
        .await
        .map_err(|error| format!("读取平台快照任务失败：{error}"))?
}

#[tauri::command]
pub(crate) fn save_openclaw_provider_base_url(
    provider_id: String,
    base_url: String,
) -> Result<(), String> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err("providerId 不能为空".to_string());
    }
    let base_url = base_url.trim();
    if base_url.is_empty() {
        return Err("baseUrl 不能为空".to_string());
    }
    let normalized_base_url = normalize_local_proxy_base_url_for_persist(base_url);

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
    let providers = root
        .entry("models")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .and_then(|models| {
            Some(
                models
                    .entry("providers")
                    .or_insert_with(|| serde_json::json!({}))
                    .as_object_mut()?,
            )
        })
        .ok_or("openclaw.json 的 models.providers 不是对象")?;

    let target_key = providers
        .keys()
        .find(|key| key.eq_ignore_ascii_case(provider_id))
        .cloned()
        .unwrap_or_else(|| provider_id.to_string());

    let provider = providers
        .entry(target_key)
        .or_insert_with(|| serde_json::json!({}));
    let provider_obj = provider.as_object_mut().ok_or("provider 配置不是对象")?;
    provider_obj.insert("baseUrl".to_string(), Value::String(normalized_base_url));

    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
pub(crate) fn delete_openclaw_provider_config(provider_id: String) -> Result<(), String> {
    let normalized_provider_id = normalize_provider_id(provider_id.trim());
    if normalized_provider_id.is_empty() {
        return Err("providerId 不能为空".to_string());
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
    let providers = root
        .entry("models")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .and_then(|models| {
            Some(
                models
                    .entry("providers")
                    .or_insert_with(|| serde_json::json!({}))
                    .as_object_mut()?,
            )
        })
        .ok_or("openclaw.json 的 models.providers 不是对象")?;

    let target_key = providers
        .keys()
        .find(|key| key.eq_ignore_ascii_case(&normalized_provider_id))
        .cloned()
        .ok_or_else(|| format!("未找到 providerId 为 {} 的模型配置。", provider_id.trim()))?;
    providers.remove(&target_key);

    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
pub(crate) fn save_openclaw_provider_config(
    config: OpenClawProviderConfigPayload,
) -> Result<(), String> {
    let normalized_provider_id = normalize_provider_id(config.provider_id.trim());
    if normalized_provider_id.is_empty() {
        return Err("providerId 不能为空".to_string());
    }

    let base_url = config.base_url.trim();
    if base_url.is_empty() {
        return Err("baseUrl 不能为空".to_string());
    }
    let normalized_base_url = normalize_local_proxy_base_url_for_persist(base_url);
    let _requested_provider_name = config.provider_name.as_deref().unwrap_or("").trim();
    let requested_model = config.model.as_deref().unwrap_or("").trim().to_string();

    let requested_api_kind = config
        .api_kind
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let requested_protocol = config
        .protocol
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let api_key = config.api_key.trim().to_string();

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

    let target_key = {
        let providers = root
            .entry("models")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .and_then(|models| {
                Some(
                    models
                        .entry("providers")
                        .or_insert_with(|| serde_json::json!({}))
                        .as_object_mut()?,
                )
            })
            .ok_or("openclaw.json 的 models.providers 不是对象")?;

        let target_key = providers
            .keys()
            .find(|key| key.eq_ignore_ascii_case(&normalized_provider_id))
            .cloned()
            .unwrap_or_else(|| normalized_provider_id.to_string());
        let provider = providers
            .entry(target_key.clone())
            .or_insert_with(|| serde_json::json!({}));
        let provider_obj = provider.as_object_mut().ok_or("provider 配置不是对象")?;

        let existing_api_kind = provider_obj
            .get("api")
            .and_then(Value::as_str)
            .and_then(|value| {
                let normalized = value.trim().to_ascii_lowercase();
                if normalized == "openai-responses" {
                    Some("openai-responses")
                } else if normalized == "anthropic-messages" {
                    Some("anthropic-messages")
                } else if normalized == "openai-completions" {
                    Some("openai-completions")
                } else {
                    None
                }
            });
        let api_kind = if requested_api_kind == "openai-responses" {
            "openai-responses"
        } else if requested_api_kind == "anthropic-messages" {
            "anthropic-messages"
        } else if requested_api_kind == "openai-completions" {
            "openai-completions"
        } else if requested_protocol == "anthropic" {
            "anthropic-messages"
        } else if requested_protocol == "openai" {
            "openai-completions"
        } else if let Some(existing_kind) = existing_api_kind {
            existing_kind
        } else {
            "openai-completions"
        };

        provider_obj.remove("name");
        provider_obj.insert("api".to_string(), Value::String(api_kind.to_string()));
        provider_obj.insert("baseUrl".to_string(), Value::String(normalized_base_url));
        provider_obj.insert("apiKey".to_string(), Value::String(api_key));
        if !matches!(provider_obj.get("models"), Some(Value::Array(_))) {
            provider_obj.insert("models".to_string(), Value::Array(Vec::new()));
        }

        if !requested_model.is_empty() {
            let existing_models = provider_obj
                .get("models")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut next_models = Vec::new();
            let mut seen_model_ids = std::collections::HashSet::new();
            for item in existing_models {
                let Some(model_id) = item
                    .as_object()
                    .and_then(|obj| obj.get("id"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                else {
                    continue;
                };
                if seen_model_ids.insert(model_id.to_string()) {
                    next_models.push(item);
                }
            }
            if seen_model_ids.insert(requested_model.clone()) {
                next_models.push(serde_json::json!({
                    "id": requested_model.clone(),
                    "name": requested_model.clone()
                }));
            }
            provider_obj.insert("models".to_string(), Value::Array(next_models));
        }

        target_key
    };

    if !requested_model.is_empty() {
        if let Some(models_obj) = root.get_mut("models").and_then(Value::as_object_mut) {
            models_obj.remove("default");
        }

        if !matches!(root.get("agents"), Some(Value::Object(_))) {
            root.insert("agents".to_string(), serde_json::json!({}));
        }
        let agents_obj = root
            .get_mut("agents")
            .and_then(Value::as_object_mut)
            .ok_or("openclaw.json 的 agents 不是对象")?;
        if !matches!(agents_obj.get("defaults"), Some(Value::Object(_))) {
            agents_obj.insert("defaults".to_string(), serde_json::json!({}));
        }
        let defaults_obj = agents_obj
            .get_mut("defaults")
            .and_then(Value::as_object_mut)
            .ok_or("openclaw.json 的 agents.defaults 不是对象")?;
        let existing_fallbacks = defaults_obj
            .get("model")
            .and_then(Value::as_object)
            .and_then(|obj| obj.get("fallbacks"))
            .cloned();
        let fallbacks = match existing_fallbacks {
            Some(Value::Array(items)) => Value::Array(items),
            _ => Value::Array(Vec::new()),
        };
        let primary_model_ref = format!("{}/{}", target_key, requested_model);
        defaults_obj.insert(
            "model".to_string(),
            serde_json::json!({
                "primary": primary_model_ref,
                "fallbacks": fallbacks
            }),
        );
    }

    write_openclaw_config_value(&source_path, &parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openclaw::test_support::{read_temp_openclaw_config_json, with_temp_openclaw_home};
    use serde_json::{json, Value};

    #[test]
    fn load_platforms_snapshot_sorts_items_and_skips_empty_base_urls() {
        with_temp_openclaw_home(
            "dragonclaw-platform-providers",
            Some(json!({
                "models": {
                    "providers": {
                        "z-provider": {
                            "name": "Zulu Provider",
                            "api": "openai-completions",
                            "baseUrl": "https://zulu.example.com",
                            "apiKey": "zulu-key",
                            "models": [
                                { "id": "zulu-model", "name": "Zulu Model" }
                            ]
                        },
                        "alpha-provider": {
                            "alias": "Alpha Provider",
                            "api": "anthropic-messages",
                            "baseUrl": "https://alpha.example.com/v1",
                            "apiKey": "alpha-key",
                            "models": [
                                { "name": "claude-sonnet" }
                            ]
                        },
                        "ignored-provider": {
                            "name": "Ignored Provider",
                            "baseUrl": "   "
                        }
                    }
                }
            })),
            |_| {
                let response =
                    load_openclaw_platforms_snapshot_blocking().expect("load platform snapshot");

                assert_eq!(response.platforms.len(), 2);
                assert_eq!(response.platforms[0].provider_id, "alpha-provider");
                assert_eq!(response.platforms[0].name, "Alpha Provider");
                assert_eq!(response.platforms[0].protocol, "anthropic");
                assert_eq!(response.platforms[0].api_path, "/messages");
                assert_eq!(response.platforms[0].model, "claude-sonnet");
                assert_eq!(response.platforms[1].provider_id, "z-provider");
                assert_eq!(response.platforms[1].name, "Zulu Provider");
                assert_eq!(response.platforms[1].protocol, "openai");
                assert_eq!(response.platforms[1].api_path, "/v1/chat/completions");
                assert_eq!(response.platforms[1].api_key, "zulu-key");
            },
        );
    }

    #[test]
    fn save_provider_config_updates_primary_model_and_deduplicates_models() {
        with_temp_openclaw_home(
            "dragonclaw-platform-providers",
            Some(json!({
                "models": {
                    "default": "legacy/provider",
                    "providers": {
                        "custom-provider": {
                            "api": "openai-completions",
                            "baseUrl": "https://legacy.example.com",
                            "apiKey": "legacy-key",
                            "models": [
                                { "id": "existing-model", "name": "Existing Model" },
                                { "id": "existing-model", "name": "Existing Model Duplicate" },
                                { "id": "other-model", "name": "Other Model" }
                            ]
                        }
                    }
                }
            })),
            |_| {
                save_openclaw_provider_config(OpenClawProviderConfigPayload {
                    provider_id: "custom-provider".to_string(),
                    provider_name: Some("Custom Provider".to_string()),
                    protocol: Some("anthropic".to_string()),
                    api_kind: None,
                    base_url: "https://api.example.com/v1".to_string(),
                    model: Some("new-model".to_string()),
                    api_key: "updated-key".to_string(),
                })
                .expect("save provider config");

                let saved = read_temp_openclaw_config_json();
                let provider = saved
                    .get("models")
                    .and_then(Value::as_object)
                    .and_then(|models| models.get("providers"))
                    .and_then(Value::as_object)
                    .and_then(|providers| providers.get("custom-provider"))
                    .and_then(Value::as_object)
                    .expect("provider object");

                assert_eq!(
                    provider.get("api").and_then(Value::as_str),
                    Some("anthropic-messages")
                );
                assert_eq!(
                    provider.get("baseUrl").and_then(Value::as_str),
                    Some("https://api.example.com/v1")
                );
                assert_eq!(
                    provider.get("apiKey").and_then(Value::as_str),
                    Some("updated-key")
                );

                let model_ids = provider
                    .get("models")
                    .and_then(Value::as_array)
                    .expect("provider model list")
                    .iter()
                    .filter_map(|item| item.get("id").and_then(Value::as_str))
                    .collect::<Vec<_>>();
                assert_eq!(
                    model_ids,
                    vec!["existing-model", "other-model", "new-model"]
                );

                let defaults = saved
                    .get("agents")
                    .and_then(Value::as_object)
                    .and_then(|agents| agents.get("defaults"))
                    .and_then(Value::as_object)
                    .and_then(|defaults| defaults.get("model"))
                    .and_then(Value::as_object)
                    .expect("agents.defaults.model object");
                assert_eq!(
                    defaults.get("primary").and_then(Value::as_str),
                    Some("custom-provider/new-model")
                );
                assert_eq!(
                    saved
                        .get("models")
                        .and_then(Value::as_object)
                        .and_then(|models| models.get("default")),
                    None
                );
            },
        );
    }
}
