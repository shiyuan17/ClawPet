use crate::openclaw::models::*;
use crate::{bytes_to_lower_hex, current_timestamp_millis, read_env_u64, sms_code_store};
use base64::Engine;
use chrono::Utc;
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha1::Sha1;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

const DEFAULT_SMS_CODE_TTL_SECONDS: u64 = 300;
const DEFAULT_SMS_COOLDOWN_SECONDS: u64 = 60;

fn read_required_env(name: &str) -> Result<String, String> {
    let raw = std::env::var(name).map_err(|_| format!("缺少环境变量：{name}"))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("环境变量不能为空：{name}"));
    }
    Ok(trimmed.to_string())
}

fn load_aliyun_sms_config() -> Result<AliyunSmsConfig, String> {
    let access_key_id = read_required_env("ALIYUN_SMS_ACCESS_KEY_ID")?;
    let access_key_secret = read_required_env("ALIYUN_SMS_ACCESS_KEY_SECRET")?;
    let sign_name = read_required_env("ALIYUN_SMS_SIGN_NAME")?;
    let template_code = read_required_env("ALIYUN_SMS_TEMPLATE_CODE")?;
    let endpoint = std::env::var("ALIYUN_SMS_ENDPOINT")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://dysmsapi.aliyuncs.com".to_string());
    let region_id = std::env::var("ALIYUN_SMS_REGION_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "cn-hangzhou".to_string());
    let code_ttl_seconds = read_env_u64(
        "ALIYUN_SMS_CODE_TTL_SECONDS",
        DEFAULT_SMS_CODE_TTL_SECONDS,
        60,
        1800,
    );
    let cooldown_seconds = read_env_u64(
        "ALIYUN_SMS_COOLDOWN_SECONDS",
        DEFAULT_SMS_COOLDOWN_SECONDS,
        10,
        300,
    );

    Ok(AliyunSmsConfig {
        access_key_id,
        access_key_secret,
        sign_name,
        template_code,
        endpoint,
        region_id,
        code_ttl_seconds,
        cooldown_seconds,
    })
}

fn normalize_mainland_phone(raw: &str) -> String {
    raw.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn is_valid_mainland_phone(phone: &str) -> bool {
    let bytes = phone.as_bytes();
    bytes.len() == 11 && bytes[0] == b'1' && (b'3'..=b'9').contains(&bytes[1])
}

fn sanitize_verification_code(raw: &str) -> String {
    raw.chars().filter(|ch| ch.is_ascii_digit()).collect()
}

fn generate_numeric_code(length: usize) -> Result<String, String> {
    if length == 0 {
        return Err("验证码长度非法。".to_string());
    }
    let mut random_bytes = vec![0u8; length];
    getrandom(&mut random_bytes).map_err(|error| format!("生成验证码失败：{error}"))?;
    let output = random_bytes
        .into_iter()
        .map(|byte| (b'0' + (byte % 10)) as char)
        .collect::<String>();
    Ok(output)
}

fn generate_aliyun_signature_nonce() -> Result<String, String> {
    let mut random_bytes = [0u8; 16];
    getrandom(&mut random_bytes).map_err(|error| format!("生成短信签名随机数失败：{error}"))?;
    Ok(bytes_to_lower_hex(&random_bytes))
}

fn aliyun_percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric()
            || byte == b'-'
            || byte == b'_'
            || byte == b'.'
            || byte == b'~'
        {
            encoded.push(byte as char);
            continue;
        }
        encoded.push('%');
        encoded.push_str(&format!("{:02X}", byte));
    }
    encoded
}

fn build_aliyun_query_string(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                aliyun_percent_encode(key),
                aliyun_percent_encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn build_aliyun_signature(
    params: &BTreeMap<String, String>,
    access_key_secret: &str,
) -> Result<String, String> {
    type HmacSha1 = Hmac<Sha1>;
    let canonical_query_string = build_aliyun_query_string(params);
    let string_to_sign = format!(
        "POST&%2F&{}",
        aliyun_percent_encode(&canonical_query_string)
    );
    let signing_key = format!("{access_key_secret}&");
    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes())
        .map_err(|error| format!("生成短信签名失败：{error}"))?;
    mac.update(string_to_sign.as_bytes());
    let result = mac.finalize().into_bytes();
    Ok(base64::engine::general_purpose::STANDARD.encode(result))
}

fn build_aliyun_send_sms_params(
    config: &AliyunSmsConfig,
    phone: &str,
    code: &str,
) -> Result<BTreeMap<String, String>, String> {
    let template_param = serde_json::to_string(&serde_json::json!({ "code": code }))
        .map_err(|error| format!("构建短信模板参数失败：{error}"))?;
    let nonce = generate_aliyun_signature_nonce()?;
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut params = BTreeMap::new();
    params.insert("Action".to_string(), "SendSms".to_string());
    params.insert("Version".to_string(), "2017-05-25".to_string());
    params.insert("RegionId".to_string(), config.region_id.clone());
    params.insert("PhoneNumbers".to_string(), phone.to_string());
    params.insert("SignName".to_string(), config.sign_name.clone());
    params.insert("TemplateCode".to_string(), config.template_code.clone());
    params.insert("TemplateParam".to_string(), template_param);
    params.insert("Format".to_string(), "JSON".to_string());
    params.insert("AccessKeyId".to_string(), config.access_key_id.clone());
    params.insert("SignatureMethod".to_string(), "HMAC-SHA1".to_string());
    params.insert("SignatureVersion".to_string(), "1.0".to_string());
    params.insert("SignatureNonce".to_string(), nonce);
    params.insert("Timestamp".to_string(), timestamp);
    Ok(params)
}

async fn call_aliyun_send_sms(
    config: &AliyunSmsConfig,
    phone: &str,
    code: &str,
) -> Result<(), String> {
    let mut params = build_aliyun_send_sms_params(config, phone, code)?;
    let signature = build_aliyun_signature(&params, &config.access_key_secret)?;
    params.insert("Signature".to_string(), signature);
    let query = build_aliyun_query_string(&params);
    let url = format!("{}/?{}", config.endpoint.trim_end_matches('/'), query);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| format!("创建短信请求客户端失败：{error}"))?;
    let response = client
        .post(&url)
        .send()
        .await
        .map_err(|error| format!("调用阿里云短信服务失败：{error}"))?;
    let status = response.status();
    let raw_text = response
        .text()
        .await
        .map_err(|error| format!("读取短信服务响应失败：{error}"))?;

    if !status.is_success() {
        let preview = raw_text.chars().take(400).collect::<String>();
        return Err(format!("短信服务返回异常状态 {status}：{preview}"));
    }

    let payload: Value = serde_json::from_str(&raw_text)
        .map_err(|error| format!("短信服务响应解析失败：{error}"))?;
    let response_code = payload
        .get("Code")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if response_code.eq_ignore_ascii_case("OK") {
        return Ok(());
    }

    let response_message = payload
        .get("Message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("未知错误");
    if response_code.is_empty() {
        Err(format!("短信发送失败：{response_message}"))
    } else {
        Err(format!(
            "短信发送失败（{response_code}）：{response_message}"
        ))
    }
}

fn clear_expired_sms_code_records(records: &mut HashMap<String, SmsCodeRecord>, now_ms: u128) {
    records.retain(|_, record| record.expires_at_ms > now_ms);
}

#[tauri::command]
pub(crate) async fn send_sms_code(phone: String) -> Result<SmsSendResponse, String> {
    let normalized_phone = normalize_mainland_phone(&phone);
    if !is_valid_mainland_phone(&normalized_phone) {
        return Err("请输入有效的中国大陆手机号。".to_string());
    }
    let config = load_aliyun_sms_config()?;

    let now_ms = current_timestamp_millis();
    {
        let store_mutex = sms_code_store();
        let mut store = store_mutex
            .lock()
            .map_err(|_| "无法获取短信验证码状态锁。".to_string())?;
        clear_expired_sms_code_records(&mut store, now_ms);
        if let Some(record) = store.get(&normalized_phone) {
            let cooldown_until_ms =
                record.last_sent_at_ms + (config.cooldown_seconds as u128).saturating_mul(1000);
            if now_ms < cooldown_until_ms {
                let remaining_seconds = ((cooldown_until_ms - now_ms) / 1000 + 1) as u64;
                return Err(format!(
                    "验证码发送过于频繁，请在 {} 秒后重试。",
                    remaining_seconds
                ));
            }
        }
    }

    let verification_code = generate_numeric_code(6)?;
    call_aliyun_send_sms(&config, &normalized_phone, &verification_code).await?;

    let saved_at_ms = current_timestamp_millis();
    let expires_at_ms = saved_at_ms + (config.code_ttl_seconds as u128).saturating_mul(1000);
    {
        let store_mutex = sms_code_store();
        let mut store = store_mutex
            .lock()
            .map_err(|_| "无法获取短信验证码状态锁。".to_string())?;
        store.insert(
            normalized_phone,
            SmsCodeRecord {
                code: verification_code,
                expires_at_ms,
                last_sent_at_ms: saved_at_ms,
            },
        );
    }

    Ok(SmsSendResponse {
        detail: "验证码已发送，请注意查收短信。".to_string(),
        cooldown_seconds: config.cooldown_seconds,
        expires_in_seconds: config.code_ttl_seconds,
    })
}

#[tauri::command]
pub(crate) fn verify_sms_code(phone: String, code: String) -> Result<SmsVerifyResponse, String> {
    let normalized_phone = normalize_mainland_phone(&phone);
    if !is_valid_mainland_phone(&normalized_phone) {
        return Err("请输入有效的中国大陆手机号。".to_string());
    }

    let normalized_code = sanitize_verification_code(&code);
    if normalized_code.len() != 6 {
        return Err("请输入 6 位数字验证码。".to_string());
    }

    let now_ms = current_timestamp_millis();
    let store_mutex = sms_code_store();
    let mut store = store_mutex
        .lock()
        .map_err(|_| "无法获取短信验证码状态锁。".to_string())?;
    clear_expired_sms_code_records(&mut store, now_ms);

    let Some(record) = store.get(&normalized_phone).cloned() else {
        return Err("请先获取短信验证码。".to_string());
    };

    if now_ms > record.expires_at_ms {
        store.remove(&normalized_phone);
        return Err("验证码已过期，请重新获取。".to_string());
    }

    if record.code != normalized_code {
        return Err("验证码错误，请重新输入。".to_string());
    }

    store.remove(&normalized_phone);

    let mut token_bytes = [0u8; 24];
    getrandom(&mut token_bytes).map_err(|error| format!("生成登录会话失败：{error}"))?;
    let session_token = bytes_to_lower_hex(&token_bytes);
    Ok(SmsVerifyResponse {
        detail: "登录成功。".to_string(),
        session_token,
    })
}
