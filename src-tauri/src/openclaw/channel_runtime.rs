use crate::openclaw::channel_config::{
    normalize_channel_identifier, resolve_channel_plugin_install_spec,
};
use crate::openclaw::channel_onboarding::trim_remote_error_detail;
use crate::openclaw::config::paths::resolve_openclaw_home_path;
use crate::openclaw::config::schema::collect_openclaw_candidate_config_paths;
use crate::openclaw::config::write::write_openclaw_config_value;
use crate::openclaw::gateway::{
    append_labeled_output_section, resolve_default_openclaw_api_url, resolve_openclaw_gateway_port,
};
use crate::openclaw::lobster::bootstrap_openclaw_runtime;
use crate::openclaw::models::*;
use crate::openclaw::runtime::run_openclaw_cli_output;
use crate::openclaw::utils::resolve_openclaw_gateway_token;
use chrono::Utc;
use reqwest::header::AUTHORIZATION;
use serde_json::Value;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

fn extract_missing_plugin_ids_from_text(raw: &str) -> Vec<String> {
    const MARKER: &str = "plugin not found:";
    let mut seen = HashSet::new();
    let mut ids = Vec::new();
    for line in raw.lines() {
        let lower = line.to_ascii_lowercase();
        let Some(marker_index) = lower.find(MARKER) else {
            continue;
        };
        let raw_candidate = line[marker_index + MARKER.len()..]
            .trim()
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|ch: char| {
                matches!(
                    ch,
                    '"' | '\''
                        | '`'
                        | ','
                        | ';'
                        | '.'
                        | ':'
                        | ')'
                        | '('
                        | '，'
                        | '。'
                        | '；'
                        | '（'
                        | '）'
                )
            })
            .trim();
        if raw_candidate.is_empty() {
            continue;
        }
        let normalized = raw_candidate.to_ascii_lowercase();
        if seen.insert(normalized.clone()) {
            ids.push(normalized);
        }
    }
    ids
}

fn prune_missing_plugin_refs_from_openclaw_config(
    missing_plugin_ids: &[String],
) -> Result<Option<String>, String> {
    let missing_set = missing_plugin_ids
        .iter()
        .map(|item| item.trim().to_ascii_lowercase())
        .filter(|item| !item.is_empty())
        .collect::<HashSet<_>>();
    if missing_set.is_empty() {
        return Ok(None);
    }

    let mut changed_paths = Vec::new();
    for config_path in collect_openclaw_candidate_config_paths() {
        let raw = match std::fs::read_to_string(&config_path) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!("无法读取 {}: {error}", config_path.display()));
            }
        };
        let mut parsed: Value = serde_json::from_str(&raw)
            .map_err(|error| format!("{} 解析失败: {error}", config_path.display()))?;
        let Some(root) = parsed.as_object_mut() else {
            continue;
        };
        let Some(plugins_obj) = root.get_mut("plugins").and_then(Value::as_object_mut) else {
            continue;
        };

        let mut changed = false;
        if let Some(allow_arr) = plugins_obj.get_mut("allow").and_then(Value::as_array_mut) {
            let before_len = allow_arr.len();
            allow_arr.retain(|item| {
                let Some(text) = item.as_str() else {
                    return true;
                };
                !missing_set.contains(&text.trim().to_ascii_lowercase())
            });
            if allow_arr.len() != before_len {
                changed = true;
            }
        }

        if let Some(entries_obj) = plugins_obj
            .get_mut("entries")
            .and_then(Value::as_object_mut)
        {
            let before_len = entries_obj.len();
            entries_obj.retain(|key, _| !missing_set.contains(&key.trim().to_ascii_lowercase()));
            if entries_obj.len() != before_len {
                changed = true;
            }
        }

        if changed {
            write_openclaw_config_value(&config_path, &parsed)?;
            changed_paths.push(config_path.display().to_string());
        }
    }

    if changed_paths.is_empty() {
        return Ok(None);
    }

    let mut removed_ids = missing_set.into_iter().collect::<Vec<_>>();
    removed_ids.sort();
    Ok(Some(format!(
        "已清理无效插件引用：{}（{}）。",
        removed_ids.join(", "),
        changed_paths.join(", ")
    )))
}

fn resolve_openclaw_channel_mirror_failure_log_path() -> PathBuf {
    resolve_openclaw_home_path()
        .join("logs")
        .join("channel-mirror-failures.log")
}

fn append_openclaw_channel_mirror_failure_log_blocking(
    payload: OpenClawChannelMirrorFailureLogPayload,
) -> Result<String, String> {
    let log_path = resolve_openclaw_channel_mirror_failure_log_path();
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!("创建频道转发日志目录失败（{}）：{error}", parent.display())
        })?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| {
            format!(
                "打开频道转发日志文件失败（{}）：{error}",
                log_path.display()
            )
        })?;

    let normalized_channel = normalize_channel_identifier(&payload.channel_type);
    let account_id = payload.account_id.trim();
    let target = payload.target.trim();
    let preview_text = payload.message_preview.trim();
    let error_text = payload.error_detail.trim();
    let record = format!(
        "[{}] channel={} account={} target={}\nmessage={}\nerror={}\n\n",
        Utc::now().to_rfc3339(),
        if normalized_channel.is_empty() {
            payload.channel_type.trim()
        } else {
            normalized_channel.as_str()
        },
        if account_id.is_empty() {
            "default"
        } else {
            account_id
        },
        if target.is_empty() { "(empty)" } else { target },
        if preview_text.is_empty() {
            "(empty)"
        } else {
            preview_text
        },
        if error_text.is_empty() {
            "(empty)"
        } else {
            error_text
        }
    );
    file.write_all(record.as_bytes()).map_err(|error| {
        format!(
            "写入频道转发日志文件失败（{}）：{error}",
            log_path.display()
        )
    })?;

    Ok(log_path.to_string_lossy().into_owned())
}

fn run_openclaw_channel_message_send_blocking(
    payload: OpenClawChannelMessageSendPayload,
) -> Result<OpenClawChannelMessageSendResult, String> {
    let started_at = std::time::Instant::now();
    let normalized_channel = normalize_channel_identifier(&payload.channel_type);
    if normalized_channel.is_empty() {
        return Err("channelType 不能为空。".to_string());
    }

    let normalized_account = payload
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();
    let target = payload.target.trim();
    if target.is_empty() {
        return Err("target 不能为空。".to_string());
    }
    let message = payload.message.trim();
    if message.is_empty() {
        return Err("message 不能为空。".to_string());
    }

    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return Err(format!("频道消息发送前自举失败：{error}"));
    }

    let args_owned = vec![
        "message".to_string(),
        "send".to_string(),
        "--channel".to_string(),
        normalized_channel.clone(),
        "--account".to_string(),
        normalized_account.clone(),
        "--target".to_string(),
        target.to_string(),
        "--message".to_string(),
        message.to_string(),
    ];
    let args_ref: Vec<&str> = args_owned.iter().map(String::as_str).collect();
    let (command_display, output) = run_openclaw_cli_output(&args_ref)
        .map_err(|error| format!("频道消息发送失败，无法调用全局 OpenClaw CLI：{error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();
    if !output.status.success() {
        let merged_detail = format!("{}\n{}", stdout.trim(), stderr.trim())
            .trim()
            .to_string();
        let mut base_stdout_sections = Vec::new();
        let mut base_stderr_sections = Vec::new();
        let mut command_sections = vec![command_display.clone()];

        let mut working_stdout = stdout.clone();
        let mut working_stderr = stderr.clone();
        let mut working_merged_detail = merged_detail.clone();

        let should_try_prune_missing_plugins = {
            let lower = merged_detail.to_ascii_lowercase();
            lower.contains("config invalid") && lower.contains("plugin not found")
        };
        if should_try_prune_missing_plugins {
            let missing_plugin_ids = extract_missing_plugin_ids_from_text(&merged_detail);
            if !missing_plugin_ids.is_empty() {
                match prune_missing_plugin_refs_from_openclaw_config(&missing_plugin_ids) {
                    Ok(Some(prune_detail)) => {
                        append_labeled_output_section(
                            &mut base_stdout_sections,
                            "first-attempt",
                            &stdout,
                        );
                        append_labeled_output_section(
                            &mut base_stderr_sections,
                            "first-attempt",
                            &stderr,
                        );
                        append_labeled_output_section(
                            &mut base_stdout_sections,
                            "config-prune",
                            &prune_detail,
                        );
                        command_sections.push(format!(
                            "sanitize openclaw.json plugins ({})",
                            missing_plugin_ids.join(", ")
                        ));

                        let retry_args_ref: Vec<&str> =
                            args_owned.iter().map(String::as_str).collect();
                        let (prune_retry_command, prune_retry_output) =
                            run_openclaw_cli_output(&retry_args_ref).map_err(|error| {
                                format!(
                                    "频道消息发送失败，自动清理插件配置后重试调用全局 OpenClaw CLI 失败：{error}"
                                )
                            })?;
                        let prune_retry_stdout =
                            String::from_utf8_lossy(&prune_retry_output.stdout).to_string();
                        let prune_retry_stderr =
                            String::from_utf8_lossy(&prune_retry_output.stderr).to_string();
                        let prune_retry_merged = format!(
                            "{}\n{}",
                            prune_retry_stdout.trim(),
                            prune_retry_stderr.trim()
                        )
                        .trim()
                        .to_string();
                        command_sections.push(prune_retry_command.clone());

                        working_stdout = prune_retry_stdout;
                        working_stderr = prune_retry_stderr;
                        working_merged_detail = prune_retry_merged;

                        if prune_retry_output.status.success() {
                            let mut stdout_sections = base_stdout_sections;
                            let mut stderr_sections = base_stderr_sections;
                            append_labeled_output_section(
                                &mut stdout_sections,
                                "retry",
                                &working_stdout,
                            );
                            append_labeled_output_section(
                                &mut stderr_sections,
                                "retry",
                                &working_stderr,
                            );
                            let detail = if !working_stdout.trim().is_empty() {
                                trim_remote_error_detail(working_stdout.trim())
                            } else if !working_stderr.trim().is_empty() {
                                trim_remote_error_detail(working_stderr.trim())
                            } else {
                                "频道消息发送成功（已自动清理插件配置并重试）。".to_string()
                            };
                            return Ok(OpenClawChannelMessageSendResult {
                                channel_type: normalized_channel,
                                account_id: normalized_account,
                                target: target.to_string(),
                                command: command_sections.join(" ; "),
                                success: true,
                                detail,
                                exit_code: prune_retry_output.status.code(),
                                stdout: stdout_sections.join("\n\n"),
                                stderr: stderr_sections.join("\n\n"),
                                duration_ms: started_at.elapsed().as_millis(),
                            });
                        }
                    }
                    Ok(None) => {}
                    Err(prune_error) => {
                        return Err(if merged_detail.is_empty() {
                            format!(
                                "频道消息发送失败（{normalized_channel}/{normalized_account}）。自动清理插件配置失败：{prune_error}"
                            )
                        } else {
                            format!(
                                "频道消息发送失败（{normalized_channel}/{normalized_account}）：{}；自动清理插件配置失败：{prune_error}",
                                trim_remote_error_detail(&merged_detail)
                            )
                        });
                    }
                }
            }
        }

        let should_try_repair = normalized_channel == "feishu" && {
            let merged_lower = working_merged_detail.to_ascii_lowercase();
            merged_lower.contains("unknown channel: feishu")
                || merged_lower.contains("unknown channel: openclaw-lark")
                || (merged_lower.contains("openclaw-lark")
                    && merged_lower.contains("failed to load"))
                || merged_lower.contains("root-alias.cjs/channel-status")
        };
        if should_try_repair {
            let repair_result =
                run_openclaw_channel_plugin_install_blocking(normalized_channel.clone());
            if !repair_result.success {
                let repair_detail = trim_remote_error_detail(&repair_result.detail);
                return Err(if working_merged_detail.is_empty() {
                    format!(
                        "频道消息发送失败（{normalized_channel}/{normalized_account}）。自动修复飞书插件失败：{repair_detail}"
                    )
                } else {
                    format!(
                        "频道消息发送失败（{normalized_channel}/{normalized_account}）：{}；自动修复飞书插件失败：{repair_detail}",
                        trim_remote_error_detail(&working_merged_detail)
                    )
                });
            }

            let retry_args_ref: Vec<&str> = args_owned.iter().map(String::as_str).collect();
            let (retry_command, retry_output) =
                run_openclaw_cli_output(&retry_args_ref).map_err(|error| {
                    format!(
                        "频道消息发送失败，自动修复飞书插件后重试调用全局 OpenClaw CLI 失败：{error}"
                    )
                })?;
            let retry_stdout = String::from_utf8_lossy(&retry_output.stdout).to_string();
            let retry_stderr = String::from_utf8_lossy(&retry_output.stderr).to_string();
            let retry_exit_code = retry_output.status.code();
            if retry_output.status.success() {
                let mut stdout_sections = base_stdout_sections;
                let mut stderr_sections = base_stderr_sections;
                append_labeled_output_section(
                    &mut stdout_sections,
                    "before-plugin-repair",
                    &working_stdout,
                );
                append_labeled_output_section(
                    &mut stderr_sections,
                    "before-plugin-repair",
                    &working_stderr,
                );
                append_labeled_output_section(
                    &mut stdout_sections,
                    "plugin-repair",
                    &repair_result.stdout,
                );
                append_labeled_output_section(
                    &mut stderr_sections,
                    "plugin-repair",
                    &repair_result.stderr,
                );
                append_labeled_output_section(&mut stdout_sections, "retry", &retry_stdout);
                append_labeled_output_section(&mut stderr_sections, "retry", &retry_stderr);
                let detail = if !retry_stdout.trim().is_empty() {
                    trim_remote_error_detail(retry_stdout.trim())
                } else if !retry_stderr.trim().is_empty() {
                    trim_remote_error_detail(retry_stderr.trim())
                } else {
                    "频道消息发送成功（已自动修复飞书插件并重试）。".to_string()
                };
                command_sections.push(repair_result.command.clone());
                command_sections.push(retry_command.clone());
                return Ok(OpenClawChannelMessageSendResult {
                    channel_type: normalized_channel,
                    account_id: normalized_account,
                    target: target.to_string(),
                    command: command_sections.join(" ; "),
                    success: true,
                    detail,
                    exit_code: retry_exit_code,
                    stdout: stdout_sections.join("\n\n"),
                    stderr: stderr_sections.join("\n\n"),
                    duration_ms: started_at.elapsed().as_millis(),
                });
            }

            let retry_merged = format!("{}\n{}", retry_stdout.trim(), retry_stderr.trim())
                .trim()
                .to_string();
            return Err(if retry_merged.is_empty() {
                format!(
                    "频道消息发送失败（{normalized_channel}/{normalized_account}）。已自动修复飞书插件并重试，但仍失败。"
                )
            } else {
                format!(
                    "频道消息发送失败（{normalized_channel}/{normalized_account}）：{}；已自动修复飞书插件并重试，但仍失败：{}",
                    if working_merged_detail.is_empty() {
                        "首次失败无详细输出".to_string()
                    } else {
                        trim_remote_error_detail(&working_merged_detail)
                    },
                    trim_remote_error_detail(&retry_merged)
                )
            });
        }

        return Err(if working_merged_detail.is_empty() {
            format!("频道消息发送失败（{normalized_channel}/{normalized_account}）。")
        } else {
            format!(
                "频道消息发送失败（{normalized_channel}/{normalized_account}）：{}",
                trim_remote_error_detail(&working_merged_detail)
            )
        });
    }

    let detail = if !stdout.trim().is_empty() {
        trim_remote_error_detail(stdout.trim())
    } else if !stderr.trim().is_empty() {
        trim_remote_error_detail(stderr.trim())
    } else {
        "频道消息发送成功。".to_string()
    };

    Ok(OpenClawChannelMessageSendResult {
        channel_type: normalized_channel,
        account_id: normalized_account,
        target: target.to_string(),
        command: command_display,
        success: true,
        detail,
        exit_code,
        stdout,
        stderr,
        duration_ms: started_at.elapsed().as_millis(),
    })
}

#[tauri::command]
pub(crate) async fn send_openclaw_channel_message(
    payload: OpenClawChannelMessageSendPayload,
) -> Result<OpenClawChannelMessageSendResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_openclaw_channel_message_send_blocking(payload)
    })
    .await
    .map_err(|error| format!("频道消息发送任务执行失败：{error}"))?
}

#[tauri::command]
pub(crate) async fn append_openclaw_channel_mirror_failure_log(
    payload: OpenClawChannelMirrorFailureLogPayload,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        append_openclaw_channel_mirror_failure_log_blocking(payload)
    })
    .await
    .map_err(|error| format!("频道转发日志写入任务执行失败：{error}"))?
}

fn run_openclaw_channel_plugin_install_blocking(
    channel_type: String,
) -> OpenClawChannelPluginInstallResult {
    let started_at = std::time::Instant::now();
    let normalized_channel = normalize_channel_identifier(&channel_type);
    if normalized_channel.is_empty() {
        return OpenClawChannelPluginInstallResult {
            channel_type: channel_type.trim().to_string(),
            plugin_id: None,
            plugin_spec: None,
            command: String::new(),
            success: false,
            detail: "channelType 不能为空。".to_string(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
        };
    }

    let Some((plugin_id, plugin_spec)) = resolve_channel_plugin_install_spec(&normalized_channel)
    else {
        return OpenClawChannelPluginInstallResult {
            channel_type: normalized_channel,
            plugin_id: None,
            plugin_spec: None,
            command: String::new(),
            success: true,
            detail: "当前频道无需额外安装插件。".to_string(),
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: started_at.elapsed().as_millis(),
        };
    };

    if let Err(error) = bootstrap_openclaw_runtime(false) {
        let command = format!(
            "openclaw plugins install {plugin_spec} && openclaw plugins enable {plugin_id}"
        );
        return OpenClawChannelPluginInstallResult {
            channel_type: normalized_channel,
            plugin_id: Some(plugin_id.to_string()),
            plugin_spec: Some(plugin_spec.to_string()),
            command,
            success: false,
            detail: format!("插件安装前自举失败：{error}"),
            exit_code: None,
            stdout: String::new(),
            stderr: error,
            duration_ms: started_at.elapsed().as_millis(),
        };
    }

    let mut stdout_sections = Vec::new();
    let mut stderr_sections = Vec::new();

    let install_args = ["plugins", "install", plugin_spec];
    let (install_command, mut install_output) = match run_openclaw_cli_output(&install_args) {
        Ok(value) => value,
        Err(error) => {
            let command = format!(
                "openclaw plugins install {plugin_spec} && openclaw plugins enable {plugin_id}"
            );
            return OpenClawChannelPluginInstallResult {
                channel_type: normalized_channel,
                plugin_id: Some(plugin_id.to_string()),
                plugin_spec: Some(plugin_spec.to_string()),
                command,
                success: false,
                detail: "插件安装失败，无法调用全局 OpenClaw CLI。".to_string(),
                exit_code: None,
                stdout: String::new(),
                stderr: error,
                duration_ms: started_at.elapsed().as_millis(),
            };
        }
    };
    let mut install_command_chain = install_command.clone();

    let mut install_stdout = String::from_utf8_lossy(&install_output.stdout).to_string();
    let mut install_stderr = String::from_utf8_lossy(&install_output.stderr).to_string();
    append_labeled_output_section(&mut stdout_sections, "plugin-install", &install_stdout);
    append_labeled_output_section(&mut stderr_sections, "plugin-install", &install_stderr);

    if !install_output.status.success() {
        let merged_detail = format!("{}\n{}", install_stdout.trim(), install_stderr.trim())
            .trim()
            .to_string();
        let already_exists = merged_detail
            .to_ascii_lowercase()
            .contains("plugin already exists");
        if already_exists {
            let uninstall_args = ["plugins", "uninstall", "--force", plugin_id];
            let (uninstall_command, uninstall_output) = match run_openclaw_cli_output(
                &uninstall_args,
            ) {
                Ok(value) => value,
                Err(error) => {
                    return OpenClawChannelPluginInstallResult {
                        channel_type: normalized_channel,
                        plugin_id: Some(plugin_id.to_string()),
                        plugin_spec: Some(plugin_spec.to_string()),
                        command: format!("{install_command} && openclaw plugins uninstall --force {plugin_id}"),
                        success: false,
                        detail: "检测到插件目录已存在，尝试卸载后重装失败（无法调用全局 OpenClaw CLI）。".to_string(),
                        exit_code: install_output.status.code(),
                        stdout: stdout_sections.join("\n\n"),
                        stderr: if stderr_sections.is_empty() {
                            error
                        } else {
                            format!("{}\n\n{error}", stderr_sections.join("\n\n"))
                        },
                        duration_ms: started_at.elapsed().as_millis(),
                    };
                }
            };
            let uninstall_stdout = String::from_utf8_lossy(&uninstall_output.stdout).to_string();
            let uninstall_stderr = String::from_utf8_lossy(&uninstall_output.stderr).to_string();
            append_labeled_output_section(
                &mut stdout_sections,
                "plugin-uninstall",
                &uninstall_stdout,
            );
            append_labeled_output_section(
                &mut stderr_sections,
                "plugin-uninstall",
                &uninstall_stderr,
            );

            if !uninstall_output.status.success() {
                let uninstall_merged =
                    format!("{}\n{}", uninstall_stdout.trim(), uninstall_stderr.trim())
                        .trim()
                        .to_string();
                return OpenClawChannelPluginInstallResult {
                    channel_type: normalized_channel,
                    plugin_id: Some(plugin_id.to_string()),
                    plugin_spec: Some(plugin_spec.to_string()),
                    command: format!("{install_command} ; {uninstall_command}"),
                    success: false,
                    detail: if uninstall_merged.is_empty() {
                        format!(
                            "插件安装失败（{plugin_spec}）：检测到已存在同名插件，自动卸载失败。"
                        )
                    } else {
                        format!(
                            "插件安装失败（{plugin_spec}）：检测到已存在同名插件，自动卸载失败：{uninstall_merged}"
                        )
                    },
                    exit_code: uninstall_output
                        .status
                        .code()
                        .or(install_output.status.code()),
                    stdout: stdout_sections.join("\n\n"),
                    stderr: stderr_sections.join("\n\n"),
                    duration_ms: started_at.elapsed().as_millis(),
                };
            }

            let reinstall_args = ["plugins", "install", plugin_spec];
            let (reinstall_command, reinstall_output) = match run_openclaw_cli_output(
                &reinstall_args,
            ) {
                Ok(value) => value,
                Err(error) => {
                    return OpenClawChannelPluginInstallResult {
                        channel_type: normalized_channel,
                        plugin_id: Some(plugin_id.to_string()),
                        plugin_spec: Some(plugin_spec.to_string()),
                        command: format!("{install_command} ; {uninstall_command} ; openclaw plugins install {plugin_spec}"),
                        success: false,
                        detail: "插件卸载后重装失败，无法调用全局 OpenClaw CLI。".to_string(),
                        exit_code: uninstall_output.status.code().or(install_output.status.code()),
                        stdout: stdout_sections.join("\n\n"),
                        stderr: if stderr_sections.is_empty() {
                            error
                        } else {
                            format!("{}\n\n{error}", stderr_sections.join("\n\n"))
                        },
                        duration_ms: started_at.elapsed().as_millis(),
                    };
                }
            };
            let reinstall_stdout = String::from_utf8_lossy(&reinstall_output.stdout).to_string();
            let reinstall_stderr = String::from_utf8_lossy(&reinstall_output.stderr).to_string();
            append_labeled_output_section(
                &mut stdout_sections,
                "plugin-install-retry",
                &reinstall_stdout,
            );
            append_labeled_output_section(
                &mut stderr_sections,
                "plugin-install-retry",
                &reinstall_stderr,
            );

            install_command_chain =
                format!("{install_command} ; {uninstall_command} ; {reinstall_command}");
            install_output = reinstall_output;
            install_stdout = reinstall_stdout;
            install_stderr = reinstall_stderr;
        }

        if !install_output.status.success() {
            let merged_detail = format!("{}\n{}", install_stdout.trim(), install_stderr.trim())
                .trim()
                .to_string();
            return OpenClawChannelPluginInstallResult {
                channel_type: normalized_channel,
                plugin_id: Some(plugin_id.to_string()),
                plugin_spec: Some(plugin_spec.to_string()),
                command: install_command_chain,
                success: false,
                detail: if merged_detail.is_empty() {
                    format!("插件安装失败（{plugin_spec}）。")
                } else {
                    format!("插件安装失败（{plugin_spec}）：{merged_detail}")
                },
                exit_code: install_output.status.code(),
                stdout: stdout_sections.join("\n\n"),
                stderr: stderr_sections.join("\n\n"),
                duration_ms: started_at.elapsed().as_millis(),
            };
        }
    }

    let enable_args = ["plugins", "enable", plugin_id];
    let (enable_command, enable_output) = match run_openclaw_cli_output(&enable_args) {
        Ok(value) => value,
        Err(error) => {
            let command = format!("{install_command_chain} && openclaw plugins enable {plugin_id}");
            return OpenClawChannelPluginInstallResult {
                channel_type: normalized_channel,
                plugin_id: Some(plugin_id.to_string()),
                plugin_spec: Some(plugin_spec.to_string()),
                command,
                success: false,
                detail: "插件已安装，但启用失败（无法调用全局 OpenClaw CLI）。".to_string(),
                exit_code: install_output.status.code(),
                stdout: stdout_sections.join("\n\n"),
                stderr: if stderr_sections.is_empty() {
                    error
                } else {
                    format!("{}\n\n{error}", stderr_sections.join("\n\n"))
                },
                duration_ms: started_at.elapsed().as_millis(),
            };
        }
    };
    let enable_stdout = String::from_utf8_lossy(&enable_output.stdout).to_string();
    let enable_stderr = String::from_utf8_lossy(&enable_output.stderr).to_string();
    append_labeled_output_section(&mut stdout_sections, "plugin-enable", &enable_stdout);
    append_labeled_output_section(&mut stderr_sections, "plugin-enable", &enable_stderr);

    if !enable_output.status.success() {
        let merged_detail = format!("{}\n{}", enable_stdout.trim(), enable_stderr.trim())
            .trim()
            .to_string();
        return OpenClawChannelPluginInstallResult {
            channel_type: normalized_channel,
            plugin_id: Some(plugin_id.to_string()),
            plugin_spec: Some(plugin_spec.to_string()),
            command: format!("{install_command_chain} && {enable_command}"),
            success: false,
            detail: if merged_detail.is_empty() {
                format!("插件已安装，但启用失败（{plugin_id}）。")
            } else {
                format!("插件已安装，但启用失败（{plugin_id}）：{merged_detail}")
            },
            exit_code: enable_output.status.code().or(install_output.status.code()),
            stdout: stdout_sections.join("\n\n"),
            stderr: stderr_sections.join("\n\n"),
            duration_ms: started_at.elapsed().as_millis(),
        };
    }

    OpenClawChannelPluginInstallResult {
        channel_type: normalized_channel,
        plugin_id: Some(plugin_id.to_string()),
        plugin_spec: Some(plugin_spec.to_string()),
        command: format!("{install_command_chain} && {enable_command}"),
        success: true,
        detail: format!("插件 {plugin_id} 已安装并启用。"),
        exit_code: enable_output
            .status
            .code()
            .or(install_output.status.code())
            .or(Some(0)),
        stdout: stdout_sections.join("\n\n"),
        stderr: stderr_sections.join("\n\n"),
        duration_ms: started_at.elapsed().as_millis(),
    }
}

#[tauri::command]
pub(crate) async fn install_openclaw_channel_plugin(
    channel_type: String,
) -> Result<OpenClawChannelPluginInstallResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_openclaw_channel_plugin_install_blocking(channel_type)
    })
    .await
    .map_err(|error| format!("频道插件安装任务执行失败：{error}"))
}

#[tauri::command]
pub(crate) async fn check_openclaw_gateway(
    endpoint: Option<String>,
) -> Result<GatewayHealthResponse, String> {
    check_openclaw_gateway_internal(endpoint).await
}

pub(crate) async fn check_openclaw_gateway_internal(
    endpoint: Option<String>,
) -> Result<GatewayHealthResponse, String> {
    let gateway_port = Some(resolve_openclaw_gateway_port());
    let endpoint = endpoint
        .filter(|value| !value.trim().is_empty())
        .or_else(resolve_default_openclaw_api_url);

    let Some(endpoint) = endpoint else {
        return Ok(GatewayHealthResponse {
            status: "unconfigured".to_string(),
            checked_url: None,
            detail: Some("未设置 OPENCLAW_API_URL。".to_string()),
            latency_ms: None,
            gateway_port,
        });
    };

    let endpoint = endpoint.trim().to_string();
    let mut candidates = Vec::new();

    if let Ok(mut url) = reqwest::Url::parse(&endpoint) {
        url.set_path("/health");
        url.set_query(None);
        url.set_fragment(None);
        candidates.push(url.to_string());
    }
    candidates.push(endpoint.clone());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|error| format!("创建网关检查客户端失败: {error}"))?;
    let gateway_token = resolve_openclaw_gateway_token();

    let mut last_error = None;

    for candidate in candidates {
        let started_at = std::time::Instant::now();
        let mut request = client.get(&candidate);
        if let Some(token) = gateway_token
            .as_deref()
            .filter(|token| !token.trim().is_empty())
        {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        match request.send().await {
            Ok(response) => {
                let latency_ms = started_at.elapsed().as_millis();
                let status = response.status();
                let detail = if status.is_success() {
                    Some(format!("HTTP {status}"))
                } else {
                    let body = response.text().await.unwrap_or_default();
                    let body = body.trim();
                    if body.is_empty() {
                        Some(format!("HTTP {status}，服务可达"))
                    } else {
                        let truncated = if body.chars().count() > 220 {
                            format!("{}...", body.chars().take(220).collect::<String>())
                        } else {
                            body.to_string()
                        };
                        Some(format!("HTTP {status}：{truncated}"))
                    }
                };

                return Ok(GatewayHealthResponse {
                    status: "online".to_string(),
                    checked_url: Some(candidate),
                    detail,
                    latency_ms: Some(latency_ms),
                    gateway_port,
                });
            }
            Err(error) => {
                last_error = Some(format!("{candidate}: {error}"));
            }
        }
    }

    Ok(GatewayHealthResponse {
        status: "offline".to_string(),
        checked_url: Some(endpoint),
        detail: last_error,
        latency_ms: None,
        gateway_port,
    })
}
