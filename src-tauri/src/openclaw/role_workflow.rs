use crate::{
    ensure_agents_list_has_main, expand_home_path, resolve_openclaw_config_path,
    resolve_openclaw_home_path, resolve_workspace_agents_root, write_openclaw_config_value,
};
use serde_json::Value;
use std::path::PathBuf;

fn strip_wrapping_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        return trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    trimmed.to_string()
}

fn parse_markdown_frontmatter_and_body(
    markdown: &str,
) -> (std::collections::HashMap<String, String>, String) {
    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let mut fields = std::collections::HashMap::new();
    let lines = normalized.split('\n').collect::<Vec<_>>();

    if lines.first().map(|line| line.trim()) != Some("---") {
        return (fields, normalized);
    }

    let mut end_index = None;
    for (index, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_index = Some(index);
            break;
        }
    }

    let Some(end) = end_index else {
        return (fields, normalized);
    };

    for line in &lines[1..end] {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        if key.is_empty() {
            continue;
        }
        fields.insert(key, strip_wrapping_quotes(raw_value));
    }

    let body = if end + 1 < lines.len() {
        lines[end + 1..].join("\n")
    } else {
        String::new()
    };
    (fields, body)
}

fn is_soul_header(header: &str) -> bool {
    let normalized = header.to_ascii_lowercase();
    let soul_keywords = [
        "identity",
        "communication",
        "style",
        "critical rule",
        "critical-rule",
        "rules you must follow",
        "身份",
        "记忆",
        "溝通",
        "沟通",
        "风格",
        "風格",
        "关键规则",
        "關鍵規則",
    ];
    soul_keywords
        .iter()
        .any(|keyword| normalized.contains(keyword))
}

fn split_openclaw_markdown_sections(body: &str) -> (String, String) {
    let mut soul_content = String::new();
    let mut agents_content = String::new();
    let mut current_section = String::new();
    let mut current_target_is_soul = false;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") || trimmed.starts_with("##\t") {
            if !current_section.is_empty() {
                if current_target_is_soul {
                    soul_content.push_str(&current_section);
                } else {
                    agents_content.push_str(&current_section);
                }
            }
            current_section.clear();
            current_target_is_soul = is_soul_header(trimmed);
        }

        current_section.push_str(line);
        current_section.push('\n');
    }

    if !current_section.is_empty() {
        if current_target_is_soul {
            soul_content.push_str(&current_section);
        } else {
            agents_content.push_str(&current_section);
        }
    }

    (soul_content, agents_content)
}

fn normalize_markdown_output(content: String) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    }
}

#[tauri::command]
pub(crate) fn install_role_workflow_agent(
    agent_id: String,
    display_name: String,
    content: String,
    source_path: Option<String>,
) -> Result<String, String> {
    let normalized_id = {
        let mut output = String::new();
        for ch in agent_id.trim().chars() {
            if ch.is_ascii_alphanumeric() {
                output.push(ch.to_ascii_lowercase());
            } else if ch == '-' || ch == '_' || ch == '/' || ch == '\\' {
                if !output.ends_with('-') {
                    output.push('-');
                }
            }
        }
        output = output.trim_matches('-').to_string();
        if output.is_empty() {
            return Err("角色 ID 为空，无法安装。".to_string());
        }
        output
    };
    if normalized_id.eq_ignore_ascii_case("main") {
        return Err("主控员工 main 受保护，不能被替换。".to_string());
    }

    let workspace_root = resolve_openclaw_home_path().join(format!("workspace-{normalized_id}"));
    std::fs::create_dir_all(&workspace_root).map_err(|error| {
        format!(
            "创建角色工作区失败（{}）：{error}",
            workspace_root.display()
        )
    })?;

    let source_label = source_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let raw_markdown = if content.trim().is_empty() {
        format!(
            "# {name}\n\n来源：{source}\n\n> 该角色由 DragonClaw 安装，请补充具体职责内容。\n",
            name = display_name.trim(),
            source = source_label
        )
    } else {
        let mut normalized = content.replace("\r\n", "\n");
        if !normalized.ends_with('\n') {
            normalized.push('\n');
        }
        normalized
    };

    let (frontmatter_fields, body_markdown_raw) =
        parse_markdown_frontmatter_and_body(&raw_markdown);
    let body_markdown = normalize_markdown_output(body_markdown_raw);
    let (soul_split, agents_split) = split_openclaw_markdown_sections(&body_markdown);
    let mut soul_markdown = normalize_markdown_output(soul_split);
    let mut agents_markdown = normalize_markdown_output(agents_split);

    if agents_markdown.is_empty() {
        agents_markdown = if body_markdown.is_empty() {
            normalize_markdown_output(raw_markdown.clone())
        } else {
            body_markdown.clone()
        };
    }

    let configured_name = display_name.trim();
    let frontmatter_name = frontmatter_fields
        .get("name")
        .map(String::as_str)
        .unwrap_or("")
        .trim();
    let next_name = if !configured_name.is_empty() {
        configured_name.to_string()
    } else if !frontmatter_name.is_empty() {
        frontmatter_name.to_string()
    } else {
        normalized_id.clone()
    };

    let frontmatter_desc = frontmatter_fields
        .get("description")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let frontmatter_vibe = frontmatter_fields
        .get("vibe")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let frontmatter_emoji = frontmatter_fields
        .get("emoji")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if soul_markdown.is_empty() {
        let soul_summary = if !frontmatter_desc.is_empty() {
            frontmatter_desc.clone()
        } else {
            format!("该角色来源于 {source_label}，请补充身份、记忆、沟通风格与关键规则。")
        };
        soul_markdown =
            format!("## 你的身份与记忆\n\n- **角色**：{next_name}\n- **定位**：{soul_summary}\n");
    }

    let identity_title = if frontmatter_emoji.is_empty() {
        format!("# {next_name}")
    } else {
        format!("# {} {next_name}", frontmatter_emoji)
    };
    let identity_summary = if !frontmatter_vibe.is_empty() {
        frontmatter_vibe
    } else if !frontmatter_desc.is_empty() {
        frontmatter_desc
    } else {
        format!("来源：{source_label}")
    };
    let identity_markdown = format!("{identity_title}\n\n{identity_summary}\n");

    let agents_md_path = workspace_root.join("AGENTS.md");
    std::fs::write(&agents_md_path, agents_markdown)
        .map_err(|error| format!("写入角色文件失败（{}）：{error}", agents_md_path.display()))?;
    let soul_md_path = workspace_root.join("SOUL.md");
    std::fs::write(&soul_md_path, soul_markdown)
        .map_err(|error| format!("写入角色文件失败（{}）：{error}", soul_md_path.display()))?;
    let identity_md_path = workspace_root.join("IDENTITY.md");
    std::fs::write(&identity_md_path, identity_markdown).map_err(|error| {
        format!(
            "写入角色文件失败（{}）：{error}",
            identity_md_path.display()
        )
    })?;

    let config_path = resolve_openclaw_config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("创建配置目录失败（{}）：{error}", parent.display()))?;
    }

    let mut parsed = if config_path.exists() {
        let raw = std::fs::read_to_string(&config_path)
            .map_err(|error| format!("读取配置失败（{}）：{error}", config_path.display()))?;
        if raw.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
        }
    } else {
        serde_json::json!({})
    };

    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象，无法安装角色。")?;
    let agents = root
        .entry("agents")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 agents 字段不是对象。")?;
    let list = agents
        .entry("list")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or("openclaw.json 的 agents.list 字段不是数组。")?;

    let workspace_hint = format!("~/.openclaw/workspace-{normalized_id}");
    let mut updated = false;
    for item in list.iter_mut() {
        let Some(obj) = item.as_object_mut() else {
            continue;
        };
        let current_id = obj
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if !current_id.eq_ignore_ascii_case(&normalized_id) {
            continue;
        }
        obj.insert("id".to_string(), Value::String(normalized_id.clone()));
        obj.insert("name".to_string(), Value::String(next_name.clone()));
        obj.insert(
            "workspace".to_string(),
            Value::String(workspace_hint.clone()),
        );
        updated = true;
        break;
    }

    if !updated {
        list.push(serde_json::json!({
            "id": normalized_id,
            "name": next_name,
            "workspace": workspace_hint
        }));
    }
    ensure_agents_list_has_main(list);

    write_openclaw_config_value(&config_path, &parsed)?;

    Ok(format!(
        "角色已安装：{}（工作区：{}）",
        agents_md_path
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("workspace"),
        agents_md_path.display()
    ))
}

#[tauri::command]
pub(crate) fn remove_role_workflow_agent(
    agent_id: String,
    delete_files: bool,
) -> Result<String, String> {
    let normalized_id = {
        let mut output = String::new();
        for ch in agent_id.trim().chars() {
            if ch.is_ascii_alphanumeric() {
                output.push(ch.to_ascii_lowercase());
            } else if ch == '-' || ch == '_' || ch == '/' || ch == '\\' {
                if !output.ends_with('-') {
                    output.push('-');
                }
            }
        }
        output = output.trim_matches('-').to_string();
        if output.is_empty() {
            return Err("角色 ID 为空，无法删除。".to_string());
        }
        output
    };
    if normalized_id.eq_ignore_ascii_case("main") {
        return Err("主控员工 main 受保护，不能删除。".to_string());
    }

    let source_path = resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象，无法删除角色。")?;

    let mut removed_name = normalized_id.clone();
    let mut configured_workspace: Option<String> = None;
    let mut removed_count = 0usize;
    if let Some(list) = root
        .get_mut("agents")
        .and_then(Value::as_object_mut)
        .and_then(|agents| agents.get_mut("list"))
        .and_then(Value::as_array_mut)
    {
        list.retain(|item| {
            let Some(obj) = item.as_object() else {
                return true;
            };
            let current_id = obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            let should_remove =
                !current_id.is_empty() && current_id.eq_ignore_ascii_case(&normalized_id);
            if should_remove {
                removed_count += 1;
                if let Some(name) = obj
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    removed_name = name.to_string();
                }
                if configured_workspace.is_none() {
                    configured_workspace = obj
                        .get("workspace")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned);
                }
            }
            !should_remove
        });
        ensure_agents_list_has_main(list);
    }

    if removed_count == 0 {
        return Err(format!("未找到角色 ID「{}」，无法删除。", normalized_id));
    }

    let mut removed_binding_count = 0usize;
    if let Some(bindings) = root.get_mut("bindings").and_then(Value::as_array_mut) {
        let before = bindings.len();
        bindings.retain(|item| {
            let target_id = item
                .get("agentId")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            !target_id.eq_ignore_ascii_case(&normalized_id)
        });
        removed_binding_count = before.saturating_sub(bindings.len());
    }

    let mut cleared_channel_binding_count = 0usize;
    if let Some(channels) = root.get_mut("channels").and_then(Value::as_object_mut) {
        for channel_section in channels.values_mut() {
            let Some(section_obj) = channel_section.as_object_mut() else {
                continue;
            };
            if let Some(accounts) = section_obj
                .get_mut("accounts")
                .and_then(Value::as_object_mut)
            {
                for account in accounts.values_mut() {
                    let Some(account_obj) = account.as_object_mut() else {
                        continue;
                    };
                    let is_target = account_obj
                        .get("agentId")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .map(|value| value.eq_ignore_ascii_case(&normalized_id))
                        .unwrap_or(false);
                    if is_target {
                        account_obj.remove("agentId");
                        cleared_channel_binding_count += 1;
                    }
                }
            }
        }
    }

    write_openclaw_config_value(&source_path, &parsed)?;

    let mut removed_paths: Vec<String> = Vec::new();
    let mut delete_warnings: Vec<String> = Vec::new();
    if delete_files {
        let mut candidates = Vec::<PathBuf>::new();
        if let Some(workspace) = configured_workspace.as_deref() {
            let expanded = expand_home_path(workspace);
            let candidate = if expanded.is_absolute() {
                expanded
            } else {
                resolve_openclaw_home_path().join(expanded)
            };
            candidates.push(candidate);
        }
        let runtime_agents_root = resolve_openclaw_config_path()
            .parent()
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".openclaw"))
            .join("agents");
        candidates.push(runtime_agents_root.join(&normalized_id));
        candidates.push(resolve_openclaw_home_path().join(format!("workspace-{normalized_id}")));
        candidates.push(resolve_workspace_agents_root().join(&normalized_id));

        let mut seen = std::collections::HashSet::new();
        for path in candidates {
            let key = path.to_string_lossy().to_string();
            if !seen.insert(key.clone()) || !path.exists() {
                continue;
            }
            let result = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
            match result {
                Ok(_) => removed_paths.push(key),
                Err(error) => delete_warnings.push(format!("{key}: {error}")),
            }
        }
    }

    let mut detail = format!("角色「{}」已删除。", removed_name);
    let binding_cleanup_count = removed_binding_count + cleared_channel_binding_count;
    if binding_cleanup_count > 0 {
        detail.push_str(&format!(" 已清理 {} 条绑定关系。", binding_cleanup_count));
    }
    if delete_files {
        if removed_paths.is_empty() {
            detail.push_str(" 未删除配置文件（未找到可删除目录或文件）。");
        } else {
            detail.push_str(&format!(
                " 已删除 {} 个配置文件/目录。",
                removed_paths.len()
            ));
        }
        if !delete_warnings.is_empty() {
            detail.push_str(" 部分配置文件删除失败：");
            detail.push_str(&delete_warnings.join("；"));
        }
    } else {
        detail.push_str(" 配置文件已保留。");
    }

    Ok(detail)
}
