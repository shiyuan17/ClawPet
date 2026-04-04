use crate::openclaw::channel_config::{
    channel_account_binding_key, channel_payload_has_content, extract_channel_form_values,
    migrate_legacy_channel_section_to_accounts, normalize_channel_identifier,
    resolve_channel_binding_maps, resolve_channel_section_from_channels_obj,
    sync_channel_accounts_from_plugin_store,
};
use crate::openclaw::channel_write;
use crate::openclaw::config::paths::resolve_openclaw_config_path;
use crate::openclaw::models::*;
use crate::openclaw::utils::value_as_object;
use serde_json::Value;

fn load_openclaw_channel_accounts_snapshot_blocking(
) -> Result<OpenClawChannelAccountsSnapshotResponse, String> {
    let _ = sync_channel_accounts_from_plugin_store("weixin");
    let _ = sync_channel_accounts_from_plugin_store("wecom");
    let _ = sync_channel_accounts_from_plugin_store("whatsapp");

    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawChannelAccountsSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 未找到，当前没有可读取的频道配置。".to_string(),
                channels: Vec::new(),
            })
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawChannelAccountsSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 解析失败，当前无法读取频道配置。".to_string(),
                channels: Vec::new(),
            })
        }
    };
    let Some(root) = value_as_object(&parsed) else {
        return Ok(OpenClawChannelAccountsSnapshotResponse {
            source_path: source_path.display().to_string(),
            detail: "openclaw.json 根节点不是对象。".to_string(),
            channels: Vec::new(),
        });
    };

    let (channel_to_agent, account_to_agent) = resolve_channel_binding_maps(root);
    let mut channel_groups = Vec::new();

    if let Some(channels_obj) = root.get("channels").and_then(Value::as_object) {
        for (channel_type, section_value) in channels_obj {
            let Some(section_obj) = section_value.as_object() else {
                continue;
            };
            if section_obj.get("enabled").and_then(Value::as_bool) == Some(false) {
                continue;
            }

            let mut section_clone = section_obj.clone();
            migrate_legacy_channel_section_to_accounts(&mut section_clone);
            let default_account_id = section_clone
                .get("defaultAccount")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("default")
                .to_string();
            let mut accounts = Vec::new();
            if let Some(accounts_obj) = section_clone.get("accounts").and_then(Value::as_object) {
                for (account_id, account_value) in accounts_obj {
                    let Some(account_obj) = account_value.as_object() else {
                        continue;
                    };
                    let enabled = account_obj
                        .get("enabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(true);
                    let configured = enabled && channel_payload_has_content(account_obj);
                    let status = if configured {
                        "connected"
                    } else {
                        "disconnected"
                    }
                    .to_string();
                    let normalized_channel = normalize_channel_identifier(channel_type);
                    let binding_key = channel_account_binding_key(channel_type, account_id);
                    let agent_id = account_to_agent
                        .get(&binding_key)
                        .cloned()
                        .or_else(|| channel_to_agent.get(&normalized_channel).cloned());
                    let display_name = account_obj
                        .get("name")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .unwrap_or_else(|| {
                            if account_id.eq_ignore_ascii_case("default") {
                                "主账号".to_string()
                            } else {
                                account_id.to_string()
                            }
                        });

                    accounts.push(OpenClawChannelAccountSnapshotItem {
                        account_id: account_id.to_string(),
                        name: display_name,
                        configured,
                        status,
                        is_default: account_id.eq_ignore_ascii_case(&default_account_id),
                        agent_id,
                    });
                }
            }

            if accounts.is_empty() {
                continue;
            }

            accounts.sort_by(|left, right| {
                if left.is_default {
                    return std::cmp::Ordering::Less;
                }
                if right.is_default {
                    return std::cmp::Ordering::Greater;
                }
                left.account_id.cmp(&right.account_id)
            });

            let group_status = if accounts.iter().any(|item| item.status == "connected") {
                "connected"
            } else {
                "disconnected"
            };

            channel_groups.push(OpenClawChannelGroupSnapshotItem {
                channel_type: channel_type.to_string(),
                default_account_id: default_account_id.to_string(),
                status: group_status.to_string(),
                accounts,
            });
        }
    }

    channel_groups.sort_by(|left, right| left.channel_type.cmp(&right.channel_type));
    Ok(OpenClawChannelAccountsSnapshotResponse {
        source_path: source_path.display().to_string(),
        detail: format!("已读取 {} 个已配置频道。", channel_groups.len()),
        channels: channel_groups,
    })
}

#[tauri::command]
pub(crate) async fn load_openclaw_channel_accounts_snapshot(
) -> Result<OpenClawChannelAccountsSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_openclaw_channel_accounts_snapshot_blocking)
        .await
        .map_err(|error| format!("读取频道快照任务失败：{error}"))?
}

#[tauri::command]
pub(crate) fn load_openclaw_channel_form_values(
    channel_type: String,
    account_id: Option<String>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => return Ok(std::collections::HashMap::new()),
    };
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = value_as_object(&parsed).ok_or("openclaw.json 根节点不是对象")?;
    let normalized_channel = normalize_channel_identifier(&channel_type);
    if normalized_channel.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let Some(section_obj) = root
        .get("channels")
        .and_then(Value::as_object)
        .and_then(|channels| {
            resolve_channel_section_from_channels_obj(channels, &normalized_channel)
        })
    else {
        return Ok(std::collections::HashMap::new());
    };

    let mut section_clone = section_obj.clone();
    migrate_legacy_channel_section_to_accounts(&mut section_clone);
    let resolved_account_id = account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default");

    if let Some(account_obj) = section_clone
        .get("accounts")
        .and_then(Value::as_object)
        .and_then(|accounts| accounts.get(resolved_account_id))
        .and_then(Value::as_object)
    {
        return Ok(extract_channel_form_values(
            &normalized_channel,
            account_obj,
        ));
    }

    return Ok(extract_channel_form_values(
        &normalized_channel,
        &section_clone,
    ));
}

#[tauri::command]
pub(crate) fn save_openclaw_channel_config(
    payload: OpenClawChannelConfigPayload,
) -> Result<(), String> {
    channel_write::save_openclaw_channel_config(payload)
}

#[tauri::command]
pub(crate) fn save_openclaw_channel_binding(
    payload: OpenClawChannelBindingPayload,
) -> Result<(), String> {
    channel_write::save_openclaw_channel_binding(payload)
}

#[tauri::command]
pub(crate) fn delete_openclaw_channel_account_config(
    payload: OpenClawChannelAccountPayload,
) -> Result<(), String> {
    channel_write::delete_openclaw_channel_account_config(payload)
}

#[tauri::command]
pub(crate) fn rename_openclaw_channel_account(
    payload: OpenClawChannelAccountRenamePayload,
) -> Result<(), String> {
    channel_write::rename_openclaw_channel_account(payload)
}

#[tauri::command]
pub(crate) fn delete_openclaw_channel_config(channel_type: String) -> Result<(), String> {
    channel_write::delete_openclaw_channel_config(channel_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openclaw::test_support::with_temp_openclaw_home;
    use serde_json::json;
    #[test]
    fn load_channel_form_values_reads_account_specific_values() {
        with_temp_openclaw_home(
            "dragonclaw-platform-channels",
            Some(json!({
                "channels": {
                    "wecom": {
                        "enabled": true,
                        "defaultAccount": "team-b",
                        "accounts": {
                            "team-a": {
                                "enabled": true,
                                "corpId": "corp-a",
                                "agentSecret": "secret-a"
                            },
                            "team-b": {
                                "enabled": true,
                                "corpId": "corp-b",
                                "agentSecret": "secret-b"
                            }
                        }
                    }
                }
            })),
            |_| {
                let values = load_openclaw_channel_form_values(
                    "wecom".to_string(),
                    Some("team-a".to_string()),
                )
                .expect("load account form values");

                assert_eq!(values.get("corpId"), Some(&"corp-a".to_string()));
                assert_eq!(values.get("agentSecret"), Some(&"secret-a".to_string()));
                assert!(!values.contains_key("enabled"));
            },
        );
    }

    #[test]
    fn load_channel_form_values_migrates_legacy_section_payloads() {
        with_temp_openclaw_home(
            "dragonclaw-platform-channels",
            Some(json!({
                "channels": {
                    "openclaw-weixin": {
                        "enabled": true,
                        "appId": "wx-app-id",
                        "appSecret": "wx-app-secret"
                    }
                }
            })),
            |_| {
                let values = load_openclaw_channel_form_values(
                    "wechat".to_string(),
                    Some("default".to_string()),
                )
                .expect("load migrated legacy values");

                assert_eq!(values.get("appId"), Some(&"wx-app-id".to_string()));
                assert_eq!(values.get("appSecret"), Some(&"wx-app-secret".to_string()));
            },
        );
    }
}
