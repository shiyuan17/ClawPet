use serde_json::Value;

pub(crate) fn save_openclaw_channel_config(
    payload: crate::OpenClawChannelConfigPayload,
) -> Result<(), String> {
    let source_path = crate::resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        super::channel_config::normalize_channel_identifier(&payload.channel_type);
    if normalized_channel.is_empty() {
        return Err("channelType 不能为空".to_string());
    }

    crate::ensure_channel_plugin_allowlist(root, &normalized_channel)?;

    if super::channel_config::is_whatsapp_channel_identifier(&normalized_channel) {
        return crate::write_openclaw_config_value(&source_path, &parsed);
    }

    if !matches!(root.get("channels"), Some(Value::Object(_))) {
        root.insert(
            "channels".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let channels_obj = root
        .get_mut("channels")
        .and_then(Value::as_object_mut)
        .ok_or("channels 不是对象")?;
    let channel_config_key =
        crate::merge_channel_alias_sections(channels_obj, &normalized_channel)?;
    let section_obj = channels_obj
        .get_mut(&channel_config_key)
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType> 不是对象")?;
    crate::migrate_legacy_channel_section_to_accounts(section_obj);

    if !matches!(section_obj.get("accounts"), Some(Value::Object(_))) {
        section_obj.insert(
            "accounts".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let accounts_obj = section_obj
        .get_mut("accounts")
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType>.accounts 不是对象")?;

    let resolved_account_id = payload
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();
    let existing_account_obj = accounts_obj
        .get(&resolved_account_id)
        .and_then(Value::as_object);
    let next_account_obj = crate::build_channel_account_config(
        &normalized_channel,
        &payload.config,
        existing_account_obj,
    );
    accounts_obj.insert(resolved_account_id.clone(), Value::Object(next_account_obj));

    if section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        section_obj.insert(
            "defaultAccount".to_string(),
            Value::String(resolved_account_id.clone()),
        );
    }
    section_obj.insert("enabled".to_string(), Value::Bool(true));
    crate::mirror_default_account_to_channel_section(section_obj);

    let (channel_to_agent, account_to_agent) =
        super::channel_config::resolve_channel_binding_maps(root);
    let binding_key = super::channel_config::channel_account_binding_key(
        &channel_config_key,
        &resolved_account_id,
    );
    let has_account_binding = account_to_agent.contains_key(&binding_key);
    let has_channel_binding = channel_to_agent.contains_key(&channel_config_key);
    if !has_account_binding && !has_channel_binding {
        let binding_account =
            if super::channel_config::is_default_channel_account_id(&resolved_account_id) {
                None
            } else {
                Some(resolved_account_id.as_str())
            };
        crate::upsert_channel_binding(root, &channel_config_key, binding_account, Some("main"));
    }

    crate::write_openclaw_config_value(&source_path, &parsed)
}

pub(crate) fn save_openclaw_channel_binding(
    payload: crate::OpenClawChannelBindingPayload,
) -> Result<(), String> {
    let source_path = crate::resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        super::channel_config::normalize_channel_identifier_for_openclaw_config(
            &payload.channel_type,
        );
    let normalized_account = payload.account_id.trim();
    if normalized_channel.is_empty() || normalized_account.is_empty() {
        return Err("channelType 与 accountId 不能为空".to_string());
    }
    let binding_account =
        if super::channel_config::is_default_channel_account_id(normalized_account) {
            None
        } else {
            Some(normalized_account)
        };

    crate::upsert_channel_binding(
        root,
        &normalized_channel,
        binding_account,
        payload.agent_id.as_deref(),
    );
    crate::write_openclaw_config_value(&source_path, &parsed)
}

pub(crate) fn delete_openclaw_channel_account_config(
    payload: crate::OpenClawChannelAccountPayload,
) -> Result<(), String> {
    let source_path = crate::resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        super::channel_config::normalize_channel_identifier_for_openclaw_config(
            &payload.channel_type,
        );
    let normalized_account = payload.account_id.trim().to_string();
    if normalized_channel.is_empty() || normalized_account.is_empty() {
        return Err("channelType 与 accountId 不能为空".to_string());
    }

    let mut removed = false;
    if let Some(channels_obj) = root.get_mut("channels").and_then(Value::as_object_mut) {
        let _ = crate::merge_channel_alias_sections(channels_obj, &normalized_channel)?;
        let mut remove_channel = false;
        if let Some(section_obj) = channels_obj
            .get_mut(&normalized_channel)
            .and_then(Value::as_object_mut)
        {
            crate::migrate_legacy_channel_section_to_accounts(section_obj);
            let default_account_id_before = section_obj
                .get("defaultAccount")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("")
                .to_string();
            if let Some(accounts_obj) = section_obj
                .get_mut("accounts")
                .and_then(Value::as_object_mut)
            {
                if accounts_obj.remove(&normalized_account).is_some() {
                    removed = true;
                    if accounts_obj.is_empty() {
                        remove_channel = true;
                    } else {
                        let default_matches_removed = !default_account_id_before.is_empty()
                            && default_account_id_before.eq_ignore_ascii_case(&normalized_account);
                        if default_matches_removed {
                            let mut next_default_ids =
                                accounts_obj.keys().cloned().collect::<Vec<_>>();
                            next_default_ids.sort();
                            if let Some(next_default) = next_default_ids.first() {
                                section_obj.insert(
                                    "defaultAccount".to_string(),
                                    Value::String(next_default.to_string()),
                                );
                            }
                        }
                        crate::mirror_default_account_to_channel_section(section_obj);
                    }
                }
            }
        }
        if remove_channel {
            channels_obj.remove(&normalized_channel);
        }
    }

    if !removed {
        return Ok(());
    }

    if super::channel_config::is_default_channel_account_id(&normalized_account) {
        crate::upsert_channel_binding(root, &normalized_channel, None, None);
    }
    crate::upsert_channel_binding(root, &normalized_channel, Some(&normalized_account), None);
    crate::write_openclaw_config_value(&source_path, &parsed)
}

pub(crate) fn rename_openclaw_channel_account(
    payload: crate::OpenClawChannelAccountRenamePayload,
) -> Result<(), String> {
    let source_path = crate::resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&source_path)
        .map_err(|error| format!("读取 openclaw.json 失败: {error}"))?;
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        super::channel_config::normalize_channel_identifier_for_openclaw_config(
            &payload.channel_type,
        );
    let normalized_account = payload.account_id.trim().to_string();
    let normalized_name = payload.name.trim().to_string();
    if normalized_channel.is_empty() || normalized_account.is_empty() {
        return Err("channelType 与 accountId 不能为空".to_string());
    }
    if normalized_name.is_empty() {
        return Err("名称不能为空".to_string());
    }

    let channels_obj = root
        .get_mut("channels")
        .and_then(Value::as_object_mut)
        .ok_or("channels 不存在或格式错误")?;
    let _ = crate::merge_channel_alias_sections(channels_obj, &normalized_channel)?;
    let section_obj = channels_obj
        .get_mut(&normalized_channel)
        .and_then(Value::as_object_mut)
        .ok_or("未找到对应频道配置")?;
    crate::migrate_legacy_channel_section_to_accounts(section_obj);
    let accounts_obj = section_obj
        .get_mut("accounts")
        .and_then(Value::as_object_mut)
        .ok_or("频道账号配置缺失")?;
    let account_obj = accounts_obj
        .get_mut(&normalized_account)
        .and_then(Value::as_object_mut)
        .ok_or("未找到对应账号配置")?;

    account_obj.insert("name".to_string(), Value::String(normalized_name));
    crate::mirror_default_account_to_channel_section(section_obj);
    crate::write_openclaw_config_value(&source_path, &parsed)
}

pub(crate) fn delete_openclaw_channel_config(channel_type: String) -> Result<(), String> {
    let source_path = crate::resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        super::channel_config::normalize_channel_identifier_for_openclaw_config(&channel_type);
    if normalized_channel.is_empty() {
        return Err("channelType 不能为空".to_string());
    }

    if let Some(channels_obj) = root.get_mut("channels").and_then(Value::as_object_mut) {
        let aliases = crate::resolve_channel_config_verification_aliases(&normalized_channel);
        let keys_to_remove = channels_obj
            .keys()
            .cloned()
            .filter(|key| {
                aliases.contains(&super::channel_config::normalize_channel_identifier(key))
            })
            .collect::<Vec<_>>();
        for key in keys_to_remove {
            channels_obj.remove(&key);
        }
    }
    crate::clear_all_channel_bindings(root, &normalized_channel);
    crate::write_openclaw_config_value(&source_path, &parsed)
}
