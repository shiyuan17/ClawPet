use serde_json::Value;
use std::path::Path;
use std::time::Duration;

fn is_local_proxy_host(url: &reqwest::Url) -> bool {
    matches!(url.host_str(), Some("localhost") | Some("127.0.0.1"))
}

fn is_openai_compatible_endpoint(endpoint: &str) -> bool {
    let normalized = endpoint.trim_end_matches('/').to_ascii_lowercase();
    normalized.ends_with("/v1/chat/completions") || normalized.ends_with("/chat/completions")
}

fn trim_remote_error_detail(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().count() <= 260 {
        return trimmed.to_string();
    }
    format!("{}...", trimmed.chars().take(260).collect::<String>())
}

pub(crate) struct GatewayBootstrapOutcome {
    pub(crate) success: bool,
    pub(crate) command: String,
    pub(crate) detail: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) struct GatewayDaemonEnsureOutcome {
    pub(crate) success: bool,
    pub(crate) command: String,
    pub(crate) detail: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

fn load_openclaw_gateway_port_from_path(config_path: &Path) -> Option<u16> {
    let raw = std::fs::read_to_string(config_path).ok()?;
    let parsed = serde_json::from_str::<Value>(&raw).ok()?;
    let port_value = parsed
        .get("gateway")
        .and_then(Value::as_object)
        .and_then(|gateway| gateway.get("port"))?;

    if let Some(port_u64) = port_value.as_u64() {
        return u16::try_from(port_u64).ok().filter(|port| *port > 0);
    }
    if let Some(port_str) = port_value.as_str() {
        return port_str.trim().parse::<u16>().ok().filter(|port| *port > 0);
    }
    None
}

fn load_openclaw_gateway_port_from_config() -> Option<u16> {
    let primary_path = crate::resolve_openclaw_config_path();
    if let Some(port) = load_openclaw_gateway_port_from_path(&primary_path) {
        return Some(port);
    }

    let fallback_path = crate::resolve_default_openclaw_config_path();
    if fallback_path != primary_path {
        return load_openclaw_gateway_port_from_path(&fallback_path);
    }

    None
}

fn parse_local_openclaw_api_url(raw: &str) -> Option<reqwest::Url> {
    let url = reqwest::Url::parse(raw).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if host != "127.0.0.1" && host != "localhost" && host != "::1" {
        return None;
    }
    Some(url)
}

pub(crate) fn normalize_local_openclaw_chat_endpoint(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let Ok(mut url) = reqwest::Url::parse(trimmed) else {
        return trimmed.to_string();
    };
    if !is_local_proxy_host(&url) {
        return trimmed.to_string();
    }

    let normalized_path = url.path().trim_end_matches('/').to_ascii_lowercase();
    if normalized_path.is_empty() || normalized_path == "/" || normalized_path == "/v1" {
        url.set_path("/v1/chat/completions");
    } else if normalized_path == "/chat/completions" {
        url.set_path("/v1/chat/completions");
    }

    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

pub(crate) fn should_try_enable_chat_completions_endpoint(endpoint: &str) -> bool {
    if !is_openai_compatible_endpoint(endpoint) {
        return false;
    }
    let Some(url) = parse_local_openclaw_api_url(endpoint) else {
        return false;
    };
    let Some(port) = url.port_or_known_default() else {
        return false;
    };
    port == resolve_openclaw_gateway_port()
}

pub(crate) fn resolve_default_openclaw_api_url() -> Option<String> {
    let configured_port = load_openclaw_gateway_port_from_config();
    let env_url = std::env::var("OPENCLAW_API_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(url_raw) = env_url {
        if let (Some(port), Some(mut local_url)) =
            (configured_port, parse_local_openclaw_api_url(&url_raw))
        {
            let _ = local_url.set_port(Some(port));
            return Some(normalize_local_openclaw_chat_endpoint(
                &local_url.to_string(),
            ));
        }
        return Some(normalize_local_openclaw_chat_endpoint(&url_raw));
    }

    configured_port
        .map(|port| format!("http://127.0.0.1:{port}/v1/chat/completions"))
        .map(|url| normalize_local_openclaw_chat_endpoint(&url))
}

pub(crate) fn resolve_openclaw_gateway_port() -> u16 {
    if let Ok(raw) = std::env::var("OPENCLAW_GATEWAY_PORT") {
        if let Ok(port) = raw.trim().parse::<u16>() {
            if port > 0 {
                return port;
            }
        }
    }

    if let Some(port) = load_openclaw_gateway_port_from_config() {
        return port;
    }

    std::env::var("OPENCLAW_API_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .and_then(|value| parse_local_openclaw_api_url(&value))
        .and_then(|url| url.port_or_known_default())
        .unwrap_or(18789)
}

pub(crate) fn find_available_loopback_port(preferred_port: u16) -> Option<u16> {
    if !is_loopback_port_listening(preferred_port) {
        return Some(preferred_port);
    }

    for offset in 1..=64u16 {
        if let Some(candidate) = preferred_port.checked_add(offset) {
            if !is_loopback_port_listening(candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn summarize_gateway_bootstrap_failure(stderr: &str, port: u16) -> String {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("eaddrinuse")
        || lower.contains("address already in use")
        || lower.contains("port is already in use")
    {
        return format!(
            "网关初始化失败：检测到端口 {port} 已被占用（常见于 ClawX 或其他 OpenClaw 实例）。请关闭占用进程后重试。"
        );
    }
    "网关初始化失败，请检查全局 OpenClaw CLI 运行环境与配置。".to_string()
}

fn is_loopback_port_listening(port: u16) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok()
}

pub(crate) fn wait_for_loopback_port_listening(
    port: u16,
    attempts: usize,
    interval_ms: u64,
) -> bool {
    for _ in 0..attempts {
        if is_loopback_port_listening(port) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(interval_ms));
    }
    false
}

fn gateway_cli_output_text(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{stdout}\n{stderr}")
}

pub(crate) fn gateway_cli_text_indicates_non_effective_success(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    lowered.contains("\"ok\": false")
        || lowered.contains("\"ok\":false")
        || lowered.contains("\"result\": \"not-loaded\"")
        || lowered.contains("\"result\":\"not-loaded\"")
        || lowered.contains("\"loaded\": false")
        || lowered.contains("\"loaded\":false")
        || lowered.contains("\"status\": \"stopped\"")
        || lowered.contains("\"status\":\"stopped\"")
        || lowered.contains("\"state\": \"stopped\"")
        || lowered.contains("\"state\":\"stopped\"")
        || lowered.contains("gateway service not loaded")
        || lowered.contains("could not find service")
}

pub(crate) fn gateway_cli_step_effective_success(output: &std::process::Output) -> bool {
    if !output.status.success() {
        return false;
    }
    !gateway_cli_text_indicates_non_effective_success(&gateway_cli_output_text(output))
}

pub(crate) fn append_labeled_output_section(target: &mut Vec<String>, label: &str, text: &str) {
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        target.push(format!("[{label}]\n{trimmed}"));
    }
}

pub(crate) fn gateway_status_payload_indicates_running(payload: &Value) -> bool {
    if let Some(ok) = payload.get("ok").and_then(Value::as_bool) {
        if !ok {
            return false;
        }
    }

    let service = payload.get("service").and_then(Value::as_object);
    let loaded = service
        .and_then(|item| item.get("loaded"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !loaded {
        return false;
    }

    if let Some(runtime) = service
        .and_then(|item| item.get("runtime"))
        .and_then(Value::as_object)
    {
        if runtime
            .get("running")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return true;
        }

        for key in ["status", "state"] {
            if let Some(value) = runtime.get(key).and_then(Value::as_str) {
                let normalized = value.trim().to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "running" | "started" | "online" | "active" | "ready"
                ) {
                    return true;
                }
            }
        }
    }

    if let Some(result) = payload.get("result").and_then(Value::as_str) {
        let normalized = result.trim().to_ascii_lowercase();
        if matches!(
            normalized.as_str(),
            "running" | "started" | "online" | "active" | "ready"
        ) {
            return true;
        }
    }

    false
}

pub(crate) fn gateway_status_payload_summary(payload: &Value) -> String {
    let loaded = payload
        .get("service")
        .and_then(Value::as_object)
        .and_then(|item| item.get("loaded"))
        .and_then(Value::as_bool);
    let runtime_running = payload
        .get("service")
        .and_then(Value::as_object)
        .and_then(|item| item.get("runtime"))
        .and_then(Value::as_object)
        .and_then(|runtime| runtime.get("running"))
        .and_then(Value::as_bool);
    let runtime_state = payload
        .get("service")
        .and_then(Value::as_object)
        .and_then(|item| item.get("runtime"))
        .and_then(Value::as_object)
        .and_then(|runtime| runtime.get("state"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let runtime_status = payload
        .get("service")
        .and_then(Value::as_object)
        .and_then(|item| item.get("runtime"))
        .and_then(Value::as_object)
        .and_then(|runtime| runtime.get("status"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let result = payload
        .get("result")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");

    format!(
        "service.loaded={:?}, runtime.running={:?}, runtime.state={}, runtime.status={}, result={}",
        loaded, runtime_running, runtime_state, runtime_status, result
    )
}

pub(crate) fn gateway_status_text_indicates_running(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    let loaded = lowered.contains("\"loaded\": true")
        || lowered.contains("\"loaded\":true")
        || lowered.contains("service loaded")
        || lowered.contains("service: loaded");
    let running = lowered.contains("\"status\": \"running\"")
        || lowered.contains("\"status\":\"running\"")
        || lowered.contains("\"state\": \"running\"")
        || lowered.contains("\"state\":\"running\"")
        || lowered.contains("runtime status: running")
        || lowered.contains("runtime state: running");
    loaded && running
}

pub(crate) fn is_gateway_health_probe_online(probe: &crate::GatewayHealthResponse) -> bool {
    probe.status.trim().eq_ignore_ascii_case("online")
}

pub(crate) fn summarize_gateway_health_probe(
    probe: Option<&crate::GatewayHealthResponse>,
) -> String {
    let Some(probe) = probe else {
        return "gateway_probe=skipped".to_string();
    };

    let url = probe.checked_url.as_deref().unwrap_or("-");
    let detail = probe.detail.as_deref().unwrap_or("-");
    format!(
        "gateway_probe_status={}, gateway_probe_url={}, gateway_probe_detail={}",
        probe.status.trim(),
        url,
        trim_remote_error_detail(detail)
    )
}

pub(crate) fn check_openclaw_gateway_health_fallback_blocking(
) -> Option<crate::GatewayHealthResponse> {
    tauri::async_runtime::block_on(async {
        crate::check_openclaw_gateway_internal(None).await.ok()
    })
}

pub(crate) fn run_openclaw_gateway_daemon_ensure_once() -> GatewayDaemonEnsureOutcome {
    let gateway_port = resolve_openclaw_gateway_port();
    let gateway_port_string = gateway_port.to_string();

    let install_args = vec![
        "gateway".to_string(),
        "install".to_string(),
        "--runtime".to_string(),
        "node".to_string(),
        "--port".to_string(),
        gateway_port_string,
        "--json".to_string(),
    ];
    let install_refs = install_args.iter().map(String::as_str).collect::<Vec<_>>();
    let (install_command, install_output) =
        match super::runtime::run_openclaw_cli_output(&install_refs) {
            Ok(value) => value,
            Err(error) => {
                return GatewayDaemonEnsureOutcome {
                    success: false,
                    command: "openclaw gateway install --runtime node --port <port> --json"
                        .to_string(),
                    detail: "后台守护进程安装失败，无法调用全局 OpenClaw CLI。".to_string(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: error,
                };
            }
        };
    let install_stdout = String::from_utf8_lossy(&install_output.stdout).to_string();
    let install_stderr = String::from_utf8_lossy(&install_output.stderr).to_string();
    let mut stdout_sections = Vec::new();
    let mut stderr_sections = Vec::new();
    append_labeled_output_section(&mut stdout_sections, "gateway-install", &install_stdout);
    append_labeled_output_section(&mut stderr_sections, "gateway-install", &install_stderr);

    if !gateway_cli_step_effective_success(&install_output) {
        let merged_detail = format!("{}\n{}", install_stdout.trim(), install_stderr.trim())
            .trim()
            .to_string();
        return GatewayDaemonEnsureOutcome {
            success: false,
            command: install_command,
            detail: if merged_detail.is_empty() {
                "后台守护进程安装失败。".to_string()
            } else {
                format!("后台守护进程安装失败：{merged_detail}")
            },
            exit_code: install_output.status.code(),
            stdout: stdout_sections.join("\n\n"),
            stderr: stderr_sections.join("\n\n"),
        };
    }

    let start_args = vec![
        "gateway".to_string(),
        "start".to_string(),
        "--json".to_string(),
    ];
    let start_refs = start_args.iter().map(String::as_str).collect::<Vec<_>>();
    let (start_command, start_output) = match super::runtime::run_openclaw_cli_output(&start_refs) {
        Ok(value) => value,
        Err(error) => {
            return GatewayDaemonEnsureOutcome {
                success: false,
                command: format!("{install_command} && openclaw gateway start --json"),
                detail: "后台守护进程启动失败，无法调用全局 OpenClaw CLI。".to_string(),
                exit_code: None,
                stdout: stdout_sections.join("\n\n"),
                stderr: if stderr_sections.is_empty() {
                    error
                } else {
                    format!("{}\n\n{error}", stderr_sections.join("\n\n"))
                },
            };
        }
    };
    let start_stdout = String::from_utf8_lossy(&start_output.stdout).to_string();
    let start_stderr = String::from_utf8_lossy(&start_output.stderr).to_string();
    append_labeled_output_section(&mut stdout_sections, "gateway-start", &start_stdout);
    append_labeled_output_section(&mut stderr_sections, "gateway-start", &start_stderr);

    if !gateway_cli_step_effective_success(&start_output) {
        let merged_detail = format!("{}\n{}", start_stdout.trim(), start_stderr.trim())
            .trim()
            .to_string();
        return GatewayDaemonEnsureOutcome {
            success: false,
            command: format!("{install_command} && {start_command}"),
            detail: if merged_detail.is_empty() {
                "后台守护进程启动失败。".to_string()
            } else {
                format!("后台守护进程启动失败：{merged_detail}")
            },
            exit_code: start_output.status.code().or(install_output.status.code()),
            stdout: stdout_sections.join("\n\n"),
            stderr: stderr_sections.join("\n\n"),
        };
    }

    let status_args = vec![
        "gateway".to_string(),
        "status".to_string(),
        "--json".to_string(),
    ];
    let status_refs = status_args.iter().map(String::as_str).collect::<Vec<_>>();
    let (status_command, status_output) =
        match super::runtime::run_openclaw_cli_output(&status_refs) {
            Ok(value) => value,
            Err(error) => {
                return GatewayDaemonEnsureOutcome {
                    success: false,
                    command: format!(
                        "{install_command} && {start_command} && openclaw gateway status --json"
                    ),
                    detail: "后台守护进程状态检查失败，无法调用全局 OpenClaw CLI。".to_string(),
                    exit_code: None,
                    stdout: stdout_sections.join("\n\n"),
                    stderr: if stderr_sections.is_empty() {
                        error
                    } else {
                        format!("{}\n\n{error}", stderr_sections.join("\n\n"))
                    },
                };
            }
        };
    let status_stdout = String::from_utf8_lossy(&status_output.stdout).to_string();
    let status_stderr = String::from_utf8_lossy(&status_output.stderr).to_string();
    append_labeled_output_section(&mut stdout_sections, "gateway-status", &status_stdout);
    append_labeled_output_section(&mut stderr_sections, "gateway-status", &status_stderr);

    let status_effective = gateway_cli_step_effective_success(&status_output);
    let merged_status_text = format!("{}\n{}", status_stdout, status_stderr);
    let status_payload =
        super::json::extract_last_json_object_from_streams(&status_stdout, &status_stderr);
    let status_running = if let Some(payload) = status_payload.as_ref() {
        gateway_status_payload_indicates_running(payload)
    } else {
        gateway_status_text_indicates_running(&merged_status_text)
    };
    let port_ready = wait_for_loopback_port_listening(gateway_port, 36, 250);
    let gateway_probe = if !status_effective || !status_running || !port_ready {
        check_openclaw_gateway_health_fallback_blocking()
    } else {
        None
    };
    let gateway_probe_online = gateway_probe
        .as_ref()
        .map(is_gateway_health_probe_online)
        .unwrap_or(false);
    let fallback_online = gateway_probe_online;
    let effective_success = status_effective || status_running || fallback_online;
    let effective_running = status_running || fallback_online;
    let effective_ready = port_ready || fallback_online || status_running;
    let status_summary = status_payload
        .as_ref()
        .map(gateway_status_payload_summary)
        .unwrap_or_else(|| "未解析到 JSON 状态对象".to_string());
    let probe_summary = summarize_gateway_health_probe(gateway_probe.as_ref());

    if !effective_success || !effective_running || !effective_ready {
        let status_detail = format!(
            "status_effective={status_effective}, status_running={status_running}, port_ready={port_ready}, fallback_online={fallback_online}, {status_summary}, {probe_summary}"
        );
        let merged_detail = format!("{}\n{}", status_stdout.trim(), status_stderr.trim())
            .trim()
            .to_string();
        return GatewayDaemonEnsureOutcome {
            success: false,
            command: format!("{install_command} && {start_command} && {status_command}"),
            detail: if merged_detail.is_empty() {
                format!("后台守护进程状态检查未通过：{status_detail}")
            } else {
                format!("后台守护进程状态检查未通过：{status_detail}。详情：{merged_detail}")
            },
            exit_code: status_output
                .status
                .code()
                .or(start_output.status.code())
                .or(install_output.status.code()),
            stdout: stdout_sections.join("\n\n"),
            stderr: stderr_sections.join("\n\n"),
        };
    }

    GatewayDaemonEnsureOutcome {
        success: true,
        command: format!("{install_command} && {start_command} && {status_command}"),
        detail: format!(
            "后台守护进程已安装并运行（status_effective={status_effective}, status_running={status_running}, port_ready={port_ready}, fallback_online={fallback_online}, {status_summary}, {probe_summary}）。"
        ),
        exit_code: status_output
            .status
            .code()
            .or(start_output.status.code())
            .or(install_output.status.code())
            .or(Some(0)),
        stdout: stdout_sections.join("\n\n"),
        stderr: stderr_sections.join("\n\n"),
    }
}

pub(crate) fn run_openclaw_gateway_bootstrap_once() -> GatewayBootstrapOutcome {
    let gateway_port = resolve_openclaw_gateway_port();

    let (command_display, output) =
        match super::runtime::run_openclaw_cli_output(&["gateway", "restart", "--json"]) {
            Ok(value) => value,
            Err(error) => {
                return GatewayBootstrapOutcome {
                    success: false,
                    command: "openclaw gateway restart".to_string(),
                    detail: "网关初始化失败，未能调用全局 OpenClaw CLI。".to_string(),
                    exit_code: None,
                    stdout: String::new(),
                    stderr: error,
                };
            }
        };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let port_ready = wait_for_loopback_port_listening(gateway_port, 12, 250);

    if port_ready {
        let effective = gateway_cli_step_effective_success(&output);
        return GatewayBootstrapOutcome {
            success: true,
            command: command_display,
            detail: if effective {
                "网关初始化完成。".to_string()
            } else {
                "网关端口已就绪（服务状态输出异常，按可用处理）。".to_string()
            },
            exit_code: output.status.code(),
            stdout,
            stderr,
        };
    }

    let merged_error_text = format!("{}\n{}", stdout.trim(), stderr.trim())
        .trim()
        .to_string();
    GatewayBootstrapOutcome {
        success: false,
        command: command_display,
        detail: if merged_error_text.is_empty() {
            summarize_gateway_bootstrap_failure("", gateway_port)
        } else {
            format!(
                "{} 详情：{}",
                summarize_gateway_bootstrap_failure(&merged_error_text, gateway_port),
                merged_error_text
            )
        },
        exit_code: output.status.code(),
        stdout,
        stderr,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        gateway_cli_text_indicates_non_effective_success, gateway_status_payload_indicates_running,
    };

    #[test]
    fn detects_gateway_not_loaded_text() {
        let raw = r#"{
  "action": "restart",
  "ok": true,
  "result": "not-loaded",
  "message": "Gateway service not loaded."
}"#;
        assert!(gateway_cli_text_indicates_non_effective_success(raw));
    }

    #[test]
    fn healthy_gateway_text_not_flagged() {
        let raw = r#"{
  "action": "start",
  "ok": true,
  "result": "started"
}"#;
        assert!(!gateway_cli_text_indicates_non_effective_success(raw));
    }

    #[test]
    fn gateway_status_payload_running_when_loaded_and_running() {
        let payload = serde_json::json!({
            "ok": true,
            "service": {
                "loaded": true,
                "runtime": {
                    "status": "running"
                }
            }
        });
        assert!(gateway_status_payload_indicates_running(&payload));
    }

    #[test]
    fn gateway_status_payload_not_running_when_not_loaded() {
        let payload = serde_json::json!({
            "ok": true,
            "service": {
                "loaded": false,
                "runtime": {
                    "status": "running"
                }
            }
        });
        assert!(!gateway_status_payload_indicates_running(&payload));
    }
}
