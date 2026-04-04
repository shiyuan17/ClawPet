use crate::openclaw::models::PendingToolCall;
use serde_json::Value;

pub(crate) fn extract_text_from_message_content(content: &Value) -> Option<String> {
    if let Some(text) = content.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    if let Some(items) = content.as_array() {
        let mut texts = Vec::new();
        for item in items {
            let Some(obj) = item.as_object() else {
                continue;
            };
            if let Some(text) = obj.get("text").and_then(Value::as_str) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    texts.push(trimmed.to_string());
                }
            }
        }
        if !texts.is_empty() {
            return Some(texts.join("\n"));
        }
    }

    None
}

pub(crate) fn sanitize_staff_output(content: &str) -> String {
    let mut normalized = content
        .replace("[[reply_to_current]]", "")
        .trim()
        .to_string();
    if normalized.starts_with('[') {
        if let Some(pos) = normalized.find(']') {
            normalized = normalized[(pos + 1)..].trim().to_string();
        }
    }
    if normalized.is_empty() {
        "最近暂无产出。".to_string()
    } else {
        normalized
    }
}

pub(crate) fn extract_message_timestamp_ms(
    message: &serde_json::Map<String, Value>,
    fallback: i64,
) -> i64 {
    crate::openclaw::staff_runtime::value_as_i64(message.get("timestamp")).unwrap_or(fallback)
}

pub(crate) fn extract_message_text(message: &serde_json::Map<String, Value>) -> Option<String> {
    message
        .get("content")
        .and_then(extract_text_from_message_content)
        .map(|text| sanitize_staff_output(&text))
}

pub(crate) fn extract_tool_calls(
    message: &serde_json::Map<String, Value>,
) -> Vec<(String, PendingToolCall)> {
    let Some(items) = message.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };

    let created_at = extract_message_timestamp_ms(message, 0);
    let mut output = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };
        if obj.get("type").and_then(Value::as_str) != Some("toolCall") {
            continue;
        }

        let Some(tool_call_id) = obj.get("id").and_then(Value::as_str) else {
            continue;
        };
        let tool_name = obj
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("tool")
            .to_string();
        let arguments = obj
            .get("arguments")
            .map(|value| serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()))
            .unwrap_or_else(|| "{}".to_string());

        output.push((
            tool_call_id.to_string(),
            PendingToolCall {
                tool_name,
                arguments,
                created_at,
            },
        ));
    }

    output
}

pub(crate) fn extract_tool_result_text(message: &serde_json::Map<String, Value>) -> Option<String> {
    let text = extract_message_text(message)?;
    let details = message.get("details").and_then(Value::as_object);
    let audio_path = details
        .and_then(|value| value.get("audioPath"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(path) = audio_path {
        if text.contains(path) {
            return Some(text);
        }
        return Some(format!("{text}\n{path}"));
    }

    Some(text)
}

pub(crate) fn infer_openclaw_response_status(text: &str) -> (u16, Option<String>) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (200, None);
    }

    if let Some(start) = trimmed.find("（") {
        if let Some(end_offset) = trimmed[start + 3..].find('）') {
            let code = &trimmed[start + 3..start + 3 + end_offset];
            if let Ok(status) = code.parse::<u16>() {
                return (status, (status >= 400).then(|| trimmed.to_string()));
            }
        }
    }

    if let Some(index) = trimmed.find("status code ") {
        let digits = trimmed[index + "status code ".len()..]
            .chars()
            .take_while(|char| char.is_ascii_digit())
            .collect::<String>();
        if let Ok(status) = digits.parse::<u16>() {
            return (status, (status >= 400).then(|| trimmed.to_string()));
        }
    }

    if let Some(index) = trimmed.find("HTTP ") {
        let digits = trimmed[index + "HTTP ".len()..]
            .chars()
            .take_while(|char| char.is_ascii_digit())
            .collect::<String>();
        if let Ok(status) = digits.parse::<u16>() {
            return (status, (status >= 400).then(|| trimmed.to_string()));
        }
    }

    let lower = trimmed.to_ascii_lowercase();
    if trimmed.contains("请求失败")
        || trimmed.contains("请求 OpenClaw 失败")
        || trimmed.contains("OpenClaw 请求失败")
        || trimmed.contains("返回错误状态")
        || trimmed.contains("连接失败")
        || trimmed.contains("连接被拒绝")
        || trimmed.contains("超时")
        || lower.starts_with("error sending request for url")
        || lower.contains(": error sending request for url")
        || lower.contains("connection refused")
        || lower.contains("failed to connect")
        || lower.contains("network error")
        || lower.contains("timed out")
        || lower.contains("dns error")
        || trimmed.contains("invalid_api_key")
        || trimmed.contains("unauthorized")
        || trimmed.contains("rate limit")
    {
        return (500, Some(trimmed.to_string()));
    }

    (200, None)
}

#[cfg(test)]
mod tests {
    use super::infer_openclaw_response_status;

    #[test]
    fn infer_runtime_status_marks_openclaw_request_failure_as_error() {
        let text = "请求 OpenClaw 失败: error sending request for url (http://127.0.0.1:19789/v1/chat/completions)";
        let (status, error) = infer_openclaw_response_status(text);
        assert_eq!(status, 500);
        assert_eq!(error.as_deref(), Some(text));
    }

    #[test]
    fn infer_runtime_status_marks_english_network_failure_as_error() {
        let text = "error sending request for url (http://127.0.0.1:19789/v1/chat/completions)";
        let (status, error) = infer_openclaw_response_status(text);
        assert_eq!(status, 500);
        assert_eq!(error.as_deref(), Some(text));
    }

    #[test]
    fn infer_runtime_status_keeps_normal_text_successful() {
        let text = "配置已经基本落地，下一步继续执行。";
        let (status, error) = infer_openclaw_response_status(text);
        assert_eq!(status, 200);
        assert!(error.is_none());
    }
}
