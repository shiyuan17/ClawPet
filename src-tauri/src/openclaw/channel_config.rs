use serde_json::Value;

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
