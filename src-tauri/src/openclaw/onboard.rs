#[cfg(target_os = "windows")]
use serde_json::Value;

pub(crate) struct OfficialOnboardOutcome {
    pub(crate) success: bool,
    pub(crate) degraded: bool,
    pub(crate) command: String,
    pub(crate) detail: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

struct OfficialOnboardCommandRun {
    command_redacted: String,
    output: std::process::Output,
    strategy_note: Option<String>,
}

fn build_official_onboard_command_redacted(
    package_spec: Option<&str>,
    gateway_port: &str,
    workspace: &str,
) -> String {
    let _ = package_spec;
    let core = format!(
        "onboard --non-interactive --accept-risk --mode local --flow quickstart --auth-choice skip --gateway-auth token --gateway-token *** --gateway-port {} --gateway-bind loopback --install-daemon --workspace {} --json",
        gateway_port, workspace
    );
    format!("openclaw {}", core)
}

fn run_official_onboard_command_with_latest_priority(
    args: &[String],
    gateway_port: &str,
    workspace: &str,
) -> Result<OfficialOnboardCommandRun, String> {
    let command_redacted = build_official_onboard_command_redacted(None, gateway_port, workspace);
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (_command_display, output) = super::runtime::run_openclaw_cli_output(&arg_refs)?;
    Ok(OfficialOnboardCommandRun {
        command_redacted,
        output,
        strategy_note: Some("使用全局 OpenClaw CLI 执行安装。".to_string()),
    })
}

#[cfg(target_os = "windows")]
fn extract_last_onboard_json(stdout: &str, stderr: &str) -> Option<Value> {
    super::json::extract_last_json_object_from_streams(stdout, stderr)
}

fn is_windows_daemon_install_soft_failure(stdout: &str, stderr: &str, detail: &str) -> bool {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (stdout, stderr, detail);
        false
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(payload) = extract_last_onboard_json(stdout, stderr) {
            let phase = payload
                .get("phase")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            let daemon_requested = payload
                .get("daemonInstall")
                .and_then(Value::as_object)
                .and_then(|item| item.get("requested"))
                .and_then(Value::as_bool)
                .unwrap_or_else(|| {
                    payload
                        .get("installDaemon")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                });
            let daemon_installed = payload
                .get("daemonInstall")
                .and_then(Value::as_object)
                .and_then(|item| item.get("installed"))
                .and_then(Value::as_bool)
                .unwrap_or(true);
            if phase.eq_ignore_ascii_case("daemon-install") && daemon_requested && !daemon_installed
            {
                return true;
            }
        }

        let lowered = format!("{detail}\n{stdout}\n{stderr}").to_ascii_lowercase();
        let has_daemon_phase = lowered.contains("phase\":\"daemon-install\"")
            || lowered.contains("phase\": \"daemon-install\"")
            || lowered.contains("gateway service install did not complete successfully");
        let daemon_not_installed = lowered.contains("\"installed\":false")
            || lowered.contains("\"installed\": false")
            || lowered.contains("gateway service install failed")
            || lowered.contains("daemon install failed")
            || lowered.contains("schtasks create failed");
        has_daemon_phase && daemon_not_installed
    }
}

pub(crate) fn run_openclaw_official_silent_onboard_once() -> OfficialOnboardOutcome {
    let token = match crate::resolve_openclaw_gateway_token_for_onboard() {
        Ok(value) => value,
        Err(error) => {
            return OfficialOnboardOutcome {
                success: false,
                degraded: false,
                command: "openclaw onboard --non-interactive ...".to_string(),
                detail: "官方静默安装失败，无法准备 gateway token。".to_string(),
                exit_code: None,
                stdout: String::new(),
                stderr: error,
            };
        }
    };
    let preferred_gateway_port = super::gateway::resolve_openclaw_gateway_port();
    let gateway_port = match super::gateway::find_available_loopback_port(preferred_gateway_port) {
        Some(port) => port,
        None => {
            return OfficialOnboardOutcome {
                success: false,
                degraded: false,
                command: "openclaw onboard --non-interactive ...".to_string(),
                detail: format!(
                    "官方静默安装失败：网关端口 {} 已被占用，且未找到可用替代端口（+64 范围）。",
                    preferred_gateway_port
                ),
                exit_code: None,
                stdout: String::new(),
                stderr: "gateway port unavailable".to_string(),
            };
        }
    };
    let port_switch_note = if gateway_port != preferred_gateway_port {
        std::env::set_var("OPENCLAW_GATEWAY_PORT", gateway_port.to_string());
        std::env::set_var(
            "OPENCLAW_API_URL",
            format!("http://127.0.0.1:{gateway_port}/v1/chat/completions"),
        );
        Some(format!(
            "检测到默认端口 {} 被占用，已自动切换到 {}。",
            preferred_gateway_port, gateway_port
        ))
    } else {
        None
    };
    let workspace = crate::resolve_workspace_main_root();
    if let Err(error) = std::fs::create_dir_all(&workspace) {
        return OfficialOnboardOutcome {
            success: false,
            degraded: false,
            command: "openclaw onboard --non-interactive ...".to_string(),
            detail: format!(
                "官方静默安装失败，创建工作区目录失败（{}）。",
                workspace.display()
            ),
            exit_code: None,
            stdout: String::new(),
            stderr: error.to_string(),
        };
    }

    let gateway_port_string = gateway_port.to_string();
    let workspace_string = workspace.display().to_string();
    let args = vec![
        "onboard".to_string(),
        "--non-interactive".to_string(),
        "--accept-risk".to_string(),
        "--mode".to_string(),
        "local".to_string(),
        "--flow".to_string(),
        "quickstart".to_string(),
        "--auth-choice".to_string(),
        "skip".to_string(),
        "--gateway-auth".to_string(),
        "token".to_string(),
        "--gateway-token".to_string(),
        token.clone(),
        "--gateway-port".to_string(),
        gateway_port_string.clone(),
        "--gateway-bind".to_string(),
        "loopback".to_string(),
        "--install-daemon".to_string(),
        "--workspace".to_string(),
        workspace_string.clone(),
        "--json".to_string(),
    ];
    let command_run = match run_official_onboard_command_with_latest_priority(
        &args,
        &gateway_port_string,
        &workspace_string,
    ) {
        Ok(value) => value,
        Err(error) => {
            return OfficialOnboardOutcome {
                success: false,
                degraded: false,
                command: build_official_onboard_command_redacted(
                    None,
                    &gateway_port_string,
                    &workspace_string,
                ),
                detail: "官方静默安装失败，无法调用 OpenClaw CLI。".to_string(),
                exit_code: None,
                stdout: String::new(),
                stderr: error,
            };
        }
    };
    let command_redacted = command_run.command_redacted;
    let strategy_note = command_run.strategy_note;
    let output = command_run.output;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        let mut detail = "已按官方 Onboard 流程完成静默安装配置。".to_string();
        if let Some(note) = strategy_note.as_deref() {
            detail.push(' ');
            detail.push_str(note);
        }
        if let Some(note) = port_switch_note {
            detail.push(' ');
            detail.push_str(&note);
        }
        OfficialOnboardOutcome {
            success: true,
            degraded: false,
            command: command_redacted,
            detail,
            exit_code: output.status.code(),
            stdout,
            stderr,
        }
    } else {
        let merged_detail = format!("{}\n{}", stdout.trim(), stderr.trim())
            .trim()
            .to_string();
        let soft_failure = is_windows_daemon_install_soft_failure(&stdout, &stderr, &merged_detail);
        if soft_failure {
            let soft_note = "守护进程安装失败，已降级为用户级登录启动项（无需管理员权限）。";
            let mut detail = if merged_detail.is_empty() {
                format!("已按官方 Onboard 流程完成核心配置。{soft_note}")
            } else {
                format!("已按官方 Onboard 流程完成核心配置。{soft_note} 详情：{merged_detail}")
            };
            if let Some(note) = strategy_note.as_deref() {
                detail.push(' ');
                detail.push_str(note);
            }
            if let Some(note) = port_switch_note {
                detail.push(' ');
                detail.push_str(&note);
            }
            return OfficialOnboardOutcome {
                success: true,
                degraded: true,
                command: command_redacted,
                detail,
                exit_code: output.status.code(),
                stdout,
                stderr,
            };
        }

        OfficialOnboardOutcome {
            success: false,
            degraded: false,
            command: command_redacted,
            detail: if merged_detail.is_empty() {
                let base = format!(
                    "官方静默安装失败（exit: {}）。",
                    output.status.code().unwrap_or(-1)
                );
                let with_strategy = if let Some(note) = strategy_note.as_deref() {
                    format!("{base} {note}")
                } else {
                    base
                };
                if let Some(note) = port_switch_note {
                    format!("{with_strategy} {note}")
                } else {
                    with_strategy
                }
            } else if let Some(strategy) = strategy_note.as_deref() {
                if let Some(note) = port_switch_note {
                    format!("官方静默安装失败：{merged_detail} {strategy} {note}")
                } else {
                    format!("官方静默安装失败：{merged_detail} {strategy}")
                }
            } else if let Some(note) = port_switch_note {
                format!("官方静默安装失败：{merged_detail} {note}")
            } else {
                format!("官方静默安装失败：{merged_detail}")
            },
            exit_code: output.status.code(),
            stdout,
            stderr,
        }
    }
}
