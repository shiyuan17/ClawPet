use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalProxyPlatform {
    protocol: String,
    base_url: String,
    path_prefix: String,
    api_key: String,
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
    headers
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}

fn build_cors_headers(request_headers: &[(String, String)]) -> Vec<(String, String)> {
    let origin = find_header_value(request_headers, "origin").unwrap_or_else(|| "*".to_string());
    let allow_headers = find_header_value(request_headers, "access-control-request-headers")
        .unwrap_or_else(|| {
            "content-type, authorization, x-api-key, anthropic-version, accept".to_string()
        });

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

fn read_http_request(
    stream: &mut std::net::TcpStream,
) -> Result<(String, String, Vec<(String, String)>, Vec<u8>), String> {
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

    stream
        .write_all(header_lines.as_bytes())
        .map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn find_platform_by_path<'a>(
    platforms: &'a [LocalProxyPlatform],
    path: &str,
) -> Option<&'a LocalProxyPlatform> {
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
    let target_url = format!(
        "{}{}",
        platform.base_url.trim_end_matches('/'),
        if actual_path.is_empty() {
            "/"
        } else {
            actual_path
        }
    );
    let protocol = platform.protocol.to_lowercase();
    let api_key = platform.api_key.clone();
    let has_authorization = has_header(&headers, "authorization");
    let has_x_api_key = has_header(&headers, "x-api-key");
    let has_anthropic_version = has_header(&headers, "anthropic-version");
    let has_openclaw_scopes = has_header(&headers, "x-openclaw-scopes");
    let should_set_openclaw_scopes = protocol != "anthropic"
        && super::gateway::should_try_enable_chat_completions_endpoint(&target_url);

    tauri::async_runtime::block_on(async move {
        let client = reqwest::Client::new();
        let method_value = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|error| format!("无效请求方法: {error}"))?;
        let mut request = client.request(method_value, target_url);

        for (key, value) in headers {
            let lower = key.to_ascii_lowercase();
            if matches!(
                lower.as_str(),
                "host" | "content-length" | "connection" | "origin"
            ) {
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

        if should_set_openclaw_scopes && !has_openclaw_scopes {
            request = request.header("x-openclaw-scopes", "operator.write");
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

fn run_local_proxy(
    listener: TcpListener,
    stop: Arc<AtomicBool>,
    platforms: Arc<Vec<LocalProxyPlatform>>,
) {
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
                                "timestamp": crate::current_timestamp_millis(),
                                "platforms": platforms
                                    .iter()
                                    .map(|platform| serde_json::json!({
                                        "pathPrefix": platform.path_prefix,
                                        "baseUrl": platform.base_url,
                                        "protocol": platform.protocol
                                    }))
                                    .collect::<Vec<_>>()
                            });
                            let body =
                                serde_json::to_vec(&payload).map_err(|error| error.to_string())?;
                            write_http_response(
                                &mut stream,
                                200,
                                Some("application/json"),
                                &cors_headers,
                                &body,
                            )?;
                            return Ok(());
                        }

                        let (status, content_type, mut response_headers, response_body) =
                            proxy_single_request(method, path, headers, body, platforms)?;
                        response_headers.extend(cors_headers);
                        write_http_response(
                            &mut stream,
                            status,
                            Some(&content_type),
                            &response_headers,
                            &response_body,
                        )?;
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
pub(crate) fn sync_local_proxy(
    port: u16,
    platforms: Vec<LocalProxyPlatform>,
) -> Result<(), String> {
    let state_mutex = crate::openclaw::state::local_proxy_state();
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
