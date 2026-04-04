use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn value_as_object(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    value.as_object()
}

pub(crate) fn resolve_channels_from_bindings(
    root: &serde_json::Map<String, Value>,
) -> std::collections::HashMap<String, String> {
    let mut by_agent: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    if let Some(arr) = root.get("bindings").and_then(Value::as_array) {
        for item in arr {
            if let Some(obj) = value_as_object(item) {
                let agent_id = obj
                    .get("agentId")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                let channel = obj
                    .get("match")
                    .and_then(value_as_object)
                    .and_then(|m| m.get("channel"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                if let (Some(a), Some(c)) = (agent_id, channel) {
                    by_agent
                        .entry(a.to_string())
                        .or_default()
                        .push(c.to_string());
                }
            }
        }
    }
    by_agent
        .into_iter()
        .map(|(k, v)| (k, v.join(", ")))
        .collect()
}

pub(crate) fn normalize_channel_identifier(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

pub(crate) fn is_weixin_channel_identifier(normalized_channel: &str) -> bool {
    matches!(
        normalized_channel,
        "weixin"
            | "wechat"
            | "wx"
            | "wechat_official_account"
            | "wechat-official-account"
            | "openclaw-weixin"
            | "openclaw_weixin"
    )
}

pub(crate) fn is_wecom_channel_identifier(normalized_channel: &str) -> bool {
    matches!(
        normalized_channel,
        "wecom"
            | "workwechat"
            | "work-wechat"
            | "work_wechat"
            | "wechatwork"
            | "qywx"
            | "openclaw-wecom"
            | "openclaw_wecom"
    )
}

pub(crate) fn is_whatsapp_channel_identifier(normalized_channel: &str) -> bool {
    matches!(
        normalized_channel,
        "whatsapp" | "wa" | "wacli" | "openclaw-whatsapp" | "openclaw_whatsapp"
    )
}

pub(crate) fn normalize_channel_identifier_for_openclaw_config(raw: &str) -> String {
    let normalized = normalize_channel_identifier(raw);
    if is_weixin_channel_identifier(&normalized) {
        "openclaw-weixin".to_string()
    } else if is_wecom_channel_identifier(&normalized) {
        "wecom".to_string()
    } else if is_whatsapp_channel_identifier(&normalized) {
        "whatsapp".to_string()
    } else {
        normalized
    }
}

pub(crate) fn normalize_account_identifier(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

pub(crate) fn is_default_channel_account_id(raw: &str) -> bool {
    normalize_account_identifier(raw) == "default"
}

pub(crate) fn channel_account_binding_key(channel_type: &str, account_id: &str) -> String {
    format!(
        "{}:{}",
        normalize_channel_identifier(channel_type),
        normalize_account_identifier(account_id)
    )
}

pub(crate) fn resolve_channel_binding_maps(
    root: &serde_json::Map<String, Value>,
) -> (
    std::collections::HashMap<String, String>,
    std::collections::HashMap<String, String>,
) {
    let mut channel_to_agent = std::collections::HashMap::new();
    let mut account_to_agent = std::collections::HashMap::new();
    let Some(bindings) = root.get("bindings").and_then(Value::as_array) else {
        return (channel_to_agent, account_to_agent);
    };

    for item in bindings {
        let Some(binding_obj) = item.as_object() else {
            continue;
        };
        let Some(agent_id) = binding_obj
            .get("agentId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(channel_type) = binding_obj
            .get("match")
            .and_then(Value::as_object)
            .and_then(|match_obj| match_obj.get("channel"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        let normalized_channel = normalize_channel_identifier(channel_type);
        let account_id = binding_obj
            .get("match")
            .and_then(Value::as_object)
            .and_then(|match_obj| match_obj.get("accountId"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .filter(|value| !is_default_channel_account_id(value));

        if let Some(account_id) = account_id {
            account_to_agent.insert(
                channel_account_binding_key(&normalized_channel, account_id),
                agent_id.to_string(),
            );
        } else {
            channel_to_agent.insert(normalized_channel, agent_id.to_string());
        }
    }

    (channel_to_agent, account_to_agent)
}

pub(crate) fn is_channel_section_reserved_key(key: &str) -> bool {
    matches!(key, "accounts" | "defaultAccount" | "enabled")
}

pub(crate) fn channel_payload_has_content(payload: &serde_json::Map<String, Value>) -> bool {
    payload.iter().any(|(key, value)| {
        if key == "enabled" || key == "name" {
            return false;
        }
        match value {
            Value::Null => false,
            Value::String(text) => !text.trim().is_empty(),
            Value::Array(items) => !items.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            _ => true,
        }
    })
}

pub(crate) fn resolve_channel_plugin_account_store_paths(channel_type: &str) -> Vec<PathBuf> {
    let openclaw_home = crate::resolve_openclaw_home_path();
    match normalize_channel_identifier_for_openclaw_config(channel_type).as_str() {
        "openclaw-weixin" => vec![
            openclaw_home.join("openclaw-weixin").join("accounts.json"),
            openclaw_home.join("weixin").join("accounts.json"),
            openclaw_home.join("wechat").join("accounts.json"),
        ],
        "wecom" => vec![
            openclaw_home.join("wecom").join("accounts.json"),
            openclaw_home.join("openclaw-wecom").join("accounts.json"),
            openclaw_home.join("wecom-openclaw").join("accounts.json"),
            openclaw_home.join("work-wechat").join("accounts.json"),
        ],
        "whatsapp" => vec![
            openclaw_home.join("whatsapp").join("accounts.json"),
            openclaw_home
                .join("openclaw-whatsapp")
                .join("accounts.json"),
            openclaw_home.join("wacli").join("accounts.json"),
        ],
        _ => Vec::new(),
    }
}

pub(crate) fn parse_channel_plugin_account_ids(value: &Value) -> Vec<String> {
    let mut account_ids = Vec::new();

    if let Some(items) = value.as_array() {
        for item in items {
            let Some(raw) = item.as_str().map(str::trim).filter(|text| !text.is_empty()) else {
                continue;
            };
            account_ids.push(raw.to_string());
        }
        return account_ids;
    }

    let Some(obj) = value.as_object() else {
        return account_ids;
    };
    if let Some(items) = obj.get("accounts").and_then(Value::as_array) {
        for item in items {
            let Some(raw) = item.as_str().map(str::trim).filter(|text| !text.is_empty()) else {
                continue;
            };
            account_ids.push(raw.to_string());
        }
        return account_ids;
    }
    for key in obj.keys() {
        let normalized = key.trim();
        if normalized.is_empty() {
            continue;
        }
        account_ids.push(normalized.to_string());
    }
    account_ids
}

pub(crate) fn load_channel_plugin_account_configs(
    channel_type: &str,
) -> Result<Vec<(String, serde_json::Map<String, Value>)>, String> {
    let mut output = Vec::<(String, serde_json::Map<String, Value>)>::new();
    let mut seen_accounts = HashSet::<String>::new();

    for accounts_index_path in resolve_channel_plugin_account_store_paths(channel_type) {
        if !accounts_index_path.exists() || !accounts_index_path.is_file() {
            continue;
        }
        let raw = std::fs::read_to_string(&accounts_index_path).map_err(|error| {
            format!(
                "读取频道账号索引失败（{}）：{error}",
                accounts_index_path.display()
            )
        })?;
        let parsed: Value = serde_json::from_str(&raw).map_err(|error| {
            format!(
                "解析频道账号索引失败（{}）：{error}",
                accounts_index_path.display()
            )
        })?;

        let account_ids = parse_channel_plugin_account_ids(&parsed);
        if account_ids.is_empty() {
            continue;
        }

        let Some(channel_state_dir) = accounts_index_path.parent() else {
            continue;
        };
        let accounts_dir = channel_state_dir.join("accounts");

        for account_id in account_ids {
            let normalized_key = normalize_account_identifier(&account_id);
            if normalized_key.is_empty() || !seen_accounts.insert(normalized_key) {
                continue;
            }

            let mut config_obj = serde_json::Map::<String, Value>::new();
            config_obj.insert(
                "pluginAccountId".to_string(),
                Value::String(account_id.clone()),
            );
            config_obj.insert("enabled".to_string(), Value::Bool(true));

            let detail_path = accounts_dir.join(format!("{account_id}.json"));
            if detail_path.exists() && detail_path.is_file() {
                if let Ok(detail_raw) = std::fs::read_to_string(&detail_path) {
                    if let Ok(detail_parsed) = serde_json::from_str::<Value>(&detail_raw) {
                        if let Some(detail_obj) = detail_parsed.as_object() {
                            if let Some(user_id) = detail_obj
                                .get("userId")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|text| !text.is_empty())
                            {
                                config_obj.insert(
                                    "userId".to_string(),
                                    Value::String(user_id.to_string()),
                                );
                            }
                            if let Some(base_url) = detail_obj
                                .get("baseUrl")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|text| !text.is_empty())
                            {
                                config_obj.insert(
                                    "baseUrl".to_string(),
                                    Value::String(base_url.to_string()),
                                );
                            }
                            if let Some(name) = detail_obj
                                .get("name")
                                .or_else(|| detail_obj.get("nickname"))
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|text| !text.is_empty())
                            {
                                config_obj
                                    .insert("name".to_string(), Value::String(name.to_string()));
                            }
                        }
                    }
                }
            }

            output.push((account_id, config_obj));
        }
    }

    Ok(output)
}

pub(crate) fn sync_channel_accounts_from_plugin_store(channel_type: &str) -> Result<bool, String> {
    let normalized_channel = normalize_channel_identifier(channel_type);
    if !is_weixin_channel_identifier(&normalized_channel)
        && !is_wecom_channel_identifier(&normalized_channel)
        && !is_whatsapp_channel_identifier(&normalized_channel)
    {
        return Ok(false);
    }
    let channel_config_key = normalize_channel_identifier_for_openclaw_config(channel_type);

    let plugin_accounts = load_channel_plugin_account_configs(&channel_config_key)?;
    if plugin_accounts.is_empty() {
        return Ok(false);
    }

    let source_path = crate::resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(error) => {
            return Err(format!(
                "读取 openclaw.json 失败（{}）：{error}",
                source_path.display()
            ))
        }
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }

    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
    ensure_channel_plugin_allowlist(root, &channel_config_key)?;

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
    let _ = merge_channel_alias_sections(channels_obj, &channel_config_key)?;
    let section_obj = channels_obj
        .get_mut(&channel_config_key)
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType> 不是对象")?;
    migrate_legacy_channel_section_to_accounts(section_obj);
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

    let mut changed = false;
    let mut first_account_id = String::new();
    for (account_id, plugin_config) in plugin_accounts {
        if first_account_id.is_empty() {
            first_account_id = account_id.clone();
        }
        let current = accounts_obj
            .get(&account_id)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let mut merged = current;
        for (key, value) in plugin_config {
            if merged.get(&key) != Some(&value) {
                merged.insert(key, value);
                changed = true;
            }
        }
        if accounts_obj.get(&account_id) != Some(&Value::Object(merged.clone())) {
            accounts_obj.insert(account_id, Value::Object(merged));
            changed = true;
        }
    }

    if section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
        && !first_account_id.is_empty()
    {
        section_obj.insert(
            "defaultAccount".to_string(),
            Value::String(first_account_id),
        );
        changed = true;
    }
    if section_obj.get("enabled").and_then(Value::as_bool) != Some(true) {
        section_obj.insert("enabled".to_string(), Value::Bool(true));
        changed = true;
    }
    mirror_default_account_to_channel_section(section_obj);

    if !changed {
        return Ok(true);
    }
    crate::write_openclaw_config_value(&source_path, &parsed)?;
    Ok(true)
}

pub(crate) fn resolve_channel_config_verification_aliases(channel_type: &str) -> HashSet<String> {
    let normalized = normalize_channel_identifier(channel_type);
    let mut aliases = vec![normalized.clone()];
    if is_weixin_channel_identifier(&normalized) {
        aliases.extend([
            "openclaw-weixin".to_string(),
            "wechat".to_string(),
            "wx".to_string(),
            "openclaw_weixin".to_string(),
            "weixin".to_string(),
            "wechat_official_account".to_string(),
            "wechat-official-account".to_string(),
        ]);
    } else if is_wecom_channel_identifier(&normalized) {
        aliases.extend([
            "wecom".to_string(),
            "workwechat".to_string(),
            "work-wechat".to_string(),
            "work_wechat".to_string(),
            "wechatwork".to_string(),
            "qywx".to_string(),
            "openclaw-wecom".to_string(),
            "openclaw_wecom".to_string(),
        ]);
    } else if is_whatsapp_channel_identifier(&normalized) {
        aliases.extend([
            "whatsapp".to_string(),
            "wa".to_string(),
            "wacli".to_string(),
            "openclaw-whatsapp".to_string(),
            "openclaw_whatsapp".to_string(),
        ]);
    }

    aliases
        .into_iter()
        .map(|value| normalize_channel_identifier(&value))
        .filter(|value| !value.is_empty())
        .collect()
}

pub(crate) fn resolve_channel_section_from_channels_obj<'a>(
    channels_obj: &'a serde_json::Map<String, Value>,
    channel_type: &str,
) -> Option<&'a serde_json::Map<String, Value>> {
    let preferred = normalize_channel_identifier_for_openclaw_config(channel_type);
    let aliases = resolve_channel_config_verification_aliases(channel_type);

    if let Some(section_obj) = channels_obj.get(&preferred).and_then(Value::as_object) {
        return Some(section_obj);
    }

    for (channel_key, section_value) in channels_obj {
        if !aliases.contains(&normalize_channel_identifier(channel_key)) {
            continue;
        }
        if let Some(section_obj) = section_value.as_object() {
            return Some(section_obj);
        }
    }

    None
}

pub(crate) fn merge_channel_section_from_alias(
    target: &mut serde_json::Map<String, Value>,
    source: &serde_json::Map<String, Value>,
) {
    migrate_legacy_channel_section_to_accounts(target);
    let mut source_clone = source.clone();
    migrate_legacy_channel_section_to_accounts(&mut source_clone);

    if !matches!(target.get("accounts"), Some(Value::Object(_))) {
        target.insert(
            "accounts".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    if let (Some(target_accounts), Some(source_accounts)) = (
        target.get_mut("accounts").and_then(Value::as_object_mut),
        source_clone.get("accounts").and_then(Value::as_object),
    ) {
        for (account_id, source_account_value) in source_accounts {
            let Some(source_account_obj) = source_account_value.as_object() else {
                continue;
            };
            if !matches!(target_accounts.get(account_id), Some(Value::Object(_))) {
                target_accounts.insert(
                    account_id.clone(),
                    Value::Object(source_account_obj.clone()),
                );
                continue;
            }
            if let Some(target_account_obj) = target_accounts
                .get_mut(account_id)
                .and_then(Value::as_object_mut)
            {
                for (key, value) in source_account_obj {
                    if !target_account_obj.contains_key(key) {
                        target_account_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    if target
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        if let Some(default_account) = source_clone
            .get("defaultAccount")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            target.insert(
                "defaultAccount".to_string(),
                Value::String(default_account.to_string()),
            );
        }
    }

    for (key, value) in &source_clone {
        if is_channel_section_reserved_key(key) {
            continue;
        }
        if !target.contains_key(key) {
            target.insert(key.clone(), value.clone());
        }
    }

    let source_enabled = source_clone
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if source_enabled && target.get("enabled").and_then(Value::as_bool) != Some(true) {
        target.insert("enabled".to_string(), Value::Bool(true));
    }

    mirror_default_account_to_channel_section(target);
}

pub(crate) fn merge_channel_alias_sections(
    channels_obj: &mut serde_json::Map<String, Value>,
    channel_type: &str,
) -> Result<String, String> {
    let canonical_key = normalize_channel_identifier_for_openclaw_config(channel_type);
    if canonical_key.is_empty() {
        return Err("channelType 不能为空".to_string());
    }

    if !matches!(channels_obj.get(&canonical_key), Some(Value::Object(_))) {
        channels_obj.insert(
            canonical_key.clone(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }

    let aliases = resolve_channel_config_verification_aliases(channel_type);
    let keys_to_merge = channels_obj
        .keys()
        .cloned()
        .filter(|channel_key| {
            let normalized_key = normalize_channel_identifier(channel_key);
            normalized_key != canonical_key && aliases.contains(&normalized_key)
        })
        .collect::<Vec<_>>();

    for alias_key in keys_to_merge {
        let Some(source_value) = channels_obj.remove(&alias_key) else {
            continue;
        };
        let Some(source_obj) = source_value.as_object() else {
            continue;
        };
        let target_obj = channels_obj
            .get_mut(&canonical_key)
            .and_then(Value::as_object_mut)
            .ok_or("channels.<channelType> 不是对象")?;
        merge_channel_section_from_alias(target_obj, source_obj);
    }

    let section_obj = channels_obj
        .get_mut(&canonical_key)
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType> 不是对象")?;
    migrate_legacy_channel_section_to_accounts(section_obj);
    Ok(canonical_key)
}

pub(crate) fn channel_section_has_configured_accounts(
    section_obj: &serde_json::Map<String, Value>,
) -> bool {
    let mut section_clone = section_obj.clone();
    migrate_legacy_channel_section_to_accounts(&mut section_clone);
    if section_clone
        .get("enabled")
        .and_then(Value::as_bool)
        .is_some_and(|enabled| !enabled)
    {
        return false;
    }

    let Some(accounts_obj) = section_clone.get("accounts").and_then(Value::as_object) else {
        return false;
    };
    accounts_obj
        .values()
        .filter_map(Value::as_object)
        .any(|account_obj| {
            account_obj
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true)
                && channel_payload_has_content(account_obj)
        })
}

pub(crate) fn has_configured_channel_account_in_openclaw_config(
    channel_type: &str,
) -> Result<bool, String> {
    let source_path = crate::resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "读取 openclaw.json 失败（{}）：{error}",
                source_path.display()
            ))
        }
    };
    let parsed: Value = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "openclaw.json 解析失败（{}）：{error}",
            source_path.display()
        )
    })?;
    let Some(root) = value_as_object(&parsed) else {
        return Ok(false);
    };
    let Some(channels_obj) = root.get("channels").and_then(Value::as_object) else {
        return Ok(false);
    };

    let aliases = resolve_channel_config_verification_aliases(channel_type);
    if aliases.is_empty() {
        return Ok(false);
    }

    for (channel_key, section_value) in channels_obj {
        if !aliases.contains(&normalize_channel_identifier(channel_key)) {
            continue;
        }
        let Some(section_obj) = section_value.as_object() else {
            continue;
        };
        if channel_section_has_configured_accounts(section_obj) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn wait_for_channel_config_sync(channel_type: &str) -> Result<bool, String> {
    let normalized = normalize_channel_identifier(channel_type);
    let max_attempts = if is_whatsapp_channel_identifier(&normalized) {
        14
    } else {
        6
    };
    for attempt in 0..max_attempts {
        let _ = sync_channel_accounts_from_plugin_store(channel_type);
        if has_configured_channel_account_in_openclaw_config(channel_type)? {
            return Ok(true);
        }
        if attempt + 1 < max_attempts {
            thread::sleep(Duration::from_millis(900));
        }
    }
    Ok(false)
}

pub(crate) fn migrate_legacy_channel_section_to_accounts(
    section_obj: &mut serde_json::Map<String, Value>,
) {
    let has_accounts = section_obj
        .get("accounts")
        .and_then(Value::as_object)
        .map(|accounts| !accounts.is_empty())
        .unwrap_or(false);
    if has_accounts {
        if !section_obj.contains_key("defaultAccount") {
            section_obj.insert(
                "defaultAccount".to_string(),
                Value::String("default".to_string()),
            );
        }
        return;
    }

    let legacy_payload = section_obj
        .iter()
        .filter(|(key, _)| !is_channel_section_reserved_key(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<serde_json::Map<String, Value>>();

    if legacy_payload.is_empty() {
        return;
    }

    let default_account_id = section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();

    let mut account_payload = legacy_payload.clone();
    if let Some(enabled) = section_obj.get("enabled").and_then(Value::as_bool) {
        account_payload.insert("enabled".to_string(), Value::Bool(enabled));
    }

    let mut accounts = serde_json::Map::new();
    accounts.insert(default_account_id.clone(), Value::Object(account_payload));
    section_obj.insert("accounts".to_string(), Value::Object(accounts));
    section_obj.insert(
        "defaultAccount".to_string(),
        Value::String(default_account_id),
    );

    let legacy_keys = legacy_payload.keys().cloned().collect::<Vec<_>>();
    for key in legacy_keys {
        section_obj.remove(&key);
    }
}

pub(crate) fn mirror_default_account_to_channel_section(
    section_obj: &mut serde_json::Map<String, Value>,
) {
    let default_account_id = section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();
    let default_payload = section_obj
        .get("accounts")
        .and_then(Value::as_object)
        .and_then(|accounts| accounts.get(&default_account_id))
        .and_then(Value::as_object)
        .cloned();

    let removable_keys = section_obj
        .keys()
        .filter(|key| !is_channel_section_reserved_key(key))
        .cloned()
        .collect::<Vec<_>>();
    for key in removable_keys {
        section_obj.remove(&key);
    }

    if let Some(default_payload) = default_payload {
        for (key, value) in default_payload {
            if key == "enabled" {
                continue;
            }
            section_obj.insert(key, value);
        }
    }

    section_obj.insert("enabled".to_string(), Value::Bool(true));
}

pub(crate) fn ensure_channel_plugin_allowlist(
    root: &mut serde_json::Map<String, Value>,
    channel_type: &str,
) -> Result<(), String> {
    let plugin_id = match normalize_channel_identifier(channel_type).as_str() {
        "feishu" => Some("openclaw-lark"),
        "dingtalk" => Some("dingtalk"),
        "weixin" | "wechat" | "wx" | "openclaw-weixin" | "openclaw_weixin" => {
            Some("openclaw-weixin")
        }
        "wecom" | "workwechat" | "work-wechat" | "work_wechat" | "wechatwork" | "qywx"
        | "openclaw-wecom" | "openclaw_wecom" => Some("wecom"),
        "qqbot" => Some("qqbot"),
        "whatsapp" | "wa" | "wacli" | "openclaw-whatsapp" | "openclaw_whatsapp" => Some("whatsapp"),
        _ => None,
    };

    let Some(plugin_id) = plugin_id else {
        return Ok(());
    };

    if !matches!(root.get("plugins"), Some(Value::Object(_))) {
        root.insert(
            "plugins".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let Some(plugins_obj) = root.get_mut("plugins").and_then(Value::as_object_mut) else {
        return Err("plugins 配置不是对象".to_string());
    };

    plugins_obj.insert("enabled".to_string(), Value::Bool(true));

    if !matches!(plugins_obj.get("allow"), Some(Value::Array(_))) {
        plugins_obj.insert("allow".to_string(), Value::Array(Vec::new()));
    }
    if let Some(allow_arr) = plugins_obj.get_mut("allow").and_then(Value::as_array_mut) {
        let exists = allow_arr.iter().any(|item| {
            item.as_str()
                .map(str::trim)
                .map(|value| value.eq_ignore_ascii_case(plugin_id))
                .unwrap_or(false)
        });
        if !exists {
            allow_arr.push(Value::String(plugin_id.to_string()));
        }
    }

    if !matches!(plugins_obj.get("entries"), Some(Value::Object(_))) {
        plugins_obj.insert(
            "entries".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    if let Some(entries_obj) = plugins_obj
        .get_mut("entries")
        .and_then(Value::as_object_mut)
    {
        if !matches!(entries_obj.get(plugin_id), Some(Value::Object(_))) {
            entries_obj.insert(
                plugin_id.to_string(),
                Value::Object(serde_json::Map::<String, Value>::new()),
            );
        }
        if let Some(entry_obj) = entries_obj
            .get_mut(plugin_id)
            .and_then(Value::as_object_mut)
        {
            entry_obj.insert("enabled".to_string(), Value::Bool(true));
        }
    }

    Ok(())
}

pub(crate) fn resolve_channel_plugin_install_spec(
    channel_type: &str,
) -> Option<(&'static str, &'static str)> {
    match normalize_channel_identifier(channel_type).as_str() {
        "feishu" => Some(("openclaw-lark", "@larksuite/openclaw-lark@2026.3.12")),
        "dingtalk" => Some(("dingtalk", "@soimy/dingtalk")),
        "whatsapp" | "wa" | "wacli" | "openclaw-whatsapp" | "openclaw_whatsapp" => {
            Some(("whatsapp", "@openclaw/whatsapp"))
        }
        _ => None,
    }
}

pub(crate) fn clear_all_channel_bindings(
    root: &mut serde_json::Map<String, Value>,
    channel_type: &str,
) {
    let normalized_channel = normalize_channel_identifier_for_openclaw_config(channel_type);
    let aliases = resolve_channel_config_verification_aliases(channel_type);
    let Some(bindings) = root.get_mut("bindings").and_then(Value::as_array_mut) else {
        return;
    };

    bindings.retain(|item| {
        let Some(binding_obj) = item.as_object() else {
            return true;
        };
        let Some(existing_channel) = binding_obj
            .get("match")
            .and_then(Value::as_object)
            .and_then(|match_obj| match_obj.get("channel"))
            .and_then(Value::as_str)
        else {
            return true;
        };

        let normalized_existing = normalize_channel_identifier(existing_channel);
        if normalized_existing == normalized_channel {
            return false;
        }
        !aliases.contains(&normalized_existing)
    });

    if bindings.is_empty() {
        root.remove("bindings");
    }
}

pub(crate) fn upsert_channel_binding(
    root: &mut serde_json::Map<String, Value>,
    channel_type: &str,
    account_id: Option<&str>,
    agent_id: Option<&str>,
) {
    let normalized_channel = normalize_channel_identifier_for_openclaw_config(channel_type);
    let channel_aliases = resolve_channel_config_verification_aliases(channel_type);
    if normalized_channel.is_empty() {
        return;
    }
    let normalized_account = account_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_account_identifier)
        .and_then(|value| {
            if value == "default" {
                None
            } else {
                Some(value)
            }
        });
    let normalized_agent = agent_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let existing_bindings = root
        .get("bindings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut next_bindings = Vec::new();

    for item in existing_bindings {
        let Some(binding_obj) = item.as_object() else {
            next_bindings.push(item);
            continue;
        };
        let Some(match_obj) = binding_obj.get("match").and_then(Value::as_object) else {
            next_bindings.push(item);
            continue;
        };
        let Some(existing_channel) = match_obj.get("channel").and_then(Value::as_str) else {
            next_bindings.push(item);
            continue;
        };

        let normalized_existing_channel = normalize_channel_identifier(existing_channel);
        if normalized_existing_channel != normalized_channel
            && !channel_aliases.contains(&normalized_existing_channel)
        {
            next_bindings.push(item);
            continue;
        }

        let existing_agent = binding_obj
            .get("agentId")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        let existing_account = match_obj
            .get("accountId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_account_identifier)
            .and_then(|value| {
                if value == "default" {
                    None
                } else {
                    Some(value)
                }
            });

        if let Some(ref target_agent) = normalized_agent {
            if !target_agent.is_empty() && existing_agent.eq_ignore_ascii_case(target_agent) {
                continue;
            }
        }

        match normalized_account.as_deref() {
            Some(target_account) => {
                if existing_account.as_deref() == Some(target_account) {
                    continue;
                }
            }
            None => {
                if existing_account.is_none() {
                    continue;
                }
            }
        }

        next_bindings.push(item);
    }

    if let Some(agent_id) = normalized_agent {
        let mut match_obj = serde_json::Map::new();
        match_obj.insert(
            "channel".to_string(),
            Value::String(normalized_channel.clone()),
        );
        if let Some(account_id) = normalized_account {
            match_obj.insert("accountId".to_string(), Value::String(account_id));
        }

        let mut binding_obj = serde_json::Map::new();
        binding_obj.insert("type".to_string(), Value::String("route".to_string()));
        binding_obj.insert("agentId".to_string(), Value::String(agent_id));
        binding_obj.insert("match".to_string(), Value::Object(match_obj));
        next_bindings.push(Value::Object(binding_obj));
    }

    if next_bindings.is_empty() {
        root.remove("bindings");
    } else {
        root.insert("bindings".to_string(), Value::Array(next_bindings));
    }
}

pub(crate) fn build_channel_account_config(
    channel_type: &str,
    incoming: &std::collections::HashMap<String, String>,
    existing: Option<&serde_json::Map<String, Value>>,
) -> serde_json::Map<String, Value> {
    let mut config = existing.cloned().unwrap_or_default();
    for (raw_key, raw_value) in incoming {
        let key = raw_key.trim();
        if key.is_empty() {
            continue;
        }
        let value = raw_value.trim();
        if value.is_empty() {
            config.remove(key);
        } else {
            config.insert(key.to_string(), Value::String(value.to_string()));
        }
    }

    let normalized_channel = normalize_channel_identifier(channel_type);

    if normalized_channel == "telegram" {
        let dm_policy = config
            .get("dmPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("pairing")
            .to_string();
        config.insert("dmPolicy".to_string(), Value::String(dm_policy));

        if let Some(allowed_users) = incoming.get("allowedUsers") {
            let users = allowed_users
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| Value::String(value.to_string()))
                .collect::<Vec<_>>();
            if users.is_empty() {
                config.remove("allowFrom");
            } else {
                config.insert("allowFrom".to_string(), Value::Array(users));
            }
        }
    }

    if normalized_channel == "discord" {
        let dm_policy = config
            .get("dmPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("pairing")
            .to_string();
        config.insert("dmPolicy".to_string(), Value::String(dm_policy));

        let group_policy = config
            .get("groupPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("allowlist")
            .to_string();
        config.insert("groupPolicy".to_string(), Value::String(group_policy));

        if !matches!(config.get("retry"), Some(Value::Object(_))) {
            config.insert(
                "retry".to_string(),
                serde_json::json!({
                    "attempts": 3,
                    "minDelayMs": 500,
                    "maxDelayMs": 30000,
                    "jitter": 0.1
                }),
            );
        }

        let guild_id = incoming
            .get("guildId")
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let channel_id = incoming
            .get("channelId")
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        if let Some(guild_id) = guild_id {
            let channel_rule = if let Some(channel_id) = channel_id {
                serde_json::json!({ channel_id: { "allow": true, "requireMention": true } })
            } else {
                serde_json::json!({ "*": { "allow": true, "requireMention": true } })
            };
            config.insert(
                "guilds".to_string(),
                serde_json::json!({
                    guild_id: {
                        "users": ["*"],
                        "requireMention": true,
                        "channels": channel_rule
                    }
                }),
            );
        }
        config.remove("guildId");
        config.remove("channelId");
        config.remove("dm");
    }

    if normalized_channel == "feishu" || normalized_channel == "wecom" {
        let dm_policy = config
            .get("dmPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("open")
            .to_string();
        config.insert("dmPolicy".to_string(), Value::String(dm_policy.clone()));

        let mut allow_from = config
            .get("allowFrom")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["*".to_string()]);
        if dm_policy == "open" && !allow_from.iter().any(|value| value == "*") {
            allow_from.push("*".to_string());
        }
        config.insert(
            "allowFrom".to_string(),
            Value::Array(
                allow_from
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
            ),
        );
    }

    config.insert("enabled".to_string(), Value::Bool(true));
    config
}

pub(crate) fn extract_channel_form_values(
    channel_type: &str,
    account_obj: &serde_json::Map<String, Value>,
) -> std::collections::HashMap<String, String> {
    let mut values = std::collections::HashMap::new();
    let normalized_channel = normalize_channel_identifier(channel_type);

    for (key, value) in account_obj {
        if key == "enabled"
            || key == "accounts"
            || key == "defaultAccount"
            || key == "name"
            || key == "guilds"
            || key == "allowFrom"
            || key == "groupPolicy"
            || key == "dm"
            || key == "retry"
            || key == "dmPolicy"
        {
            continue;
        }
        if let Some(text) = value
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            values.insert(key.to_string(), text.to_string());
        }
    }

    if normalized_channel == "telegram" {
        if !values.contains_key("allowedUsers") {
            if let Some(users) = account_obj.get("allowFrom").and_then(Value::as_array) {
                let merged = users
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                if !merged.is_empty() {
                    values.insert("allowedUsers".to_string(), merged);
                }
            }
        }
    } else if normalized_channel == "discord" {
        if let Some(guilds) = account_obj.get("guilds").and_then(Value::as_object) {
            if let Some((guild_id, guild_obj)) = guilds.iter().next() {
                if !guild_id.trim().is_empty() {
                    values.insert("guildId".to_string(), guild_id.to_string());
                }
                if let Some(channels) = guild_obj.get("channels").and_then(Value::as_object) {
                    if let Some((channel_id, _)) = channels.iter().find(|(key, _)| *key != "*") {
                        if !channel_id.trim().is_empty() {
                            values.insert("channelId".to_string(), channel_id.to_string());
                        }
                    }
                }
            }
        }
    }

    values
}
