use crate::openclaw::models::*;
use crate::openclaw::utils::generate_ephemeral_openclaw_gateway_token;
use crate::{
    current_timestamp_millis, has_configured_channel_account_in_openclaw_config,
    is_wecom_channel_identifier, is_weixin_channel_identifier, is_whatsapp_channel_identifier,
    normalize_channel_identifier, open_external_url, openclaw_channel_qr_binding_sessions,
    prepend_global_openclaw_cli_to_command_path, resolve_openclaw_config_path,
    resolve_openclaw_gateway_port, resolve_openclaw_gateway_token, resolve_openclaw_home_path,
    run_openclaw_cli_output, run_openclaw_gateway_bootstrap_once,
    sanitize_openclaw_plugin_load_paths, suppress_windows_command_window,
    sync_channel_accounts_from_plugin_store, wait_for_channel_config_sync,
};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub(crate) fn trim_remote_error_detail(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().count() <= 260 {
        return trimmed.to_string();
    }
    format!("{}...", trimmed.chars().take(260).collect::<String>())
}

fn build_openclaw_channel_qr_binding_snapshot(
    state: &OpenClawChannelQrBindingSessionState,
) -> OpenClawChannelQrBindingSessionSnapshot {
    OpenClawChannelQrBindingSessionSnapshot {
        session_id: state.session_id.clone(),
        channel_type: state.channel_type.clone(),
        status: state.status.clone(),
        qr_url: state.qr_url.clone(),
        qr_ascii: state.qr_ascii.clone(),
        detail: state.detail.clone(),
        logs: state.logs.clone(),
        started_at_ms: state.started_at_ms,
        updated_at_ms: state.updated_at_ms,
    }
}

pub(crate) fn is_whatsapp_ascii_qr_line(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return false;
    }

    let mut has_blocks = false;
    for ch in trimmed.chars() {
        match ch {
            '█' | '▀' | '▄' => has_blocks = true,
            ' ' => {}
            _ => return false,
        }
    }

    has_blocks
}

fn finalize_whatsapp_ascii_qr_capture(session: &mut OpenClawChannelQrBindingSessionState) {
    if session.qr_ascii_buffer.len() < 8 {
        session.qr_ascii_collecting = false;
        session.qr_ascii_buffer.clear();
        return;
    }

    session.qr_ascii = Some(session.qr_ascii_buffer.join("\n"));
    session.qr_ascii_collecting = false;
    session.qr_ascii_buffer.clear();
    session.status = "waiting_scan".to_string();
    session.detail = Some("二维码已生成，请使用手机扫码完成绑定。".to_string());
}

pub(crate) fn strip_ansi_escape_sequences(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        output.push(ch);
    }

    output
}

fn extract_http_urls_from_text(raw: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut cursor = 0usize;

    while cursor < raw.len() {
        let remain = &raw[cursor..];
        let next_http = remain.find("http://");
        let next_https = remain.find("https://");
        let Some(relative_start) = (match (next_http, next_https) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(value), None) => Some(value),
            (None, Some(value)) => Some(value),
            (None, None) => None,
        }) else {
            break;
        };

        let start = cursor + relative_start;
        let tail = &raw[start..];
        let relative_end = tail.find(|ch: char| {
            ch.is_whitespace()
                || ch == '"'
                || ch == '\''
                || ch == '<'
                || ch == '>'
                || ch == '，'
                || ch == '。'
        });
        let end = start + relative_end.unwrap_or(tail.len());
        if end <= start {
            break;
        }

        let candidate = raw[start..end].trim_end_matches(|ch: char| {
            ch == ','
                || ch == '.'
                || ch == ';'
                || ch == ')'
                || ch == '('
                || ch == '，'
                || ch == '。'
                || ch == '；'
                || ch == '）'
                || ch == '（'
        });
        if candidate.starts_with("http://") || candidate.starts_with("https://") {
            urls.push(candidate.to_string());
        }

        cursor = end;
    }

    urls
}

fn extract_channel_qr_url_from_line(channel_type: &str, line: &str) -> Option<String> {
    let urls = extract_http_urls_from_text(line);
    if urls.is_empty() {
        return None;
    }

    let normalized = normalize_channel_identifier(channel_type);
    if is_weixin_channel_identifier(&normalized) {
        if let Some(value) = urls
            .iter()
            .find(|url| url.contains("liteapp.weixin.qq.com"))
            .cloned()
        {
            return Some(value);
        }
    } else if is_wecom_channel_identifier(&normalized) {
        if let Some(value) = urls
            .iter()
            .find(|url| url.contains("work.weixin.qq.com/ai/qc/"))
            .cloned()
        {
            return Some(value);
        }
    } else if is_whatsapp_channel_identifier(&normalized) {
        if let Some(value) = urls
            .iter()
            .find(|url| {
                let lower = url.to_ascii_lowercase();
                lower.contains("whatsapp.com") || lower.contains("wa.me")
            })
            .cloned()
        {
            return Some(value);
        }
    }

    urls.into_iter().next()
}

pub(crate) fn update_openclaw_channel_qr_binding_session_from_line(
    session: &mut OpenClawChannelQrBindingSessionState,
    raw_line: &str,
) {
    let cleaned = strip_ansi_escape_sequences(raw_line)
        .replace('\r', "")
        .trim()
        .to_string();
    if cleaned.is_empty() {
        return;
    }

    session.updated_at_ms = current_timestamp_millis();
    session.logs.push(cleaned.clone());
    if session.logs.len() > 120 {
        let extra = session.logs.len() - 120;
        session.logs.drain(0..extra);
    }
    let cleaned_lower = cleaned.to_ascii_lowercase();
    let is_whatsapp = is_whatsapp_channel_identifier(&session.channel_type);

    if is_whatsapp {
        let is_qr_heading = cleaned_lower.contains("scan this qr in whatsapp")
            && cleaned_lower.contains("linked devices");
        if is_qr_heading {
            if session.qr_ascii_collecting && !session.qr_ascii_buffer.is_empty() {
                finalize_whatsapp_ascii_qr_capture(session);
            }
            session.qr_ascii_collecting = true;
            session.qr_ascii_buffer.clear();
        } else if session.qr_ascii_collecting && is_whatsapp_ascii_qr_line(&cleaned) {
            session.qr_ascii_buffer.push(cleaned.clone());
            if session.qr_ascii_buffer.len() >= 12 {
                session.qr_ascii = Some(session.qr_ascii_buffer.join("\n"));
                session.status = "waiting_scan".to_string();
                session.detail = Some("二维码已生成，请使用手机扫码完成绑定。".to_string());
            }
        } else if session.qr_ascii_collecting
            && !cleaned_lower.contains("waiting for whatsapp connection")
        {
            finalize_whatsapp_ascii_qr_capture(session);
        }
    }

    if session.qr_url.is_none() {
        if let Some(url) = extract_channel_qr_url_from_line(&session.channel_type, &cleaned) {
            session.qr_url = Some(url);
            session.status = "waiting_scan".to_string();
            session.detail = Some("二维码已生成，请使用手机扫码完成绑定。".to_string());
        }
    }

    let has_qr_payload = session.qr_url.is_some() || session.qr_ascii.is_some();

    if cleaned.contains("等待扫码")
        || cleaned.contains("等待连接结果")
        || cleaned.contains("请使用微信扫描")
        || cleaned.contains("请使用企业微信扫描")
        || cleaned_lower.contains("waiting for whatsapp connection")
        || cleaned_lower.contains("scan the qr")
        || cleaned_lower.contains("scan qr")
        || cleaned_lower.contains("waiting for scan")
        || cleaned_lower.contains("waiting for qr")
    {
        if session.status == "running" || (is_whatsapp && has_qr_payload) {
            session.status = "waiting_scan".to_string();
        }
    }

    if cleaned.contains("扫码成功")
        || cleaned.contains("连接成功")
        || cleaned.contains("已自动获取")
        || cleaned.contains("与微信连接成功")
        || cleaned_lower.contains("login successful")
        || cleaned_lower.contains("logged in")
    {
        session.detail = Some(cleaned.clone());
    }

    if cleaned.contains("失败") || cleaned_lower.contains("error") {
        session.detail = Some(cleaned);
    }

    if cleaned_lower.contains("unsupported channel: whatsapp") {
        session.status = "error".to_string();
        session.detail = Some(
            "当前 OpenClaw CLI 版本不支持 WhatsApp 频道。请升级到 2026.3.13 或更高版本后重试。"
                .to_string(),
        );
    }
}

fn prune_openclaw_channel_qr_binding_sessions() {
    let now = current_timestamp_millis();
    let mut sessions = match openclaw_channel_qr_binding_sessions().lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    sessions.retain(|_, session_state| {
        let Ok(state) = session_state.lock() else {
            return false;
        };
        let age_ms = now.saturating_sub(state.updated_at_ms);
        let finished = state.status == "success" || state.status == "error";
        if finished {
            age_ms <= 10 * 60 * 1000
        } else {
            age_ms <= 30 * 60 * 1000
        }
    });
}

#[tauri::command]
pub(crate) fn start_openclaw_channel_qr_binding(
    channel_type: String,
) -> Result<OpenClawChannelQrBindingSessionSnapshot, String> {
    let normalized_input = normalize_channel_identifier(&channel_type);
    let normalized_channel = if is_weixin_channel_identifier(&normalized_input) {
        "weixin".to_string()
    } else if is_wecom_channel_identifier(&normalized_input) {
        "wecom".to_string()
    } else if is_whatsapp_channel_identifier(&normalized_input) {
        "whatsapp".to_string()
    } else {
        return Err(
            "当前仅支持微信（weixin）、企业微信（wecom）和 WhatsApp（whatsapp）二维码绑定。"
                .to_string(),
        );
    };

    if let Err(error) = sanitize_openclaw_plugin_load_paths() {
        eprintln!("[dragonclaw] sanitize plugins.load.paths failed before qr binding: {error}");
    }

    prune_openclaw_channel_qr_binding_sessions();

    let session_id = generate_ephemeral_openclaw_gateway_token()
        .unwrap_or_else(|_| format!("qr-{}", current_timestamp_millis()));
    let openclaw_state_dir = resolve_openclaw_home_path().display().to_string();
    let openclaw_config_path = resolve_openclaw_config_path().display().to_string();
    let login_command = match normalized_channel.as_str() {
        "weixin" => {
            "(openclaw channels login --channel openclaw-weixin) || (openclaw channels login --channel weixin)"
                .to_string()
        }
        "wecom" => {
            "(openclaw channels login --channel wecom) || (openclaw channels login --channel openclaw-wecom)"
                .to_string()
        }
        _ => "openclaw channels login --channel whatsapp --verbose".to_string(),
    };
    let install_command = match normalized_channel.as_str() {
        "weixin" => Some("npx -y @tencent-weixin/openclaw-weixin-cli@latest install".to_string()),
        "wecom" => Some(if cfg!(target_os = "windows") {
            // WeCom CLI 默认首项即“扫码接入（推荐）”，通过换行可自动确认。
            "echo.| npx -y @wecom/wecom-openclaw-cli@latest install".to_string()
        } else {
            "printf '\\n' | npx -y @wecom/wecom-openclaw-cli@latest install".to_string()
        }),
        "whatsapp" => Some(
            "openclaw plugins install @openclaw/whatsapp && openclaw plugins enable whatsapp"
                .to_string(),
        ),
        _ => None,
    };
    let command_line = if normalized_channel == "whatsapp" {
        if let Some(install_command) = install_command {
            format!(
                "(({install_command}) || (echo [whatsapp] plugin install fallback)) && ({login_command})"
            )
        } else {
            login_command.clone()
        }
    } else if let Some(install_command) = install_command {
        format!("({login_command}) || (({install_command}) && ({login_command}))")
    } else {
        login_command.clone()
    };

    let now = current_timestamp_millis();
    let session_state = Arc::new(Mutex::new(OpenClawChannelQrBindingSessionState {
        session_id: session_id.clone(),
        channel_type: normalized_channel.clone(),
        status: "running".to_string(),
        qr_url: None,
        qr_ascii: None,
        qr_ascii_collecting: false,
        qr_ascii_buffer: Vec::new(),
        detail: Some(format!(
            "已启动{}绑定流程，正在获取二维码...",
            match normalized_channel.as_str() {
                "weixin" => "微信",
                "wecom" => "企业微信",
                _ => "WhatsApp",
            }
        )),
        logs: vec![
            format!("启动命令：{command_line}"),
            format!("配置路径：{openclaw_config_path}"),
        ],
        started_at_ms: now,
        updated_at_ms: now,
    }));

    {
        let mut sessions = openclaw_channel_qr_binding_sessions()
            .lock()
            .map_err(|_| "无法获取二维码会话状态锁。".to_string())?;
        sessions.insert(session_id.clone(), session_state.clone());
    }

    let session_state_for_thread = session_state.clone();
    thread::spawn(move || {
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            suppress_windows_command_window(&mut cmd);
            cmd.args(["/C", &command_line]);
            cmd
        } else {
            let mut cmd = Command::new("/bin/sh");
            cmd.args(["-lc", &command_line]);
            cmd
        };
        suppress_windows_command_window(&mut command);
        let global_cli_path_prefix = prepend_global_openclaw_cli_to_command_path(&mut command);
        command
            .env("OPENCLAW_NO_RESPAWN", "1")
            .env("OPENCLAW_EMBEDDED_IN", "DragonClaw")
            .env("OPENCLAW_HOME", &openclaw_state_dir)
            .env("OPENCLAW_STATE_DIR", &openclaw_state_dir)
            .env("CLAWDBOT_STATE_DIR", &openclaw_state_dir)
            .env("OPENCLAW_CONFIG_PATH", &openclaw_config_path)
            .env("OPENCLAW_CONFIG", &openclaw_config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Ok(mut state) = session_state_for_thread.lock() {
            state.updated_at_ms = current_timestamp_millis();
            if let Some(path_prefix) = global_cli_path_prefix {
                state.logs.push(format!(
                    "已将全局 OpenClaw CLI 置于 PATH 前缀：{path_prefix}"
                ));
            } else {
                state.logs.push(
                    "未找到全局 OpenClaw CLI 路径，当前将使用系统 PATH 中的 openclaw。".to_string(),
                );
            }
        }

        let mut child = match command.spawn() {
            Ok(value) => value,
            Err(error) => {
                if let Ok(mut state) = session_state_for_thread.lock() {
                    state.status = "error".to_string();
                    state.detail = Some(format!("启动二维码绑定流程失败：{error}"));
                    state.updated_at_ms = current_timestamp_millis();
                }
                return;
            }
        };

        let mut reader_jobs: Vec<JoinHandle<()>> = Vec::new();

        if let Some(stdout) = child.stdout.take() {
            let stdout_state = session_state_for_thread.clone();
            reader_jobs.push(thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    let Ok(content) = line else {
                        continue;
                    };
                    if let Ok(mut state) = stdout_state.lock() {
                        update_openclaw_channel_qr_binding_session_from_line(&mut state, &content);
                    } else {
                        break;
                    }
                }
            }));
        }

        if let Some(stderr) = child.stderr.take() {
            let stderr_state = session_state_for_thread.clone();
            reader_jobs.push(thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    let Ok(content) = line else {
                        continue;
                    };
                    if let Ok(mut state) = stderr_state.lock() {
                        update_openclaw_channel_qr_binding_session_from_line(&mut state, &content);
                    } else {
                        break;
                    }
                }
            }));
        }

        let wait_result = child.wait();
        for job in reader_jobs {
            let _ = job.join();
        }

        if let Ok(mut state) = session_state_for_thread.lock() {
            state.updated_at_ms = current_timestamp_millis();
            if state.qr_ascii_collecting && !state.qr_ascii_buffer.is_empty() {
                finalize_whatsapp_ascii_qr_capture(&mut state);
            }
            let config_sync_result = wait_for_channel_config_sync(&state.channel_type);
            match wait_result {
                Ok(status) => {
                    let code_text = status
                        .code()
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    match config_sync_result {
                        Ok(true) => {
                            state.status = "success".to_string();
                            if !status.success() {
                                state.logs.push(format!(
                                    "命令退出码 {code_text}，但已检测到频道配置写入。"
                                ));
                            }
                            if state.detail.is_none() {
                                state.detail =
                                    Some("绑定流程已完成，并检测到频道配置。".to_string());
                            }
                        }
                        Ok(false) => {
                            state.status = "error".to_string();
                            let saw_whatsapp_install_prompt = state.channel_type == "whatsapp"
                                && state.logs.iter().any(|line| {
                                    let lowered = line.to_ascii_lowercase();
                                    lowered.contains("install whatsapp plugin")
                                        || lowered.contains("use local plugin path")
                                        || lowered.contains("@openclaw/whatsapp")
                                });
                            if saw_whatsapp_install_prompt {
                                state.detail = Some(format!(
                                    "扫码流程结束（exit code: {code_text}），检测到 WhatsApp 插件未完成安装。请先执行 `openclaw plugins install @openclaw/whatsapp && openclaw plugins enable whatsapp`，再点击“重新获取二维码”。"
                                ));
                            } else {
                                state.detail = Some(format!(
                                    "扫码流程结束（exit code: {code_text}），但未在 openclaw 配置中检测到 {} 频道可用账号。请点击“重新获取二维码”重试；如仍失败，请检查 CLI 输出日志。",
                                    match state.channel_type.as_str() {
                                        "weixin" => "微信",
                                        "wecom" => "企业微信",
                                        _ => "WhatsApp",
                                    }
                                ));
                            }
                        }
                        Err(error) => {
                            state.status = "error".to_string();
                            state.detail = Some(format!("绑定流程结束后校验频道配置失败：{error}"));
                        }
                    }
                }
                Err(error) => {
                    state.status = "error".to_string();
                    state.detail = Some(format!("等待绑定流程结束失败：{error}"));
                }
            }
        }
    });

    let state = session_state
        .lock()
        .map_err(|_| "无法读取二维码会话状态。".to_string())?;
    Ok(build_openclaw_channel_qr_binding_snapshot(&state))
}

#[tauri::command]
pub(crate) fn poll_openclaw_channel_qr_binding(
    session_id: String,
) -> Result<OpenClawChannelQrBindingSessionSnapshot, String> {
    prune_openclaw_channel_qr_binding_sessions();

    let normalized_id = session_id.trim().to_string();
    if normalized_id.is_empty() {
        return Err("sessionId 不能为空。".to_string());
    }

    let state_handle = {
        let sessions = openclaw_channel_qr_binding_sessions()
            .lock()
            .map_err(|_| "无法获取二维码会话状态锁。".to_string())?;
        sessions
            .get(&normalized_id)
            .cloned()
            .ok_or_else(|| format!("未找到二维码会话：{normalized_id}"))?
    };

    let mut state = state_handle
        .lock()
        .map_err(|_| "二维码会话状态读取失败。".to_string())?;
    let normalized_status = state.status.trim().to_ascii_lowercase();
    if normalized_status != "success" && normalized_status != "error" {
        let _ = sync_channel_accounts_from_plugin_store(&state.channel_type);
        if has_configured_channel_account_in_openclaw_config(&state.channel_type)? {
            state.status = "success".to_string();
            state.detail = Some("二维码绑定成功，已检测到频道配置。".to_string());
            state.updated_at_ms = current_timestamp_millis();
        }
    }
    Ok(build_openclaw_channel_qr_binding_snapshot(&state))
}

#[tauri::command]
pub(crate) fn clear_openclaw_channel_qr_binding_session(session_id: String) -> Result<(), String> {
    let normalized_id = session_id.trim().to_string();
    if normalized_id.is_empty() {
        return Ok(());
    }
    let mut sessions = openclaw_channel_qr_binding_sessions()
        .lock()
        .map_err(|_| "无法获取二维码会话状态锁。".to_string())?;
    sessions.remove(&normalized_id);
    Ok(())
}

#[tauri::command]
pub(crate) async fn request_feishu_openclaw_qr() -> Result<FeishuOnboardingQrResponse, String> {
    let endpoint = "https://accounts.feishu.cn/oauth/v1/app/registration";
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| format!("创建飞书连接客户端失败：{error}"))?;

    let init_response = client
        .post(endpoint)
        .form(&[("action", "init")])
        .send()
        .await
        .map_err(|error| format!("请求飞书创建会话失败（init）：{error}"))?;
    let init_status = init_response.status();
    let init_body = init_response
        .text()
        .await
        .map_err(|error| format!("读取飞书创建会话响应失败（init）：{error}"))?;
    if !init_status.is_success() {
        let detail = trim_remote_error_detail(&init_body);
        return Err(if detail.is_empty() {
            format!("请求飞书创建会话失败（init，HTTP {init_status}）。")
        } else {
            format!("请求飞书创建会话失败（init，HTTP {init_status}）：{detail}")
        });
    }
    let init_payload = serde_json::from_str::<FeishuOnboardingInitResponse>(&init_body)
        .map_err(|error| format!("解析飞书创建会话响应失败（init）：{error}"))?;
    let supports_client_secret = init_payload
        .supported_auth_methods
        .iter()
        .any(|item| item.trim().eq_ignore_ascii_case("client_secret"));
    if !supports_client_secret {
        return Err("当前飞书环境暂不支持 client_secret 授权，请升级插件后重试。".to_string());
    }

    let begin_response = client
        .post(endpoint)
        .form(&[
            ("action", "begin"),
            ("archetype", "PersonalAgent"),
            ("auth_method", "client_secret"),
            ("request_user_info", "open_id"),
        ])
        .send()
        .await
        .map_err(|error| format!("请求飞书创建码失败（begin）：{error}"))?;
    let begin_status = begin_response.status();
    let begin_body = begin_response
        .text()
        .await
        .map_err(|error| format!("读取飞书创建码响应失败（begin）：{error}"))?;
    if !begin_status.is_success() {
        let detail = trim_remote_error_detail(&begin_body);
        return Err(if detail.is_empty() {
            format!("请求飞书创建码失败（begin，HTTP {begin_status}）。")
        } else {
            format!("请求飞书创建码失败（begin，HTTP {begin_status}）：{detail}")
        });
    }
    let begin_payload = serde_json::from_str::<FeishuOnboardingBeginResponse>(&begin_body)
        .map_err(|error| format!("解析飞书创建码响应失败（begin）：{error}"))?;

    let raw_qr_url = begin_payload
        .verification_uri_complete
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "飞书返回的二维码链接为空，请稍后重试。".to_string())?;
    let user_code = begin_payload
        .user_code
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "飞书返回的创建码为空，请稍后重试。".to_string())?;
    let device_code = begin_payload
        .device_code
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "飞书返回的设备码为空，请稍后重试。".to_string())?;

    let expires_in_seconds = begin_payload.expire_in.unwrap_or(600).clamp(60, 3600);
    let poll_interval_seconds = begin_payload.interval.unwrap_or(5).clamp(1, 30);
    let expires_at_ms =
        current_timestamp_millis() + u128::from(expires_in_seconds).saturating_mul(1000);

    let qr_url = if let Ok(mut parsed) = reqwest::Url::parse(&raw_qr_url) {
        parsed.query_pairs_mut().append_pair("from", "onboard");
        parsed.to_string()
    } else {
        raw_qr_url
    };

    Ok(FeishuOnboardingQrResponse {
        qr_url,
        user_code,
        device_code,
        poll_interval_seconds,
        expires_in_seconds,
        expires_at_ms,
    })
}

fn map_feishu_poll_error_to_response(
    error_code_raw: &str,
    error_description: Option<String>,
    tenant_brand: Option<String>,
) -> FeishuOnboardingPollResponse {
    let normalized = error_code_raw.trim().to_ascii_lowercase();
    let message = error_description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let (status, fallback_message) = match normalized.as_str() {
        "authorization_pending" => ("pending", "飞书侧尚未完成授权，请完成扫码后再检查。"),
        "slow_down" => ("pending", "请求过于频繁，请稍后几秒再次检查。"),
        "access_denied" => ("denied", "你已拒绝授权，请重新获取创建码并扫码。"),
        "expired_token" | "invalid_grant" => ("expired", "创建码已过期，请重新获取创建码。"),
        _ => ("error", "飞书返回了异常状态，请稍后重试。"),
    };

    FeishuOnboardingPollResponse {
        status: status.to_string(),
        message: Some(
            message
                .unwrap_or_else(|| fallback_message.to_string())
                .trim()
                .to_string(),
        ),
        app_id: None,
        app_secret: None,
        tenant_brand,
    }
}

async fn poll_feishu_registration_once(
    client: &reqwest::Client,
    endpoint: &str,
    device_code: &str,
) -> Result<FeishuOnboardingPollRawResponse, String> {
    let response = client
        .post(endpoint)
        .form(&[("action", "poll"), ("device_code", device_code)])
        .send()
        .await
        .map_err(|error| format!("请求飞书创建结果失败（poll）：{error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("读取飞书创建结果失败（poll）：{error}"))?;
    if !status.is_success() {
        let detail = trim_remote_error_detail(&body);
        return Err(if detail.is_empty() {
            format!("请求飞书创建结果失败（poll，HTTP {status}）。")
        } else {
            format!("请求飞书创建结果失败（poll，HTTP {status}）：{detail}")
        });
    }
    serde_json::from_str::<FeishuOnboardingPollRawResponse>(&body)
        .map_err(|error| format!("解析飞书创建结果失败（poll）：{error}"))
}

#[tauri::command]
pub(crate) async fn poll_feishu_openclaw_qr_result(
    device_code: String,
) -> Result<FeishuOnboardingPollResponse, String> {
    let normalized_device_code = device_code.trim().to_string();
    if normalized_device_code.is_empty() {
        return Err("设备码为空，请重新获取创建码。".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| format!("创建飞书查询客户端失败：{error}"))?;

    let mut poll_payload = poll_feishu_registration_once(
        &client,
        "https://accounts.feishu.cn/oauth/v1/app/registration",
        &normalized_device_code,
    )
    .await?;

    let tenant_brand = poll_payload
        .user_info
        .as_ref()
        .and_then(|info| info.tenant_brand.as_ref())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());

    // Lark 租户需要切换到 accounts.larksuite.com 再拉取一次结果。
    if tenant_brand.as_deref() == Some("lark")
        && poll_payload
            .error
            .as_deref()
            .map(str::trim)
            .map(|value| value.eq_ignore_ascii_case("authorization_pending"))
            .unwrap_or(false)
    {
        poll_payload = poll_feishu_registration_once(
            &client,
            "https://accounts.larksuite.com/oauth/v1/app/registration",
            &normalized_device_code,
        )
        .await?;
    }

    let app_id = poll_payload
        .client_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let app_secret = poll_payload
        .client_secret
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if let (Some(app_id), Some(app_secret)) = (app_id, app_secret) {
        return Ok(FeishuOnboardingPollResponse {
            status: "success".to_string(),
            message: Some("已获取飞书凭证。".to_string()),
            app_id: Some(app_id),
            app_secret: Some(app_secret),
            tenant_brand,
        });
    }

    if let Some(error_code) = poll_payload.error {
        let error_description = poll_payload
            .error_description
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        return Ok(map_feishu_poll_error_to_response(
            &error_code,
            error_description,
            tenant_brand,
        ));
    }

    Ok(FeishuOnboardingPollResponse {
        status: "pending".to_string(),
        message: Some("飞书侧尚未完成授权，请完成扫码后再检查。".to_string()),
        app_id: None,
        app_secret: None,
        tenant_brand,
    })
}

fn build_openclaw_control_ui_url_fallback() -> String {
    let port = resolve_openclaw_gateway_port();
    let token = resolve_openclaw_gateway_token()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut url = format!("http://127.0.0.1:{port}/");
    if let Some(token) = token {
        url.push_str(&format!("#token={token}"));
    }
    url
}

pub(crate) fn extract_dashboard_url_from_text(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let candidate = if let Some(value) = trimmed.strip_prefix("Dashboard URL:") {
            value.trim()
        } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed
        } else {
            continue;
        };
        if let Ok(url) = reqwest::Url::parse(candidate) {
            if matches!(url.scheme(), "http" | "https") {
                return Some(url.to_string());
            }
        }
    }
    None
}

fn resolve_openclaw_dashboard_url() -> Result<String, String> {
    let (command_display, output) = run_openclaw_cli_output(&["dashboard", "--no-open"])?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let detail = format!("{}\n{}", stdout.trim(), stderr.trim())
            .trim()
            .to_string();
        return Err(if detail.is_empty() {
            format!("OpenClaw dashboard 命令执行失败（{command_display}）。")
        } else {
            format!("OpenClaw dashboard 命令执行失败（{command_display}）：{detail}")
        });
    }

    let merged = format!("{}\n{}", stdout, stderr);
    extract_dashboard_url_from_text(&merged)
        .ok_or_else(|| format!("OpenClaw dashboard 输出未包含可解析 URL（{command_display}）。"))
}

#[tauri::command]
pub(crate) fn build_openclaw_control_ui_url() -> Result<String, String> {
    Ok(build_openclaw_control_ui_url_fallback())
}

fn open_openclaw_control_ui_blocking() -> Result<String, String> {
    let gateway_bootstrap = run_openclaw_gateway_bootstrap_once();
    if !gateway_bootstrap.success {
        return Err(format!(
            "网关未就绪，无法打开 OpenClaw 控制台：{}",
            gateway_bootstrap.detail
        ));
    }

    let url = match resolve_openclaw_dashboard_url() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[dragonclaw] resolve dashboard url failed: {error}");
            build_openclaw_control_ui_url_fallback()
        }
    };
    open_external_url(url.clone())?;
    Ok(url)
}

#[tauri::command]
pub(crate) async fn open_openclaw_control_ui() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(open_openclaw_control_ui_blocking)
        .await
        .map_err(|error| format!("打开 OpenClaw 控制台任务执行失败：{error}"))?
}

#[cfg(test)]
mod tests {
    use super::{
        extract_dashboard_url_from_text, is_whatsapp_ascii_qr_line,
        update_openclaw_channel_qr_binding_session_from_line,
    };
    use crate::openclaw::models::OpenClawChannelQrBindingSessionState;

    #[test]
    fn extracts_dashboard_url_from_cli_output() {
        let raw = "Dashboard URL: http://127.0.0.1:18789/#token=abc123\nBrowser launch disabled.";
        let parsed = extract_dashboard_url_from_text(raw);
        assert_eq!(
            parsed.as_deref(),
            Some("http://127.0.0.1:18789/#token=abc123")
        );
    }

    #[test]
    fn detects_whatsapp_ascii_qr_line() {
        assert!(is_whatsapp_ascii_qr_line("██▀▀▄█"));
        assert!(is_whatsapp_ascii_qr_line(" ▄▄▄ █ "));
        assert!(!is_whatsapp_ascii_qr_line(
            "Waiting for WhatsApp connection..."
        ));
    }

    #[test]
    fn captures_whatsapp_ascii_qr_from_cli_stream() {
        let mut session = OpenClawChannelQrBindingSessionState {
            session_id: "qr-test".to_string(),
            channel_type: "whatsapp".to_string(),
            status: "running".to_string(),
            qr_url: None,
            qr_ascii: None,
            qr_ascii_collecting: false,
            qr_ascii_buffer: Vec::new(),
            detail: None,
            logs: Vec::new(),
            started_at_ms: 0,
            updated_at_ms: 0,
        };

        update_openclaw_channel_qr_binding_session_from_line(
            &mut session,
            "Scan this QR in WhatsApp (Linked Devices):",
        );
        for _ in 0..12 {
            update_openclaw_channel_qr_binding_session_from_line(&mut session, "██▀▀▄█");
        }

        assert_eq!(session.status, "waiting_scan");
        assert!(session.qr_ascii.is_some());
        assert!(session
            .qr_ascii
            .as_deref()
            .unwrap_or_default()
            .contains("██▀▀▄█"));
    }
}
