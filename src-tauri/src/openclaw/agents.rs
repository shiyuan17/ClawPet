use serde_json::Value;

fn read_string_or_primary<'a>(value: &'a Value) -> Option<&'a str> {
    value
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .or_else(|| {
            value
                .as_object()
                .and_then(|obj| obj.get("primary"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
        })
}

pub(crate) fn load_agent_context_messages(agent_id: &str) -> Vec<crate::OpenClawMessage> {
    let config_path = crate::resolve_openclaw_config_path();
    let configured_workspace = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|parsed| {
            parsed
                .get("agents")
                .and_then(Value::as_object)
                .and_then(|agents| agents.get("list"))
                .and_then(Value::as_array)
                .and_then(|list| {
                    list.iter().find(|item| {
                        item.as_object()
                            .and_then(|obj| obj.get("id").and_then(Value::as_str))
                            .map(|id| id.eq_ignore_ascii_case(agent_id))
                            .unwrap_or(false)
                    })
                })
                .and_then(|item| {
                    item.as_object()
                        .and_then(|obj| obj.get("workspace").and_then(Value::as_str))
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                })
        });
    let workspace_root =
        crate::resolve_workspace_root_for_agent(agent_id, configured_workspace.as_deref());

    let context_files = [
        "SOUL.md",
        "IDENTITY.md",
        "MEMORY.md",
        "BOOTSTRAP.md",
        "HEARTBEAT.md",
        "AGENTS.md",
    ];

    let mut sections = Vec::new();
    for file_name in context_files {
        let path = workspace_root.join(file_name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                let tag = file_name.trim_end_matches(".md");
                sections.push(format!("<{tag}>\n{trimmed}\n</{tag}>"));
            }
        }
    }

    if sections.is_empty() {
        return Vec::new();
    }

    vec![crate::OpenClawMessage {
        role: "system".to_string(),
        content: sections.join("\n\n"),
    }]
}

pub(crate) fn resolve_agent_model_from_config(agent_id: &str) -> Option<String> {
    let config_path = crate::resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(config_path).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let agents = parsed.get("agents")?.as_object()?;
    let list = agents.get("list")?.as_array()?;
    let agent = list.iter().find(|item| {
        item.as_object()
            .and_then(|obj| obj.get("id").and_then(Value::as_str))
            .map(|id| id.eq_ignore_ascii_case(agent_id))
            .unwrap_or(false)
    })?;
    let model = agent
        .as_object()?
        .get("model")
        .and_then(read_string_or_primary)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    model.or_else(|| {
        agents
            .get("defaults")
            .and_then(Value::as_object)
            .and_then(|d| d.get("model"))
            .and_then(read_string_or_primary)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    })
}
