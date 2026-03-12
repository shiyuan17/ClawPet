use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenClawMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenClawRequest {
    messages: Vec<OpenClawMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenClawHttpResponse {
    text: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiMessage {
    content: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiChoice {
    message: Option<OpenAiMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenAiChatResponse {
    choices: Option<Vec<OpenAiChoice>>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct OpenClawResponse {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GatewayHealthResponse {
    status: String,
    checked_url: Option<String>,
    detail: Option<String>,
    latency_ms: Option<u128>,
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    messages: Vec<OpenClawMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AnthropicResponse {
    content: Option<Vec<AnthropicContentBlock>>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LocalProxyPlatform {
    protocol: String,
    base_url: String,
    path_prefix: String,
    api_key: String,
}

#[derive(Default)]
struct LocalProxyState {
    stop_signal: Option<Arc<AtomicBool>>,
    handle: Option<JoinHandle<()>>,
}

static LOCAL_PROXY_STATE: OnceLock<Mutex<LocalProxyState>> = OnceLock::new();

fn local_proxy_state() -> &'static Mutex<LocalProxyState> {
    LOCAL_PROXY_STATE.get_or_init(|| Mutex::new(LocalProxyState::default()))
}

fn load_env_file(path: &Path) {
    if path.exists() {
        let _ = dotenvy::from_path(path);
    }
}

fn load_openclaw_env() {
    if let Ok(current_dir) = std::env::current_dir() {
        load_env_file(&current_dir.join(".env"));
        load_env_file(&current_dir.join("../.env"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    load_env_file(&manifest_dir.join(".env"));
    if let Some(workspace_dir) = manifest_dir.parent() {
        load_env_file(&workspace_dir.join(".env"));
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            load_env_file(&exe_dir.join(".env"));
            load_env_file(&exe_dir.join("../.env"));
        }
    }
}

fn is_openai_compatible_endpoint(endpoint: &str) -> bool {
    endpoint
        .trim_end_matches('/')
        .ends_with("/v1/chat/completions")
}

fn extract_openai_content(content: &serde_json::Value) -> Option<String> {
    match content {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Array(items) => {
            let text = items
                .iter()
                .filter_map(|item| item.get("text").and_then(|value| value.as_str()))
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}

fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        return "/platform".to_string();
    }

    let normalized = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    normalized.trim_end_matches('/').to_string()
}

fn parse_request_path(first_line: &str) -> Option<(String, String)> {
    let mut parts = first_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    Some((method, path))
}

fn find_header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.clone())
}

fn has_header(headers: &[(String, String)], name: &str) -> bool {
    headers.iter().any(|(key, _)| key.eq_ignore_ascii_case(name))
}

fn build_cors_headers(request_headers: &[(String, String)]) -> Vec<(String, String)> {
    let origin = find_header_value(request_headers, "origin").unwrap_or_else(|| "*".to_string());
    let allow_headers = find_header_value(request_headers, "access-control-request-headers")
        .unwrap_or_else(|| "content-type, authorization, x-api-key, anthropic-version, accept".to_string());

    vec![
        ("Access-Control-Allow-Origin".to_string(), origin),
        ("Access-Control-Allow-Headers".to_string(), allow_headers),
        (
            "Access-Control-Allow-Methods".to_string(),
            "GET, POST, PUT, PATCH, DELETE, OPTIONS".to_string(),
        ),
        (
            "Vary".to_string(),
            "Origin, Access-Control-Request-Headers".to_string(),
        ),
    ]
}

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn read_http_request(stream: &mut std::net::TcpStream) -> Result<(String, String, Vec<(String, String)>, Vec<u8>), String> {
    let mut buffer = Vec::new();
    let mut header_end = None;
    let mut chunk = [0_u8; 4096];

    while header_end.is_none() {
        let bytes = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes]);
        if let Some(position) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            header_end = Some(position + 4);
        }
        if buffer.len() > 1024 * 1024 {
            return Err("请求头过大".to_string());
        }
    }

    let header_end = header_end.ok_or_else(|| "无法解析请求头".to_string())?;
    let header_text = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = header_text.split("\r\n").filter(|line| !line.is_empty());
    let first_line = lines.next().ok_or_else(|| "缺少请求行".to_string())?;
    let (method, path) = parse_request_path(first_line).ok_or_else(|| "无效请求行".to_string())?;

    let mut headers = Vec::new();
    let mut content_length = 0_usize;
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            let key = name.trim().to_string();
            let normalized = value.trim().to_string();
            if key.eq_ignore_ascii_case("content-length") {
                content_length = normalized.parse::<usize>().unwrap_or(0);
            }
            headers.push((key, normalized));
        }
    }

    let mut body = buffer[header_end..].to_vec();
    while body.len() < content_length {
        let bytes = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..bytes]);
    }

    body.truncate(content_length);
    Ok((method, path, headers, body))
}

fn write_http_response(
    stream: &mut std::net::TcpStream,
    status: u16,
    content_type: Option<&str>,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        _ => "OK",
    };

    let mut header_lines = format!("HTTP/1.1 {status} {reason}\r\n");
    let mut has_content_type = false;

    for (key, value) in headers {
        if key.eq_ignore_ascii_case("content-length") || key.eq_ignore_ascii_case("connection") {
            continue;
        }
        if key.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        header_lines.push_str(&format!("{key}: {value}\r\n"));
    }

    if !has_content_type {
        header_lines.push_str(&format!(
            "Content-Type: {}\r\n",
            content_type.unwrap_or("application/json")
        ));
    }

    header_lines.push_str(&format!("Content-Length: {}\r\n", body.len()));
    header_lines.push_str("Connection: close\r\n\r\n");

    stream.write_all(header_lines.as_bytes()).map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn find_platform_by_path<'a>(platforms: &'a [LocalProxyPlatform], path: &str) -> Option<&'a LocalProxyPlatform> {
    platforms
        .iter()
        .filter(|platform| path.starts_with(&normalize_prefix(&platform.path_prefix)))
        .max_by_key(|platform| platform.path_prefix.len())
}

fn proxy_single_request(
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    platforms: Arc<Vec<LocalProxyPlatform>>,
) -> Result<(u16, String, Vec<(String, String)>, Vec<u8>), String> {
    if method.eq_ignore_ascii_case("OPTIONS") {
        return Ok((204, "application/json".to_string(), Vec::new(), Vec::new()));
    }

    let platform = match find_platform_by_path(&platforms, &path) {
        Some(platform) => platform,
        None => {
            let available_prefixes = platforms
                .iter()
                .map(|platform| platform.path_prefix.clone())
                .collect::<Vec<_>>();
            let payload = serde_json::json!({
                "error": "Platform not found",
                "path": path,
                "availablePrefixes": available_prefixes
            });
            return Ok((
                404,
                "application/json".to_string(),
                Vec::new(),
                serde_json::to_vec(&payload).map_err(|error| error.to_string())?,
            ));
        }
    };
    let prefix = normalize_prefix(&platform.path_prefix);
    let actual_path = path.strip_prefix(&prefix).unwrap_or("/");
    let target_url = format!("{}{}", platform.base_url.trim_end_matches('/'), if actual_path.is_empty() { "/" } else { actual_path });
    let protocol = platform.protocol.to_lowercase();
    let api_key = platform.api_key.clone();
    let has_authorization = has_header(&headers, "authorization");
    let has_x_api_key = has_header(&headers, "x-api-key");
    let has_anthropic_version = has_header(&headers, "anthropic-version");

    tauri::async_runtime::block_on(async move {
        let client = reqwest::Client::new();
        let method_value =
            reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| format!("无效请求方法: {error}"))?;
        let mut request = client.request(method_value, target_url);

        for (key, value) in headers {
            let lower = key.to_ascii_lowercase();
            if matches!(lower.as_str(), "host" | "content-length" | "connection" | "origin") {
                continue;
            }
            request = request.header(&key, value);
        }

        if protocol == "anthropic" {
            if !has_x_api_key && !api_key.trim().is_empty() {
                request = request.header("x-api-key", api_key);
            }
            if !has_anthropic_version {
                request = request.header("anthropic-version", "2023-06-01");
            }
        } else if !has_authorization && !api_key.trim().is_empty() {
            request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
        }

        let response = request
            .body(body)
            .send()
            .await
            .map_err(|error| format!("代理请求失败: {error}"))?;

        let status = response.status().as_u16();
        let response_headers = response
            .headers()
            .iter()
            .filter_map(|(key, value)| {
                if matches!(
                    key.as_str().to_ascii_lowercase().as_str(),
                    "content-length" | "connection" | "transfer-encoding"
                ) {
                    return None;
                }
                value
                    .to_str()
                    .ok()
                    .map(|parsed| (key.to_string(), parsed.to_string()))
            })
            .collect::<Vec<_>>();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/json")
            .to_string();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("读取代理响应失败: {error}"))?
            .to_vec();

        Ok((status, content_type, response_headers, bytes))
    })
}

fn run_local_proxy(listener: TcpListener, stop: Arc<AtomicBool>, platforms: Arc<Vec<LocalProxyPlatform>>) {
    let _ = listener.set_nonblocking(true);

    while !stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let platforms = Arc::clone(&platforms);
                thread::spawn(move || {
                    let result = (|| -> Result<(), String> {
                        let (method, path, headers, body) = read_http_request(&mut stream)?;
                        let cors_headers = build_cors_headers(&headers);

                        if method.eq_ignore_ascii_case("GET") && path == "/health" {
                            let payload = serde_json::json!({
                                "status": "ok",
                                "timestamp": current_timestamp_millis(),
                                "platforms": platforms
                                    .iter()
                                    .map(|platform| serde_json::json!({
                                        "pathPrefix": platform.path_prefix,
                                        "baseUrl": platform.base_url,
                                        "protocol": platform.protocol
                                    }))
                                    .collect::<Vec<_>>()
                            });
                            let body = serde_json::to_vec(&payload).map_err(|error| error.to_string())?;
                            write_http_response(&mut stream, 200, Some("application/json"), &cors_headers, &body)?;
                            return Ok(());
                        }

                        let (status, content_type, mut response_headers, response_body) =
                            proxy_single_request(method, path, headers, body, platforms)?;
                        response_headers.extend(cors_headers);
                        write_http_response(&mut stream, status, Some(&content_type), &response_headers, &response_body)?;
                        Ok(())
                    })();

                    if let Err(error) = result {
                        let fallback = format!(r#"{{"error":"{error}"}}"#);
                        let _ = write_http_response(
                            &mut stream,
                            502,
                            Some("application/json"),
                            &[("Access-Control-Allow-Origin".to_string(), "*".to_string())],
                            fallback.as_bytes(),
                        );
                    }
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(40));
            }
            Err(_) => break,
        }
    }
}

#[tauri::command]
fn sync_local_proxy(port: u16, platforms: Vec<LocalProxyPlatform>) -> Result<(), String> {
    let state_mutex = local_proxy_state();
    let mut state = state_mutex
        .lock()
        .map_err(|_| "无法获取本地代理状态锁".to_string())?;

    if let Some(stop) = state.stop_signal.take() {
        stop.store(true, Ordering::SeqCst);
    }
    if let Some(handle) = state.handle.take() {
        let _ = handle.join();
    }

    if platforms.is_empty() {
        return Ok(());
    }

    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| format!("无法启动本地代理，端口 {port} 可能已被占用: {error}"))?;
    let stop_signal = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop_signal);
    let thread_platforms = Arc::new(platforms);
    let handle = thread::spawn(move || run_local_proxy(listener, thread_stop, thread_platforms));

    state.stop_signal = Some(stop_signal);
    state.handle = Some(handle);
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
async fn openclaw_chat(
    messages: Vec<OpenClawMessage>,
    endpoint: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    protocol: Option<String>,
) -> Result<OpenClawResponse, String> {
    let endpoint = endpoint
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("OPENCLAW_API_URL").ok())
        .ok_or_else(|| "未设置可用的聊天接口地址。".to_string())?;
    let request_protocol = protocol.unwrap_or_else(|| "openai".to_string()).to_lowercase();
    let is_openai_compatible = is_openai_compatible_endpoint(&endpoint);
    let gateway_token = std::env::var("OPENCLAW_GATEWAY_TOKEN").ok();
    let api_key = api_key
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("OPENCLAW_API_KEY").ok());
    let model = model
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("OPENCLAW_MODEL").ok());

    let client = reqwest::Client::new();
    let mut request = client
        .post(endpoint)
        .header(CONTENT_TYPE, "application/json");

    if request_protocol == "anthropic" {
        if let Some(api_key) = api_key.as_deref().filter(|value| !value.trim().is_empty()) {
            request = request.header("x-api-key", api_key);
        }
        request = request.header("anthropic-version", "2023-06-01");
    } else if let Some(token) = gateway_token.as_deref().filter(|token| !token.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    } else if let Some(api_key) = api_key.as_deref().filter(|api_key| !api_key.trim().is_empty()) {
        request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
    }

    request = if request_protocol == "anthropic" {
        let model = model.ok_or_else(|| "Anthropic 协议需要模型配置。".to_string())?;
        let system = messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        let anthropic_messages = messages
            .into_iter()
            .filter(|message| message.role != "system")
            .map(|message| AnthropicMessage {
                role: if message.role == "assistant" {
                    "assistant".to_string()
                } else {
                    "user".to_string()
                },
                content: message.content,
            })
            .collect::<Vec<_>>();

        request.json(&AnthropicRequest {
            model,
            max_tokens: 1024,
            system: if system.is_empty() { None } else { Some(system) },
            messages: anthropic_messages,
        })
    } else if is_openai_compatible {
        request.json(&OpenAiChatRequest { model, messages })
    } else {
        request.json(&OpenClawRequest { messages })
    };

    let response = request
        .send()
        .await
        .map_err(|error| format!("请求 OpenClaw 失败: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "未返回错误详情".to_string());
        return Err(format!("OpenClaw 返回错误状态 {status}: {body}"));
    }

    let (text, raw, usage) = if request_protocol == "anthropic" {
        let payload = response
            .json::<AnthropicResponse>()
            .await
            .map_err(|error| format!("解析 Anthropic 响应失败: {error}"))?;

        let text = payload
            .content
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| item.block_type.as_deref() == Some("text"))
            .filter_map(|item| item.text)
            .collect::<Vec<_>>()
            .join("\n");
        let raw = serde_json::to_string_pretty(&payload).ok();
        let usage = payload
            .usage
            .as_ref()
            .and_then(|value| serde_json::to_value(value).ok());

        (text, raw, usage)
    } else if is_openai_compatible {
        let payload = response
            .json::<OpenAiChatResponse>()
            .await
            .map_err(|error| format!("解析 OpenClaw Gateway 响应失败: {error}"))?;

        let text = payload
            .choices
            .clone()
            .and_then(|choices| choices.into_iter().next())
            .and_then(|choice| choice.message)
            .and_then(|message| message.content)
            .and_then(|content| extract_openai_content(&content))
            .unwrap_or_else(|| "OpenClaw Gateway 返回了空内容。".to_string());
        let raw = serde_json::to_string_pretty(&payload).ok();
        let usage = payload
            .usage
            .as_ref()
            .and_then(|value| serde_json::to_value(value).ok());

        (text, raw, usage)
    } else {
        let payload = response
            .json::<OpenClawHttpResponse>()
            .await
            .map_err(|error| format!("解析 OpenClaw 响应失败: {error}"))?;

        let text = payload
            .text
            .clone()
            .or(payload.content.clone())
            .unwrap_or_else(|| "OpenClaw 返回了空内容。".to_string());
        let raw = serde_json::to_string_pretty(&payload).ok();

        (text, raw, None)
    };

    Ok(OpenClawResponse {
        text: if text.is_empty() {
            "接口返回了空内容。".to_string()
        } else {
            text
        },
        raw,
        usage,
    })
}

#[tauri::command]
async fn check_openclaw_gateway(endpoint: Option<String>) -> Result<GatewayHealthResponse, String> {
    let endpoint = endpoint
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("OPENCLAW_API_URL").ok());

    let Some(endpoint) = endpoint else {
        return Ok(GatewayHealthResponse {
            status: "unconfigured".to_string(),
            checked_url: None,
            detail: Some("未设置 OPENCLAW_API_URL。".to_string()),
            latency_ms: None,
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

    let mut last_error = None;

    for candidate in candidates {
        let started_at = std::time::Instant::now();
        match client.get(&candidate).send().await {
            Ok(response) => {
                let latency_ms = started_at.elapsed().as_millis();
                let status = response.status();
                let detail = if status.is_success() {
                    Some(format!("HTTP {status}"))
                } else {
                    Some(format!("HTTP {status}，服务可达"))
                };

                return Ok(GatewayHealthResponse {
                    status: "online".to_string(),
                    checked_url: Some(candidate),
                    detail,
                    latency_ms: Some(latency_ms),
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
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_openclaw_env();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![quit_app, openclaw_chat, sync_local_proxy, check_openclaw_gateway])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_always_on_top(true);
                let _ = window.set_shadow(false);
                let _ = window.set_skip_taskbar(false);
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let size = monitor.size();
                    let position = monitor.position();
                    let _ = window.set_position(tauri::Position::Physical(*position));
                    let _ = window.set_size(tauri::Size::Physical(*size));
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
