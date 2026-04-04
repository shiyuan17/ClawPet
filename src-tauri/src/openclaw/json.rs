use serde_json::Value;

fn parse_json_object_from_line_tail(line: &str) -> Option<Value> {
    extract_last_json_object_from_text(line)
}

fn extract_last_json_object_from_text(raw: &str) -> Option<Value> {
    let cleaned = crate::strip_ansi_escape_sequences(raw).replace('\r', "");
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if value.is_object() {
            return Some(value);
        }
    }

    let mut latest: Option<Value> = None;
    let mut depth = 0usize;
    let mut start_index: Option<usize> = None;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in cleaned.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start_index = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start_index {
                        let end = index + ch.len_utf8();
                        let candidate = &cleaned[start..end];
                        if let Ok(value) = serde_json::from_str::<Value>(candidate) {
                            if value.is_object() {
                                latest = Some(value);
                            }
                        }
                    }
                    start_index = None;
                }
            }
            _ => {}
        }
    }

    latest
}

pub(crate) fn extract_last_json_object_from_streams(stdout: &str, stderr: &str) -> Option<Value> {
    let stderr_cleaned = crate::strip_ansi_escape_sequences(stderr).replace('\r', "");
    let stdout_cleaned = crate::strip_ansi_escape_sequences(stdout).replace('\r', "");

    if let Some(value) = extract_last_json_object_from_text(&stderr_cleaned) {
        return Some(value);
    }
    if let Some(value) = extract_last_json_object_from_text(&stdout_cleaned) {
        return Some(value);
    }

    for line in stderr_cleaned.lines().rev() {
        if let Some(value) = parse_json_object_from_line_tail(line) {
            return Some(value);
        }
    }
    for line in stdout_cleaned.lines().rev() {
        if let Some(value) = parse_json_object_from_line_tail(line) {
            return Some(value);
        }
    }

    None
}
