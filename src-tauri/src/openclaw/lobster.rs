pub(crate) fn bootstrap_openclaw_runtime(install_cli: bool) -> Result<Vec<String>, String> {
    let cli_candidates = crate::collect_openclaw_cli_command_candidates();
    let Some(cli_path) = cli_candidates.first() else {
        return Err(format!(
            "未找到全局 OpenClaw CLI（openclaw）。请先安装并确保 PATH 可见。{}",
            crate::official_openclaw_install_hint_for_platform()
        ));
    };
    let openclaw_home = crate::resolve_openclaw_home_path();

    let mut notes = vec![
        format!("home={}", openclaw_home.display()),
        format!("config={}", crate::resolve_openclaw_config_path().display()),
        format!("cli={}", cli_path.display()),
    ];

    match crate::sanitize_openclaw_channel_schema() {
        Ok(Some(detail)) => notes.push(format!("channel_schema_sanitized={detail}")),
        Ok(None) => {}
        Err(error) => notes.push(format!("channel_schema_sanitize_error={error}")),
    }
    match crate::sanitize_openclaw_plugin_load_paths() {
        Ok(Some(detail)) => notes.push(format!("plugin_load_paths_sanitized={detail}")),
        Ok(None) => {}
        Err(error) => notes.push(format!("plugin_load_paths_sanitize_error={error}")),
    }
    match super::config::schema::ensure_openclaw_chat_completions_endpoint_enabled_outcome() {
        outcome if outcome.any_success() => {
            notes.push(format!(
                "gateway_mode_local_and_chat_completions={}",
                outcome.detail()
            ));
        }
        outcome => {
            return Err(format!(
                "OpenClaw 网关配置未就绪（需要 gateway.mode=local 与 chatCompletions.enabled=true）：{}",
                outcome.detail()
            ));
        }
    }

    match crate::run_openclaw_cli_output(&["--version"]) {
        Ok((command_display, output)) if output.status.success() => {
            let version_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version_text.is_empty() {
                notes.push(format!("cli_version={version_text}"));
            } else {
                notes.push(format!("cli_check={command_display}"));
            }
        }
        Ok((command_display, output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                format!(
                    "全局 OpenClaw CLI 自检失败（{}，exit: {}）。",
                    command_display,
                    output.status.code().unwrap_or(-1)
                )
            } else {
                format!(
                    "全局 OpenClaw CLI 自检失败（{}）：{}",
                    command_display, stderr
                )
            });
        }
        Err(error) => return Err(error),
    }

    if install_cli {
        notes.push("cli_mode=global-only".to_string());
    }

    Ok(notes)
}

pub(crate) fn detect_openclaw_installation() -> (bool, Option<String>, Option<String>, String) {
    let cli_candidates = crate::collect_openclaw_cli_command_candidates();
    let binary = cli_candidates
        .first()
        .map(|path| path.display().to_string());
    if binary.is_none() {
        return (
            false,
            None,
            None,
            format!(
                "未找到全局 OpenClaw CLI（openclaw）。请先安装并确保 PATH 可见。{}",
                crate::official_openclaw_install_hint_for_platform()
            ),
        );
    }

    match crate::run_openclaw_cli_output(&["--version"]) {
        Ok((command_display, result)) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            let version = if stdout.is_empty() {
                if stderr.is_empty() {
                    None
                } else {
                    Some(stderr)
                }
            } else {
                Some(stdout)
            };
            (
                true,
                version,
                binary,
                format!("已检测到全局 OpenClaw CLI（{command_display}）。"),
            )
        }
        Ok((command_display, result)) => {
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            let detail = if stderr.is_empty() {
                format!(
                    "OpenClaw 命令执行失败（{command_display}，exit: {}）。",
                    result.status.code().unwrap_or(-1)
                )
            } else {
                format!("OpenClaw 命令执行失败（{command_display}）：{stderr}")
            };
            (false, None, binary, detail)
        }
        Err(error) => (false, None, binary, error),
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreinstalledSkillManifestItem {
    slug: String,
    #[serde(default)]
    auto_enable: bool,
}

#[derive(Debug, serde::Deserialize)]
struct PreinstalledSkillManifest {
    #[serde(default)]
    skills: Vec<PreinstalledSkillManifestItem>,
}

fn resolve_lobster_backup_root() -> std::path::PathBuf {
    let home = crate::resolve_openclaw_home_path();
    home.parent()
        .map(|parent| parent.join(".openclaw-backups"))
        .unwrap_or_else(|| std::path::PathBuf::from(".openclaw-backups"))
}

fn metadata_modified_at_ms(metadata: &std::fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn collect_dir_size_bytes(path: &std::path::Path) -> u64 {
    let Ok(metadata) = std::fs::metadata(path) else {
        return 0;
    };
    if metadata.is_file() {
        return metadata.len();
    }
    if !metadata.is_dir() {
        return 0;
    }

    let mut total = 0_u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            total = total.saturating_add(collect_dir_size_bytes(&entry.path()));
        }
    }
    total
}

fn copy_directory_recursive(
    source: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("源目录不存在：{}", source.display()));
    }
    if !source.is_dir() {
        return Err(format!("源路径不是目录：{}", source.display()));
    }

    std::fs::create_dir_all(target)
        .map_err(|error| format!("创建目录失败 {}: {error}", target.display()))?;

    let entries = std::fs::read_dir(source)
        .map_err(|error| format!("读取目录失败 {}: {error}", source.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("读取目录项失败: {error}"))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|error| format!("读取文件类型失败 {}: {error}", source_path.display()))?;

        if file_type.is_dir() {
            copy_directory_recursive(&source_path, &target_path)?;
            continue;
        }

        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("创建目录失败 {}: {error}", parent.display()))?;
        }

        std::fs::copy(&source_path, &target_path).map_err(|error| {
            format!(
                "复制文件失败 {} -> {}: {error}",
                source_path.display(),
                target_path.display()
            )
        })?;
    }

    Ok(())
}

fn resolve_preinstalled_skills_dir() -> Option<std::path::PathBuf> {
    let mut candidates = Vec::new();
    for root in crate::resolve_resource_root_candidates() {
        candidates.push(root.join("preinstalled-skills"));
        candidates.push(root.join("resources").join("preinstalled-skills"));
    }

    candidates.into_iter().find(|path| path.is_dir())
}

fn load_preinstalled_skill_auto_enable_map(
    source_root: &std::path::Path,
) -> std::collections::HashMap<String, bool> {
    let manifest_path = source_root.join("preinstalled-manifest.json");
    let raw = match std::fs::read_to_string(&manifest_path) {
        Ok(value) => value,
        Err(_) => return std::collections::HashMap::new(),
    };
    let parsed = match serde_json::from_str::<PreinstalledSkillManifest>(&raw) {
        Ok(value) => value,
        Err(_) => return std::collections::HashMap::new(),
    };

    parsed
        .skills
        .into_iter()
        .filter_map(|item| {
            let slug = item.slug.trim().to_ascii_lowercase();
            if slug.is_empty() {
                None
            } else {
                Some((slug, item.auto_enable))
            }
        })
        .collect()
}

fn sync_preinstalled_skills_to_openclaw_home() -> Result<String, String> {
    let source_root = resolve_preinstalled_skills_dir()
        .ok_or_else(|| "未找到 preinstalled-skills 目录，无法同步预置技能。".to_string())?;
    let target_root = crate::resolve_openclaw_home_path().join("skills");
    std::fs::create_dir_all(&target_root).map_err(|error| {
        format!(
            "创建 OpenClaw 技能目录失败（{}）：{error}",
            target_root.display()
        )
    })?;

    let auto_enable_map = load_preinstalled_skill_auto_enable_map(&source_root);
    let source_entries = std::fs::read_dir(&source_root)
        .map_err(|error| format!("读取预置技能目录失败（{}）：{error}", source_root.display()))?;

    let mut synced_slugs = Vec::<String>::new();
    for entry in source_entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let slug = entry.file_name().to_string_lossy().trim().to_string();
        if slug.is_empty() || slug.starts_with('.') {
            continue;
        }

        let source_skill_dir = entry.path();
        if !source_skill_dir.join("SKILL.md").exists() {
            continue;
        }

        let target_skill_dir = target_root.join(&slug);
        copy_directory_recursive(&source_skill_dir, &target_skill_dir)?;
        synced_slugs.push(slug);
    }

    if synced_slugs.is_empty() {
        return Ok(format!(
            "未发现可同步的预置技能目录（{}）。",
            source_root.display()
        ));
    }

    let config_path = crate::resolve_openclaw_config_path();
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
            serde_json::from_str::<serde_json::Value>(&raw)
                .unwrap_or_else(|_| serde_json::json!({}))
        }
    } else {
        serde_json::json!({})
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }

    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象，无法同步技能。")?;
    let skills = root
        .entry("skills")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 skills 字段不是对象。")?;
    let entries = skills
        .entry("entries")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 skills.entries 字段不是对象。")?;

    let mut auto_enabled_count = 0usize;
    for slug in &synced_slugs {
        let auto_enable = auto_enable_map
            .get(&slug.to_ascii_lowercase())
            .copied()
            .unwrap_or(true);
        if !auto_enable {
            continue;
        }

        let entry = entries
            .entry(slug.to_string())
            .or_insert_with(|| serde_json::json!({}));
        let has_explicit_enabled = entry
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .is_some();
        if !has_explicit_enabled {
            *entry = serde_json::json!({ "enabled": true });
            auto_enabled_count += 1;
        }
    }

    crate::write_openclaw_config_value(&config_path, &parsed)?;

    synced_slugs.sort();
    let preview = if synced_slugs.len() <= 6 {
        synced_slugs.join(", ")
    } else {
        format!(
            "{}, 等 {} 项",
            synced_slugs
                .iter()
                .take(6)
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
            synced_slugs.len()
        )
    };

    Ok(format!(
        "已同步 {} 个预置技能到 {}，按默认策略自动启用 {} 项（{}）。",
        synced_slugs.len(),
        target_root.display(),
        auto_enabled_count,
        preview
    ))
}

fn collect_lobster_backups() -> Vec<crate::LobsterBackupItem> {
    let backup_root = resolve_lobster_backup_root();
    let Ok(entries) = std::fs::read_dir(&backup_root) else {
        return Vec::new();
    };

    let mut backups = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        backups.push(crate::LobsterBackupItem {
            name,
            path: path.display().to_string(),
            created_at_ms: metadata_modified_at_ms(&metadata),
            size_bytes: collect_dir_size_bytes(&path),
        });
    }

    backups.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    backups
}

fn run_lobster_install_action() -> crate::LobsterActionResult {
    let started_at = std::time::Instant::now();
    match bootstrap_openclaw_runtime(true) {
        Ok(mut notes) => {
            let official_onboard = super::onboard::run_openclaw_official_silent_onboard_once();
            notes.push(format!(
                "official_onboard: success={}, command={}",
                official_onboard.success, official_onboard.command
            ));
            notes.push(format!(
                "official_onboard_detail={}",
                official_onboard.detail
            ));

            let endpoint_config = if official_onboard.success {
                super::config::schema::ensure_openclaw_chat_completions_endpoint_enabled_outcome()
            } else {
                super::config::schema::ChatCompletionsEndpointEnableOutcome::default()
            };
            if official_onboard.success {
                notes.push(format!(
                    "chat_completions_endpoint: success={}, changed={}",
                    endpoint_config.any_success(),
                    endpoint_config.changed()
                ));
                notes.push(format!(
                    "chat_completions_endpoint_detail={}",
                    endpoint_config.detail()
                ));
            } else {
                notes.push(
                    "chat_completions_endpoint: success=false, changed=false, skipped=true"
                        .to_string(),
                );
                notes.push(
                    "chat_completions_endpoint_detail=已跳过写入，因为官方静默安装失败。"
                        .to_string(),
                );
            }

            let gateway_daemon = if official_onboard.success {
                super::gateway::run_openclaw_gateway_daemon_ensure_once()
            } else {
                super::gateway::GatewayDaemonEnsureOutcome {
                    success: false,
                    command: "openclaw gateway install/start/status --json (skipped)".to_string(),
                    detail: "已跳过后台守护进程校验，因为官方静默安装失败。".to_string(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                }
            };
            notes.push(format!(
                "gateway_daemon: success={}, command={}",
                gateway_daemon.success, gateway_daemon.command
            ));
            notes.push(format!("gateway_daemon_detail={}", gateway_daemon.detail));

            let preinstalled_skill_sync = if official_onboard.success {
                match sync_preinstalled_skills_to_openclaw_home() {
                    Ok(detail) => (true, detail),
                    Err(error) => (false, error),
                }
            } else {
                (
                    false,
                    "已跳过预置技能同步，因为官方静默安装失败。".to_string(),
                )
            };
            notes.push(format!(
                "preinstalled_skills: success={}, detail={}",
                preinstalled_skill_sync.0, preinstalled_skill_sync.1
            ));

            let (installed, version, _, detail) = detect_openclaw_installation();
            let chat_endpoint_ready = official_onboard.success && endpoint_config.any_success();
            let success = installed
                && official_onboard.success
                && chat_endpoint_ready
                && gateway_daemon.success
                && preinstalled_skill_sync.0;
            let official_warning = if official_onboard.success {
                if official_onboard.degraded {
                    format!(" 官方静默安装提示：{}", official_onboard.detail)
                } else {
                    String::new()
                }
            } else {
                format!(" 官方静默安装提示：{}", official_onboard.detail)
            };
            let daemon_warning = if gateway_daemon.success {
                String::new()
            } else {
                format!(" 后台守护进程提示：{}", gateway_daemon.detail)
            };
            let chat_endpoint_warning = if official_onboard.success && endpoint_config.any_success()
            {
                String::new()
            } else if official_onboard.success {
                format!(" 聊天端点配置提示：{}", endpoint_config.detail())
            } else {
                String::new()
            };
            let chat_endpoint_note = if official_onboard.success {
                format!(" 聊天端点配置：{}", endpoint_config.detail())
            } else {
                String::new()
            };
            let preinstalled_skill_warning = if preinstalled_skill_sync.0 {
                String::new()
            } else {
                format!(" 预置技能同步提示：{}", preinstalled_skill_sync.1)
            };
            let preinstalled_skill_note = if preinstalled_skill_sync.0 {
                format!(" 预置技能同步：{}", preinstalled_skill_sync.1)
            } else {
                String::new()
            };

            let mut runtime_logs = Vec::new();
            if !official_onboard.stdout.trim().is_empty() {
                runtime_logs.push(format!(
                    "[official-onboard]\n{}",
                    official_onboard.stdout.trim()
                ));
            }
            if !gateway_daemon.stdout.trim().is_empty() {
                runtime_logs.push(gateway_daemon.stdout.trim().to_string());
            }
            let stdout = if runtime_logs.is_empty() {
                notes.join("\n")
            } else {
                format!("{}\n\n{}", notes.join("\n"), runtime_logs.join("\n\n"))
            };
            let stderr = format!(
                "{}{}{}{}",
                official_onboard.stderr,
                if official_onboard.stderr.trim().is_empty() {
                    ""
                } else {
                    "\n"
                },
                gateway_daemon.stderr,
                if gateway_daemon.stderr.trim().is_empty() {
                    ""
                } else {
                    "\n"
                }
            )
            .trim()
            .to_string();

            crate::LobsterActionResult {
                action: "install".to_string(),
                command: format!(
                    "{} && {} && {}",
                    official_onboard.command,
                    "write openclaw.json gateway.http.endpoints.chatCompletions.enabled=true",
                    gateway_daemon.command
                ),
                success,
                detail: if success {
                    format!(
                        "OpenClaw 官方静默安装完成。{}{}{}{}{}{}",
                        version
                            .map(|value| format!("当前版本：{value}。"))
                            .unwrap_or_default(),
                        if detail.trim().is_empty() {
                            String::new()
                        } else {
                            format!(" {detail}")
                        },
                        official_warning,
                        chat_endpoint_note,
                        preinstalled_skill_note,
                        daemon_warning
                    )
                } else {
                    format!(
                        "OpenClaw 官方静默安装后仍未就绪：{detail} 官方静默安装结果：{} 聊天端点配置结果：{} 后台守护进程校验结果：{}{}{}{}",
                        official_onboard.detail,
                        endpoint_config.detail(),
                        gateway_daemon.detail,
                        official_warning,
                        chat_endpoint_warning,
                        preinstalled_skill_warning
                    )
                },
                exit_code: if success {
                    gateway_daemon
                        .exit_code
                        .or(official_onboard.exit_code)
                        .or(Some(0))
                } else {
                    None
                },
                stdout,
                stderr,
                duration_ms: started_at.elapsed().as_millis(),
                backup_path: None,
            }
        }
        Err(error) => crate::LobsterActionResult {
            action: "install".to_string(),
            command: "openclaw onboard --non-interactive".to_string(),
            success: false,
            detail: format!("OpenClaw 官方静默安装预检失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        },
    }
}

fn run_lobster_upgrade_action() -> crate::LobsterActionResult {
    let started_at = std::time::Instant::now();
    match bootstrap_openclaw_runtime(true) {
        Ok(mut notes) => {
            let official_onboard = super::onboard::run_openclaw_official_silent_onboard_once();
            notes.push(format!(
                "official_onboard: success={}, command={}",
                official_onboard.success, official_onboard.command
            ));
            notes.push(format!(
                "official_onboard_detail={}",
                official_onboard.detail
            ));

            let endpoint_config = if official_onboard.success {
                super::config::schema::ensure_openclaw_chat_completions_endpoint_enabled_outcome()
            } else {
                super::config::schema::ChatCompletionsEndpointEnableOutcome::default()
            };
            if official_onboard.success {
                notes.push(format!(
                    "chat_completions_endpoint: success={}, changed={}",
                    endpoint_config.any_success(),
                    endpoint_config.changed()
                ));
                notes.push(format!(
                    "chat_completions_endpoint_detail={}",
                    endpoint_config.detail()
                ));
            } else {
                notes.push(
                    "chat_completions_endpoint: success=false, changed=false, skipped=true"
                        .to_string(),
                );
                notes.push(
                    "chat_completions_endpoint_detail=已跳过写入，因为官方静默安装失败。"
                        .to_string(),
                );
            }

            let gateway_daemon = if official_onboard.success {
                super::gateway::run_openclaw_gateway_daemon_ensure_once()
            } else {
                super::gateway::GatewayDaemonEnsureOutcome {
                    success: false,
                    command: "openclaw gateway install/start/status --json (skipped)".to_string(),
                    detail: "已跳过后台守护进程校验，因为官方静默安装失败。".to_string(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::new(),
                }
            };
            notes.push(format!(
                "gateway_daemon: success={}, command={}",
                gateway_daemon.success, gateway_daemon.command
            ));
            notes.push(format!("gateway_daemon_detail={}", gateway_daemon.detail));

            let preinstalled_skill_sync = if official_onboard.success {
                match sync_preinstalled_skills_to_openclaw_home() {
                    Ok(detail) => (true, detail),
                    Err(error) => (false, error),
                }
            } else {
                (
                    false,
                    "已跳过预置技能同步，因为官方静默安装失败。".to_string(),
                )
            };
            notes.push(format!(
                "preinstalled_skills: success={}, detail={}",
                preinstalled_skill_sync.0, preinstalled_skill_sync.1
            ));

            let (installed, version, _, detail) = detect_openclaw_installation();
            let chat_endpoint_ready = official_onboard.success && endpoint_config.any_success();
            let success = installed
                && official_onboard.success
                && chat_endpoint_ready
                && gateway_daemon.success
                && preinstalled_skill_sync.0;
            let official_warning = if official_onboard.success {
                if official_onboard.degraded {
                    format!(" 官方静默安装提示：{}", official_onboard.detail)
                } else {
                    String::new()
                }
            } else {
                format!(" 官方静默安装提示：{}", official_onboard.detail)
            };
            let daemon_warning = if gateway_daemon.success {
                String::new()
            } else {
                format!(" 后台守护进程提示：{}", gateway_daemon.detail)
            };
            let chat_endpoint_warning = if official_onboard.success && endpoint_config.any_success()
            {
                String::new()
            } else if official_onboard.success {
                format!(" 聊天端点配置提示：{}", endpoint_config.detail())
            } else {
                String::new()
            };
            let chat_endpoint_note = if official_onboard.success {
                format!(" 聊天端点配置：{}", endpoint_config.detail())
            } else {
                String::new()
            };
            let preinstalled_skill_warning = if preinstalled_skill_sync.0 {
                String::new()
            } else {
                format!(" 预置技能同步提示：{}", preinstalled_skill_sync.1)
            };
            let preinstalled_skill_note = if preinstalled_skill_sync.0 {
                format!(" 预置技能同步：{}", preinstalled_skill_sync.1)
            } else {
                String::new()
            };

            let mut runtime_logs = Vec::new();
            if !official_onboard.stdout.trim().is_empty() {
                runtime_logs.push(format!(
                    "[official-onboard]\n{}",
                    official_onboard.stdout.trim()
                ));
            }
            if !gateway_daemon.stdout.trim().is_empty() {
                runtime_logs.push(gateway_daemon.stdout.trim().to_string());
            }
            let stdout = if runtime_logs.is_empty() {
                notes.join("\n")
            } else {
                format!("{}\n\n{}", notes.join("\n"), runtime_logs.join("\n\n"))
            };
            let stderr = format!(
                "{}{}{}{}",
                official_onboard.stderr,
                if official_onboard.stderr.trim().is_empty() {
                    ""
                } else {
                    "\n"
                },
                gateway_daemon.stderr,
                if gateway_daemon.stderr.trim().is_empty() {
                    ""
                } else {
                    "\n"
                }
            )
            .trim()
            .to_string();

            crate::LobsterActionResult {
                action: "upgrade".to_string(),
                command: format!(
                    "{} && {} && {}",
                    official_onboard.command,
                    "write openclaw.json gateway.http.endpoints.chatCompletions.enabled=true",
                    gateway_daemon.command
                ),
                success,
                detail: if success {
                    format!(
                        "OpenClaw 官方静默升级完成。{}{}{}{}{}{}",
                        version
                            .map(|value| format!("当前版本：{value}。"))
                            .unwrap_or_default(),
                        if detail.trim().is_empty() {
                            String::new()
                        } else {
                            format!(" {detail}")
                        },
                        official_warning,
                        chat_endpoint_note,
                        preinstalled_skill_note,
                        daemon_warning
                    )
                } else {
                    format!(
                        "OpenClaw 官方静默升级后仍未就绪：{detail} 官方静默安装结果：{} 聊天端点配置结果：{} 后台守护进程校验结果：{}{}{}{}",
                        official_onboard.detail,
                        endpoint_config.detail(),
                        gateway_daemon.detail,
                        official_warning,
                        chat_endpoint_warning,
                        preinstalled_skill_warning
                    )
                },
                exit_code: if success {
                    gateway_daemon
                        .exit_code
                        .or(official_onboard.exit_code)
                        .or(Some(0))
                } else {
                    None
                },
                stdout,
                stderr,
                duration_ms: started_at.elapsed().as_millis(),
                backup_path: None,
            }
        }
        Err(error) => crate::LobsterActionResult {
            action: "upgrade".to_string(),
            command: "openclaw onboard --non-interactive".to_string(),
            success: false,
            detail: format!("OpenClaw 官方静默升级预检失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        },
    }
}

fn run_lobster_restart_gateway_action() -> crate::LobsterActionResult {
    let started_at = std::time::Instant::now();
    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return crate::LobsterActionResult {
            action: "restart_gateway".to_string(),
            command: "openclaw gateway restart".to_string(),
            success: false,
            detail: format!("网关重启前自举失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        };
    }

    let gateway_bootstrap = super::gateway::run_openclaw_gateway_bootstrap_once();
    crate::LobsterActionResult {
        action: "restart_gateway".to_string(),
        command: gateway_bootstrap.command.clone(),
        success: gateway_bootstrap.success,
        detail: if gateway_bootstrap.success {
            "网关重启完成。".to_string()
        } else {
            format!("网关重启结果：{}", gateway_bootstrap.detail)
        },
        exit_code: gateway_bootstrap.exit_code,
        stdout: gateway_bootstrap.stdout,
        stderr: gateway_bootstrap.stderr,
        duration_ms: started_at.elapsed().as_millis(),
        backup_path: None,
    }
}

fn run_lobster_start_gateway_action() -> crate::LobsterActionResult {
    let started_at = std::time::Instant::now();
    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return crate::LobsterActionResult {
            action: "start_gateway".to_string(),
            command: "openclaw gateway start".to_string(),
            success: false,
            detail: format!("网关启动前自举失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        };
    }

    let gateway_daemon = super::gateway::run_openclaw_gateway_daemon_ensure_once();
    crate::LobsterActionResult {
        action: "start_gateway".to_string(),
        command: gateway_daemon.command.clone(),
        success: gateway_daemon.success,
        detail: if gateway_daemon.success {
            "网关启动完成。".to_string()
        } else {
            format!("网关启动结果：{}", gateway_daemon.detail)
        },
        exit_code: gateway_daemon.exit_code,
        stdout: gateway_daemon.stdout,
        stderr: gateway_daemon.stderr,
        duration_ms: started_at.elapsed().as_millis(),
        backup_path: None,
    }
}

fn run_lobster_pause_gateway_action() -> crate::LobsterActionResult {
    let started_at = std::time::Instant::now();
    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return crate::LobsterActionResult {
            action: "pause_gateway".to_string(),
            command: "openclaw gateway stop --json".to_string(),
            success: false,
            detail: format!("网关暂停前自举失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        };
    }

    match super::runtime::run_openclaw_cli_output(&["gateway", "stop", "--json"]) {
        Ok((command_display, output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let merged_text = format!("{}\n{}", stdout.trim(), stderr.trim()).to_ascii_lowercase();
            let likely_already_stopped = merged_text.contains("already stopped")
                || merged_text.contains("not running")
                || merged_text.contains("not-loaded")
                || merged_text.contains("not loaded")
                || merged_text.contains("service not loaded")
                || merged_text.contains("\"status\":\"stopped\"")
                || merged_text.contains("\"status\": \"stopped\"");
            let port_still_ready = super::gateway::wait_for_loopback_port_listening(
                super::gateway::resolve_openclaw_gateway_port(),
                6,
                180,
            );
            let success = !port_still_ready && (output.status.success() || likely_already_stopped);

            crate::LobsterActionResult {
                action: "pause_gateway".to_string(),
                command: command_display,
                success,
                detail: if success {
                    if likely_already_stopped {
                        "网关已处于暂停状态。".to_string()
                    } else {
                        "网关已暂停。".to_string()
                    }
                } else if port_still_ready {
                    "停止命令已执行，但网关端口仍在线，请稍后重试或执行“重启网关”。".to_string()
                } else {
                    let merged_detail = format!("{}\n{}", stdout.trim(), stderr.trim())
                        .trim()
                        .to_string();
                    if merged_detail.is_empty() {
                        "网关暂停失败，请查看日志输出。".to_string()
                    } else {
                        format!("网关暂停失败：{merged_detail}")
                    }
                },
                exit_code: output.status.code(),
                stdout,
                stderr,
                duration_ms: started_at.elapsed().as_millis(),
                backup_path: None,
            }
        }
        Err(error) => crate::LobsterActionResult {
            action: "pause_gateway".to_string(),
            command: "openclaw gateway stop --json".to_string(),
            success: false,
            detail: "网关暂停失败，无法调用全局 OpenClaw CLI。".to_string(),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        },
    }
}

fn run_lobster_backup_action() -> crate::LobsterActionResult {
    let started_at = std::time::Instant::now();
    let openclaw_home = crate::resolve_openclaw_home_path();
    if !openclaw_home.exists() {
        return crate::LobsterActionResult {
            action: "backup".to_string(),
            command: "backup openclaw home".to_string(),
            success: false,
            detail: format!("未找到龙虾目录：{}", openclaw_home.display()),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        };
    }

    let backup_root = resolve_lobster_backup_root();
    if let Err(error) = std::fs::create_dir_all(&backup_root) {
        return crate::LobsterActionResult {
            action: "backup".to_string(),
            command: "backup openclaw home".to_string(),
            success: false,
            detail: format!("创建备份目录失败 {}: {error}", backup_root.display()),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        };
    }

    let backup_name = format!("openclaw-backup-{}", crate::current_timestamp_millis());
    let backup_path = backup_root.join(&backup_name);
    if let Err(error) = copy_directory_recursive(&openclaw_home, &backup_path) {
        return crate::LobsterActionResult {
            action: "backup".to_string(),
            command: "backup openclaw home".to_string(),
            success: false,
            detail: error,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        };
    }

    let size_mb = (collect_dir_size_bytes(&backup_path) as f64) / (1024.0 * 1024.0);
    crate::LobsterActionResult {
        action: "backup".to_string(),
        command: "backup openclaw home".to_string(),
        success: true,
        detail: format!("备份完成：{}（{size_mb:.2} MB）", backup_path.display()),
        exit_code: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        duration_ms: started_at.elapsed().as_millis(),
        backup_path: Some(backup_path.display().to_string()),
    }
}

fn run_lobster_restore_action(backup_path: Option<String>) -> crate::LobsterActionResult {
    let started_at = std::time::Instant::now();
    let selected_backup = backup_path
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(trimmed))
            }
        })
        .or_else(|| {
            collect_lobster_backups()
                .first()
                .map(|item| std::path::PathBuf::from(&item.path))
        });

    let Some(selected_backup) = selected_backup else {
        return crate::LobsterActionResult {
            action: "restore".to_string(),
            command: "restore openclaw backup".to_string(),
            success: false,
            detail: "未找到可恢复的备份。".to_string(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: None,
        };
    };

    if !selected_backup.exists() || !selected_backup.is_dir() {
        return crate::LobsterActionResult {
            action: "restore".to_string(),
            command: "restore openclaw backup".to_string(),
            success: false,
            detail: format!("备份目录不存在：{}", selected_backup.display()),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: Some(selected_backup.display().to_string()),
        };
    }

    let openclaw_home = crate::resolve_openclaw_home_path();
    let restore_parent = openclaw_home
        .parent()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let home_name = openclaw_home
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "openclaw".to_string());
    let restore_stamp = crate::current_timestamp_millis();
    let stage_path = restore_parent.join(format!(".{home_name}.restore-stage-{restore_stamp}"));
    let old_backup_path = restore_parent.join(format!(".{home_name}.pre-restore-{restore_stamp}"));

    if stage_path.exists() {
        let _ = std::fs::remove_dir_all(&stage_path);
    }

    if let Err(error) = copy_directory_recursive(&selected_backup, &stage_path) {
        return crate::LobsterActionResult {
            action: "restore".to_string(),
            command: "restore openclaw backup".to_string(),
            success: false,
            detail: format!("复制恢复内容失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: Some(selected_backup.display().to_string()),
        };
    }

    if openclaw_home.exists() {
        if let Err(error) = std::fs::rename(&openclaw_home, &old_backup_path) {
            let _ = std::fs::remove_dir_all(&stage_path);
            return crate::LobsterActionResult {
                action: "restore".to_string(),
                command: "restore openclaw backup".to_string(),
                success: false,
                detail: format!("保存当前目录失败：{error}"),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                duration_ms: started_at.elapsed().as_millis(),
                backup_path: Some(selected_backup.display().to_string()),
            };
        }
    }

    if let Err(error) = std::fs::rename(&stage_path, &openclaw_home) {
        if !openclaw_home.exists() && old_backup_path.exists() {
            let _ = std::fs::rename(&old_backup_path, &openclaw_home);
        }
        return crate::LobsterActionResult {
            action: "restore".to_string(),
            command: "restore openclaw backup".to_string(),
            success: false,
            detail: format!("应用恢复目录失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
            backup_path: Some(selected_backup.display().to_string()),
        };
    }

    let old_path_hint = if old_backup_path.exists() {
        format!("旧目录已保留：{}", old_backup_path.display())
    } else {
        "恢复前目录不存在。".to_string()
    };

    crate::LobsterActionResult {
        action: "restore".to_string(),
        command: "restore openclaw backup".to_string(),
        success: true,
        detail: format!(
            "已从 {} 恢复龙虾配置。{}",
            selected_backup.display(),
            old_path_hint
        ),
        exit_code: Some(0),
        stdout: String::new(),
        stderr: String::new(),
        duration_ms: started_at.elapsed().as_millis(),
        backup_path: Some(selected_backup.display().to_string()),
    }
}

pub(crate) fn load_lobster_snapshot_blocking() -> Result<crate::LobsterSnapshotResponse, String> {
    let (installed, version, binary, detail) = detect_openclaw_installation();
    let openclaw_home = crate::resolve_openclaw_home_path();
    let backup_dir = resolve_lobster_backup_root();
    Ok(crate::LobsterSnapshotResponse {
        openclaw_installed: installed,
        openclaw_version: version,
        openclaw_binary: binary,
        openclaw_home: openclaw_home.display().to_string(),
        backup_dir: backup_dir.display().to_string(),
        detail,
        backups: collect_lobster_backups(),
        install_wizard_open_every_launch: crate::read_env_bool(
            "DRAGONCLAW_INSTALL_WIZARD_OPEN_EVERY_LAUNCH",
            false,
        ),
    })
}

pub(crate) fn check_openclaw_runtime_status_blocking(
) -> Result<crate::OpenClawRuntimeStatusResponse, String> {
    let gateway_port = Some(crate::resolve_openclaw_gateway_port());
    let sanitize_notice = match crate::sanitize_openclaw_models_provider_schema() {
        Ok(value) => value,
        Err(error) => Some(error),
    };
    let (installed, version, _binary, installation_detail) = detect_openclaw_installation();

    if !installed {
        return Ok(crate::OpenClawRuntimeStatusResponse {
            installed: false,
            healthy: false,
            status: "not_installed".to_string(),
            command: "openclaw gateway status --json".to_string(),
            detail: installation_detail,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            gateway_port,
        });
    }

    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return Ok(crate::OpenClawRuntimeStatusResponse {
            installed: true,
            healthy: false,
            status: "bootstrap_failed".to_string(),
            command: "openclaw gateway status --json".to_string(),
            detail: format!("OpenClaw 已安装，但运行时初始化失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            gateway_port,
        });
    }

    let (command_display, output) =
        match crate::run_openclaw_cli_output(&["gateway", "status", "--json"]) {
            Ok(value) => value,
            Err(error) => {
                return Ok(crate::OpenClawRuntimeStatusResponse {
                    installed: true,
                    healthy: false,
                    status: "status_command_failed".to_string(),
                    command: "openclaw gateway status --json".to_string(),
                    detail: "OpenClaw 状态检查失败，无法调用 CLI。".to_string(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: error,
                    gateway_port,
                });
            }
        };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let merged_output = format!("{}\n{}", stdout.trim(), stderr.trim())
        .trim()
        .to_string();
    let status_effective = crate::gateway_cli_step_effective_success(&output);
    let status_payload = super::json::extract_last_json_object_from_streams(&stdout, &stderr);
    let status_running = if let Some(payload) = status_payload.as_ref() {
        crate::gateway_status_payload_indicates_running(payload)
    } else {
        crate::gateway_status_text_indicates_running(&merged_output)
    };
    let port_ready = super::gateway::wait_for_loopback_port_listening(
        crate::resolve_openclaw_gateway_port(),
        4,
        180,
    );
    let gateway_probe = if port_ready && (!status_effective || !status_running) {
        crate::check_openclaw_gateway_health_fallback_blocking()
    } else {
        None
    };
    let gateway_probe_online = gateway_probe
        .as_ref()
        .map(crate::is_gateway_health_probe_online)
        .unwrap_or(false);
    let fallback_online = port_ready && gateway_probe_online;
    let healthy = (status_effective && status_running && port_ready) || fallback_online;

    let status_summary = status_payload
        .as_ref()
        .map(crate::gateway_status_payload_summary)
        .unwrap_or_else(|| "未解析到 JSON 状态对象".to_string());
    let probe_summary = crate::summarize_gateway_health_probe(gateway_probe.as_ref());
    let version_label = version
        .map(|value| format!("版本：{value}。"))
        .unwrap_or_default();
    let status_detail = format!(
        "status_effective={status_effective}, status_running={status_running}, port_ready={port_ready}, fallback_online={fallback_online}, {status_summary}, {probe_summary}"
    );
    let detail_core = if healthy {
        format!("OpenClaw 运行状态正常。{version_label} {status_detail}")
    } else if merged_output.is_empty() {
        format!("OpenClaw 状态异常。{status_detail}")
    } else {
        format!("OpenClaw 状态异常。{status_detail}。详情：{merged_output}")
    };
    let detail = if let Some(notice) = sanitize_notice {
        format!("{detail_core} 配置兼容性处理：{notice}")
    } else {
        detail_core
    };

    Ok(crate::OpenClawRuntimeStatusResponse {
        installed: true,
        healthy,
        status: if healthy {
            "online".to_string()
        } else if status_running {
            "degraded".to_string()
        } else {
            "offline".to_string()
        },
        command: command_display,
        detail,
        exit_code: output.status.code(),
        stdout,
        stderr,
        gateway_port,
    })
}

pub(crate) fn load_lobster_install_guide_blocking(
) -> Result<crate::LobsterInstallGuideResponse, String> {
    let mut checks: Vec<crate::LobsterInstallCheckItem> = Vec::new();

    let global_cli = crate::collect_openclaw_cli_command_candidates()
        .into_iter()
        .next();
    checks.push(crate::LobsterInstallCheckItem {
        id: "runtime".to_string(),
        title: "全局 OpenClaw CLI".to_string(),
        status: if global_cli.is_some() {
            "success".to_string()
        } else {
            "failed".to_string()
        },
        detail: global_cli
            .map(|path| format!("已找到全局 CLI：{}", path.display()))
            .unwrap_or_else(|| {
                "未找到全局 openclaw 命令。请先执行 `npm i -g openclaw`（或安装到 ~/.petclaw/node）并确保 PATH 可见。".to_string()
            }),
    });

    let node_check = match crate::resolve_openclaw_node_runtime() {
        Ok((path, version)) => crate::LobsterInstallCheckItem {
            id: "nodejs".to_string(),
            title: "Node.js 执行器".to_string(),
            status: "success".to_string(),
            detail: format!(
                "已找到 Node 可执行文件：{}（版本 {}）",
                path.display(),
                version
            ),
        },
        Err(error) => crate::LobsterInstallCheckItem {
            id: "nodejs".to_string(),
            title: "Node.js 执行器".to_string(),
            status: "failed".to_string(),
            detail: error,
        },
    };
    checks.push(node_check);

    let (openclaw_installed, openclaw_version, _, openclaw_detail) = detect_openclaw_installation();
    let cli_blocking =
        openclaw_detail.contains("未找到 Node.js") || openclaw_detail.contains("运行条件不满足");
    checks.push(crate::LobsterInstallCheckItem {
        id: "openclaw-cli".to_string(),
        title: "OpenClaw CLI 可执行性".to_string(),
        status: if openclaw_installed {
            "success".to_string()
        } else if cli_blocking {
            "failed".to_string()
        } else {
            "warning".to_string()
        },
        detail: if openclaw_installed {
            openclaw_version
                .map(|value| format!("可执行，当前版本：{value}"))
                .unwrap_or_else(|| "可执行，版本号待确认。".to_string())
        } else if cli_blocking {
            format!("尚未通过 OpenClaw CLI 自检，安装步骤已阻断。{openclaw_detail}")
        } else {
            format!("尚未通过 OpenClaw CLI 自检，安装步骤仍可继续。{openclaw_detail}")
        },
    });

    checks.push(crate::LobsterInstallCheckItem {
        id: "official-onboard".to_string(),
        title: "官方静默安装命令".to_string(),
        status: "success".to_string(),
        detail: format!(
            "将通过全局 openclaw CLI 执行（不使用内置运行时）。命令参数：openclaw onboard --non-interactive --accept-risk --mode local --flow quickstart --auth-choice skip --gateway-auth token --gateway-token *** --gateway-port {} --gateway-bind loopback --install-daemon --workspace {} --json",
            crate::resolve_openclaw_gateway_port(),
            crate::resolve_workspace_main_root().display()
        ),
    });

    let ready = !checks.iter().any(|item| item.status == "failed");
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    Ok(crate::LobsterInstallGuideResponse {
        os: os.to_string(),
        ready,
        checks,
    })
}

pub(crate) fn run_lobster_action_blocking(
    action: String,
    backup_path: Option<String>,
) -> Result<crate::LobsterActionResult, String> {
    let normalized = action.trim().to_ascii_lowercase();
    match crate::sanitize_openclaw_models_provider_schema() {
        Ok(Some(detail)) => {
            eprintln!("[dragonclaw] openclaw provider config sanitized before action: {detail}");
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("[dragonclaw] openclaw provider config sanitize failed: {error}");
        }
    }
    let result = match normalized.as_str() {
        "install" => run_lobster_install_action(),
        "restart_gateway" => run_lobster_restart_gateway_action(),
        "start_gateway" => run_lobster_start_gateway_action(),
        "pause_gateway" => run_lobster_pause_gateway_action(),
        "auto_fix" => {
            let started_at = std::time::Instant::now();
            if let Err(error) = bootstrap_openclaw_runtime(false) {
                crate::LobsterActionResult {
                    action: "auto_fix".to_string(),
                    command: "openclaw doctor --fix --yes --non-interactive".to_string(),
                    success: false,
                    detail: format!("自动修复前自举失败：{error}"),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: error,
                    duration_ms: started_at.elapsed().as_millis(),
                    backup_path: None,
                }
            } else {
                match crate::run_openclaw_cli_output(&[
                    "doctor",
                    "--fix",
                    "--yes",
                    "--non-interactive",
                ]) {
                    Ok((command_display, output)) => crate::LobsterActionResult {
                        action: "auto_fix".to_string(),
                        command: command_display,
                        success: output.status.success(),
                        detail: if output.status.success() {
                            "自动修复执行完成。".to_string()
                        } else {
                            "自动修复执行失败，请查看日志输出。".to_string()
                        },
                        exit_code: output.status.code(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                        duration_ms: started_at.elapsed().as_millis(),
                        backup_path: None,
                    },
                    Err(error) => crate::LobsterActionResult {
                        action: "auto_fix".to_string(),
                        command: "openclaw doctor --fix --yes --non-interactive".to_string(),
                        success: false,
                        detail: "自动修复执行失败，请查看日志输出。".to_string(),
                        exit_code: None,
                        stdout: String::new(),
                        stderr: error,
                        duration_ms: started_at.elapsed().as_millis(),
                        backup_path: None,
                    },
                }
            }
        }
        "backup" => run_lobster_backup_action(),
        "restore" => run_lobster_restore_action(backup_path),
        "upgrade" => run_lobster_upgrade_action(),
        _ => {
            return Err(format!("不支持的龙虾操作：{action}"));
        }
    };
    Ok(result)
}
