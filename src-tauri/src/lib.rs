use base64::Engine;
use chrono::Utc;
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::Sha1;
use std::collections::{BTreeMap, HashMap, HashSet};
#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
#[cfg(target_os = "macos")]
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};
#[cfg(not(target_os = "macos"))]
use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn dragonclaw_show_user_notification(
        title_utf8: *const c_char,
        body_utf8: *const c_char,
        error_out: *mut *mut c_char,
    ) -> c_int;
    fn dragonclaw_show_legacy_user_notification(
        title_utf8: *const c_char,
        body_utf8: *const c_char,
        error_out: *mut *mut c_char,
    ) -> c_int;
    fn dragonclaw_free_c_string(value: *mut c_char);
}

#[cfg(target_os = "macos")]
fn run_macos_notification_ffi(
    ffi: unsafe extern "C" fn(*const c_char, *const c_char, *mut *mut c_char) -> c_int,
    title: &str,
    body: Option<&str>,
) -> Result<(), String> {
    let title_c = CString::new(title).map_err(|_| "通知标题包含无效字符。".to_string())?;
    let body_c =
        CString::new(body.unwrap_or("")).map_err(|_| "通知正文包含无效字符。".to_string())?;
    let mut error_ptr: *mut c_char = std::ptr::null_mut();

    let result = unsafe { ffi(title_c.as_ptr(), body_c.as_ptr(), &mut error_ptr) };
    if result == 1 {
        return Ok(());
    }

    let message = if error_ptr.is_null() {
        "系统通知发送失败。".to_string()
    } else {
        let message = unsafe { CStr::from_ptr(error_ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe {
            dragonclaw_free_c_string(error_ptr);
        }
        message
    };

    Err(message)
}

#[cfg(target_os = "macos")]
fn show_macos_user_notification(title: &str, body: Option<&str>) -> Result<(), String> {
    run_macos_notification_ffi(dragonclaw_show_user_notification, title, body)
}

#[cfg(target_os = "macos")]
fn show_macos_legacy_user_notification(title: &str, body: Option<&str>) -> Result<(), String> {
    run_macos_notification_ffi(dragonclaw_show_legacy_user_notification, title, body)
}

#[cfg(target_os = "macos")]
fn is_running_from_macos_app_bundle() -> bool {
    std::env::current_exe()
        .ok()
        .map(|path| {
            path.ancestors().any(|ancestor| {
                ancestor
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| value.eq_ignore_ascii_case("app"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn show_macos_dev_helper_notification(title: &str, body: Option<&str>) -> Result<(), String> {
    let helper_app_path = option_env!("DRAGONCLAW_DEV_NOTIFIER_APP")
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "未找到开发态通知助手。".to_string())?;

    let mut command = Command::new("open");
    command
        .arg("-g")
        .arg("-na")
        .arg(helper_app_path)
        .arg("--args")
        .arg(title);
    if let Some(body_text) = body {
        command.arg(body_text);
    }

    let status = command
        .status()
        .map_err(|error| format!("启动开发态通知助手失败：{error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("开发态通知助手退出异常：{status}"))
    }
}

#[cfg(target_os = "macos")]
fn show_macos_osascript_dialog(title: &str, body: Option<&str>) -> Result<(), String> {
    let body_text = body.unwrap_or("");
    let mut command = Command::new("osascript");
    command
        .arg("-e")
        .arg("on run argv")
        .arg("-e")
        .arg("set theTitle to item 1 of argv")
        .arg("-e")
        .arg("set theBody to \"\"")
        .arg("-e")
        .arg("if (count of argv) > 1 then set theBody to item 2 of argv")
        .arg("-e")
        .arg("display dialog theBody with title theTitle buttons {\"知道了\"} default button 1 giving up after 12 with icon note")
        .arg("-e")
        .arg("end run")
        .arg("--")
        .arg(title)
        .arg(body_text)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("调用 osascript 显示系统弹窗失败：{error}"))
}

#[cfg(target_os = "macos")]
fn show_macos_osascript_notification(title: &str, body: Option<&str>) -> Result<(), String> {
    let body_text = body.unwrap_or("");
    let status = Command::new("osascript")
        .arg("-e")
        .arg("on run argv")
        .arg("-e")
        .arg("display notification (item 2 of argv) with title (item 1 of argv)")
        .arg("-e")
        .arg("end run")
        .arg("--")
        .arg(title)
        .arg(body_text)
        .status()
        .map_err(|error| format!("调用 osascript 发送系统通知失败：{error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("osascript 发送系统通知失败，退出码：{status}"))
    }
}

#[cfg(target_os = "macos")]
fn show_platform_system_notification(
    _app: &tauri::AppHandle,
    title: &str,
    body: Option<&str>,
) -> Result<(), String> {
    if !is_running_from_macos_app_bundle() {
        eprintln!("[dragonclaw] using osascript dialog path in dev runtime");
        return show_macos_osascript_dialog(title, body)
            .or_else(|dialog_error| {
                eprintln!("[dragonclaw] osascript dialog failed: {dialog_error}");
                show_macos_dev_helper_notification(title, body)
            })
            .or_else(|helper_error| {
                eprintln!("[dragonclaw] dev helper notification failed: {helper_error}");
                show_macos_legacy_user_notification(title, body)
            })
            .or_else(|legacy_error| {
                eprintln!("[dragonclaw] legacy macOS notification failed: {legacy_error}");
                show_macos_osascript_notification(title, body)
            });
    }

    eprintln!("[dragonclaw] using modern macOS notification path in app bundle runtime");
    show_macos_user_notification(title, body).or_else(|modern_error| {
        eprintln!("[dragonclaw] modern macOS notification failed: {modern_error}");
        show_macos_legacy_user_notification(title, body)
    })
}

#[cfg(not(target_os = "macos"))]
fn show_platform_system_notification(
    app: &tauri::AppHandle,
    title: &str,
    body: Option<&str>,
) -> Result<(), String> {
    let mut builder = app.notification().builder().title(title.to_string());
    if let Some(body_text) = body {
        builder = builder.body(body_text.to_string());
    }

    builder
        .show()
        .map_err(|error| format!("系统通知发送失败：{error}"))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OpenClawMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenClawRequest {
    messages: Vec<OpenClawMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
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
    gateway_port: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FeishuOnboardingQrResponse {
    qr_url: String,
    user_code: String,
    device_code: String,
    poll_interval_seconds: u64,
    expires_in_seconds: u64,
    expires_at_ms: u128,
}

#[derive(Debug, Deserialize)]
struct FeishuOnboardingInitResponse {
    #[serde(default)]
    supported_auth_methods: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FeishuOnboardingBeginResponse {
    device_code: Option<String>,
    verification_uri_complete: Option<String>,
    user_code: Option<String>,
    expire_in: Option<u64>,
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct FeishuOnboardingPollUserInfo {
    tenant_brand: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FeishuOnboardingPollRawResponse {
    client_id: Option<String>,
    client_secret: Option<String>,
    user_info: Option<FeishuOnboardingPollUserInfo>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FeishuOnboardingPollResponse {
    status: String,
    message: Option<String>,
    app_id: Option<String>,
    app_secret: Option<String>,
    tenant_brand: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelQrBindingSessionSnapshot {
    session_id: String,
    channel_type: String,
    status: String,
    qr_url: Option<String>,
    qr_ascii: Option<String>,
    detail: Option<String>,
    logs: Vec<String>,
    started_at_ms: u128,
    updated_at_ms: u128,
}

#[derive(Debug, Clone)]
struct OpenClawChannelQrBindingSessionState {
    session_id: String,
    channel_type: String,
    status: String,
    qr_url: Option<String>,
    qr_ascii: Option<String>,
    qr_ascii_collecting: bool,
    qr_ascii_buffer: Vec<String>,
    detail: Option<String>,
    logs: Vec<String>,
    started_at_ms: u128,
    updated_at_ms: u128,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LobsterBackupItem {
    name: String,
    path: String,
    created_at_ms: u128,
    size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LobsterSnapshotResponse {
    openclaw_installed: bool,
    openclaw_version: Option<String>,
    openclaw_binary: Option<String>,
    openclaw_home: String,
    backup_dir: String,
    detail: String,
    backups: Vec<LobsterBackupItem>,
    install_wizard_open_every_launch: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawRuntimeStatusResponse {
    installed: bool,
    healthy: bool,
    status: String,
    command: String,
    detail: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    gateway_port: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LobsterActionResult {
    action: String,
    command: String,
    success: bool,
    detail: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    duration_ms: u128,
    backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelPluginInstallResult {
    channel_type: String,
    plugin_id: Option<String>,
    plugin_spec: Option<String>,
    command: String,
    success: bool,
    detail: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    duration_ms: u128,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelMessageSendResult {
    channel_type: String,
    account_id: String,
    target: String,
    command: String,
    success: bool,
    detail: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    duration_ms: u128,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LobsterInstallCheckItem {
    id: String,
    title: String,
    status: String,
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LobsterInstallGuideResponse {
    os: String,
    ready: bool,
    checks: Vec<LobsterInstallCheckItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreinstalledSkillManifestItem {
    slug: String,
    #[serde(default)]
    auto_enable: bool,
}

#[derive(Debug, Deserialize)]
struct PreinstalledSkillManifest {
    #[serde(default)]
    skills: Vec<PreinstalledSkillManifestItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffMemberSnapshot {
    agent_id: String,
    display_name: String,
    role_label: String,
    channel: String,
    model: String,
    workspace: String,
    tools_profile: String,
    tools_enabled_count: usize,
    status_label: String,
    current_work_label: String,
    current_work: String,
    recent_output: String,
    scheduled_label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffSnapshotResponse {
    mission_statement: String,
    source_path: String,
    detail: String,
    members: Vec<StaffMemberSnapshot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskSnapshotItem {
    id: String,
    name: String,
    agent_id: String,
    session_target: String,
    enabled: bool,
    delete_after_run: bool,
    status_kind: String,
    status_label: String,
    summary: String,
    next_run_at_ms: Option<i64>,
    created_at_ms: Option<i64>,
    updated_at_ms: Option<i64>,
    schedule_kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskSnapshotResponse {
    source_path: String,
    detail: String,
    jobs: Vec<TaskSnapshotItem>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SourceFileSnapshotItem {
    id: String,
    title: String,
    summary: String,
    content: String,
    source_path: String,
    relative_path: String,
    facet_key: String,
    facet_label: String,
    category: String,
    updated_at_ms: i64,
    exists: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceFileSnapshotResponse {
    source_path: String,
    detail: String,
    items: Vec<SourceFileSnapshotItem>,
}

/// 已安装技能项：来自 ~/.openclaw/skills 与 workspace/skills，对应 openclaw 技能信息（非文档编辑）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenClawSkillListItem {
    id: String,
    name: String,
    description: String,
    enabled: bool,
    relative_path: String,
    source_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawSkillsListResponse {
    source_path: String,
    /// 内置技能（来自 openclaw.json skills.allowBundled + entries）
    built_in: Vec<OpenClawSkillListItem>,
    /// 安装技能（来自 ~/.openclaw/skills 与 workspace/skills 下的 SKILL.md）
    installed: Vec<OpenClawSkillListItem>,
}

/// 已配置工具项：来自 openclaw tools.profile / allow/deny，对应已安装工具信息（非 TOOLS.md 编辑）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenClawToolListItem {
    id: String,
    name: String,
    description: String,
    category: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawToolsListResponse {
    profile: String,
    profile_label: String,
    tools: Vec<OpenClawToolListItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawPlatformSnapshotItem {
    id: String,
    provider_id: String,
    name: String,
    protocol: String,
    base_url: String,
    path_prefix: String,
    api_path: String,
    api_key: String,
    model: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawPlatformSnapshotResponse {
    source_path: String,
    detail: String,
    platforms: Vec<OpenClawPlatformSnapshotItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelAccountSnapshotItem {
    account_id: String,
    name: String,
    configured: bool,
    status: String,
    is_default: bool,
    agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelGroupSnapshotItem {
    channel_type: String,
    default_account_id: String,
    status: String,
    accounts: Vec<OpenClawChannelAccountSnapshotItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelAccountsSnapshotResponse {
    source_path: String,
    detail: String,
    channels: Vec<OpenClawChannelGroupSnapshotItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelConfigPayload {
    channel_type: String,
    account_id: Option<String>,
    config: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelBindingPayload {
    channel_type: String,
    account_id: String,
    agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelMessageSendPayload {
    channel_type: String,
    account_id: Option<String>,
    target: String,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelMirrorFailureLogPayload {
    channel_type: String,
    account_id: String,
    target: String,
    message_preview: String,
    error_detail: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelAccountPayload {
    channel_type: String,
    account_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawChannelAccountRenamePayload {
    channel_type: String,
    account_id: String,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawMessageLogItem {
    id: String,
    session_id: String,
    platform_id: String,
    platform_name: String,
    protocol: String,
    method: String,
    endpoint: String,
    base_url: Option<String>,
    path: Option<String>,
    request_body: String,
    response_status: u16,
    response_body: String,
    stream_summary: Option<String>,
    duration: u32,
    first_token_time: Option<u32>,
    error: Option<String>,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
    created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawMessageLogResponse {
    detail: String,
    logs: Vec<OpenClawMessageLogItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawAgentSessionSnapshotItem {
    session_key: String,
    session_target: String,
    session_id: String,
    updated_at_ms: i64,
    message_count: usize,
    preview: String,
    last_channel: Option<String>,
    chat_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawAgentSessionsSnapshotResponse {
    detail: String,
    sessions: Vec<OpenClawAgentSessionSnapshotItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawAgentSessionHistoryMessage {
    id: String,
    role: String,
    text: String,
    created_at_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenClawAgentSessionHistoryResponse {
    detail: String,
    session_key: String,
    messages: Vec<OpenClawAgentSessionHistoryMessage>,
}

#[derive(Debug, Clone)]
struct PendingToolCall {
    tool_name: String,
    arguments: String,
    created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioFilePayload {
    data_url: String,
    mime_type: String,
    file_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalMediaFilePayload {
    data_url: String,
    mime_type: String,
    file_name: String,
    byte_length: usize,
}

#[derive(Debug, Clone)]
struct EditableScope {
    facet_key: String,
    facet_label: String,
    workspace_root: PathBuf,
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    messages: Vec<OpenClawMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenClawProviderConfigPayload {
    provider_id: String,
    provider_name: Option<String>,
    protocol: Option<String>,
    api_kind: Option<String>,
    base_url: String,
    model: Option<String>,
    api_key: String,
}

#[derive(Default)]
struct LocalProxyState {
    stop_signal: Option<Arc<AtomicBool>>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
struct SmsCodeRecord {
    code: String,
    expires_at_ms: u128,
    last_sent_at_ms: u128,
}

#[derive(Debug, Clone)]
struct AliyunSmsConfig {
    access_key_id: String,
    access_key_secret: String,
    sign_name: String,
    template_code: String,
    endpoint: String,
    region_id: String,
    code_ttl_seconds: u64,
    cooldown_seconds: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SmsSendResponse {
    detail: String,
    cooldown_seconds: u64,
    expires_in_seconds: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SmsVerifyResponse {
    detail: String,
    session_token: String,
}

static LOCAL_PROXY_STATE: OnceLock<Mutex<LocalProxyState>> = OnceLock::new();
static APP_RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();
static SMS_CODE_STORE: OnceLock<Mutex<HashMap<String, SmsCodeRecord>>> = OnceLock::new();
static OPENCLAW_CHANNEL_QR_BINDING_SESSIONS: OnceLock<
    Mutex<HashMap<String, Arc<Mutex<OpenClawChannelQrBindingSessionState>>>>,
> = OnceLock::new();

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(target_os = "windows")]
fn suppress_windows_command_window(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
fn suppress_windows_command_window(command: &mut Command) -> &mut Command {
    command
}

#[cfg(target_os = "windows")]
fn normalize_windows_path_for_child_process(path: &Path) -> PathBuf {
    let raw = path.as_os_str().to_string_lossy();
    if let Some(stripped) = raw.strip_prefix("\\\\?\\UNC\\") {
        return PathBuf::from(format!("\\\\{stripped}"));
    }
    if let Some(stripped) = raw.strip_prefix("\\\\?\\") {
        return PathBuf::from(stripped);
    }
    path.to_path_buf()
}

#[cfg(not(target_os = "windows"))]
fn normalize_windows_path_for_child_process(path: &Path) -> PathBuf {
    path.to_path_buf()
}

fn local_proxy_state() -> &'static Mutex<LocalProxyState> {
    LOCAL_PROXY_STATE.get_or_init(|| Mutex::new(LocalProxyState::default()))
}

fn sms_code_store() -> &'static Mutex<HashMap<String, SmsCodeRecord>> {
    SMS_CODE_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn openclaw_channel_qr_binding_sessions(
) -> &'static Mutex<HashMap<String, Arc<Mutex<OpenClawChannelQrBindingSessionState>>>> {
    OPENCLAW_CHANNEL_QR_BINDING_SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn set_app_resource_dir(path: PathBuf) {
    let _ = APP_RESOURCE_DIR.set(path);
}

fn load_env_file(path: &Path) {
    if path.exists() {
        let _ = dotenvy::from_path(path);
    }
}

fn read_env_bool(name: &str, default: bool) -> bool {
    let raw = match std::env::var(name) {
        Ok(value) => value,
        Err(_) => return default,
    };

    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
}

fn read_env_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
    let raw = match std::env::var(name) {
        Ok(value) => value,
        Err(_) => return default.clamp(min, max),
    };

    let parsed = match raw.trim().parse::<u64>() {
        Ok(value) => value,
        Err(_) => return default.clamp(min, max),
    };

    parsed.clamp(min, max)
}

fn load_openclaw_env() {
    // Preserve explicit shell-exported token and prevent project .env from
    // silently pinning gateway.auth.token across clean rebuilds.
    let gateway_token_before_dotenv = std::env::var("OPENCLAW_GATEWAY_TOKEN").ok();

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

    match gateway_token_before_dotenv {
        Some(token) => std::env::set_var("OPENCLAW_GATEWAY_TOKEN", token),
        None => std::env::remove_var("OPENCLAW_GATEWAY_TOKEN"),
    }
}

fn load_openclaw_gateway_token_from_config() -> Option<String> {
    let config_path = resolve_openclaw_config_path();
    let config_text = std::fs::read_to_string(config_path).ok()?;
    let config = serde_json::from_str::<serde_json::Value>(&config_text).ok()?;
    let auth = config.get("gateway")?.get("auth")?;
    let mode = auth.get("mode")?.as_str()?;
    if mode != "token" {
        return None;
    }

    auth.get("token")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn generate_ephemeral_openclaw_gateway_token() -> Result<String, String> {
    let mut random_bytes = [0u8; 24];
    getrandom(&mut random_bytes).map_err(|error| format!("生成网关令牌失败: {error}"))?;
    Ok(bytes_to_lower_hex(&random_bytes))
}

fn resolve_openclaw_gateway_token() -> Option<String> {
    std::env::var("OPENCLAW_GATEWAY_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(load_openclaw_gateway_token_from_config)
}

fn resolve_openclaw_gateway_token_for_onboard() -> Result<String, String> {
    if let Some(token) = resolve_openclaw_gateway_token() {
        return Ok(token);
    }
    generate_ephemeral_openclaw_gateway_token()
}

fn resolve_openclaw_config_path() -> PathBuf {
    resolve_openclaw_home_path().join("openclaw.json")
}

fn official_openclaw_install_hint_for_platform() -> String {
    if cfg!(target_os = "windows") {
        return "请先按官方安装脚本安装 OpenClaw（Windows PowerShell）：`iwr -useb https://openclaw.ai/install.ps1 | iex`；安装后执行 `openclaw onboard --install-daemon`。".to_string();
    }
    "请先按官方安装脚本安装 OpenClaw（macOS/Linux/WSL2）：`curl -fsSL https://openclaw.ai/install.sh | bash`；安装后执行 `openclaw onboard --install-daemon`。".to_string()
}

#[derive(Default, Debug, Clone)]
struct ChatCompletionsEndpointEnableOutcome {
    changed_paths: Vec<String>,
    unchanged_paths: Vec<String>,
    failures: Vec<String>,
}

impl ChatCompletionsEndpointEnableOutcome {
    fn any_success(&self) -> bool {
        !self.changed_paths.is_empty() || !self.unchanged_paths.is_empty()
    }

    fn changed(&self) -> bool {
        !self.changed_paths.is_empty()
    }

    fn detail(&self) -> String {
        let mut segments = Vec::new();
        if !self.changed_paths.is_empty() {
            segments.push(format!(
                "已写入 gateway.mode=local 且 gateway.http.endpoints.chatCompletions.enabled=true（{}）",
                self.changed_paths.join(", ")
            ));
        }
        if !self.unchanged_paths.is_empty() {
            segments.push(format!(
                "配置已就绪（gateway.mode=local 且 chatCompletions.enabled=true）（{}）",
                self.unchanged_paths.join(", ")
            ));
        }
        if !self.failures.is_empty() {
            segments.push(format!("写入失败（{}）", self.failures.join("；")));
        }
        if segments.is_empty() {
            "未找到可处理的 openclaw 配置路径。".to_string()
        } else {
            segments.join("；")
        }
    }
}

fn collect_openclaw_candidate_config_paths() -> Vec<PathBuf> {
    vec![resolve_openclaw_config_path()]
}

fn ensure_openclaw_chat_completions_endpoint_enabled_at_path(
    config_path: &Path,
) -> Result<bool, String> {
    let raw = match std::fs::read_to_string(&config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }

    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
    let gateway_obj = root
        .entry("gateway")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway 字段不是对象")?;
    let mut changed = false;
    let gateway_mode_ready = gateway_obj
        .get("mode")
        .and_then(Value::as_str)
        .map(|value| value.trim().eq_ignore_ascii_case("local"))
        .unwrap_or(false);
    if !gateway_mode_ready {
        gateway_obj.insert("mode".to_string(), Value::String("local".to_string()));
        changed = true;
    }
    let http_obj = gateway_obj
        .entry("http")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway.http 字段不是对象")?;
    let endpoints_obj = http_obj
        .entry("endpoints")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway.http.endpoints 字段不是对象")?;
    let chat_obj = endpoints_obj
        .entry("chatCompletions")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 gateway.http.endpoints.chatCompletions 字段不是对象")?;

    if chat_obj.get("enabled").and_then(Value::as_bool) != Some(true) {
        chat_obj.insert("enabled".to_string(), Value::Bool(true));
        changed = true;
    }
    if !changed {
        return Ok(false);
    }
    write_openclaw_config_value(&config_path, &parsed)?;
    Ok(true)
}

fn ensure_openclaw_chat_completions_endpoint_enabled_outcome(
) -> ChatCompletionsEndpointEnableOutcome {
    let mut outcome = ChatCompletionsEndpointEnableOutcome::default();
    for path in collect_openclaw_candidate_config_paths() {
        let display = path.display().to_string();
        match ensure_openclaw_chat_completions_endpoint_enabled_at_path(&path) {
            Ok(true) => outcome.changed_paths.push(display),
            Ok(false) => outcome.unchanged_paths.push(display),
            Err(error) => outcome.failures.push(format!("{display}: {error}")),
        }
    }
    outcome
}

fn ensure_openclaw_chat_completions_endpoint_enabled() -> Result<bool, String> {
    let outcome = ensure_openclaw_chat_completions_endpoint_enabled_outcome();
    if outcome.any_success() {
        if !outcome.failures.is_empty() {
            eprintln!(
                "OpenClaw chatCompletions 端点自动启用存在部分失败：{}",
                outcome.detail()
            );
        }
        return Ok(outcome.changed());
    }

    Err(format!(
        "无法确保 gateway.mode=local 与 gateway.http.endpoints.chatCompletions.enabled=true：{}",
        outcome.detail()
    ))
}

fn remove_unknown_keys_from_object(
    object: &mut serde_json::Map<String, Value>,
    allowed_keys: &[&str],
) -> Vec<String> {
    let keys = object.keys().cloned().collect::<Vec<_>>();
    let mut removed = Vec::new();
    for key in keys {
        if !allowed_keys.iter().any(|allowed| key == *allowed) {
            object.remove(&key);
            removed.push(key);
        }
    }
    removed
}

fn sanitize_openclaw_models_provider_schema_at_path(
    config_path: &Path,
) -> Result<Option<String>, String> {
    let raw = match std::fs::read_to_string(config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let Some(root_obj) = parsed.as_object_mut() else {
        return Ok(None);
    };
    let Some(models_obj) = root_obj.get_mut("models").and_then(Value::as_object_mut) else {
        return Ok(None);
    };
    let Some(providers_obj) = models_obj
        .get_mut("providers")
        .and_then(Value::as_object_mut)
    else {
        return Ok(None);
    };

    let provider_allowed_keys = [
        "baseUrl",
        "apiKey",
        "auth",
        "api",
        "injectNumCtxForOpenAICompat",
        "headers",
        "authHeader",
        "models",
    ];
    let model_allowed_keys = [
        "id",
        "name",
        "api",
        "reasoning",
        "input",
        "cost",
        "contextWindow",
        "maxTokens",
        "headers",
        "compat",
    ];

    let mut changed = false;
    let mut notes = Vec::new();

    for (provider_id, provider_value) in providers_obj.iter_mut() {
        let Some(provider_obj) = provider_value.as_object_mut() else {
            continue;
        };

        let removed_provider_keys =
            remove_unknown_keys_from_object(provider_obj, &provider_allowed_keys);
        if !removed_provider_keys.is_empty() {
            changed = true;
            for key in removed_provider_keys {
                notes.push(format!("{provider_id}.{key}"));
            }
        }

        if !matches!(provider_obj.get("models"), Some(Value::Array(_))) {
            provider_obj.insert("models".to_string(), Value::Array(Vec::new()));
            changed = true;
            notes.push(format!("{provider_id}.models"));
        }

        let Some(models_array) = provider_obj.get_mut("models").and_then(Value::as_array_mut)
        else {
            continue;
        };

        let existing_models = std::mem::take(models_array);
        let mut next_models = Vec::with_capacity(existing_models.len());
        for (index, model_value) in existing_models.into_iter().enumerate() {
            let Some(mut model_obj) = model_value.as_object().cloned() else {
                changed = true;
                notes.push(format!("{provider_id}.models[{index}]"));
                continue;
            };

            let removed_model_keys =
                remove_unknown_keys_from_object(&mut model_obj, &model_allowed_keys);
            if !removed_model_keys.is_empty() {
                changed = true;
                for key in removed_model_keys {
                    notes.push(format!("{provider_id}.models[{index}].{key}"));
                }
            }

            let model_id = model_obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let model_name = model_obj
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);

            match (model_id, model_name) {
                (None, None) => {
                    changed = true;
                    notes.push(format!("{provider_id}.models[{index}].id/name"));
                    continue;
                }
                (None, Some(name)) => {
                    model_obj.insert("id".to_string(), Value::String(name.clone()));
                    changed = true;
                    notes.push(format!("{provider_id}.models[{index}].id"));
                    model_obj.insert("name".to_string(), Value::String(name));
                }
                (Some(id), None) => {
                    model_obj.insert("name".to_string(), Value::String(id));
                    changed = true;
                    notes.push(format!("{provider_id}.models[{index}].name"));
                }
                (Some(_), Some(_)) => {}
            }

            next_models.push(Value::Object(model_obj));
        }
        *models_array = next_models;
    }

    if !changed {
        return Ok(None);
    }

    write_openclaw_config_value(config_path, &parsed)?;
    let summary = if notes.is_empty() {
        "providers schema normalized".to_string()
    } else if notes.len() > 8 {
        format!(
            "{} 等 {} 项",
            notes.iter().take(8).cloned().collect::<Vec<_>>().join(", "),
            notes.len()
        )
    } else {
        notes.join(", ")
    };
    Ok(Some(format!("{} -> {}", config_path.display(), summary)))
}

fn sanitize_openclaw_models_provider_schema() -> Result<Option<String>, String> {
    let mut changed_paths = Vec::new();
    let mut failures = Vec::new();
    for path in collect_openclaw_candidate_config_paths() {
        match sanitize_openclaw_models_provider_schema_at_path(&path) {
            Ok(Some(detail)) => changed_paths.push(detail),
            Ok(None) => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    if changed_paths.is_empty() && failures.is_empty() {
        return Ok(None);
    }
    if !changed_paths.is_empty() && failures.is_empty() {
        return Ok(Some(format!(
            "已自动清理 OpenClaw 模型配置中的不兼容字段：{}",
            changed_paths.join("；")
        )));
    }
    if changed_paths.is_empty() && !failures.is_empty() {
        return Err(format!(
            "清理 OpenClaw 模型配置失败：{}",
            failures.join("；")
        ));
    }

    Ok(Some(format!(
        "已部分清理 OpenClaw 模型配置：{}；剩余失败：{}",
        changed_paths.join("；"),
        failures.join("；")
    )))
}

fn sanitize_openclaw_channel_schema_at_path(config_path: &Path) -> Result<Option<String>, String> {
    let raw = match std::fs::read_to_string(config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let Some(root_obj) = parsed.as_object_mut() else {
        return Ok(None);
    };

    let mut changed = false;
    let mut notes = Vec::new();

    if let Some(channels_obj) = root_obj.get_mut("channels").and_then(Value::as_object_mut) {
        let channel_keys = channels_obj.keys().cloned().collect::<Vec<_>>();
        let mut merge_targets = HashSet::new();
        for channel_key in channel_keys {
            let normalized = normalize_channel_identifier(&channel_key);
            let canonical = normalize_channel_identifier_for_openclaw_config(&channel_key);
            if canonical != normalized {
                merge_targets.insert(canonical.clone());
                notes.push(format!("channels.{channel_key}->channels.{canonical}"));
            }
        }
        if !merge_targets.is_empty() {
            changed = true;
            for target in merge_targets {
                let _ = merge_channel_alias_sections(channels_obj, &target)?;
            }
        }
    }

    if let Some(bindings_arr) = root_obj.get_mut("bindings").and_then(Value::as_array_mut) {
        for (index, binding_value) in bindings_arr.iter_mut().enumerate() {
            let Some(binding_obj) = binding_value.as_object_mut() else {
                continue;
            };
            let Some(match_obj) = binding_obj.get_mut("match").and_then(Value::as_object_mut)
            else {
                continue;
            };
            let Some(channel_raw) = match_obj
                .get("channel")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };

            let canonical = normalize_channel_identifier_for_openclaw_config(channel_raw);
            let normalized = normalize_channel_identifier(channel_raw);
            if canonical != normalized {
                match_obj.insert("channel".to_string(), Value::String(canonical.clone()));
                changed = true;
                notes.push(format!(
                    "bindings[{index}].match.channel: {normalized}->{canonical}"
                ));
            }
        }
    }

    if !changed {
        return Ok(None);
    }

    write_openclaw_config_value(config_path, &parsed)?;
    let summary = if notes.is_empty() {
        "channel schema normalized".to_string()
    } else if notes.len() > 8 {
        format!(
            "{} 等 {} 项",
            notes.iter().take(8).cloned().collect::<Vec<_>>().join(", "),
            notes.len()
        )
    } else {
        notes.join(", ")
    };
    Ok(Some(format!("{} -> {}", config_path.display(), summary)))
}

fn sanitize_openclaw_channel_schema() -> Result<Option<String>, String> {
    let mut changed_paths = Vec::new();
    let mut failures = Vec::new();
    for path in collect_openclaw_candidate_config_paths() {
        match sanitize_openclaw_channel_schema_at_path(&path) {
            Ok(Some(detail)) => changed_paths.push(detail),
            Ok(None) => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    if changed_paths.is_empty() && failures.is_empty() {
        return Ok(None);
    }
    if !changed_paths.is_empty() && failures.is_empty() {
        return Ok(Some(format!(
            "已自动迁移 OpenClaw 频道配置中的旧 channel id：{}",
            changed_paths.join("；")
        )));
    }
    if changed_paths.is_empty() && !failures.is_empty() {
        return Err(format!(
            "迁移 OpenClaw 频道配置失败：{}",
            failures.join("；")
        ));
    }

    Ok(Some(format!(
        "已部分迁移 OpenClaw 频道配置：{}；剩余失败：{}",
        changed_paths.join("；"),
        failures.join("；")
    )))
}

fn normalize_path_for_plugin_compare(path: &Path) -> String {
    let display = path.display().to_string();
    #[cfg(target_os = "windows")]
    {
        display.replace('\\', "/").to_ascii_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        display.replace('\\', "/")
    }
}

fn sanitize_openclaw_plugin_load_paths_at_path(config_path: &Path) -> Result<Option<String>, String> {
    let raw = match std::fs::read_to_string(config_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let Some(root_obj) = parsed.as_object_mut() else {
        return Ok(None);
    };
    let Some(plugins_obj) = root_obj.get_mut("plugins").and_then(Value::as_object_mut) else {
        return Ok(None);
    };
    let Some(load_obj) = plugins_obj.get_mut("load").and_then(Value::as_object_mut) else {
        return Ok(None);
    };
    let Some(paths_arr) = load_obj.get_mut("paths").and_then(Value::as_array_mut) else {
        return Ok(None);
    };

    let project_root = resolve_project_root();
    let project_root_norm = normalize_path_for_plugin_compare(&project_root);
    let project_prefix = format!("{project_root_norm}/");

    let mut changed = false;
    let mut removed_project_paths = 0usize;
    let mut seen = HashSet::new();
    let mut next_paths = Vec::new();

    for value in paths_arr.iter() {
        let Some(path_raw) = value.as_str() else {
            changed = true;
            continue;
        };
        let trimmed = path_raw.trim();
        if trimmed.is_empty() {
            changed = true;
            continue;
        }

        let normalized = normalize_path_for_plugin_compare(Path::new(trimmed));
        if normalized == project_root_norm || normalized.starts_with(&project_prefix) {
            changed = true;
            removed_project_paths = removed_project_paths.saturating_add(1);
            continue;
        }

        if !seen.insert(normalized) {
            changed = true;
            continue;
        }

        next_paths.push(Value::String(trimmed.to_string()));
    }

    if !changed {
        return Ok(None);
    }

    *paths_arr = next_paths;
    write_openclaw_config_value(config_path, &parsed)?;
    Ok(Some(format!(
        "{} -> removed_project_paths={removed_project_paths}",
        config_path.display()
    )))
}

fn sanitize_openclaw_plugin_load_paths() -> Result<Option<String>, String> {
    let mut changed_paths = Vec::new();
    let mut failures = Vec::new();
    for path in collect_openclaw_candidate_config_paths() {
        match sanitize_openclaw_plugin_load_paths_at_path(&path) {
            Ok(Some(detail)) => changed_paths.push(detail),
            Ok(None) => {}
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }

    if changed_paths.is_empty() && failures.is_empty() {
        return Ok(None);
    }
    if !changed_paths.is_empty() && failures.is_empty() {
        return Ok(Some(format!(
            "已清理 plugins.load.paths 中的项目内路径与重复项：{}",
            changed_paths.join("；")
        )));
    }
    if changed_paths.is_empty() && !failures.is_empty() {
        return Err(format!(
            "清理 plugins.load.paths 失败：{}",
            failures.join("；")
        ));
    }

    Ok(Some(format!(
        "已部分清理 plugins.load.paths：{}；剩余失败：{}",
        changed_paths.join("；"),
        failures.join("；")
    )))
}

fn resolve_user_home_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let trimmed = user_profile.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }

        let home_drive = std::env::var("HOMEDRIVE")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let home_path = std::env::var("HOMEPATH")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if let (Some(drive), Some(path)) = (home_drive, home_path) {
            return Some(PathBuf::from(format!("{drive}{path}")));
        }
    }

    if let Ok(home_dir) = std::env::var("HOME") {
        let trimmed = home_dir.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    None
}

fn resolve_default_openclaw_home_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(home_dir) = resolve_user_home_path() {
            return home_dir.join(".openclaw");
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let trimmed = local_app_data.trim();
            if !trimmed.is_empty() {
                return PathBuf::from(trimmed).join("OpenClaw");
            }
        }
        if let Ok(app_data) = std::env::var("APPDATA") {
            let trimmed = app_data.trim();
            if !trimmed.is_empty() {
                return PathBuf::from(trimmed).join("OpenClaw");
            }
        }
        return std::env::temp_dir().join("openclaw");
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home_dir) = resolve_user_home_path() {
            return home_dir.join(".openclaw");
        }
        PathBuf::from(".openclaw")
    }
}

fn resolve_default_openclaw_config_path() -> PathBuf {
    resolve_default_openclaw_home_path().join("openclaw.json")
}

fn resolve_openclaw_home_path() -> PathBuf {
    resolve_default_openclaw_home_path()
}

fn resolve_workspace_main_root() -> PathBuf {
    resolve_openclaw_home_path().join("workspace-main")
}

fn resolve_workspace_agents_root() -> PathBuf {
    resolve_openclaw_home_path().join("workspaces")
}

fn expand_home_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return resolve_openclaw_home_path();
    }

    if trimmed == "~" {
        return resolve_user_home_path().unwrap_or_else(|| PathBuf::from(trimmed));
    }

    if let Some(suffix) = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"))
    {
        return resolve_user_home_path()
            .map(|home| home.join(suffix))
            .unwrap_or_else(|| PathBuf::from(trimmed));
    }

    PathBuf::from(trimmed)
}

fn resolve_workspace_root_for_agent(agent_id: &str, configured_workspace: Option<&str>) -> PathBuf {
    if let Some(workspace) = configured_workspace {
        let expanded = expand_home_path(workspace);
        if expanded.is_absolute() {
            return expanded;
        }
        return resolve_openclaw_home_path().join(expanded);
    }

    let preferred = resolve_openclaw_home_path().join(format!("workspace-{agent_id}"));
    if preferred.exists() {
        return preferred;
    }

    let legacy = resolve_workspace_agents_root().join(agent_id);
    if legacy.exists() {
        return legacy;
    }

    preferred
}

fn load_staff_mission_statement() -> String {
    let fallback = "构建可持续自治的 AI 员工体系，持续完成高价值任务。".to_string();
    let current_dir = match std::env::current_dir() {
        Ok(value) => value,
        Err(_) => return fallback,
    };
    let agent_path = current_dir.join("AGENTS.md");
    let raw = match std::fs::read_to_string(agent_path) {
        Ok(value) => value,
        Err(_) => return fallback,
    };

    raw.lines()
        .map(str::trim)
        .find(|line| line.starts_with("- ") && line.to_ascii_lowercase().contains("objective"))
        .map(|line| line.trim_start_matches('-').trim().to_string())
        .filter(|line| line.len() > 8)
        .unwrap_or(fallback)
}

fn value_as_object(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    value.as_object()
}

fn read_string_or_primary<'a>(value: &'a Value) -> Option<&'a str> {
    value
        .as_str()
        .or_else(|| {
            value_as_object(value)
                .and_then(|obj| obj.get("primary"))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_provider_id(raw: &str) -> String {
    let mut cleaned = String::new();
    for ch in raw.trim().chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            cleaned.push(ch.to_ascii_lowercase());
        } else if ch == '/' || ch == ':' || ch == '.' || ch.is_whitespace() {
            cleaned.push('-');
        }
    }

    let normalized = cleaned
        .trim_matches('-')
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if normalized.is_empty() {
        "platform".to_string()
    } else {
        normalized
    }
}

fn humanize_provider_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "未命名平台".to_string();
    }

    let parts = trimmed
        .split(|ch: char| ch == '-' || ch == '_' || ch == '/' || ch == ':' || ch == '.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return trimmed.to_string();
    }

    let words = parts
        .into_iter()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            match lower.as_str() {
                "openai" => "OpenAI".to_string(),
                "api" => "API".to_string(),
                "ai" => "AI".to_string(),
                "aws" => "AWS".to_string(),
                _ => {
                    if !part.is_ascii() {
                        return part.to_string();
                    }
                    let mut chars = lower.chars();
                    match chars.next() {
                        Some(first) => {
                            let mut word = first.to_ascii_uppercase().to_string();
                            word.push_str(chars.as_str());
                            word
                        }
                        None => String::new(),
                    }
                }
            }
        })
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    if words.is_empty() {
        trimmed.to_string()
    } else {
        words.join(" ")
    }
}

#[allow(dead_code)]
fn infer_provider_name_from_base_url(base_url: &str) -> Option<String> {
    let raw = base_url.trim();
    if raw.is_empty() {
        return None;
    }

    let candidate = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };

    let mut hostname = reqwest::Url::parse(&candidate)
        .ok()
        .and_then(|url| {
            url.host_str()
                .map(|value| value.trim().to_ascii_lowercase())
        })
        .unwrap_or_default();

    if hostname.is_empty() {
        let without_scheme = raw
            .strip_prefix("http://")
            .or_else(|| raw.strip_prefix("https://"))
            .unwrap_or(raw);
        hostname = without_scheme
            .split(['/', '?', '#'])
            .next()
            .unwrap_or_default()
            .split(':')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
    }

    if hostname.is_empty() {
        return None;
    }

    let parts = hostname
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }

    let mut index = parts.len().saturating_sub(2);
    if parts.len() >= 3 {
        let top_level = parts[parts.len() - 1];
        let second_level = parts[parts.len() - 2];
        if top_level.len() == 2 && second_level.len() <= 3 {
            index = parts.len().saturating_sub(3);
        }
    }

    let token = parts
        .get(index)
        .copied()
        .unwrap_or_default()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>();
    let token = token.trim_matches('-').trim_matches('_').to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn infer_platform_protocol(api_value: Option<&str>) -> String {
    let api = api_value
        .unwrap_or("openai-completions")
        .to_ascii_lowercase();
    if api.contains("anthropic") {
        "anthropic".to_string()
    } else {
        "openai".to_string()
    }
}

fn infer_platform_api_path(protocol: &str, api_value: Option<&str>, base_url: &str) -> String {
    let api = api_value.unwrap_or("").trim().to_ascii_lowercase();
    if api.contains("openai-responses") {
        if base_url
            .trim_end_matches('/')
            .to_ascii_lowercase()
            .ends_with("/v1")
        {
            return "/responses".to_string();
        }
        return "/v1/responses".to_string();
    }

    let normalized_base = base_url.trim_end_matches('/').to_ascii_lowercase();
    if protocol.eq_ignore_ascii_case("anthropic") {
        if normalized_base.ends_with("/v1") {
            "/messages".to_string()
        } else {
            "/v1/messages".to_string()
        }
    } else if normalized_base.ends_with("/v1") {
        "/chat/completions".to_string()
    } else {
        "/v1/chat/completions".to_string()
    }
}

fn is_local_proxy_host(url: &reqwest::Url) -> bool {
    matches!(url.host_str(), Some("localhost") | Some("127.0.0.1"))
}

fn normalize_local_proxy_base_url_for_persist(base_url: &str) -> String {
    let normalized = base_url.trim().trim_end_matches('/').to_string();
    if normalized.is_empty() {
        return normalized;
    }

    let Ok(mut url) = reqwest::Url::parse(&normalized) else {
        return normalized;
    };
    if !is_local_proxy_host(&url) {
        return normalized;
    }

    let mut path = url.path().trim_end_matches('/').to_string();
    let lower = path.to_ascii_lowercase();
    if lower.ends_with("/v1/chat/completions") {
        path = path[..path.len() - "/v1/chat/completions".len()].to_string();
    } else if lower.ends_with("/chat/completions") {
        path = path[..path.len() - "/chat/completions".len()].to_string();
    } else if lower.ends_with("/v1/responses") {
        path = path[..path.len() - "/v1/responses".len()].to_string();
    } else if lower.ends_with("/responses") {
        path = path[..path.len() - "/responses".len()].to_string();
    } else if lower.ends_with("/v1/messages") {
        path = path[..path.len() - "/v1/messages".len()].to_string();
    } else if lower.ends_with("/messages") {
        path = path[..path.len() - "/messages".len()].to_string();
    }

    if path.to_ascii_lowercase().ends_with("/v1") {
        path = path[..path.len() - "/v1".len()].to_string();
    }

    url.set_path(if path.is_empty() { "/" } else { &path });
    url.set_query(None);
    url.set_fragment(None);
    url.to_string().trim_end_matches('/').to_string()
}

fn write_openclaw_config_value(source_path: &Path, parsed: &Value) -> Result<(), String> {
    let output = serde_json::to_string_pretty(parsed)
        .map_err(|error| format!("序列化 openclaw.json 失败: {error}"))?;
    match write_openclaw_config_to_path(source_path, &output) {
        Ok(()) => Ok(()),
        Err(primary_error) if primary_error.is_permission_denied() => {
            let fallback_path = resolve_default_openclaw_config_path();
            if fallback_path == source_path {
                return Err(primary_error.describe());
            }

            write_openclaw_config_to_path(&fallback_path, &output).map_err(|fallback_error| {
                format!(
                    "{}；已尝试回退到 {}，但仍失败：{}",
                    primary_error.describe(),
                    fallback_path.display(),
                    fallback_error.describe()
                )
            })?;
            set_openclaw_runtime_paths(&fallback_path);
            Ok(())
        }
        Err(error) => Err(error.describe()),
    }
}

#[derive(Debug)]
enum OpenClawConfigWriteStage {
    EnsureDir(PathBuf),
    WriteFile(PathBuf),
}

#[derive(Debug)]
struct OpenClawConfigWriteError {
    stage: OpenClawConfigWriteStage,
    error: std::io::Error,
}

impl OpenClawConfigWriteError {
    fn is_permission_denied(&self) -> bool {
        self.error.kind() == std::io::ErrorKind::PermissionDenied
    }

    fn describe(&self) -> String {
        match &self.stage {
            OpenClawConfigWriteStage::EnsureDir(path) => {
                format!(
                    "创建 openclaw 配置目录失败（{}）: {}",
                    path.display(),
                    self.error
                )
            }
            OpenClawConfigWriteStage::WriteFile(path) => {
                format!(
                    "写入 openclaw.json 失败（{}）: {}",
                    path.display(),
                    self.error
                )
            }
        }
    }
}

fn write_openclaw_config_to_path(
    source_path: &Path,
    output: &str,
) -> Result<(), OpenClawConfigWriteError> {
    if let Some(parent) = source_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| OpenClawConfigWriteError {
            stage: OpenClawConfigWriteStage::EnsureDir(parent.to_path_buf()),
            error,
        })?;
    }
    if let Ok(existing) = std::fs::read_to_string(source_path) {
        if existing == output {
            return Ok(());
        }
    }
    std::fs::write(source_path, output).map_err(|error| OpenClawConfigWriteError {
        stage: OpenClawConfigWriteStage::WriteFile(source_path.to_path_buf()),
        error,
    })?;
    Ok(())
}

fn set_openclaw_runtime_paths(config_path: &Path) {
    std::env::set_var("OPENCLAW_CONFIG_PATH", config_path);
    if let Some(home_path) = config_path.parent() {
        std::env::set_var("OPENCLAW_HOME", home_path);
    }
}

/// 从 openclaw.json 的 bindings 中按 agentId 匹配，收集每个 agent 的 match.channel，多个用 ", " 拼接。
fn resolve_channels_from_bindings(
    root: &serde_json::Map<String, Value>,
) -> std::collections::HashMap<String, String> {
    let mut by_agent: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    if let Some(arr) = root.get("bindings").and_then(Value::as_array) {
        for item in arr {
            if let Some(obj) = value_as_object(item) {
                let agent_id = obj
                    .get("agentId")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                let channel = obj
                    .get("match")
                    .and_then(value_as_object)
                    .and_then(|m| m.get("channel"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                if let (Some(a), Some(c)) = (agent_id, channel) {
                    by_agent
                        .entry(a.to_string())
                        .or_default()
                        .push(c.to_string());
                }
            }
        }
    }
    by_agent
        .into_iter()
        .map(|(k, v)| (k, v.join(", ")))
        .collect()
}

fn normalize_channel_identifier(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn is_weixin_channel_identifier(normalized_channel: &str) -> bool {
    matches!(
        normalized_channel,
        "weixin"
            | "wechat"
            | "wx"
            | "wechat_official_account"
            | "wechat-official-account"
            | "openclaw-weixin"
            | "openclaw_weixin"
    )
}

fn is_wecom_channel_identifier(normalized_channel: &str) -> bool {
    matches!(
        normalized_channel,
        "wecom"
            | "workwechat"
            | "work-wechat"
            | "work_wechat"
            | "wechatwork"
            | "qywx"
            | "openclaw-wecom"
            | "openclaw_wecom"
    )
}

fn is_whatsapp_channel_identifier(normalized_channel: &str) -> bool {
    matches!(
        normalized_channel,
        "whatsapp" | "wa" | "wacli" | "openclaw-whatsapp" | "openclaw_whatsapp"
    )
}

fn normalize_channel_identifier_for_openclaw_config(raw: &str) -> String {
    let normalized = normalize_channel_identifier(raw);
    if is_weixin_channel_identifier(&normalized) {
        "openclaw-weixin".to_string()
    } else if is_wecom_channel_identifier(&normalized) {
        "wecom".to_string()
    } else if is_whatsapp_channel_identifier(&normalized) {
        "whatsapp".to_string()
    } else {
        normalized
    }
}

fn normalize_account_identifier(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn is_default_channel_account_id(raw: &str) -> bool {
    normalize_account_identifier(raw) == "default"
}

fn channel_account_binding_key(channel_type: &str, account_id: &str) -> String {
    format!(
        "{}:{}",
        normalize_channel_identifier(channel_type),
        normalize_account_identifier(account_id)
    )
}

fn resolve_channel_binding_maps(
    root: &serde_json::Map<String, Value>,
) -> (
    std::collections::HashMap<String, String>,
    std::collections::HashMap<String, String>,
) {
    let mut channel_to_agent = std::collections::HashMap::new();
    let mut account_to_agent = std::collections::HashMap::new();
    let Some(bindings) = root.get("bindings").and_then(Value::as_array) else {
        return (channel_to_agent, account_to_agent);
    };

    for item in bindings {
        let Some(binding_obj) = item.as_object() else {
            continue;
        };
        let Some(agent_id) = binding_obj
            .get("agentId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(channel_type) = binding_obj
            .get("match")
            .and_then(Value::as_object)
            .and_then(|match_obj| match_obj.get("channel"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        let normalized_channel = normalize_channel_identifier(channel_type);
        let account_id = binding_obj
            .get("match")
            .and_then(Value::as_object)
            .and_then(|match_obj| match_obj.get("accountId"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .filter(|value| !is_default_channel_account_id(value));

        if let Some(account_id) = account_id {
            account_to_agent.insert(
                channel_account_binding_key(&normalized_channel, account_id),
                agent_id.to_string(),
            );
        } else {
            channel_to_agent.insert(normalized_channel, agent_id.to_string());
        }
    }

    (channel_to_agent, account_to_agent)
}

fn is_channel_section_reserved_key(key: &str) -> bool {
    matches!(key, "accounts" | "defaultAccount" | "enabled")
}

fn channel_payload_has_content(payload: &serde_json::Map<String, Value>) -> bool {
    payload.iter().any(|(key, value)| {
        if key == "enabled" || key == "name" {
            return false;
        }
        match value {
            Value::Null => false,
            Value::String(text) => !text.trim().is_empty(),
            Value::Array(items) => !items.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            _ => true,
        }
    })
}

fn resolve_channel_plugin_account_store_paths(channel_type: &str) -> Vec<PathBuf> {
    let openclaw_home = resolve_openclaw_home_path();
    match normalize_channel_identifier_for_openclaw_config(channel_type).as_str() {
        "openclaw-weixin" => vec![
            openclaw_home.join("openclaw-weixin").join("accounts.json"),
            openclaw_home.join("weixin").join("accounts.json"),
            openclaw_home.join("wechat").join("accounts.json"),
        ],
        "wecom" => vec![
            openclaw_home.join("wecom").join("accounts.json"),
            openclaw_home.join("openclaw-wecom").join("accounts.json"),
            openclaw_home.join("wecom-openclaw").join("accounts.json"),
            openclaw_home.join("work-wechat").join("accounts.json"),
        ],
        "whatsapp" => vec![
            openclaw_home.join("whatsapp").join("accounts.json"),
            openclaw_home.join("openclaw-whatsapp").join("accounts.json"),
            openclaw_home.join("wacli").join("accounts.json"),
        ],
        _ => Vec::new(),
    }
}

fn parse_channel_plugin_account_ids(value: &Value) -> Vec<String> {
    let mut account_ids = Vec::new();

    if let Some(items) = value.as_array() {
        for item in items {
            let Some(raw) = item.as_str().map(str::trim).filter(|text| !text.is_empty()) else {
                continue;
            };
            account_ids.push(raw.to_string());
        }
        return account_ids;
    }

    let Some(obj) = value.as_object() else {
        return account_ids;
    };
    if let Some(items) = obj.get("accounts").and_then(Value::as_array) {
        for item in items {
            let Some(raw) = item.as_str().map(str::trim).filter(|text| !text.is_empty()) else {
                continue;
            };
            account_ids.push(raw.to_string());
        }
        return account_ids;
    }
    for key in obj.keys() {
        let normalized = key.trim();
        if normalized.is_empty() {
            continue;
        }
        account_ids.push(normalized.to_string());
    }
    account_ids
}

fn load_channel_plugin_account_configs(
    channel_type: &str,
) -> Result<Vec<(String, serde_json::Map<String, Value>)>, String> {
    let mut output = Vec::<(String, serde_json::Map<String, Value>)>::new();
    let mut seen_accounts = HashSet::<String>::new();

    for accounts_index_path in resolve_channel_plugin_account_store_paths(channel_type) {
        if !accounts_index_path.exists() || !accounts_index_path.is_file() {
            continue;
        }
        let raw = std::fs::read_to_string(&accounts_index_path).map_err(|error| {
            format!(
                "读取频道账号索引失败（{}）：{error}",
                accounts_index_path.display()
            )
        })?;
        let parsed: Value = serde_json::from_str(&raw).map_err(|error| {
            format!(
                "解析频道账号索引失败（{}）：{error}",
                accounts_index_path.display()
            )
        })?;

        let account_ids = parse_channel_plugin_account_ids(&parsed);
        if account_ids.is_empty() {
            continue;
        }

        let Some(channel_state_dir) = accounts_index_path.parent() else {
            continue;
        };
        let accounts_dir = channel_state_dir.join("accounts");

        for account_id in account_ids {
            let normalized_key = normalize_account_identifier(&account_id);
            if normalized_key.is_empty() || !seen_accounts.insert(normalized_key) {
                continue;
            }

            let mut config_obj = serde_json::Map::<String, Value>::new();
            config_obj.insert(
                "pluginAccountId".to_string(),
                Value::String(account_id.clone()),
            );
            config_obj.insert("enabled".to_string(), Value::Bool(true));

            let detail_path = accounts_dir.join(format!("{account_id}.json"));
            if detail_path.exists() && detail_path.is_file() {
                if let Ok(detail_raw) = std::fs::read_to_string(&detail_path) {
                    if let Ok(detail_parsed) = serde_json::from_str::<Value>(&detail_raw) {
                        if let Some(detail_obj) = detail_parsed.as_object() {
                            if let Some(user_id) = detail_obj
                                .get("userId")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|text| !text.is_empty())
                            {
                                config_obj.insert(
                                    "userId".to_string(),
                                    Value::String(user_id.to_string()),
                                );
                            }
                            if let Some(base_url) = detail_obj
                                .get("baseUrl")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|text| !text.is_empty())
                            {
                                config_obj.insert(
                                    "baseUrl".to_string(),
                                    Value::String(base_url.to_string()),
                                );
                            }
                            if let Some(name) = detail_obj
                                .get("name")
                                .or_else(|| detail_obj.get("nickname"))
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|text| !text.is_empty())
                            {
                                config_obj
                                    .insert("name".to_string(), Value::String(name.to_string()));
                            }
                        }
                    }
                }
            }

            output.push((account_id, config_obj));
        }
    }

    Ok(output)
}

fn sync_channel_accounts_from_plugin_store(channel_type: &str) -> Result<bool, String> {
    let normalized_channel = normalize_channel_identifier(channel_type);
    if !is_weixin_channel_identifier(&normalized_channel)
        && !is_wecom_channel_identifier(&normalized_channel)
        && !is_whatsapp_channel_identifier(&normalized_channel)
    {
        return Ok(false);
    }
    let channel_config_key = normalize_channel_identifier_for_openclaw_config(channel_type);

    let plugin_accounts = load_channel_plugin_account_configs(&channel_config_key)?;
    if plugin_accounts.is_empty() {
        return Ok(false);
    }

    let source_path = resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(error) => {
            return Err(format!(
                "读取 openclaw.json 失败（{}）：{error}",
                source_path.display()
            ))
        }
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }

    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
    ensure_channel_plugin_allowlist(root, &channel_config_key)?;

    if !matches!(root.get("channels"), Some(Value::Object(_))) {
        root.insert(
            "channels".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let channels_obj = root
        .get_mut("channels")
        .and_then(Value::as_object_mut)
        .ok_or("channels 不是对象")?;
    let _ = merge_channel_alias_sections(channels_obj, &channel_config_key)?;
    let section_obj = channels_obj
        .get_mut(&channel_config_key)
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType> 不是对象")?;
    migrate_legacy_channel_section_to_accounts(section_obj);
    if !matches!(section_obj.get("accounts"), Some(Value::Object(_))) {
        section_obj.insert(
            "accounts".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let accounts_obj = section_obj
        .get_mut("accounts")
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType>.accounts 不是对象")?;

    let mut changed = false;
    let mut first_account_id = String::new();
    for (account_id, plugin_config) in plugin_accounts {
        if first_account_id.is_empty() {
            first_account_id = account_id.clone();
        }
        let current = accounts_obj
            .get(&account_id)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let mut merged = current;
        for (key, value) in plugin_config {
            if merged.get(&key) != Some(&value) {
                merged.insert(key, value);
                changed = true;
            }
        }
        if accounts_obj.get(&account_id) != Some(&Value::Object(merged.clone())) {
            accounts_obj.insert(account_id, Value::Object(merged));
            changed = true;
        }
    }

    if section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
        && !first_account_id.is_empty()
    {
        section_obj.insert(
            "defaultAccount".to_string(),
            Value::String(first_account_id),
        );
        changed = true;
    }
    if section_obj.get("enabled").and_then(Value::as_bool) != Some(true) {
        section_obj.insert("enabled".to_string(), Value::Bool(true));
        changed = true;
    }
    mirror_default_account_to_channel_section(section_obj);

    if !changed {
        return Ok(true);
    }
    write_openclaw_config_value(&source_path, &parsed)?;
    Ok(true)
}

fn resolve_channel_config_verification_aliases(channel_type: &str) -> HashSet<String> {
    let normalized = normalize_channel_identifier(channel_type);
    let mut aliases = vec![normalized.clone()];
    if is_weixin_channel_identifier(&normalized) {
        aliases.extend([
            "openclaw-weixin".to_string(),
            "wechat".to_string(),
            "wx".to_string(),
            "openclaw_weixin".to_string(),
            "weixin".to_string(),
            "wechat_official_account".to_string(),
            "wechat-official-account".to_string(),
        ]);
    } else if is_wecom_channel_identifier(&normalized) {
        aliases.extend([
            "wecom".to_string(),
            "workwechat".to_string(),
            "work-wechat".to_string(),
            "work_wechat".to_string(),
            "wechatwork".to_string(),
            "qywx".to_string(),
            "openclaw-wecom".to_string(),
            "openclaw_wecom".to_string(),
        ]);
    } else if is_whatsapp_channel_identifier(&normalized) {
        aliases.extend([
            "whatsapp".to_string(),
            "wa".to_string(),
            "wacli".to_string(),
            "openclaw-whatsapp".to_string(),
            "openclaw_whatsapp".to_string(),
        ]);
    }

    aliases
        .into_iter()
        .map(|value| normalize_channel_identifier(&value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn resolve_channel_section_from_channels_obj<'a>(
    channels_obj: &'a serde_json::Map<String, Value>,
    channel_type: &str,
) -> Option<&'a serde_json::Map<String, Value>> {
    let preferred = normalize_channel_identifier_for_openclaw_config(channel_type);
    let aliases = resolve_channel_config_verification_aliases(channel_type);

    if let Some(section_obj) = channels_obj.get(&preferred).and_then(Value::as_object) {
        return Some(section_obj);
    }

    for (channel_key, section_value) in channels_obj {
        if !aliases.contains(&normalize_channel_identifier(channel_key)) {
            continue;
        }
        if let Some(section_obj) = section_value.as_object() {
            return Some(section_obj);
        }
    }

    None
}

fn merge_channel_section_from_alias(
    target: &mut serde_json::Map<String, Value>,
    source: &serde_json::Map<String, Value>,
) {
    migrate_legacy_channel_section_to_accounts(target);
    let mut source_clone = source.clone();
    migrate_legacy_channel_section_to_accounts(&mut source_clone);

    if !matches!(target.get("accounts"), Some(Value::Object(_))) {
        target.insert(
            "accounts".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    if let (Some(target_accounts), Some(source_accounts)) = (
        target.get_mut("accounts").and_then(Value::as_object_mut),
        source_clone.get("accounts").and_then(Value::as_object),
    ) {
        for (account_id, source_account_value) in source_accounts {
            let Some(source_account_obj) = source_account_value.as_object() else {
                continue;
            };
            if !matches!(target_accounts.get(account_id), Some(Value::Object(_))) {
                target_accounts.insert(
                    account_id.clone(),
                    Value::Object(source_account_obj.clone()),
                );
                continue;
            }
            if let Some(target_account_obj) = target_accounts
                .get_mut(account_id)
                .and_then(Value::as_object_mut)
            {
                for (key, value) in source_account_obj {
                    if !target_account_obj.contains_key(key) {
                        target_account_obj.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    if target
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        if let Some(default_account) = source_clone
            .get("defaultAccount")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            target.insert(
                "defaultAccount".to_string(),
                Value::String(default_account.to_string()),
            );
        }
    }

    for (key, value) in &source_clone {
        if is_channel_section_reserved_key(key) {
            continue;
        }
        if !target.contains_key(key) {
            target.insert(key.clone(), value.clone());
        }
    }

    let source_enabled = source_clone
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if source_enabled && target.get("enabled").and_then(Value::as_bool) != Some(true) {
        target.insert("enabled".to_string(), Value::Bool(true));
    }

    mirror_default_account_to_channel_section(target);
}

fn merge_channel_alias_sections(
    channels_obj: &mut serde_json::Map<String, Value>,
    channel_type: &str,
) -> Result<String, String> {
    let canonical_key = normalize_channel_identifier_for_openclaw_config(channel_type);
    if canonical_key.is_empty() {
        return Err("channelType 不能为空".to_string());
    }

    if !matches!(channels_obj.get(&canonical_key), Some(Value::Object(_))) {
        channels_obj.insert(
            canonical_key.clone(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }

    let aliases = resolve_channel_config_verification_aliases(channel_type);
    let keys_to_merge = channels_obj
        .keys()
        .cloned()
        .filter(|channel_key| {
            let normalized_key = normalize_channel_identifier(channel_key);
            normalized_key != canonical_key && aliases.contains(&normalized_key)
        })
        .collect::<Vec<_>>();

    for alias_key in keys_to_merge {
        let Some(source_value) = channels_obj.remove(&alias_key) else {
            continue;
        };
        let Some(source_obj) = source_value.as_object() else {
            continue;
        };
        let target_obj = channels_obj
            .get_mut(&canonical_key)
            .and_then(Value::as_object_mut)
            .ok_or("channels.<channelType> 不是对象")?;
        merge_channel_section_from_alias(target_obj, source_obj);
    }

    let section_obj = channels_obj
        .get_mut(&canonical_key)
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType> 不是对象")?;
    migrate_legacy_channel_section_to_accounts(section_obj);
    Ok(canonical_key)
}

fn channel_section_has_configured_accounts(section_obj: &serde_json::Map<String, Value>) -> bool {
    let mut section_clone = section_obj.clone();
    migrate_legacy_channel_section_to_accounts(&mut section_clone);
    if section_clone
        .get("enabled")
        .and_then(Value::as_bool)
        .is_some_and(|enabled| !enabled)
    {
        return false;
    }

    let Some(accounts_obj) = section_clone.get("accounts").and_then(Value::as_object) else {
        return false;
    };
    accounts_obj
        .values()
        .filter_map(Value::as_object)
        .any(|account_obj| {
            account_obj
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true)
                && channel_payload_has_content(account_obj)
        })
}

fn has_configured_channel_account_in_openclaw_config(channel_type: &str) -> Result<bool, String> {
    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "读取 openclaw.json 失败（{}）：{error}",
                source_path.display()
            ))
        }
    };
    let parsed: Value = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "openclaw.json 解析失败（{}）：{error}",
            source_path.display()
        )
    })?;
    let Some(root) = value_as_object(&parsed) else {
        return Ok(false);
    };
    let Some(channels_obj) = root.get("channels").and_then(Value::as_object) else {
        return Ok(false);
    };

    let aliases = resolve_channel_config_verification_aliases(channel_type);
    if aliases.is_empty() {
        return Ok(false);
    }

    for (channel_key, section_value) in channels_obj {
        if !aliases.contains(&normalize_channel_identifier(channel_key)) {
            continue;
        }
        let Some(section_obj) = section_value.as_object() else {
            continue;
        };
        if channel_section_has_configured_accounts(section_obj) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn wait_for_channel_config_sync(channel_type: &str) -> Result<bool, String> {
    let normalized = normalize_channel_identifier(channel_type);
    let max_attempts = if is_whatsapp_channel_identifier(&normalized) {
        14
    } else {
        6
    };
    for attempt in 0..max_attempts {
        let _ = sync_channel_accounts_from_plugin_store(channel_type);
        if has_configured_channel_account_in_openclaw_config(channel_type)? {
            return Ok(true);
        }
        if attempt + 1 < max_attempts {
            thread::sleep(Duration::from_millis(900));
        }
    }
    Ok(false)
}

fn migrate_legacy_channel_section_to_accounts(section_obj: &mut serde_json::Map<String, Value>) {
    let has_accounts = section_obj
        .get("accounts")
        .and_then(Value::as_object)
        .map(|accounts| !accounts.is_empty())
        .unwrap_or(false);
    if has_accounts {
        if !section_obj.contains_key("defaultAccount") {
            section_obj.insert(
                "defaultAccount".to_string(),
                Value::String("default".to_string()),
            );
        }
        return;
    }

    let legacy_payload = section_obj
        .iter()
        .filter(|(key, _)| !is_channel_section_reserved_key(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<serde_json::Map<String, Value>>();

    if legacy_payload.is_empty() {
        return;
    }

    let default_account_id = section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();

    let mut account_payload = legacy_payload.clone();
    if let Some(enabled) = section_obj.get("enabled").and_then(Value::as_bool) {
        account_payload.insert("enabled".to_string(), Value::Bool(enabled));
    }

    let mut accounts = serde_json::Map::new();
    accounts.insert(default_account_id.clone(), Value::Object(account_payload));
    section_obj.insert("accounts".to_string(), Value::Object(accounts));
    section_obj.insert(
        "defaultAccount".to_string(),
        Value::String(default_account_id),
    );

    let legacy_keys = legacy_payload.keys().cloned().collect::<Vec<_>>();
    for key in legacy_keys {
        section_obj.remove(&key);
    }
}

fn mirror_default_account_to_channel_section(section_obj: &mut serde_json::Map<String, Value>) {
    let default_account_id = section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();
    let default_payload = section_obj
        .get("accounts")
        .and_then(Value::as_object)
        .and_then(|accounts| accounts.get(&default_account_id))
        .and_then(Value::as_object)
        .cloned();

    let removable_keys = section_obj
        .keys()
        .filter(|key| !is_channel_section_reserved_key(key))
        .cloned()
        .collect::<Vec<_>>();
    for key in removable_keys {
        section_obj.remove(&key);
    }

    if let Some(default_payload) = default_payload {
        for (key, value) in default_payload {
            if key == "enabled" {
                continue;
            }
            section_obj.insert(key, value);
        }
    }

    section_obj.insert("enabled".to_string(), Value::Bool(true));
}

fn ensure_channel_plugin_allowlist(
    root: &mut serde_json::Map<String, Value>,
    channel_type: &str,
) -> Result<(), String> {
    let plugin_id = match normalize_channel_identifier(channel_type).as_str() {
        "feishu" => Some("openclaw-lark"),
        "dingtalk" => Some("dingtalk"),
        "weixin" | "wechat" | "wx" | "openclaw-weixin" | "openclaw_weixin" => {
            Some("openclaw-weixin")
        }
        "wecom" | "workwechat" | "work-wechat" | "work_wechat" | "wechatwork" | "qywx"
        | "openclaw-wecom" | "openclaw_wecom" => Some("wecom"),
        "qqbot" => Some("qqbot"),
        "whatsapp" | "wa" | "wacli" | "openclaw-whatsapp" | "openclaw_whatsapp" => {
            Some("whatsapp")
        }
        _ => None,
    };

    let Some(plugin_id) = plugin_id else {
        return Ok(());
    };

    if !matches!(root.get("plugins"), Some(Value::Object(_))) {
        root.insert(
            "plugins".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let Some(plugins_obj) = root.get_mut("plugins").and_then(Value::as_object_mut) else {
        return Err("plugins 配置不是对象".to_string());
    };

    plugins_obj.insert("enabled".to_string(), Value::Bool(true));

    if !matches!(plugins_obj.get("allow"), Some(Value::Array(_))) {
        plugins_obj.insert("allow".to_string(), Value::Array(Vec::new()));
    }
    if let Some(allow_arr) = plugins_obj.get_mut("allow").and_then(Value::as_array_mut) {
        let exists = allow_arr.iter().any(|item| {
            item.as_str()
                .map(str::trim)
                .map(|value| value.eq_ignore_ascii_case(plugin_id))
                .unwrap_or(false)
        });
        if !exists {
            allow_arr.push(Value::String(plugin_id.to_string()));
        }
    }

    if !matches!(plugins_obj.get("entries"), Some(Value::Object(_))) {
        plugins_obj.insert(
            "entries".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    if let Some(entries_obj) = plugins_obj
        .get_mut("entries")
        .and_then(Value::as_object_mut)
    {
        if !matches!(entries_obj.get(plugin_id), Some(Value::Object(_))) {
            entries_obj.insert(
                plugin_id.to_string(),
                Value::Object(serde_json::Map::<String, Value>::new()),
            );
        }
        if let Some(entry_obj) = entries_obj
            .get_mut(plugin_id)
            .and_then(Value::as_object_mut)
        {
            entry_obj.insert("enabled".to_string(), Value::Bool(true));
        }
    }

    Ok(())
}

fn resolve_channel_plugin_install_spec(channel_type: &str) -> Option<(&'static str, &'static str)> {
    match normalize_channel_identifier(channel_type).as_str() {
        "feishu" => Some(("openclaw-lark", "@larksuite/openclaw-lark@2026.3.12")),
        "dingtalk" => Some(("dingtalk", "@soimy/dingtalk")),
        "whatsapp" | "wa" | "wacli" | "openclaw-whatsapp" | "openclaw_whatsapp" => {
            Some(("whatsapp", "@openclaw/whatsapp"))
        }
        _ => None,
    }
}

fn clear_all_channel_bindings(root: &mut serde_json::Map<String, Value>, channel_type: &str) {
    let normalized_channel = normalize_channel_identifier_for_openclaw_config(channel_type);
    let aliases = resolve_channel_config_verification_aliases(channel_type);
    let Some(bindings) = root.get_mut("bindings").and_then(Value::as_array_mut) else {
        return;
    };

    bindings.retain(|item| {
        let Some(binding_obj) = item.as_object() else {
            return true;
        };
        let Some(existing_channel) = binding_obj
            .get("match")
            .and_then(Value::as_object)
            .and_then(|match_obj| match_obj.get("channel"))
            .and_then(Value::as_str)
        else {
            return true;
        };

        let normalized_existing = normalize_channel_identifier(existing_channel);
        if normalized_existing == normalized_channel {
            return false;
        }
        !aliases.contains(&normalized_existing)
    });

    if bindings.is_empty() {
        root.remove("bindings");
    }
}

fn upsert_channel_binding(
    root: &mut serde_json::Map<String, Value>,
    channel_type: &str,
    account_id: Option<&str>,
    agent_id: Option<&str>,
) {
    let normalized_channel = normalize_channel_identifier_for_openclaw_config(channel_type);
    let channel_aliases = resolve_channel_config_verification_aliases(channel_type);
    if normalized_channel.is_empty() {
        return;
    }
    let normalized_account = account_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_account_identifier)
        .and_then(|value| {
            if value == "default" {
                None
            } else {
                Some(value)
            }
        });
    let normalized_agent = agent_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let existing_bindings = root
        .get("bindings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut next_bindings = Vec::new();

    for item in existing_bindings {
        let Some(binding_obj) = item.as_object() else {
            next_bindings.push(item);
            continue;
        };
        let Some(match_obj) = binding_obj.get("match").and_then(Value::as_object) else {
            next_bindings.push(item);
            continue;
        };
        let Some(existing_channel) = match_obj.get("channel").and_then(Value::as_str) else {
            next_bindings.push(item);
            continue;
        };

        let normalized_existing_channel = normalize_channel_identifier(existing_channel);
        if normalized_existing_channel != normalized_channel
            && !channel_aliases.contains(&normalized_existing_channel)
        {
            next_bindings.push(item);
            continue;
        }

        let existing_agent = binding_obj
            .get("agentId")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        let existing_account = match_obj
            .get("accountId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_account_identifier)
            .and_then(|value| {
                if value == "default" {
                    None
                } else {
                    Some(value)
                }
            });

        if let Some(ref target_agent) = normalized_agent {
            if !target_agent.is_empty() && existing_agent.eq_ignore_ascii_case(target_agent) {
                continue;
            }
        }

        match normalized_account.as_deref() {
            Some(target_account) => {
                if existing_account.as_deref() == Some(target_account) {
                    continue;
                }
            }
            None => {
                if existing_account.is_none() {
                    continue;
                }
            }
        }

        next_bindings.push(item);
    }

    if let Some(agent_id) = normalized_agent {
        let mut match_obj = serde_json::Map::new();
        match_obj.insert(
            "channel".to_string(),
            Value::String(normalized_channel.clone()),
        );
        if let Some(account_id) = normalized_account {
            match_obj.insert("accountId".to_string(), Value::String(account_id));
        }

        let mut binding_obj = serde_json::Map::new();
        binding_obj.insert("type".to_string(), Value::String("route".to_string()));
        binding_obj.insert("agentId".to_string(), Value::String(agent_id));
        binding_obj.insert("match".to_string(), Value::Object(match_obj));
        next_bindings.push(Value::Object(binding_obj));
    }

    if next_bindings.is_empty() {
        root.remove("bindings");
    } else {
        root.insert("bindings".to_string(), Value::Array(next_bindings));
    }
}

fn build_channel_account_config(
    channel_type: &str,
    incoming: &std::collections::HashMap<String, String>,
    existing: Option<&serde_json::Map<String, Value>>,
) -> serde_json::Map<String, Value> {
    let mut config = existing.cloned().unwrap_or_default();
    for (raw_key, raw_value) in incoming {
        let key = raw_key.trim();
        if key.is_empty() {
            continue;
        }
        let value = raw_value.trim();
        if value.is_empty() {
            config.remove(key);
        } else {
            config.insert(key.to_string(), Value::String(value.to_string()));
        }
    }

    let normalized_channel = normalize_channel_identifier(channel_type);

    if normalized_channel == "telegram" {
        let dm_policy = config
            .get("dmPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("pairing")
            .to_string();
        config.insert("dmPolicy".to_string(), Value::String(dm_policy));

        if let Some(allowed_users) = incoming.get("allowedUsers") {
            let users = allowed_users
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| Value::String(value.to_string()))
                .collect::<Vec<_>>();
            if users.is_empty() {
                config.remove("allowFrom");
            } else {
                config.insert("allowFrom".to_string(), Value::Array(users));
            }
        }
    }

    if normalized_channel == "discord" {
        let dm_policy = config
            .get("dmPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("pairing")
            .to_string();
        config.insert("dmPolicy".to_string(), Value::String(dm_policy));

        let group_policy = config
            .get("groupPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("allowlist")
            .to_string();
        config.insert("groupPolicy".to_string(), Value::String(group_policy));

        if !matches!(config.get("retry"), Some(Value::Object(_))) {
            config.insert(
                "retry".to_string(),
                serde_json::json!({
                    "attempts": 3,
                    "minDelayMs": 500,
                    "maxDelayMs": 30000,
                    "jitter": 0.1
                }),
            );
        }

        let guild_id = incoming
            .get("guildId")
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let channel_id = incoming
            .get("channelId")
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        if let Some(guild_id) = guild_id {
            let channel_rule = if let Some(channel_id) = channel_id {
                serde_json::json!({ channel_id: { "allow": true, "requireMention": true } })
            } else {
                serde_json::json!({ "*": { "allow": true, "requireMention": true } })
            };
            config.insert(
                "guilds".to_string(),
                serde_json::json!({
                    guild_id: {
                        "users": ["*"],
                        "requireMention": true,
                        "channels": channel_rule
                    }
                }),
            );
        }
        config.remove("guildId");
        config.remove("channelId");
        config.remove("dm");
    }

    if normalized_channel == "feishu" || normalized_channel == "wecom" {
        let dm_policy = config
            .get("dmPolicy")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("open")
            .to_string();
        config.insert("dmPolicy".to_string(), Value::String(dm_policy.clone()));

        let mut allow_from = config
            .get("allowFrom")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["*".to_string()]);
        if dm_policy == "open" && !allow_from.iter().any(|value| value == "*") {
            allow_from.push("*".to_string());
        }
        config.insert(
            "allowFrom".to_string(),
            Value::Array(
                allow_from
                    .into_iter()
                    .map(Value::String)
                    .collect::<Vec<_>>(),
            ),
        );
    }

    config.insert("enabled".to_string(), Value::Bool(true));
    config
}

fn extract_channel_form_values(
    channel_type: &str,
    account_obj: &serde_json::Map<String, Value>,
) -> std::collections::HashMap<String, String> {
    let mut values = std::collections::HashMap::new();
    let normalized_channel = normalize_channel_identifier(channel_type);

    for (key, value) in account_obj {
        if key == "enabled"
            || key == "accounts"
            || key == "defaultAccount"
            || key == "name"
            || key == "guilds"
            || key == "allowFrom"
            || key == "groupPolicy"
            || key == "dm"
            || key == "retry"
            || key == "dmPolicy"
        {
            continue;
        }
        if let Some(text) = value
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            values.insert(key.to_string(), text.to_string());
        }
    }

    if normalized_channel == "telegram" {
        if !values.contains_key("allowedUsers") {
            if let Some(users) = account_obj.get("allowFrom").and_then(Value::as_array) {
                let merged = users
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                if !merged.is_empty() {
                    values.insert("allowedUsers".to_string(), merged);
                }
            }
        }
    } else if normalized_channel == "discord" {
        if let Some(guilds) = account_obj.get("guilds").and_then(Value::as_object) {
            if let Some((guild_id, guild_obj)) = guilds.iter().next() {
                if !guild_id.trim().is_empty() {
                    values.insert("guildId".to_string(), guild_id.to_string());
                }
                if let Some(channels) = guild_obj.get("channels").and_then(Value::as_object) {
                    if let Some((channel_id, _)) = channels.iter().find(|(key, _)| *key != "*") {
                        if !channel_id.trim().is_empty() {
                            values.insert("channelId".to_string(), channel_id.to_string());
                        }
                    }
                }
            }
        }
    }

    values
}

fn load_staff_from_runtime_dirs(
    scheduled_agents: &std::collections::HashSet<String>,
    channels_by_agent: &std::collections::HashMap<String, String>,
    default_model: &str,
    default_tools_profile: &str,
) -> Result<Vec<StaffMemberSnapshot>, String> {
    let agents_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents");

    let entries = match std::fs::read_dir(&agents_path) {
        Ok(value) => value,
        Err(_) => return Ok(Vec::new()),
    };

    let mut members = Vec::new();
    let preferred_model = default_model.trim();
    let preferred_model = if preferred_model.is_empty() || preferred_model == "未标注" {
        None
    } else {
        Some(preferred_model.to_string())
    };
    let preferred_tools_profile = {
        let normalized = default_tools_profile.trim();
        if normalized.is_empty() {
            "default".to_string()
        } else {
            normalized.to_string()
        }
    };
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let agent_id = entry.file_name().to_string_lossy().trim().to_string();
        if agent_id.is_empty() {
            continue;
        }
        let runtime_summary = load_runtime_session_summary(&agent_id);
        let status_label = runtime_summary
            .latest_updated_at_ms
            .map(derive_status_label)
            .unwrap_or_else(|| "待命".to_string());
        let (current_work, recent_output) = runtime_summary
            .latest_session_file
            .as_deref()
            .map(load_recent_activity_from_session_file)
            .unwrap_or((None, None));
        let channel = channels_by_agent
            .get(&agent_id)
            .cloned()
            .unwrap_or_default();
        members.push(StaffMemberSnapshot {
            agent_id: agent_id.clone(),
            display_name: if agent_id.eq_ignore_ascii_case("main") {
                MAIN_STAFF_DISPLAY_NAME.to_string()
            } else {
                agent_id.clone()
            },
            role_label: humanize_agent_role(&agent_id),
            channel,
            model: preferred_model
                .clone()
                .or(runtime_summary.latest_model)
                .unwrap_or_else(|| "未标注".to_string()),
            workspace: "未标注".to_string(),
            tools_profile: preferred_tools_profile.clone(),
            tools_enabled_count: openclaw_profile_tool_ids(&preferred_tools_profile).len(),
            status_label,
            current_work_label: "正在处理什么".to_string(),
            current_work: current_work.unwrap_or_else(|| "当前无实时任务".to_string()),
            recent_output: recent_output.unwrap_or_else(|| "最近暂无产出。".to_string()),
            scheduled_label: if scheduled_agents.contains(&agent_id) {
                "已排班".to_string()
            } else {
                "未排班".to_string()
            },
        });
    }

    members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    Ok(members)
}

fn humanize_agent_role(agent_id: &str) -> String {
    match agent_id.trim().to_lowercase().as_str() {
        "main" => "主控员工".to_string(),
        "gateway" => "网关员工".to_string(),
        other => format!("{other} 员工"),
    }
}

const MAIN_STAFF_DISPLAY_NAME: &str = "超级管理者";

fn ensure_agents_list_has_main(list: &mut Vec<Value>) {
    let mut has_main = false;
    for item in list.iter_mut() {
        let Some(obj) = item.as_object_mut() else {
            continue;
        };
        let current_id = obj
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if !current_id.eq_ignore_ascii_case("main") {
            continue;
        }
        has_main = true;
        obj.insert("id".to_string(), Value::String("main".to_string()));
        obj.insert(
            "name".to_string(),
            Value::String(MAIN_STAFF_DISPLAY_NAME.to_string()),
        );
        let has_workspace = obj
            .get("workspace")
            .and_then(Value::as_str)
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
        if !has_workspace {
            obj.insert(
                "workspace".to_string(),
                Value::String("~/.openclaw/workspace-main".to_string()),
            );
        }
        break;
    }

    if !has_main {
        list.push(serde_json::json!({
            "id": "main",
            "name": MAIN_STAFF_DISPLAY_NAME,
            "workspace": "~/.openclaw/workspace-main"
        }));
    }
}

fn build_main_staff_snapshot(
    scheduled_agents: &std::collections::HashSet<String>,
    channels_by_agent: &std::collections::HashMap<String, String>,
    default_model: &str,
    default_tools_profile: &str,
) -> StaffMemberSnapshot {
    let runtime_summary = load_runtime_session_summary("main");
    let status_label = runtime_summary
        .latest_updated_at_ms
        .map(derive_status_label)
        .unwrap_or_else(|| "待命".to_string());
    let (current_work, recent_output) = runtime_summary
        .latest_session_file
        .as_deref()
        .map(load_recent_activity_from_session_file)
        .unwrap_or((None, None));

    StaffMemberSnapshot {
        agent_id: "main".to_string(),
        display_name: MAIN_STAFF_DISPLAY_NAME.to_string(),
        role_label: humanize_agent_role("main"),
        channel: channels_by_agent.get("main").cloned().unwrap_or_default(),
        model: runtime_summary
            .latest_model
            .unwrap_or_else(|| default_model.trim().to_string()),
        workspace: "~/.openclaw/workspace-main".to_string(),
        tools_profile: default_tools_profile.trim().to_string(),
        tools_enabled_count: openclaw_profile_tool_ids(default_tools_profile).len(),
        status_label,
        current_work_label: "正在处理什么".to_string(),
        current_work: current_work.unwrap_or_else(|| "当前无实时任务".to_string()),
        recent_output: recent_output.unwrap_or_else(|| "最近暂无产出。".to_string()),
        scheduled_label: if scheduled_agents.contains("main") {
            "已排班".to_string()
        } else {
            "未排班".to_string()
        },
    }
}

fn extract_text_from_message_content(content: &Value) -> Option<String> {
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

fn sanitize_staff_output(content: &str) -> String {
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

fn extract_message_timestamp_ms(message: &serde_json::Map<String, Value>, fallback: i64) -> i64 {
    value_as_i64(message.get("timestamp")).unwrap_or(fallback)
}

fn extract_message_text(message: &serde_json::Map<String, Value>) -> Option<String> {
    message
        .get("content")
        .and_then(extract_text_from_message_content)
        .map(|text| sanitize_staff_output(&text))
}

fn extract_tool_calls(message: &serde_json::Map<String, Value>) -> Vec<(String, PendingToolCall)> {
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

fn extract_tool_result_text(message: &serde_json::Map<String, Value>) -> Option<String> {
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

fn infer_openclaw_response_status(text: &str) -> (u16, Option<String>) {
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

fn extract_usage_numbers(
    message: &serde_json::Map<String, Value>,
) -> (Option<u32>, Option<u32>, Option<u32>, Option<u32>) {
    let usage = message.get("usage").and_then(Value::as_object);
    let prompt_tokens = usage
        .and_then(|value| value.get("input"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let completion_tokens = usage
        .and_then(|value| value.get("output"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());
    let total_tokens = usage
        .and_then(|value| value.get("totalTokens"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| match (prompt_tokens, completion_tokens) {
            (Some(input), Some(output)) => Some(input.saturating_add(output)),
            _ => None,
        });
    let cache_read_input_tokens = usage
        .and_then(|value| value.get("cacheRead"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok());

    (
        prompt_tokens,
        completion_tokens,
        total_tokens,
        cache_read_input_tokens,
    )
}

fn load_openclaw_message_logs_from_session_file(
    agent_id: &str,
    session_file: &Path,
) -> Vec<OpenClawMessageLogItem> {
    let raw = match std::fs::read_to_string(session_file) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let session_file_name = session_file
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown-session");
    let session_id = session_file
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(session_file_name);
    let endpoint = format!("openclaw://runtime/{agent_id}/{session_id}");
    let base_url = format!("openclaw://runtime/{agent_id}");
    let path = format!("/{session_id}");
    let platform_id = format!("openclaw-runtime-{agent_id}");
    let platform_name = format!("OpenClaw 运行时 / {agent_id}");

    let mut pending_user: Option<(String, i64)> = None;
    let mut pending_tool_calls: std::collections::HashMap<String, PendingToolCall> =
        std::collections::HashMap::new();
    let mut output = Vec::new();

    for (line_index, line) in raw.lines().enumerate() {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let Some(obj) = parsed.as_object() else {
            continue;
        };
        if obj.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }
        let Some(message) = obj.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let fallback_created_at = i64::try_from(line_index).unwrap_or(0);
        let created_at = extract_message_timestamp_ms(message, fallback_created_at);

        if role == "user" {
            let Some(text) = extract_message_text(message) else {
                continue;
            };
            pending_user = Some((text, created_at));
            continue;
        }

        if role == "assistant" {
            for (tool_call_id, tool_call) in extract_tool_calls(message) {
                pending_tool_calls.insert(tool_call_id, tool_call);
            }

            let Some(text) = extract_message_text(message) else {
                continue;
            };

            let (request_body, request_at) = pending_user
                .take()
                .unwrap_or_else(|| ("".to_string(), created_at));
            let duration = created_at.saturating_sub(request_at);
            let duration = u32::try_from(duration).unwrap_or(u32::MAX);
            let (response_status, error) = infer_openclaw_response_status(&text);
            let (prompt_tokens, completion_tokens, total_tokens, cache_read_input_tokens) =
                extract_usage_numbers(message);

            output.push(OpenClawMessageLogItem {
                id: format!("runtime-{agent_id}-{session_id}-{created_at}"),
                session_id: format!("runtime-{agent_id}-{session_id}"),
                platform_id: platform_id.clone(),
                platform_name: platform_name.clone(),
                protocol: "openai".to_string(),
                method: "MESSAGE".to_string(),
                endpoint: endpoint.clone(),
                base_url: Some(base_url.clone()),
                path: Some(path.clone()),
                request_body,
                response_status,
                response_body: text.clone(),
                stream_summary: Some(text),
                duration,
                first_token_time: Some(duration),
                error,
                prompt_tokens,
                completion_tokens,
                total_tokens,
                cache_read_input_tokens,
                created_at,
            });
            continue;
        }

        if role != "toolResult" {
            continue;
        }

        let tool_call_id = message
            .get("toolCallId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let tool_name = message
            .get("toolName")
            .and_then(Value::as_str)
            .unwrap_or("tool");
        let Some(text) = extract_tool_result_text(message) else {
            continue;
        };
        let pending_tool_call = pending_tool_calls.remove(&tool_call_id);
        let request_at = pending_tool_call
            .as_ref()
            .map(|value| value.created_at)
            .unwrap_or(created_at);
        let duration = created_at.saturating_sub(request_at);
        let duration = u32::try_from(duration).unwrap_or(u32::MAX);
        let (response_status, error) = infer_openclaw_response_status(&text);
        let resolved_tool_name = pending_tool_call
            .as_ref()
            .map(|value| value.tool_name.clone())
            .unwrap_or_else(|| tool_name.to_string());
        let request_body = pending_tool_call
            .map(|value| value.arguments)
            .unwrap_or_else(|| "{}".to_string());

        output.push(OpenClawMessageLogItem {
            id: format!("runtime-{agent_id}-{session_id}-tool-{created_at}"),
            session_id: format!("runtime-{agent_id}-{session_id}"),
            platform_id: platform_id.clone(),
            platform_name: format!("{platform_name} / {resolved_tool_name}"),
            protocol: "openai".to_string(),
            method: format!("TOOL:{resolved_tool_name}"),
            endpoint: format!("{endpoint}/tool/{resolved_tool_name}"),
            base_url: Some(base_url.clone()),
            path: Some(format!("{path}/tool/{resolved_tool_name}")),
            request_body,
            response_status,
            response_body: text.clone(),
            stream_summary: Some(text),
            duration,
            first_token_time: Some(duration),
            error,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            cache_read_input_tokens: None,
            created_at,
        });
    }

    output
}

#[tauri::command]
fn load_openclaw_message_logs() -> Result<OpenClawMessageLogResponse, String> {
    let agents_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents");

    let entries = match std::fs::read_dir(&agents_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawMessageLogResponse {
                detail: "未找到 OpenClaw 运行时会话目录。".to_string(),
                logs: Vec::new(),
            })
        }
    };

    let mut session_files = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let agent_id = entry.file_name().to_string_lossy().trim().to_string();
        if agent_id.is_empty() {
            continue;
        }

        let sessions_dir = entry.path().join("sessions");
        let Ok(files) = std::fs::read_dir(&sessions_dir) else {
            continue;
        };

        for file in files {
            let Ok(file) = file else {
                continue;
            };
            let path = file.path();
            if path.extension().and_then(|value| value.to_str()) != Some("jsonl") {
                continue;
            }

            let modified_at = file
                .metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| value.as_millis())
                .and_then(|value| i64::try_from(value).ok())
                .unwrap_or_default();

            session_files.push((modified_at, agent_id.clone(), path));
        }
    }

    session_files.sort_by(|left, right| right.0.cmp(&left.0));

    let mut logs = Vec::new();
    for (_, agent_id, path) in session_files.into_iter().take(12) {
        logs.extend(load_openclaw_message_logs_from_session_file(
            &agent_id, &path,
        ));
    }

    logs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    logs.truncate(180);

    Ok(OpenClawMessageLogResponse {
        detail: format!("已从 OpenClaw 运行时会话读取 {} 条消息日志。", logs.len()),
        logs,
    })
}

fn humanize_scope_label(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("main") {
        return "Main".to_string();
    }
    if trimmed.is_empty() {
        return "未标注".to_string();
    }
    trimmed.to_string()
}

fn load_editable_scopes() -> Vec<EditableScope> {
    let mut scopes = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let main_root = resolve_workspace_main_root();
    scopes.push(EditableScope {
        facet_key: "main".to_string(),
        facet_label: "Main".to_string(),
        workspace_root: main_root.clone(),
    });
    seen.insert("main".to_string());

    let config_path = resolve_openclaw_config_path();
    if let Ok(raw) = std::fs::read_to_string(config_path) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&raw) {
            if let Some(list) = parsed
                .get("agents")
                .and_then(Value::as_object)
                .and_then(|value| value.get("list"))
                .and_then(Value::as_array)
            {
                for row in list {
                    let Some(obj) = row.as_object() else {
                        continue;
                    };
                    let agent_id = obj
                        .get("id")
                        .and_then(Value::as_str)
                        .or_else(|| obj.get("name").and_then(Value::as_str))
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .unwrap_or("");
                    if agent_id.is_empty() {
                        continue;
                    }
                    let facet_key = agent_id.to_lowercase();
                    if seen.contains(&facet_key) {
                        continue;
                    }
                    let configured_workspace = obj
                        .get("workspace")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty());
                    let workspace_root = if facet_key == "main" {
                        main_root.clone()
                    } else {
                        resolve_workspace_root_for_agent(agent_id, configured_workspace)
                    };
                    scopes.push(EditableScope {
                        facet_key: facet_key.clone(),
                        facet_label: humanize_scope_label(agent_id),
                        workspace_root,
                    });
                    seen.insert(facet_key);
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(resolve_workspace_agents_root()) {
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let agent_id = entry.file_name().to_string_lossy().trim().to_string();
            if agent_id.is_empty() {
                continue;
            }
            let facet_key = agent_id.to_lowercase();
            if seen.contains(&facet_key) {
                continue;
            }
            scopes.push(EditableScope {
                facet_key: facet_key.clone(),
                facet_label: humanize_scope_label(&agent_id),
                workspace_root: entry.path(),
            });
            seen.insert(facet_key);
        }
    }

    scopes.sort_by(|left, right| {
        if left.facet_key == "main" {
            return std::cmp::Ordering::Less;
        }
        if right.facet_key == "main" {
            return std::cmp::Ordering::Greater;
        }
        left.facet_label.cmp(&right.facet_label)
    });
    scopes
}

fn safe_read_source_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Some(String::new());
    }
    Some(trimmed.to_string())
}

fn build_source_file_title(path: &Path, content: &str) -> String {
    for line in content.lines().map(str::trim) {
        if let Some(value) = line.strip_prefix("# ") {
            let heading = value.trim();
            if !heading.is_empty() {
                return heading.to_string();
            }
        }
    }

    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("未命名文件")
        .to_string()
}

fn build_source_file_summary(content: &str) -> String {
    let normalized = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(4)
        .collect::<Vec<_>>()
        .join(" ");
    if normalized.is_empty() {
        "文件内容为空。".to_string()
    } else if normalized.chars().count() > 180 {
        normalized.chars().take(180).collect::<String>()
    } else {
        normalized
    }
}

fn build_source_file_item(
    path: &Path,
    relative_base: &Path,
    facet_key: &str,
    facet_label: &str,
    category: &str,
) -> Option<SourceFileSnapshotItem> {
    let meta = std::fs::metadata(path).ok();
    let exists = meta.as_ref().map(|value| value.is_file()).unwrap_or(false);
    let content = if exists {
        safe_read_source_file(path)?
    } else {
        String::new()
    };
    let updated_at_ms = meta
        .as_ref()
        .and_then(|value| value.modified().ok())
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis() as i64)
        .unwrap_or(0);
    let title = build_source_file_title(path, &content);
    let summary = if exists {
        build_source_file_summary(&content)
    } else {
        "文件缺失，保存时会自动创建。".to_string()
    };
    let source_path = path.display().to_string();
    let relative_path = path
        .strip_prefix(relative_base)
        .ok()
        .map(|value| value.display().to_string())
        .unwrap_or_else(|| source_path.clone());
    let id = format!(
        "{facet_key}-{}",
        relative_path.replace(['/', '\\', ' '], "-").to_lowercase()
    );

    Some(SourceFileSnapshotItem {
        id,
        title,
        summary,
        content,
        source_path,
        relative_path,
        facet_key: facet_key.to_string(),
        facet_label: facet_label.to_string(),
        category: category.to_string(),
        updated_at_ms,
        exists,
    })
}

fn load_memory_file_items() -> Vec<SourceFileSnapshotItem> {
    let mut output = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let main_root = resolve_workspace_main_root();
    let scopes = load_editable_scopes();
    let memory_candidates = [
        "MEMORY.md",
        "SOUL.md",
        "TOOLS.md",
        "IDENTITY.md",
        "USER.md",
        "HEARTBEAT.md",
        "BOOTSTRAP.md",
        "AGENTS.md",
    ];

    let append = |output: &mut Vec<SourceFileSnapshotItem>,
                  seen: &mut std::collections::HashSet<String>,
                  item: Option<SourceFileSnapshotItem>| {
        if let Some(row) = item {
            if seen.insert(row.source_path.clone()) {
                output.push(row);
            }
        }
    };

    for file_name in memory_candidates {
        append(
            &mut output,
            &mut seen,
            build_source_file_item(
                &main_root.join(file_name),
                &main_root,
                "main",
                "Main",
                "Main",
            ),
        );
    }

    if let Ok(entries) = std::fs::read_dir(main_root.join("memory")) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !["md", "markdown", "txt"].contains(&ext.as_str()) {
                continue;
            }
            append(
                &mut output,
                &mut seen,
                build_source_file_item(&path, &main_root, "main", "Main", "Main 记忆记录"),
            );
        }
    }

    for scope in scopes.iter().filter(|scope| scope.facet_key != "main") {
        for file_name in memory_candidates {
            append(
                &mut output,
                &mut seen,
                build_source_file_item(
                    &scope.workspace_root.join(file_name),
                    &scope.workspace_root,
                    &scope.facet_key,
                    &scope.facet_label,
                    &scope.facet_label,
                ),
            );
        }

        if let Ok(entries) = std::fs::read_dir(scope.workspace_root.join("memory")) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !["md", "markdown", "txt"].contains(&ext.as_str()) {
                    continue;
                }
                append(
                    &mut output,
                    &mut seen,
                    build_source_file_item(
                        &path,
                        &scope.workspace_root,
                        &scope.facet_key,
                        &scope.facet_label,
                        &format!("{} 记忆记录", scope.facet_label),
                    ),
                );
            }
        }
    }

    output.sort_by(|left, right| {
        (left.facet_key != "main")
            .cmp(&(right.facet_key != "main"))
            .then_with(|| left.facet_label.cmp(&right.facet_label))
            .then_with(|| right.updated_at_ms.cmp(&left.updated_at_ms))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    output
}

fn resolve_memory_db_path(scope: &EditableScope) -> PathBuf {
    let file_name = if scope.facet_key == "main" {
        "main.sqlite".to_string()
    } else {
        format!("{}.sqlite", scope.facet_key)
    };
    resolve_openclaw_home_path().join("memory").join(file_name)
}

fn query_sqlite_count(db_path: &Path, table_name: &str) -> Option<usize> {
    if !db_path.exists() {
        return None;
    }

    let mut command = Command::new("sqlite3");
    suppress_windows_command_window(&mut command);
    let output = command
        .arg(db_path)
        .arg(format!("select count(*) from {table_name};"))
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()?
        .trim()
        .parse::<usize>()
        .ok()
}

fn summarize_memory_store(scopes: &[EditableScope]) -> (usize, usize, usize) {
    let mut db_count = 0;
    let mut file_count = 0;
    let mut chunk_count = 0;

    for scope in scopes {
        let db_path = resolve_memory_db_path(scope);
        if !db_path.exists() {
            continue;
        }
        db_count += 1;
        file_count += query_sqlite_count(&db_path, "files").unwrap_or(0);
        chunk_count += query_sqlite_count(&db_path, "chunks").unwrap_or(0);
    }

    (db_count, file_count, chunk_count)
}

fn load_document_file_items() -> Vec<SourceFileSnapshotItem> {
    let mut output = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let main_root = resolve_workspace_main_root();
    let scopes = load_editable_scopes();
    let main_candidates = [
        "AGENTS.md",
        "IDENTITY.md",
        "SOUL.md",
        "BOOTSTRAP.md",
        "HEARTBEAT.md",
        "TOOLS.md",
        "README.md",
        "NOTEBOOK.md",
        "focus.md",
        "inbox.md",
        "routines.md",
    ];

    let append = |output: &mut Vec<SourceFileSnapshotItem>,
                  seen: &mut std::collections::HashSet<String>,
                  item: Option<SourceFileSnapshotItem>| {
        if let Some(row) = item {
            if seen.insert(row.source_path.clone()) {
                output.push(row);
            }
        }
    };

    for file_name in main_candidates {
        append(
            &mut output,
            &mut seen,
            build_source_file_item(
                &main_root.join(file_name),
                &main_root,
                "main",
                "Main",
                "Main 核心文档",
            ),
        );
    }

    for scope in scopes.iter().filter(|scope| scope.facet_key != "main") {
        for file_name in [
            "AGENTS.md",
            "IDENTITY.md",
            "SOUL.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
            "TOOLS.md",
            "README.md",
            "NOTEBOOK.md",
            "focus.md",
            "inbox.md",
            "routines.md",
        ] {
            append(
                &mut output,
                &mut seen,
                build_source_file_item(
                    &scope.workspace_root.join(file_name),
                    &scope.workspace_root,
                    &scope.facet_key,
                    &scope.facet_label,
                    &format!("{} 核心文档", scope.facet_label),
                ),
            );
        }
    }

    output.sort_by(|left, right| {
        (left.facet_key != "main")
            .cmp(&(right.facet_key != "main"))
            .then_with(|| left.facet_label.cmp(&right.facet_label))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    output
}

fn load_skill_file_items() -> Vec<SourceFileSnapshotItem> {
    let mut output = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let candidates = [
        resolve_openclaw_home_path()
            .join("workspace")
            .join("skills"),
        resolve_openclaw_home_path().join("skills"),
    ];

    for root in candidates {
        let entries = match std::fs::read_dir(&root) {
            Ok(value) => value,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }

            let skill_file = path.join("SKILL.md");
            let key = skill_file.display().to_string();
            if seen.contains(&key) {
                continue;
            }

            if let Some(item) = build_source_file_item(
                &skill_file,
                &root,
                "openclaw-skills",
                "OpenClaw",
                "OpenClaw Skills",
            ) {
                seen.insert(key);
                output.push(item);
            }
        }
    }

    output.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    output
}

fn load_tool_file_items(agent_id: Option<&str>) -> Vec<SourceFileSnapshotItem> {
    let scopes = load_editable_scopes();
    let filtered_scopes = scopes.into_iter().filter(|scope| {
        if scope.facet_key == "main" {
            return false;
        }

        match agent_id {
            Some(value) => scope.facet_key.eq_ignore_ascii_case(value.trim()),
            None => true,
        }
    });

    let mut output = Vec::new();
    for scope in filtered_scopes {
        if let Some(item) = build_source_file_item(
            &scope.workspace_root.join("TOOLS.md"),
            &scope.workspace_root,
            &scope.facet_key,
            &scope.facet_label,
            &format!("{} Tools", scope.facet_label),
        ) {
            output.push(item);
        }
    }

    output.sort_by(|left, right| left.facet_label.cmp(&right.facet_label));
    output
}

/// 从 relative_path（如 "transcribe/SKILL.md"）得到技能 id（如 "transcribe"），用于匹配 openclaw.json skills.entries
fn openclaw_skill_id_from_path(relative_path: &str) -> String {
    let path = relative_path.trim().replace('\\', "/");
    path.split('/')
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string()
}

/// OpenClaw 完整内置技能清单 (id, 描述)
static OPENCLAW_BUILTIN_SKILLS: &[(&str, &str)] = &[
    (
        "1password",
        "1Password 密码管理，设置并使用 1Password CLI 来管理密码",
    ),
    (
        "apple-notes",
        "通过 macOS 上的「备忘录」CLI 管理 Apple Notes",
    ),
    ("apple-reminders", "通过 remindctl 的 CLI 管理苹果提醒事项"),
    ("bear-notes", "通过 grizzly CLI 搜索和管理 Bear 笔记"),
    ("blogwatcher", "博客监控，定期监听博客更新"),
    ("blucli", "BlueBubbles CLI 操作"),
    ("bluebubbles", "BlueBubbles iMessage 收发消息"),
    ("camsnap", "RTSP/ONVIF 摄像头截图"),
    ("canvas", "Canvas 画布操作"),
    ("clawhub", "ClawHub CLI - 技能市场，安装/管理技能"),
    ("coding-agent", "编程代理，辅助代码编写与调试"),
    ("discord", "Discord 消息收发与频道管理"),
    ("eightctl", "Eight Sleep 智能床控制"),
    ("gemini", "Gemini CLI 问答与推理"),
    ("gh-issues", "GitHub Issues 自动处理"),
    ("gifgrep", "GIF 搜索与下载"),
    ("github", "GitHub CLI 仓库与 PR 操作"),
    ("gog", "Google Workspace（Gmail / Calendar / Drive）"),
    ("goplaces", "地点搜索与导航"),
    ("healthcheck", "安全审计与系统健康检查"),
    ("himalaya", "邮件客户端，收发管理邮件"),
    ("imsg", "iMessage / SMS 收发消息"),
    ("mcporter", "Minecraft 相关操作"),
    ("model-usage", "模型使用统计与费用追踪"),
    ("nano-banana-pro", "Banana Pro 设备管理"),
    ("nano-pdf", "PDF 生成与读取操作"),
    ("notion", "Notion 笔记与数据库操作"),
    ("obsidian", "Obsidian 笔记管理"),
    ("openai-image-gen", "OpenAI DALL·E 图像生成"),
    ("openai-whisper", "本地 Whisper 语音识别"),
    ("openai-whisper-api", "OpenAI Whisper API 语音识别"),
    ("openhue", "Philips Hue 智能灯控制"),
    ("oracle", "Oracle 数据库查询与管理"),
    ("ordercli", "订单管理 CLI"),
    ("peekaboo", "macOS UI 自动化截图工具"),
    ("sag", "ElevenLabs TTS 语音合成"),
    ("session-logs", "会话日志查看与管理"),
    ("sherpa-onnx-tts", "本地 Sherpa-ONNX TTS 语音合成"),
    ("skill-creator", "创建和编辑 Skills"),
    ("slack", "Slack 消息与频道操作"),
    ("songsee", "歌曲识别"),
    ("sonoscli", "Sonos 音响控制"),
    ("spotify-player", "Spotify 播放器控制"),
    ("summarize", "内容摘要与转录"),
    ("things-mac", "Things 3 任务管理"),
    ("tmux", "Tmux 会话管理"),
    ("trello", "Trello 看板任务管理"),
    ("video-frames", "视频帧提取与分析"),
    ("voice-call", "语音通话"),
    ("wacli", "WhatsApp CLI 消息收发"),
    ("weather", "天气查询"),
    ("xurl", "URL 处理与内容抓取"),
];

#[tauri::command]
fn load_openclaw_skills_list() -> Result<OpenClawSkillsListResponse, String> {
    let items = load_skill_file_items();
    let config_path = resolve_openclaw_config_path();
    let mut entries_enabled: std::collections::HashMap<String, bool> =
        std::collections::HashMap::new();
    if let Ok(raw) = std::fs::read_to_string(&config_path) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&raw) {
            if let Some(skills_root) = parsed.get("skills").and_then(Value::as_object) {
                if let Some(entries) = skills_root.get("entries").and_then(Value::as_object) {
                    for (key, val) in entries {
                        let enabled = val.get("enabled").and_then(Value::as_bool).unwrap_or(true);
                        entries_enabled.insert(key.trim().to_lowercase(), enabled);
                    }
                }
            }
        }
    }

    let source_path = resolve_openclaw_home_path()
        .join("skills")
        .display()
        .to_string();

    // 内置技能：使用完整硬编码清单，从 entries_enabled 读取启用状态
    let built_in: Vec<OpenClawSkillListItem> = OPENCLAW_BUILTIN_SKILLS
        .iter()
        .map(|(id, desc)| {
            let key_lower = id.to_lowercase();
            let enabled = entries_enabled.get(&key_lower).copied().unwrap_or(true);
            OpenClawSkillListItem {
                id: id.to_string(),
                name: id.to_string(),
                description: desc.to_string(),
                enabled,
                relative_path: String::new(),
                source_path: String::new(),
            }
        })
        .collect();

    // 内置技能 ID 集合，用于排除
    let builtin_ids: std::collections::HashSet<String> = OPENCLAW_BUILTIN_SKILLS
        .iter()
        .map(|(id, _)| id.to_lowercase())
        .collect();

    // 扫描插件目录里的额外技能（~/.openclaw/openclaw-*/skills/）
    let openclaw_home = resolve_openclaw_home_path();
    let mut plugin_items: Vec<SourceFileSnapshotItem> = Vec::new();
    if let Ok(home_entries) = std::fs::read_dir(&openclaw_home) {
        for entry in home_entries.flatten() {
            let dir_name = entry.file_name();
            let dir_str = dir_name.to_string_lossy();
            if !dir_str.starts_with("openclaw-") {
                continue;
            }
            let plugin_skills_dir = entry.path().join("skills");
            if let Ok(skill_dirs) = std::fs::read_dir(&plugin_skills_dir) {
                for skill_entry in skill_dirs.flatten() {
                    let skill_path = skill_entry.path();
                    let Ok(ft) = skill_entry.file_type() else {
                        continue;
                    };
                    if !ft.is_dir() {
                        continue;
                    }
                    let skill_file = skill_path.join("SKILL.md");
                    if let Some(item) = build_source_file_item(
                        &skill_file,
                        &plugin_skills_dir,
                        "openclaw-skills",
                        "OpenClaw",
                        "OpenClaw Skills",
                    ) {
                        plugin_items.push(item);
                    }
                }
            }
        }
    }

    fn make_skill_item(
        skill_id: String,
        item_id: String,
        title: String,
        content: String,
        relative_path: String,
        source_path: String,
        entries_enabled: &std::collections::HashMap<String, bool>,
    ) -> OpenClawSkillListItem {
        let key_lower = skill_id.to_lowercase();
        let enabled = entries_enabled.get(&key_lower).copied().unwrap_or(true);
        let title_is_filename = title.eq_ignore_ascii_case("SKILL.md") || title.trim().is_empty();
        let display_name = if title_is_filename {
            skill_id.clone()
        } else {
            title
        };
        let description = {
            let clean: String = content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .take(3)
                .collect::<Vec<_>>()
                .join(" ");
            let truncated = if clean.chars().count() > 180 {
                clean.chars().take(180).collect::<String>()
            } else {
                clean
            };
            if truncated.is_empty() {
                "暂无描述。".to_string()
            } else {
                truncated
            }
        };
        OpenClawSkillListItem {
            id: item_id,
            name: display_name,
            description,
            enabled,
            relative_path,
            source_path,
        }
    }

    // 用户安装技能：~/.openclaw/skills/ 下不在内置清单里的技能
    let installed: Vec<OpenClawSkillListItem> = items
        .into_iter()
        .filter(|item| {
            let skill_id = openclaw_skill_id_from_path(&item.relative_path);
            !builtin_ids.contains(&skill_id.to_lowercase())
        })
        .map(|item| {
            let skill_id = openclaw_skill_id_from_path(&item.relative_path);
            make_skill_item(
                skill_id,
                item.id,
                item.title,
                item.content,
                item.relative_path,
                item.source_path,
                &entries_enabled,
            )
        })
        .chain(
            // 插件技能（openclaw-lark 等）
            plugin_items.into_iter().map(|item| {
                let skill_id = openclaw_skill_id_from_path(&item.relative_path);
                make_skill_item(
                    skill_id,
                    item.id,
                    item.title,
                    item.content,
                    item.relative_path,
                    item.source_path,
                    &entries_enabled,
                )
            }),
        )
        .collect();

    Ok(OpenClawSkillsListResponse {
        source_path,
        built_in,
        installed,
    })
}

/// 更新 openclaw.json 中某技能的启用状态（skills.entries[skill_id].enabled）
#[tauri::command]
fn save_openclaw_skill_enabled(skill_id: String, enabled: bool) -> Result<(), String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut parsed: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
    let skills = root
        .entry("skills")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("skills 不是对象")?;
    let entries = skills
        .entry("entries")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("skills.entries 不是对象")?;
    let skill_key = skill_id.trim();
    if skill_key.is_empty() {
        return Err("技能 id 不能为空".to_string());
    }
    let entry = entries
        .entry(skill_key.to_string())
        .or_insert_with(|| serde_json::json!({}));
    let entry_obj = entry.as_object_mut().ok_or("skills.entries 项不是对象")?;
    entry_obj.insert("enabled".to_string(), Value::Bool(enabled));
    write_openclaw_config_value(&config_path, &parsed)
}

/// OpenClaw 内置工具清单（与 docs 一致）：id, 名称, 描述, 分类
const OPENCLAW_TOOL_INVENTORY: &[(&str, &str, &str)] = &[
    // group:fs
    ("read", "read", "读取文件内容"),
    ("write", "write", "创建或覆盖文件"),
    ("edit", "edit", "精确编辑文件"),
    ("apply_patch", "apply_patch", "应用补丁（多块编辑）"),
    // group:runtime
    ("exec", "exec", "执行 shell 命令"),
    ("bash", "bash", "Bash 执行"),
    ("process", "process", "管理后台进程"),
    // group:web
    ("web_search", "web_search", "网页搜索"),
    ("web_fetch", "web_fetch", "抓取网页内容"),
    // group:memory
    ("memory_search", "memory_search", "记忆语义搜索"),
    ("memory_get", "memory_get", "读取记忆文件"),
    // group:sessions
    ("sessions_list", "sessions_list", "会话列表"),
    ("sessions_history", "sessions_history", "会话历史"),
    ("sessions_send", "sessions_send", "发送会话消息"),
    ("sessions_spawn", "sessions_spawn", "创建会话"),
    ("session_status", "session_status", "会话状态"),
    // group:messaging
    ("message", "message", "消息发送"),
    // group:ui
    ("browser", "browser", "浏览器控制"),
    ("canvas", "canvas", "Canvas 节点"),
    // group:automation
    ("cron", "cron", "定时任务"),
    ("gateway", "gateway", "网关"),
    // group:nodes
    ("nodes", "nodes", "节点发现与配对"),
    // other
    ("image", "image", "图像生成/处理"),
];

fn openclaw_tool_category(tool_id: &str) -> &'static str {
    match tool_id {
        "read" | "write" | "edit" | "apply_patch" => "Files",
        "exec" | "bash" | "process" => "Runtime",
        "web_search" | "web_fetch" => "Web",
        "memory_search" | "memory_get" => "Memory",
        "sessions_list" | "sessions_history" | "sessions_send" | "sessions_spawn"
        | "session_status" => "Sessions",
        "message" => "Messaging",
        "browser" | "canvas" => "UI",
        "cron" | "gateway" => "Automation",
        "nodes" => "Nodes",
        _ => "Other",
    }
}

/// 解析 profile 得到基础允许的工具 id 集合；full = 全部
fn openclaw_profile_tool_ids(profile: &str) -> std::collections::HashSet<String> {
    let profile = profile.trim().to_ascii_lowercase();
    let mut set = std::collections::HashSet::new();
    match profile.as_str() {
        "full" | "default" | "" => {
            for (id, _, _) in OPENCLAW_TOOL_INVENTORY {
                set.insert((*id).to_string());
            }
            return set;
        }
        "minimal" => {
            set.insert("session_status".to_string());
            return set;
        }
        "coding" => {
            for id in &[
                "read",
                "write",
                "edit",
                "apply_patch",
                "exec",
                "bash",
                "process",
                "sessions_list",
                "sessions_history",
                "sessions_send",
                "sessions_spawn",
                "session_status",
                "memory_search",
                "memory_get",
                "image",
            ] {
                set.insert((*id).to_string());
            }
            return set;
        }
        "messaging" => {
            for id in &[
                "message",
                "sessions_list",
                "sessions_history",
                "sessions_send",
                "session_status",
            ] {
                set.insert((*id).to_string());
            }
            return set;
        }
        _ => {}
    }
    set
}

/// 从 config 的 allow/deny 数组解析出允许的工具 id（* 表示全部）；deny 优先
fn openclaw_resolve_tools_from_config(
    root: &serde_json::Map<String, Value>,
    agent_obj: Option<&serde_json::Map<String, Value>>,
) -> (String, String, std::collections::HashSet<String>) {
    let tools_root = root.get("tools").and_then(value_as_object);
    let _agents_list = root
        .get("agents")
        .and_then(value_as_object)
        .and_then(|o| o.get("list"))
        .and_then(Value::as_array);
    let default_profile = tools_root
        .and_then(|t| t.get("profile").and_then(Value::as_str))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");
    let mut profile = default_profile.to_string();
    let mut allow: Option<Vec<String>> = tools_root
        .and_then(|t| t.get("allow"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                .collect()
        });
    let mut deny: Vec<String> = tools_root
        .and_then(|t| t.get("deny"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_ascii_lowercase()))
                .collect()
        })
        .unwrap_or_default();

    if let Some(agent) = agent_obj {
        if let Some(tools) = agent.get("tools").and_then(value_as_object) {
            if let Some(p) = tools.get("profile").and_then(Value::as_str) {
                let p = p.trim();
                if !p.is_empty() {
                    profile = p.to_string();
                }
            }
            if let Some(a) = tools.get("allow").and_then(Value::as_array) {
                allow = Some(
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .collect(),
                );
            }
            if let Some(d) = tools.get("deny").and_then(Value::as_array) {
                deny = d
                    .iter()
                    .filter_map(|v| {
                        v.as_str()
                            .map(|s| s.trim().to_ascii_lowercase().to_string())
                    })
                    .collect();
            }
        }
    }

    let profile_label = match profile.to_ascii_lowercase().as_str() {
        "full" | "default" | "" => "Full",
        "minimal" => "Minimal",
        "coding" => "Coding",
        "messaging" => "Messaging",
        _ => profile.as_str(),
    }
    .to_string();

    let mut allowed_ids = openclaw_profile_tool_ids(&profile);
    if let Some(ref allow_list) = allow {
        if !allow_list.is_empty() && !allow_list.iter().any(|s| s.eq_ignore_ascii_case("*")) {
            let mut from_allow = std::collections::HashSet::new();
            for entry in allow_list {
                let e = entry.trim().to_ascii_lowercase();
                if e == "*" {
                    for (id, _, _) in OPENCLAW_TOOL_INVENTORY {
                        from_allow.insert((*id).to_string());
                    }
                    break;
                }
                if e.starts_with("group:") {
                    let group = e.strip_prefix("group:").unwrap_or("").trim();
                    for (id, _, _) in OPENCLAW_TOOL_INVENTORY {
                        let in_group = match group {
                            "fs" => matches!(*id, "read" | "write" | "edit" | "apply_patch"),
                            "runtime" => matches!(*id, "exec" | "bash" | "process"),
                            "web" => matches!(*id, "web_search" | "web_fetch"),
                            "memory" => matches!(*id, "memory_search" | "memory_get"),
                            "sessions" => id.starts_with("session"),
                            "messaging" => *id == "message",
                            "ui" => matches!(*id, "browser" | "canvas"),
                            "automation" => matches!(*id, "cron" | "gateway"),
                            "nodes" => *id == "nodes",
                            "openclaw" => true,
                            _ => false,
                        };
                        if in_group {
                            from_allow.insert((*id).to_string());
                        }
                    }
                } else {
                    from_allow.insert(entry.trim().to_string());
                }
            }
            if !from_allow.is_empty() {
                allowed_ids = from_allow;
            }
        }
    }
    for d in &deny {
        if d == "*" {
            allowed_ids.clear();
            break;
        }
        if d.starts_with("group:") {
            let group = d.strip_prefix("group:").unwrap_or("").trim();
            let in_deny_group = |id: &str| -> bool {
                matches!(
                    (group, id),
                    ("fs", "read" | "write" | "edit" | "apply_patch")
                        | ("runtime", "exec" | "bash" | "process")
                        | ("web", "web_search" | "web_fetch")
                        | ("memory", "memory_search" | "memory_get")
                        | ("messaging", "message")
                        | ("ui", "browser" | "canvas")
                        | ("automation", "cron" | "gateway")
                        | ("nodes", "nodes")
                ) || (group == "sessions" && id.starts_with("session"))
                    || (group == "openclaw")
            };
            allowed_ids.retain(|id| !in_deny_group(id.as_str()));
        } else {
            allowed_ids.retain(|id| !id.eq_ignore_ascii_case(d));
        }
    }

    (profile, profile_label, allowed_ids)
}

#[tauri::command]
fn load_openclaw_tools_list(agent_id: Option<String>) -> Result<OpenClawToolsListResponse, String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let parsed: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let root = value_as_object(&parsed).ok_or("openclaw.json 根节点不是对象")?;

    let agent_obj = if let Some(ref id) = agent_id {
        let id_trim = id.trim();
        if id_trim.is_empty() {
            None
        } else {
            root.get("agents")
                .and_then(Value::as_object)
                .and_then(|a| a.get("list"))
                .and_then(Value::as_array)
                .and_then(|list| {
                    list.iter().find_map(|item| {
                        let obj = value_as_object(item)?;
                        let aid = obj
                            .get("id")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .unwrap_or("");
                        if aid.eq_ignore_ascii_case(id_trim) {
                            Some(obj)
                        } else {
                            None
                        }
                    })
                })
        }
    } else {
        None
    };

    let (profile, profile_label, allowed_ids) = openclaw_resolve_tools_from_config(root, agent_obj);

    let tools: Vec<OpenClawToolListItem> = OPENCLAW_TOOL_INVENTORY
        .iter()
        .map(|(id, name, desc)| {
            let enabled = allowed_ids.contains(&id.to_ascii_lowercase());
            OpenClawToolListItem {
                id: (*id).to_string(),
                name: (*name).to_string(),
                description: (*desc).to_string(),
                category: openclaw_tool_category(id).to_string(),
                enabled,
            }
        })
        .collect();

    Ok(OpenClawToolsListResponse {
        profile,
        profile_label,
        tools,
    })
}

/// 保存 OpenClaw 工具配置：
/// - scope = "agent"：写入 agents.list[].tools（需要 agent_id）
/// - scope = "global"：写入根 tools
/// enabled_tool_ids 为最终启用集合（会转换为 allow/deny）
#[tauri::command]
fn save_openclaw_tools_config(
    agent_id: Option<String>,
    scope: Option<String>,
    profile: Option<String>,
    enabled_tool_ids: Vec<String>,
) -> Result<(), String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut parsed: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let scope_mode = scope
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("agent")
        .to_ascii_lowercase();

    let canonical_tool_ids: std::collections::HashSet<String> = OPENCLAW_TOOL_INVENTORY
        .iter()
        .map(|(id, _, _)| (*id).to_string())
        .collect();

    let mut enabled_set = std::collections::HashSet::new();
    for raw_id in enabled_tool_ids {
        let trimmed = raw_id.trim();
        if trimmed.is_empty() {
            continue;
        }
        let canonical = OPENCLAW_TOOL_INVENTORY
            .iter()
            .find_map(|(id, _, _)| {
                if id.eq_ignore_ascii_case(trimmed) {
                    Some((*id).to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| trimmed.to_ascii_lowercase());
        enabled_set.insert(canonical);
    }

    // 按固定顺序输出 allow，便于 diff 稳定可读。
    let mut ordered_allow: Vec<String> = OPENCLAW_TOOL_INVENTORY
        .iter()
        .map(|(id, _, _)| (*id).to_string())
        .filter(|id| enabled_set.contains(id))
        .collect();
    let mut extras: Vec<String> = enabled_set
        .iter()
        .filter(|id| !canonical_tool_ids.contains((*id).as_str()))
        .cloned()
        .collect();
    extras.sort();
    let has_extras = !extras.is_empty();
    ordered_allow.extend(extras);

    let known_enabled_count = OPENCLAW_TOOL_INVENTORY
        .iter()
        .filter(|(id, _, _)| enabled_set.contains(*id))
        .count();

    let allow = if known_enabled_count == OPENCLAW_TOOL_INVENTORY.len() && !has_extras {
        vec![Value::String("*".to_string())]
    } else if ordered_allow.is_empty() {
        vec![Value::String("*".to_string())]
    } else {
        ordered_allow
            .iter()
            .map(|id| Value::String(id.to_string()))
            .collect()
    };

    let deny = if ordered_allow.is_empty() {
        Some(vec![Value::String("*".to_string())])
    } else {
        None
    };

    let profile_value = profile
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default")
        .to_string();

    let tools_obj = match scope_mode.as_str() {
        "global" => root
            .entry("tools")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .ok_or("tools 不是对象")?,
        "agent" => {
            let agent_key = agent_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or("scope=agent 时 agentId 不能为空")?;
            let agents = root
                .entry("agents")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or("agents 不是对象")?;
            let list = agents
                .entry("list")
                .or_insert_with(|| serde_json::json!([]))
                .as_array_mut()
                .ok_or("agents.list 不是数组")?;

            let target = list
                .iter_mut()
                .find_map(|item| {
                    let obj = item.as_object_mut()?;
                    let id = obj
                        .get("id")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .unwrap_or("");
                    if id.eq_ignore_ascii_case(agent_key) {
                        Some(obj)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| format!("未找到 id 为 {} 的员工。", agent_key))?;

            target
                .entry("tools")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
                .ok_or("agents.list[].tools 不是对象")?
        }
        _ => {
            return Err("scope 仅支持 agent 或 global".to_string());
        }
    };

    tools_obj.insert("profile".to_string(), Value::String(profile_value));
    tools_obj.insert("allow".to_string(), Value::Array(allow));
    if let Some(deny_arr) = deny {
        tools_obj.insert("deny".to_string(), Value::Array(deny_arr));
    } else {
        tools_obj.remove("deny");
    }

    write_openclaw_config_value(&config_path, &parsed)
}

#[tauri::command]
fn save_openclaw_agent_model(agent_id: String, model: String) -> Result<(), String> {
    let normalized_agent_id = agent_id.trim();
    if normalized_agent_id.is_empty() {
        return Err("agentId 不能为空".to_string());
    }
    let normalized_model = model.trim();
    if normalized_model.is_empty() {
        return Err("model 不能为空".to_string());
    }

    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let agents = root
        .entry("agents")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 agents 不是对象")?;

    let mut updated_in_list = false;
    let has_non_empty_list = agents
        .get("list")
        .and_then(Value::as_array)
        .map(|list| !list.is_empty())
        .unwrap_or(false);

    if let Some(list) = agents.get_mut("list").and_then(Value::as_array_mut) {
        if let Some(target) = list.iter_mut().find_map(|item| {
            let obj = item.as_object_mut()?;
            let id = obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            if id.eq_ignore_ascii_case(normalized_agent_id) {
                Some(obj)
            } else {
                None
            }
        }) {
            target.insert(
                "model".to_string(),
                Value::String(normalized_model.to_string()),
            );
            updated_in_list = true;
        }
    }

    if !updated_in_list {
        if has_non_empty_list {
            return Err(format!("未找到 id 为 {} 的员工。", normalized_agent_id));
        }

        if !matches!(agents.get("defaults"), Some(Value::Object(_))) {
            agents.insert("defaults".to_string(), serde_json::json!({}));
        }
        let defaults = agents
            .get_mut("defaults")
            .and_then(Value::as_object_mut)
            .ok_or("openclaw.json 的 agents.defaults 不是对象")?;
        let existing_fallbacks = defaults
            .get("model")
            .and_then(Value::as_object)
            .and_then(|obj| obj.get("fallbacks"))
            .cloned();
        let fallbacks = match existing_fallbacks {
            Some(Value::Array(items)) => Value::Array(items),
            _ => Value::Array(Vec::new()),
        };
        defaults.insert(
            "model".to_string(),
            serde_json::json!({
                "primary": normalized_model.to_string(),
                "fallbacks": fallbacks
            }),
        );
    }

    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn load_memory_file_snapshot() -> Result<SourceFileSnapshotResponse, String> {
    let scopes = load_editable_scopes();
    let items = load_memory_file_items();
    let (db_count, db_file_count, db_chunk_count) = summarize_memory_store(&scopes);
    let scope_count = scopes.len();
    let source_path = format!(
        "{} | {}",
        resolve_workspace_main_root().display(),
        resolve_openclaw_home_path().join("memory").display()
    );
    let existing_count = items.iter().filter(|item| item.exists).count();
    let missing_count = items.len().saturating_sub(existing_count);
    let detail = if items.is_empty() {
        format!(
            "已扫描 {} 个工作区与 {} 个记忆库，当前未发现可展示的文件型记忆。memory-core 状态：{} files / {} chunks。",
            scope_count, db_count, db_file_count, db_chunk_count
        )
    } else {
        format!(
            "已整理 {} 个记忆文件槽位（存在 {}，缺失 {}），并检查了 {} 个记忆库（{} files / {} chunks）。",
            items.len(), existing_count, missing_count, db_count, db_file_count, db_chunk_count
        )
    };
    Ok(SourceFileSnapshotResponse {
        source_path,
        detail,
        items,
    })
}

#[tauri::command]
fn load_document_file_snapshot() -> Result<SourceFileSnapshotResponse, String> {
    let items = load_document_file_items();
    let existing_count = items.iter().filter(|item| item.exists).count();
    let missing_count = items.len().saturating_sub(existing_count);
    Ok(SourceFileSnapshotResponse {
        source_path: resolve_workspace_main_root().display().to_string(),
        detail: format!(
            "已整理 {} 份核心文件（存在 {}，缺失 {}）。",
            items.len(),
            existing_count,
            missing_count
        ),
        items,
    })
}

#[tauri::command]
fn load_openclaw_resource_snapshot(
    kind: String,
    agent_id: Option<String>,
) -> Result<SourceFileSnapshotResponse, String> {
    let normalized_kind = kind.trim().to_ascii_lowercase();

    if normalized_kind == "skill" {
        let items = load_skill_file_items();
        let existing_count = items.iter().filter(|item| item.exists).count();
        let missing_count = items.len().saturating_sub(existing_count);
        return Ok(SourceFileSnapshotResponse {
            source_path: resolve_openclaw_home_path()
                .join("skills")
                .display()
                .to_string(),
            detail: format!(
                "已整理 {} 份 OpenClaw skills（存在 {}，缺失 {}）。",
                items.len(),
                existing_count,
                missing_count
            ),
            items,
        });
    }

    if normalized_kind == "tool" {
        let items = load_tool_file_items(agent_id.as_deref());
        let existing_count = items.iter().filter(|item| item.exists).count();
        let missing_count = items.len().saturating_sub(existing_count);
        let label = agent_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("全部员工");
        return Ok(SourceFileSnapshotResponse {
            source_path: resolve_openclaw_home_path().display().to_string(),
            detail: format!(
                "已整理 {} 的 {} 份工具文件（存在 {}，缺失 {}）。",
                label,
                items.len(),
                existing_count,
                missing_count
            ),
            items,
        });
    }

    Err("不支持的资源类型。".to_string())
}

fn guess_audio_mime_type(path: &Path) -> &'static str {
    match guess_media_mime_type(path) {
        "application/octet-stream" => "application/octet-stream",
        resolved => resolved,
    }
}

fn guess_media_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("avif") => "image/avif",
        Some("heic") | Some("heif") => "image/heic",
        Some("tif") | Some("tiff") => "image/tiff",
        Some("ico") => "image/x-icon",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("m4a") => "audio/mp4",
        Some("aac") => "audio/aac",
        Some("ogg") => "audio/ogg",
        Some("flac") => "audio/flac",
        Some("opus") => "audio/opus",
        Some("wma") => "audio/x-ms-wma",
        Some("amr") => "audio/amr",
        Some("aif") | Some("aiff") => "audio/aiff",
        Some("caf") => "audio/x-caf",
        Some("alac") => "audio/alac",
        Some("mp4") | Some("m4v") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("mkv") => "video/x-matroska",
        Some("avi") => "video/x-msvideo",
        Some("wmv") => "video/x-ms-wmv",
        Some("flv") => "video/x-flv",
        Some("ogv") => "video/ogg",
        Some("3gp") => "video/3gpp",
        Some("mpeg") | Some("mpg") => "video/mpeg",
        Some("ts") => "video/mp2t",
        Some("m2ts") | Some("mts") => "video/mp2t",
        Some("html") | Some("htm") => "text/html",
        _ => "application/octet-stream",
    }
}

#[tauri::command]
fn read_local_audio_file(path: String) -> Result<AudioFilePayload, String> {
    let resolved = PathBuf::from(path.trim());
    if resolved.as_os_str().is_empty() {
        return Err("音频路径不能为空。".to_string());
    }

    if !resolved.is_file() {
        return Err(format!("音频文件不存在：{}", resolved.display()));
    }

    let bytes = std::fs::read(&resolved)
        .map_err(|error| format!("读取音频文件失败（{}）：{error}", resolved.display()))?;
    let mime_type = guess_audio_mime_type(&resolved).to_string();
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let data_url = format!("data:{mime_type};base64,{encoded}");
    let file_name = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("audio")
        .to_string();

    Ok(AudioFilePayload {
        data_url,
        mime_type,
        file_name,
    })
}

const LOCAL_MEDIA_PREVIEW_MAX_BYTES: usize = 36 * 1024 * 1024;
const CHAT_ATTACHMENT_PERSIST_MAX_BYTES: usize = 16 * 1024 * 1024;

#[tauri::command]
fn read_local_media_file(path: String) -> Result<LocalMediaFilePayload, String> {
    let normalized = path.trim();
    if normalized.is_empty() {
        return Err("本地文件路径不能为空。".to_string());
    }

    let expanded = expand_home_path(normalized);
    let resolved = if expanded.exists() {
        std::fs::canonicalize(&expanded).unwrap_or(expanded)
    } else {
        expanded
    };

    if !resolved.exists() || !resolved.is_file() {
        return Err(format!("本地文件不存在：{}", resolved.display()));
    }

    let metadata = std::fs::metadata(&resolved)
        .map_err(|error| format!("读取文件信息失败（{}）：{error}", resolved.display()))?;
    let byte_length = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    if byte_length > LOCAL_MEDIA_PREVIEW_MAX_BYTES {
        let max_mb = LOCAL_MEDIA_PREVIEW_MAX_BYTES / (1024 * 1024);
        return Err(format!(
            "文件过大（{} MB），暂不支持内存预览，请点击文件夹图标打开。",
            max_mb
        ));
    }

    let bytes = std::fs::read(&resolved)
        .map_err(|error| format!("读取本地文件失败（{}）：{error}", resolved.display()))?;
    let mime_type = guess_media_mime_type(&resolved).to_string();
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let data_url = format!("data:{mime_type};base64,{encoded}");
    let file_name = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("media")
        .to_string();

    Ok(LocalMediaFilePayload {
        data_url,
        mime_type,
        file_name,
        byte_length,
    })
}

fn guess_media_extension_from_mime_type(mime_type_raw: &str) -> &'static str {
    match mime_type_raw.trim().to_ascii_lowercase().as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/svg+xml" => "svg",
        "image/heic" => "heic",
        "image/heif" => "heif",
        "image/avif" => "avif",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "audio/mp4" => "m4a",
        "audio/aac" => "aac",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "video/x-msvideo" => "avi",
        "video/x-matroska" => "mkv",
        "text/html" => "html",
        "application/xhtml+xml" => "html",
        _ => "bin",
    }
}

fn sanitize_chat_attachment_file_stem(file_name: &str) -> String {
    let candidate = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("clipboard");
    let sanitized: String = candidate
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&ch) {
                ch
            } else if ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('_').trim();
    if trimmed.is_empty() {
        "clipboard".to_string()
    } else {
        trimmed.chars().take(48).collect()
    }
}

fn build_chat_attachment_output_root(workspace: Option<String>) -> PathBuf {
    if let Some(candidate) = workspace {
        let trimmed = candidate.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_absolute() {
                return path.join(".dragonclaw").join("chat-attachments");
            }
        }
    }
    resolve_openclaw_home_path().join("chat-attachments")
}

#[tauri::command]
fn persist_chat_attachment_data_url(
    file_name: String,
    data_url: String,
    workspace: Option<String>,
) -> Result<String, String> {
    let normalized = data_url.trim();
    if normalized.is_empty() {
        return Err("附件内容为空，无法保存。".to_string());
    }
    if !normalized.starts_with("data:") {
        return Err("仅支持 data URL 附件保存。".to_string());
    }

    let payload = &normalized["data:".len()..];
    let Some((meta, encoded)) = payload.split_once(',') else {
        return Err("附件 data URL 格式无效。".to_string());
    };

    if !meta.to_ascii_lowercase().contains(";base64") {
        return Err("附件 data URL 必须为 base64 编码。".to_string());
    }

    let mime_type = meta
        .split(';')
        .next()
        .map(|value| value.trim().to_ascii_lowercase())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let extension = Path::new(&file_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| guess_media_extension_from_mime_type(&mime_type).to_string());

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .map_err(|error| format!("附件数据解码失败：{error}"))?;
    if bytes.is_empty() {
        return Err("附件内容为空，无法保存。".to_string());
    }
    if bytes.len() > CHAT_ATTACHMENT_PERSIST_MAX_BYTES {
        let max_mb = CHAT_ATTACHMENT_PERSIST_MAX_BYTES / (1024 * 1024);
        return Err(format!("附件过大，超过 {max_mb} MB 限制。"));
    }

    let output_root = build_chat_attachment_output_root(workspace);
    std::fs::create_dir_all(&output_root)
        .map_err(|error| format!("创建附件目录失败（{}）：{error}", output_root.display()))?;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let mut random_bytes = [0_u8; 4];
    if getrandom(&mut random_bytes).is_err() {
        random_bytes = [0x12, 0x34, 0x56, 0x78];
    }
    let random_hex = random_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let safe_stem = sanitize_chat_attachment_file_stem(&file_name);
    let output_name = format!("{safe_stem}-{now_ms}-{random_hex}.{extension}");
    let output_path = output_root.join(output_name);

    std::fs::write(&output_path, bytes)
        .map_err(|error| format!("保存附件失败（{}）：{error}", output_path.display()))?;

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
fn save_source_file(kind: String, source_path: String, content: String) -> Result<String, String> {
    let allowed = if kind == "memory" {
        load_memory_file_items()
    } else if kind == "document" {
        load_document_file_items()
    } else if kind == "skill" {
        load_skill_file_items()
    } else if kind == "tool" {
        load_tool_file_items(None)
    } else {
        return Err("不支持的文件类型。".to_string());
    };

    let Some(target) = allowed
        .into_iter()
        .find(|item| std::path::Path::new(&item.source_path) == std::path::Path::new(&source_path))
    else {
        return Err("目标文件不在允许编辑范围内。".to_string());
    };

    if let Some(parent) = Path::new(&target.source_path).parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(&target.source_path, content).map_err(|error| error.to_string())?;
    Ok(target.source_path)
}

fn strip_wrapping_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        return trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    trimmed.to_string()
}

fn parse_markdown_frontmatter_and_body(
    markdown: &str,
) -> (std::collections::HashMap<String, String>, String) {
    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let mut fields = std::collections::HashMap::new();
    let lines = normalized.split('\n').collect::<Vec<_>>();

    if lines.first().map(|line| line.trim()) != Some("---") {
        return (fields, normalized);
    }

    let mut end_index = None;
    for (index, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_index = Some(index);
            break;
        }
    }

    let Some(end) = end_index else {
        return (fields, normalized);
    };

    for line in &lines[1..end] {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        if key.is_empty() {
            continue;
        }
        fields.insert(key, strip_wrapping_quotes(raw_value));
    }

    let body = if end + 1 < lines.len() {
        lines[end + 1..].join("\n")
    } else {
        String::new()
    };
    (fields, body)
}

fn is_soul_header(header: &str) -> bool {
    let normalized = header.to_ascii_lowercase();
    let soul_keywords = [
        "identity",
        "communication",
        "style",
        "critical rule",
        "critical-rule",
        "rules you must follow",
        "身份",
        "记忆",
        "溝通",
        "沟通",
        "风格",
        "風格",
        "关键规则",
        "關鍵規則",
    ];
    soul_keywords
        .iter()
        .any(|keyword| normalized.contains(keyword))
}

fn split_openclaw_markdown_sections(body: &str) -> (String, String) {
    let mut soul_content = String::new();
    let mut agents_content = String::new();
    let mut current_section = String::new();
    let mut current_target_is_soul = false;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") || trimmed.starts_with("##\t") {
            if !current_section.is_empty() {
                if current_target_is_soul {
                    soul_content.push_str(&current_section);
                } else {
                    agents_content.push_str(&current_section);
                }
            }
            current_section.clear();
            current_target_is_soul = is_soul_header(trimmed);
        }

        current_section.push_str(line);
        current_section.push('\n');
    }

    if !current_section.is_empty() {
        if current_target_is_soul {
            soul_content.push_str(&current_section);
        } else {
            agents_content.push_str(&current_section);
        }
    }

    (soul_content, agents_content)
}

fn normalize_markdown_output(content: String) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}\n")
    }
}

#[tauri::command]
fn install_role_workflow_agent(
    agent_id: String,
    display_name: String,
    content: String,
    source_path: Option<String>,
) -> Result<String, String> {
    let normalized_id = {
        let mut output = String::new();
        for ch in agent_id.trim().chars() {
            if ch.is_ascii_alphanumeric() {
                output.push(ch.to_ascii_lowercase());
            } else if ch == '-' || ch == '_' || ch == '/' || ch == '\\' {
                if !output.ends_with('-') {
                    output.push('-');
                }
            }
        }
        output = output.trim_matches('-').to_string();
        if output.is_empty() {
            return Err("角色 ID 为空，无法安装。".to_string());
        }
        output
    };
    if normalized_id.eq_ignore_ascii_case("main") {
        return Err("主控员工 main 受保护，不能被替换。".to_string());
    }

    let workspace_root = resolve_openclaw_home_path().join(format!("workspace-{normalized_id}"));
    std::fs::create_dir_all(&workspace_root).map_err(|error| {
        format!(
            "创建角色工作区失败（{}）：{error}",
            workspace_root.display()
        )
    })?;

    let source_label = source_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let raw_markdown = if content.trim().is_empty() {
        format!(
            "# {name}\n\n来源：{source}\n\n> 该角色由 DragonClaw 安装，请补充具体职责内容。\n",
            name = display_name.trim(),
            source = source_label
        )
    } else {
        let mut normalized = content.replace("\r\n", "\n");
        if !normalized.ends_with('\n') {
            normalized.push('\n');
        }
        normalized
    };

    let (frontmatter_fields, body_markdown_raw) =
        parse_markdown_frontmatter_and_body(&raw_markdown);
    let body_markdown = normalize_markdown_output(body_markdown_raw);
    let (soul_split, agents_split) = split_openclaw_markdown_sections(&body_markdown);
    let mut soul_markdown = normalize_markdown_output(soul_split);
    let mut agents_markdown = normalize_markdown_output(agents_split);

    if agents_markdown.is_empty() {
        agents_markdown = if body_markdown.is_empty() {
            normalize_markdown_output(raw_markdown.clone())
        } else {
            body_markdown.clone()
        };
    }

    let configured_name = display_name.trim();
    let frontmatter_name = frontmatter_fields
        .get("name")
        .map(String::as_str)
        .unwrap_or("")
        .trim();
    let next_name = if !configured_name.is_empty() {
        configured_name.to_string()
    } else if !frontmatter_name.is_empty() {
        frontmatter_name.to_string()
    } else {
        normalized_id.clone()
    };

    let frontmatter_desc = frontmatter_fields
        .get("description")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let frontmatter_vibe = frontmatter_fields
        .get("vibe")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let frontmatter_emoji = frontmatter_fields
        .get("emoji")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if soul_markdown.is_empty() {
        let soul_summary = if !frontmatter_desc.is_empty() {
            frontmatter_desc.clone()
        } else {
            format!("该角色来源于 {source_label}，请补充身份、记忆、沟通风格与关键规则。")
        };
        soul_markdown =
            format!("## 你的身份与记忆\n\n- **角色**：{next_name}\n- **定位**：{soul_summary}\n");
    }

    let identity_title = if frontmatter_emoji.is_empty() {
        format!("# {next_name}")
    } else {
        format!("# {} {next_name}", frontmatter_emoji)
    };
    let identity_summary = if !frontmatter_vibe.is_empty() {
        frontmatter_vibe
    } else if !frontmatter_desc.is_empty() {
        frontmatter_desc
    } else {
        format!("来源：{source_label}")
    };
    let identity_markdown = format!("{identity_title}\n\n{identity_summary}\n");

    let agents_md_path = workspace_root.join("AGENTS.md");
    std::fs::write(&agents_md_path, agents_markdown)
        .map_err(|error| format!("写入角色文件失败（{}）：{error}", agents_md_path.display()))?;
    let soul_md_path = workspace_root.join("SOUL.md");
    std::fs::write(&soul_md_path, soul_markdown)
        .map_err(|error| format!("写入角色文件失败（{}）：{error}", soul_md_path.display()))?;
    let identity_md_path = workspace_root.join("IDENTITY.md");
    std::fs::write(&identity_md_path, identity_markdown).map_err(|error| {
        format!(
            "写入角色文件失败（{}）：{error}",
            identity_md_path.display()
        )
    })?;

    let config_path = resolve_openclaw_config_path();
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
            serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
        }
    } else {
        serde_json::json!({})
    };

    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象，无法安装角色。")?;
    let agents = root
        .entry("agents")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("openclaw.json 的 agents 字段不是对象。")?;
    let list = agents
        .entry("list")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or("openclaw.json 的 agents.list 字段不是数组。")?;

    let workspace_hint = format!("~/.openclaw/workspace-{normalized_id}");
    let mut updated = false;
    for item in list.iter_mut() {
        let Some(obj) = item.as_object_mut() else {
            continue;
        };
        let current_id = obj
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if !current_id.eq_ignore_ascii_case(&normalized_id) {
            continue;
        }
        obj.insert("id".to_string(), Value::String(normalized_id.clone()));
        obj.insert("name".to_string(), Value::String(next_name.clone()));
        obj.insert(
            "workspace".to_string(),
            Value::String(workspace_hint.clone()),
        );
        updated = true;
        break;
    }

    if !updated {
        list.push(serde_json::json!({
            "id": normalized_id,
            "name": next_name,
            "workspace": workspace_hint
        }));
    }
    ensure_agents_list_has_main(list);

    write_openclaw_config_value(&config_path, &parsed)?;

    Ok(format!(
        "角色已安装：{}（工作区：{}）",
        agents_md_path
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("workspace"),
        agents_md_path.display()
    ))
}

#[tauri::command]
fn remove_role_workflow_agent(agent_id: String, delete_files: bool) -> Result<String, String> {
    let normalized_id = {
        let mut output = String::new();
        for ch in agent_id.trim().chars() {
            if ch.is_ascii_alphanumeric() {
                output.push(ch.to_ascii_lowercase());
            } else if ch == '-' || ch == '_' || ch == '/' || ch == '\\' {
                if !output.ends_with('-') {
                    output.push('-');
                }
            }
        }
        output = output.trim_matches('-').to_string();
        if output.is_empty() {
            return Err("角色 ID 为空，无法删除。".to_string());
        }
        output
    };
    if normalized_id.eq_ignore_ascii_case("main") {
        return Err("主控员工 main 受保护，不能删除。".to_string());
    }

    let source_path = resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象，无法删除角色。")?;

    let mut removed_name = normalized_id.clone();
    let mut configured_workspace: Option<String> = None;
    let mut removed_count = 0usize;
    if let Some(list) = root
        .get_mut("agents")
        .and_then(Value::as_object_mut)
        .and_then(|agents| agents.get_mut("list"))
        .and_then(Value::as_array_mut)
    {
        list.retain(|item| {
            let Some(obj) = item.as_object() else {
                return true;
            };
            let current_id = obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            let should_remove =
                !current_id.is_empty() && current_id.eq_ignore_ascii_case(&normalized_id);
            if should_remove {
                removed_count += 1;
                if let Some(name) = obj
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    removed_name = name.to_string();
                }
                if configured_workspace.is_none() {
                    configured_workspace = obj
                        .get("workspace")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned);
                }
            }
            !should_remove
        });
        ensure_agents_list_has_main(list);
    }

    if removed_count == 0 {
        return Err(format!("未找到角色 ID「{}」，无法删除。", normalized_id));
    }

    let mut removed_binding_count = 0usize;
    if let Some(bindings) = root.get_mut("bindings").and_then(Value::as_array_mut) {
        let before = bindings.len();
        bindings.retain(|item| {
            let target_id = item
                .get("agentId")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("");
            !target_id.eq_ignore_ascii_case(&normalized_id)
        });
        removed_binding_count = before.saturating_sub(bindings.len());
    }

    let mut cleared_channel_binding_count = 0usize;
    if let Some(channels) = root.get_mut("channels").and_then(Value::as_object_mut) {
        for channel_section in channels.values_mut() {
            let Some(section_obj) = channel_section.as_object_mut() else {
                continue;
            };
            if let Some(accounts) = section_obj
                .get_mut("accounts")
                .and_then(Value::as_object_mut)
            {
                for account in accounts.values_mut() {
                    let Some(account_obj) = account.as_object_mut() else {
                        continue;
                    };
                    let is_target = account_obj
                        .get("agentId")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .map(|value| value.eq_ignore_ascii_case(&normalized_id))
                        .unwrap_or(false);
                    if is_target {
                        account_obj.remove("agentId");
                        cleared_channel_binding_count += 1;
                    }
                }
            }
        }
    }

    write_openclaw_config_value(&source_path, &parsed)?;

    let mut removed_paths: Vec<String> = Vec::new();
    let mut delete_warnings: Vec<String> = Vec::new();
    if delete_files {
        let mut candidates = Vec::<PathBuf>::new();
        if let Some(workspace) = configured_workspace.as_deref() {
            let expanded = expand_home_path(workspace);
            let candidate = if expanded.is_absolute() {
                expanded
            } else {
                resolve_openclaw_home_path().join(expanded)
            };
            candidates.push(candidate);
        }
        let runtime_agents_root = resolve_openclaw_config_path()
            .parent()
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".openclaw"))
            .join("agents");
        candidates.push(runtime_agents_root.join(&normalized_id));
        candidates.push(resolve_openclaw_home_path().join(format!("workspace-{normalized_id}")));
        candidates.push(resolve_workspace_agents_root().join(&normalized_id));

        let mut seen = std::collections::HashSet::new();
        for path in candidates {
            let key = path.to_string_lossy().to_string();
            if !seen.insert(key.clone()) || !path.exists() {
                continue;
            }
            let result = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
            match result {
                Ok(_) => removed_paths.push(key),
                Err(error) => delete_warnings.push(format!("{key}: {error}")),
            }
        }
    }

    let mut detail = format!("角色「{}」已删除。", removed_name);
    let binding_cleanup_count = removed_binding_count + cleared_channel_binding_count;
    if binding_cleanup_count > 0 {
        detail.push_str(&format!(" 已清理 {} 条绑定关系。", binding_cleanup_count));
    }
    if delete_files {
        if removed_paths.is_empty() {
            detail.push_str(" 未删除配置文件（未找到可删除目录或文件）。");
        } else {
            detail.push_str(&format!(
                " 已删除 {} 个配置文件/目录。",
                removed_paths.len()
            ));
        }
        if !delete_warnings.is_empty() {
            detail.push_str(" 部分配置文件删除失败：");
            detail.push_str(&delete_warnings.join("；"));
        }
    } else {
        detail.push_str(" 配置文件已保留。");
    }

    Ok(detail)
}

fn load_recent_activity_from_session_file(session_file: &str) -> (Option<String>, Option<String>) {
    let raw = match std::fs::read_to_string(session_file) {
        Ok(value) => value,
        Err(_) => return (None, None),
    };

    let mut current_work = None;
    let mut recent_output = None;

    for line in raw.lines().rev() {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let Some(message) = parsed.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let content = message.get("content");

        if recent_output.is_none() && role == "assistant" {
            if let Some(content) = content.and_then(extract_text_from_message_content) {
                recent_output = Some(sanitize_staff_output(&content));
            }
        }

        if current_work.is_none() && role == "user" {
            if let Some(content) = content.and_then(extract_text_from_message_content) {
                current_work = Some(sanitize_staff_output(&content));
            }
        }

        if current_work.is_some() && recent_output.is_some() {
            break;
        }
    }

    (current_work, recent_output)
}

fn derive_status_label(updated_at_ms: i64) -> String {
    let now_ms = current_timestamp_millis() as i64;
    let delta = now_ms.saturating_sub(updated_at_ms);
    if delta <= 45 * 60 * 1000 {
        "工作中".to_string()
    } else {
        "待命".to_string()
    }
}

fn load_scheduled_agents() -> std::collections::HashSet<String> {
    let cron_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");
    let raw = match std::fs::read_to_string(cron_path) {
        Ok(value) => value,
        Err(_) => return std::collections::HashSet::new(),
    };
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => return std::collections::HashSet::new(),
    };

    let mut agents = std::collections::HashSet::new();
    if let Some(jobs) = parsed.get("jobs").and_then(Value::as_array) {
        for job in jobs {
            let Some(obj) = job.as_object() else {
                continue;
            };
            let enabled = obj.get("enabled").and_then(Value::as_bool).unwrap_or(true);
            if !enabled {
                continue;
            }
            for key in ["agentId", "ownerAgentId", "sessionTarget"] {
                if let Some(agent_id) = obj.get(key).and_then(Value::as_str) {
                    let trimmed = agent_id.trim();
                    if !trimmed.is_empty() && trimmed != "isolated" {
                        agents.insert(trimmed.to_string());
                    }
                }
            }
        }
    }
    agents
}

#[derive(Debug, Default)]
struct RuntimeSessionSummary {
    latest_updated_at_ms: Option<i64>,
    latest_model: Option<String>,
    latest_session_file: Option<String>,
}

fn load_runtime_session_summary(agent_id: &str) -> RuntimeSessionSummary {
    let sessions_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents")
        .join(agent_id)
        .join("sessions")
        .join("sessions.json");

    let raw = match std::fs::read_to_string(sessions_path) {
        Ok(value) => value,
        Err(_) => return RuntimeSessionSummary::default(),
    };
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => return RuntimeSessionSummary::default(),
    };

    let mut best = RuntimeSessionSummary::default();
    let mut best_updated_at = i64::MIN;
    let Some(entries) = parsed.as_object() else {
        return best;
    };

    for value in entries.values() {
        let Some(session) = value.as_object() else {
            continue;
        };
        let updated_at = session
            .get("updatedAt")
            .and_then(Value::as_i64)
            .or_else(|| session.get("lastActivityAt").and_then(Value::as_i64))
            .unwrap_or(0);
        if updated_at < best_updated_at {
            continue;
        }

        let latest_model = session
            .get("model")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let latest_session_file = session
            .get("sessionFile")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        best_updated_at = updated_at;
        best = RuntimeSessionSummary {
            latest_updated_at_ms: if updated_at > 0 {
                Some(updated_at)
            } else {
                None
            },
            latest_model,
            latest_session_file,
        };
    }

    best
}

fn derive_session_target_from_session_key(session_key: &str, agent_id: &str) -> Option<String> {
    let normalized_agent_id = agent_id.trim().to_lowercase();
    if normalized_agent_id.is_empty() {
        return None;
    }
    let normalized_session_key = session_key.trim().to_lowercase();
    let prefix = format!("agent:{normalized_agent_id}:");
    if !normalized_session_key.starts_with(&prefix) {
        return None;
    }
    let target = normalized_session_key[prefix.len()..].trim();
    if target.is_empty() {
        return Some(normalized_agent_id);
    }
    Some(target.to_string())
}

fn count_session_messages_from_file(session_file: &str) -> usize {
    let raw = match std::fs::read_to_string(session_file) {
        Ok(value) => value,
        Err(_) => return 0,
    };

    let mut count = 0usize;
    for line in raw.lines() {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if parsed.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }
        let Some(message) = parsed.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if role == "user" || role == "assistant" {
            count = count.saturating_add(1);
        }
    }
    count
}

fn build_session_preview_text(session_file: &str) -> String {
    let (current_work, recent_output) = load_recent_activity_from_session_file(session_file);
    if let Some(output) = recent_output {
        if !output.trim().is_empty() {
            return output;
        }
    }
    if let Some(work) = current_work {
        if !work.trim().is_empty() {
            return work;
        }
    }
    "暂无消息".to_string()
}

#[tauri::command]
fn load_openclaw_agent_sessions_snapshot(
    agent_id: String,
) -> Result<OpenClawAgentSessionsSnapshotResponse, String> {
    let normalized_agent_id = agent_id.trim().to_lowercase();
    if normalized_agent_id.is_empty() {
        return Ok(OpenClawAgentSessionsSnapshotResponse {
            detail: "agentId 为空，无法读取会话列表。".to_string(),
            sessions: Vec::new(),
        });
    }

    let sessions_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents")
        .join(&normalized_agent_id)
        .join("sessions")
        .join("sessions.json");

    let raw = match std::fs::read_to_string(&sessions_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawAgentSessionsSnapshotResponse {
                detail: format!("未找到 {} 的 OpenClaw 会话文件。", normalized_agent_id),
                sessions: Vec::new(),
            })
        }
    };
    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return Err(format!("会话文件解析失败: {error}"));
        }
    };

    let Some(entries) = parsed.as_object() else {
        return Ok(OpenClawAgentSessionsSnapshotResponse {
            detail: "会话文件格式异常。".to_string(),
            sessions: Vec::new(),
        });
    };

    let mut sessions = Vec::new();
    for (session_key, value) in entries {
        let Some(session_target) =
            derive_session_target_from_session_key(session_key, &normalized_agent_id)
        else {
            continue;
        };
        let Some(session_obj) = value.as_object() else {
            continue;
        };

        let session_id = session_obj
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_default();
        let updated_at_ms = value_as_i64(session_obj.get("updatedAt"))
            .or_else(|| value_as_i64(session_obj.get("lastActivityAt")))
            .unwrap_or(0);
        let session_file = session_obj
            .get("sessionFile")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned);
        let fallback_session_file = if session_id.is_empty() {
            None
        } else {
            let fallback_path = sessions_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("{session_id}.jsonl"));
            if fallback_path.exists() {
                Some(fallback_path.display().to_string())
            } else {
                None
            }
        };
        let resolved_session_file = session_file.or(fallback_session_file);
        let message_count = resolved_session_file
            .as_deref()
            .map(count_session_messages_from_file)
            .unwrap_or(0);
        let preview = resolved_session_file
            .as_deref()
            .map(build_session_preview_text)
            .unwrap_or_else(|| "暂无消息".to_string());
        let last_channel = session_obj
            .get("lastChannel")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned);
        let chat_type = session_obj
            .get("chatType")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned);

        sessions.push(OpenClawAgentSessionSnapshotItem {
            session_key: session_key.to_string(),
            session_target,
            session_id,
            updated_at_ms,
            message_count,
            preview,
            last_channel,
            chat_type,
        });
    }

    sessions.sort_by(|left, right| {
        right
            .updated_at_ms
            .cmp(&left.updated_at_ms)
            .then_with(|| left.session_target.cmp(&right.session_target))
    });

    Ok(OpenClawAgentSessionsSnapshotResponse {
        detail: format!(
            "已读取 {} 的 {} 条 OpenClaw 会话。",
            normalized_agent_id,
            sessions.len()
        ),
        sessions,
    })
}

#[tauri::command]
fn load_openclaw_agent_session_history(
    agent_id: String,
    session_key: String,
) -> Result<OpenClawAgentSessionHistoryResponse, String> {
    let normalized_agent_id = agent_id.trim().to_lowercase();
    let normalized_session_key = session_key.trim().to_lowercase();
    if normalized_agent_id.is_empty() || normalized_session_key.is_empty() {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "agentId 或 sessionKey 为空。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    }

    if derive_session_target_from_session_key(&normalized_session_key, &normalized_agent_id)
        .is_none()
    {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "会话不属于当前 Agent。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    }

    let sessions_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("agents")
        .join(&normalized_agent_id)
        .join("sessions")
        .join("sessions.json");

    let raw = match std::fs::read_to_string(&sessions_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawAgentSessionHistoryResponse {
                detail: "未找到 OpenClaw 会话文件。".to_string(),
                session_key: normalized_session_key,
                messages: Vec::new(),
            })
        }
    };
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("会话文件解析失败: {error}"))?;
    let Some(entries) = parsed.as_object() else {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "会话文件格式异常。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    };

    let mut matched_session: Option<&serde_json::Map<String, Value>> = None;
    for (key, value) in entries {
        if key.trim().to_lowercase() != normalized_session_key {
            continue;
        }
        if let Some(session_obj) = value.as_object() {
            matched_session = Some(session_obj);
            break;
        }
    }

    let Some(session_obj) = matched_session else {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "未找到该会话。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    };

    let session_id = session_obj
        .get("sessionId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    let session_file = session_obj
        .get("sessionFile")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            if session_id.is_empty() {
                return None;
            }
            let fallback = sessions_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(format!("{session_id}.jsonl"));
            if fallback.exists() {
                Some(fallback.display().to_string())
            } else {
                None
            }
        });

    let Some(session_file) = session_file else {
        return Ok(OpenClawAgentSessionHistoryResponse {
            detail: "该会话缺少可读的 sessionFile。".to_string(),
            session_key: normalized_session_key,
            messages: Vec::new(),
        });
    };

    let session_raw = std::fs::read_to_string(&session_file)
        .map_err(|error| format!("读取会话消息失败: {error}"))?;
    let mut messages = Vec::new();
    for (index, line) in session_raw.lines().enumerate() {
        let parsed_line: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if parsed_line.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }
        let Some(message) = parsed_line.get("message").and_then(Value::as_object) else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if role != "user" && role != "assistant" {
            continue;
        }
        let Some(text) = extract_message_text(message) else {
            continue;
        };
        let fallback_created_at = i64::try_from(index).unwrap_or(0);
        let created_at_ms = extract_message_timestamp_ms(message, fallback_created_at);
        let message_id = message
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("msg-{index}"));
        messages.push(OpenClawAgentSessionHistoryMessage {
            id: message_id,
            role: role.to_string(),
            text,
            created_at_ms,
        });
    }

    Ok(OpenClawAgentSessionHistoryResponse {
        detail: format!(
            "已读取会话 {} 的 {} 条消息。",
            normalized_session_key,
            messages.len()
        ),
        session_key: normalized_session_key,
        messages,
    })
}

fn value_as_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(Value::as_i64).or_else(|| {
        value
            .and_then(Value::as_u64)
            .and_then(|value| i64::try_from(value).ok())
    })
}

fn extract_agent_id_from_session_key(value: Option<&Value>) -> Option<String> {
    let session_key = value.and_then(Value::as_str).map(str::trim)?;
    if session_key.is_empty() {
        return None;
    }

    let rest = session_key.strip_prefix("agent:")?;
    let candidate = rest.split(':').next().map(str::trim).unwrap_or("");
    if candidate.is_empty() {
        return None;
    }

    Some(candidate.to_string())
}

fn extract_task_payload_summary(payload: Option<&Value>) -> String {
    let Some(payload) = payload.and_then(Value::as_object) else {
        return "未提供任务说明。".to_string();
    };

    for key in ["text", "prompt", "message", "summary", "description"] {
        if let Some(value) = payload.get(key).and_then(Value::as_str) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    if let Some(kind) = payload.get("kind").and_then(Value::as_str) {
        let trimmed = kind.trim();
        if !trimmed.is_empty() {
            return format!("任务载荷类型：{trimmed}");
        }
    }

    "未提供任务说明。".to_string()
}

fn derive_task_status(enabled: bool, next_run_at_ms: Option<i64>, now_ms: i64) -> (String, String) {
    if !enabled {
        return ("disabled".to_string(), "已停用".to_string());
    }

    if let Some(next_run_at_ms) = next_run_at_ms {
        if next_run_at_ms <= now_ms {
            return ("late".to_string(), "待执行".to_string());
        }
    }

    ("scheduled".to_string(), "已启用".to_string())
}

#[tauri::command]
fn load_task_snapshot(agent_id: Option<String>) -> Result<TaskSnapshotResponse, String> {
    let requested_agent_id = agent_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let requested_agent_lower = requested_agent_id
        .as_deref()
        .map(|value| value.to_ascii_lowercase());
    let source_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");

    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(TaskSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "cron/jobs.json 未找到，当前没有可读取的任务调度数据。".to_string(),
                jobs: Vec::new(),
            })
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => {
            return Ok(TaskSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "cron/jobs.json 解析失败，当前无法读取任务调度数据。".to_string(),
                jobs: Vec::new(),
            })
        }
    };

    let Some(items) = parsed.get("jobs").and_then(Value::as_array) else {
        return Ok(TaskSnapshotResponse {
            source_path: source_path.display().to_string(),
            detail: "cron/jobs.json 中没有 jobs 数组。".to_string(),
            jobs: Vec::new(),
        });
    };

    let now_ms = current_timestamp_millis() as i64;
    let mut jobs = Vec::new();

    for item in items {
        let Some(obj) = item.as_object() else {
            continue;
        };

        let id = obj
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("unknown-job")
            .to_string();
        let name = obj
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&id)
            .to_string();
        let agent_id = obj
            .get("agentId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                obj.get("ownerAgentId")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            })
            .or_else(|| extract_agent_id_from_session_key(obj.get("sessionKey")))
            .unwrap_or_else(|| "未标注".to_string());
        if let Some(expected_agent) = requested_agent_lower.as_deref() {
            if agent_id.to_ascii_lowercase() != expected_agent {
                continue;
            }
        }
        let session_target = obj
            .get("sessionTarget")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&agent_id)
            .to_string();
        let enabled = obj.get("enabled").and_then(Value::as_bool).unwrap_or(true);
        let delete_after_run = obj
            .get("deleteAfterRun")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let created_at_ms = value_as_i64(obj.get("createdAtMs"));
        let updated_at_ms = value_as_i64(obj.get("updatedAtMs"));
        let schedule = obj.get("schedule").and_then(Value::as_object);
        let schedule_kind = schedule
            .and_then(|value| value.get("kind"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("unknown")
            .to_string();
        let next_run_at_ms = obj
            .get("state")
            .and_then(Value::as_object)
            .and_then(|value| value.get("nextRunAtMs"))
            .and_then(Value::as_i64)
            .or_else(|| {
                obj.get("state")
                    .and_then(Value::as_object)
                    .and_then(|value| value.get("nextRunAtMs"))
                    .and_then(Value::as_u64)
                    .and_then(|value| i64::try_from(value).ok())
            });
        let summary = extract_task_payload_summary(obj.get("payload"));
        let (status_kind, status_label) = derive_task_status(enabled, next_run_at_ms, now_ms);

        jobs.push(TaskSnapshotItem {
            id,
            name,
            agent_id,
            session_target,
            enabled,
            delete_after_run,
            status_kind,
            status_label,
            summary,
            next_run_at_ms,
            created_at_ms,
            updated_at_ms,
            schedule_kind,
        });
    }

    jobs.sort_by(|left, right| {
        let left_rank = match left.status_kind.as_str() {
            "late" => 0,
            "scheduled" => 1,
            "disabled" => 2,
            _ => 3,
        };
        let right_rank = match right.status_kind.as_str() {
            "late" => 0,
            "scheduled" => 1,
            "disabled" => 2,
            _ => 3,
        };

        left_rank
            .cmp(&right_rank)
            .then_with(|| {
                left.next_run_at_ms
                    .unwrap_or(i64::MAX)
                    .cmp(&right.next_run_at_ms.unwrap_or(i64::MAX))
            })
            .then_with(|| {
                right
                    .updated_at_ms
                    .unwrap_or(0)
                    .cmp(&left.updated_at_ms.unwrap_or(0))
            })
    });

    let detail = if let Some(requested_agent_id) = requested_agent_id {
        format!(
            "已从 cron/jobs.json 读取 {} 条任务（角色：{}）。",
            jobs.len(),
            requested_agent_id
        )
    } else {
        format!("已从 cron/jobs.json 读取 {} 条任务。", jobs.len())
    };

    Ok(TaskSnapshotResponse {
        source_path: source_path.display().to_string(),
        detail,
        jobs,
    })
}

#[tauri::command]
fn set_task_enabled(task_id: String, enabled: bool) -> Result<(), String> {
    let source_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");

    let raw = std::fs::read_to_string(&source_path)
        .map_err(|err| format!("无法读取 cron/jobs.json: {}", err))?;

    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|err| format!("cron/jobs.json 解析失败: {}", err))?;

    let jobs = parsed
        .get_mut("jobs")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "cron/jobs.json 中没有 jobs 数组。".to_string())?;

    let mut found = false;
    for job in jobs.iter_mut() {
        if let Some(obj) = job.as_object_mut() {
            let id = obj.get("id").and_then(Value::as_str).unwrap_or("").trim();
            if id == task_id.trim() {
                obj.insert("enabled".to_string(), Value::Bool(enabled));
                obj.insert(
                    "updatedAtMs".to_string(),
                    Value::Number(serde_json::Number::from(current_timestamp_millis() as i64)),
                );
                found = true;
                break;
            }
        }
    }

    if !found {
        return Err(format!("未找到 id 为 {} 的任务。", task_id));
    }

    let output =
        serde_json::to_string_pretty(&parsed).map_err(|err| format!("序列化失败: {}", err))?;
    std::fs::write(&source_path, output)
        .map_err(|err| format!("写入 cron/jobs.json 失败: {}", err))?;

    Ok(())
}

#[tauri::command]
fn delete_task(task_id: String) -> Result<(), String> {
    let source_path = resolve_openclaw_config_path()
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".openclaw"))
        .join("cron")
        .join("jobs.json");

    let raw = std::fs::read_to_string(&source_path)
        .map_err(|err| format!("无法读取 cron/jobs.json: {}", err))?;

    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|err| format!("cron/jobs.json 解析失败: {}", err))?;

    let jobs = parsed
        .get_mut("jobs")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "cron/jobs.json 中没有 jobs 数组。".to_string())?;

    let target_id = task_id.trim();
    if target_id.is_empty() {
        return Err("任务 id 不能为空。".to_string());
    }

    let before_len = jobs.len();
    jobs.retain(|job| {
        job.as_object()
            .and_then(|obj| obj.get("id"))
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            != target_id
    });

    if jobs.len() == before_len {
        return Err(format!("未找到 id 为 {} 的任务。", task_id));
    }

    let output =
        serde_json::to_string_pretty(&parsed).map_err(|err| format!("序列化失败: {}", err))?;
    std::fs::write(&source_path, output)
        .map_err(|err| format!("写入 cron/jobs.json 失败: {}", err))?;

    Ok(())
}

fn load_openclaw_platforms_snapshot_blocking() -> Result<OpenClawPlatformSnapshotResponse, String> {
    let source_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&source_path)
        .map_err(|error| format!("无法读取 openclaw.json: {error}"))?;
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = value_as_object(&parsed).ok_or("openclaw.json 根节点不是对象")?;

    let providers = root
        .get("models")
        .and_then(value_as_object)
        .and_then(|models| models.get("providers"))
        .and_then(value_as_object)
        .ok_or("openclaw.json 中未找到 models.providers 配置")?;

    let mut platforms = Vec::new();
    for (provider_id, item) in providers {
        let Some(provider) = value_as_object(item) else {
            continue;
        };

        let base_url = provider
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("")
            .to_string();
        if base_url.is_empty() {
            continue;
        }

        let normalized_provider_id = normalize_provider_id(provider_id);
        let api_kind = provider.get("api").and_then(Value::as_str);
        let protocol = infer_platform_protocol(api_kind);
        let api_path = infer_platform_api_path(&protocol, api_kind, &base_url);
        let model = provider
            .get("models")
            .and_then(Value::as_array)
            .and_then(|list| {
                list.iter().find_map(|model| {
                    value_as_object(model)
                        .and_then(|obj| obj.get("id").and_then(Value::as_str))
                        .or_else(|| {
                            value_as_object(model)
                                .and_then(|obj| obj.get("name").and_then(Value::as_str))
                        })
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(|value| value.to_string())
                })
            })
            .unwrap_or_default();

        let name = provider
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .or_else(|| {
                provider
                    .get("alias")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| value.to_string())
            })
            .unwrap_or_else(|| humanize_provider_name(provider_id));
        let api_key = provider
            .get("apiKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string();

        platforms.push(OpenClawPlatformSnapshotItem {
            id: format!("openclaw-provider-{normalized_provider_id}"),
            provider_id: provider_id.to_string(),
            name,
            protocol,
            base_url,
            path_prefix: format!("/{normalized_provider_id}"),
            api_path,
            api_key,
            model,
        });
    }

    platforms.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(OpenClawPlatformSnapshotResponse {
        source_path: source_path.display().to_string(),
        detail: format!("已从 openclaw.json 读取 {} 个平台。", platforms.len()),
        platforms,
    })
}

#[tauri::command]
async fn load_openclaw_platforms_snapshot() -> Result<OpenClawPlatformSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_openclaw_platforms_snapshot_blocking)
        .await
        .map_err(|error| format!("读取平台快照任务失败：{error}"))?
}

fn load_openclaw_channel_accounts_snapshot_blocking(
) -> Result<OpenClawChannelAccountsSnapshotResponse, String> {
    let _ = sync_channel_accounts_from_plugin_store("weixin");
    let _ = sync_channel_accounts_from_plugin_store("wecom");
    let _ = sync_channel_accounts_from_plugin_store("whatsapp");

    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawChannelAccountsSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 未找到，当前没有可读取的频道配置。".to_string(),
                channels: Vec::new(),
            })
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => {
            return Ok(OpenClawChannelAccountsSnapshotResponse {
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 解析失败，当前无法读取频道配置。".to_string(),
                channels: Vec::new(),
            })
        }
    };
    let Some(root) = value_as_object(&parsed) else {
        return Ok(OpenClawChannelAccountsSnapshotResponse {
            source_path: source_path.display().to_string(),
            detail: "openclaw.json 根节点不是对象。".to_string(),
            channels: Vec::new(),
        });
    };

    let (channel_to_agent, account_to_agent) = resolve_channel_binding_maps(root);
    let mut channel_groups = Vec::new();

    if let Some(channels_obj) = root.get("channels").and_then(Value::as_object) {
        for (channel_type, section_value) in channels_obj {
            let Some(section_obj) = section_value.as_object() else {
                continue;
            };
            if section_obj.get("enabled").and_then(Value::as_bool) == Some(false) {
                continue;
            }

            let mut section_clone = section_obj.clone();
            migrate_legacy_channel_section_to_accounts(&mut section_clone);
            let default_account_id = section_clone
                .get("defaultAccount")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("default")
                .to_string();
            let mut accounts = Vec::new();
            if let Some(accounts_obj) = section_clone.get("accounts").and_then(Value::as_object) {
                for (account_id, account_value) in accounts_obj {
                    let Some(account_obj) = account_value.as_object() else {
                        continue;
                    };
                    let enabled = account_obj
                        .get("enabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(true);
                    let configured = enabled && channel_payload_has_content(account_obj);
                    let status = if configured {
                        "connected"
                    } else {
                        "disconnected"
                    }
                    .to_string();
                    let normalized_channel = normalize_channel_identifier(channel_type);
                    let binding_key = channel_account_binding_key(channel_type, account_id);
                    let agent_id = account_to_agent
                        .get(&binding_key)
                        .cloned()
                        .or_else(|| channel_to_agent.get(&normalized_channel).cloned());
                    let display_name = account_obj
                        .get("name")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .unwrap_or_else(|| {
                            if account_id.eq_ignore_ascii_case("default") {
                                "主账号".to_string()
                            } else {
                                account_id.to_string()
                            }
                        });

                    accounts.push(OpenClawChannelAccountSnapshotItem {
                        account_id: account_id.to_string(),
                        name: display_name,
                        configured,
                        status,
                        is_default: account_id.eq_ignore_ascii_case(&default_account_id),
                        agent_id,
                    });
                }
            }

            if accounts.is_empty() {
                continue;
            }

            accounts.sort_by(|left, right| {
                if left.is_default {
                    return std::cmp::Ordering::Less;
                }
                if right.is_default {
                    return std::cmp::Ordering::Greater;
                }
                left.account_id.cmp(&right.account_id)
            });

            let group_status = if accounts.iter().any(|item| item.status == "connected") {
                "connected"
            } else {
                "disconnected"
            };

            channel_groups.push(OpenClawChannelGroupSnapshotItem {
                channel_type: channel_type.to_string(),
                default_account_id: default_account_id.to_string(),
                status: group_status.to_string(),
                accounts,
            });
        }
    }

    channel_groups.sort_by(|left, right| left.channel_type.cmp(&right.channel_type));
    Ok(OpenClawChannelAccountsSnapshotResponse {
        source_path: source_path.display().to_string(),
        detail: format!("已读取 {} 个已配置频道。", channel_groups.len()),
        channels: channel_groups,
    })
}

#[tauri::command]
async fn load_openclaw_channel_accounts_snapshot(
) -> Result<OpenClawChannelAccountsSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_openclaw_channel_accounts_snapshot_blocking)
        .await
        .map_err(|error| format!("读取频道快照任务失败：{error}"))?
}

#[tauri::command]
fn load_openclaw_channel_form_values(
    channel_type: String,
    account_id: Option<String>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => return Ok(std::collections::HashMap::new()),
    };
    let parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = value_as_object(&parsed).ok_or("openclaw.json 根节点不是对象")?;
    let normalized_channel = normalize_channel_identifier(&channel_type);
    if normalized_channel.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let Some(section_obj) = root
        .get("channels")
        .and_then(Value::as_object)
        .and_then(|channels| {
            resolve_channel_section_from_channels_obj(channels, &normalized_channel)
        })
    else {
        return Ok(std::collections::HashMap::new());
    };

    let mut section_clone = section_obj.clone();
    migrate_legacy_channel_section_to_accounts(&mut section_clone);
    let resolved_account_id = account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default");

    if let Some(account_obj) = section_clone
        .get("accounts")
        .and_then(Value::as_object)
        .and_then(|accounts| accounts.get(resolved_account_id))
        .and_then(Value::as_object)
    {
        return Ok(extract_channel_form_values(
            &normalized_channel,
            account_obj,
        ));
    }

    return Ok(extract_channel_form_values(
        &normalized_channel,
        &section_clone,
    ));
}

#[tauri::command]
fn save_openclaw_channel_config(payload: OpenClawChannelConfigPayload) -> Result<(), String> {
    let source_path = resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel = normalize_channel_identifier(&payload.channel_type);
    if normalized_channel.is_empty() {
        return Err("channelType 不能为空".to_string());
    }

    ensure_channel_plugin_allowlist(root, &normalized_channel)?;

    if is_whatsapp_channel_identifier(&normalized_channel) {
        return write_openclaw_config_value(&source_path, &parsed);
    }

    if !matches!(root.get("channels"), Some(Value::Object(_))) {
        root.insert(
            "channels".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let channels_obj = root
        .get_mut("channels")
        .and_then(Value::as_object_mut)
        .ok_or("channels 不是对象")?;
    let channel_config_key = merge_channel_alias_sections(channels_obj, &normalized_channel)?;
    let section_obj = channels_obj
        .get_mut(&channel_config_key)
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType> 不是对象")?;
    migrate_legacy_channel_section_to_accounts(section_obj);

    if !matches!(section_obj.get("accounts"), Some(Value::Object(_))) {
        section_obj.insert(
            "accounts".to_string(),
            Value::Object(serde_json::Map::<String, Value>::new()),
        );
    }
    let accounts_obj = section_obj
        .get_mut("accounts")
        .and_then(Value::as_object_mut)
        .ok_or("channels.<channelType>.accounts 不是对象")?;

    let resolved_account_id = payload
        .account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string();
    let existing_account_obj = accounts_obj
        .get(&resolved_account_id)
        .and_then(Value::as_object);
    let next_account_obj =
        build_channel_account_config(&normalized_channel, &payload.config, existing_account_obj);
    accounts_obj.insert(resolved_account_id.clone(), Value::Object(next_account_obj));

    if section_obj
        .get("defaultAccount")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        section_obj.insert(
            "defaultAccount".to_string(),
            Value::String(resolved_account_id.clone()),
        );
    }
    section_obj.insert("enabled".to_string(), Value::Bool(true));
    mirror_default_account_to_channel_section(section_obj);

    let (channel_to_agent, account_to_agent) = resolve_channel_binding_maps(root);
    let binding_key = channel_account_binding_key(&channel_config_key, &resolved_account_id);
    let has_account_binding = account_to_agent.contains_key(&binding_key);
    let has_channel_binding = channel_to_agent.contains_key(&channel_config_key);
    if !has_account_binding && !has_channel_binding {
        let binding_account = if is_default_channel_account_id(&resolved_account_id) {
            None
        } else {
            Some(resolved_account_id.as_str())
        };
        upsert_channel_binding(root, &channel_config_key, binding_account, Some("main"));
    }

    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn save_openclaw_channel_binding(payload: OpenClawChannelBindingPayload) -> Result<(), String> {
    let source_path = resolve_openclaw_config_path();
    let mut parsed = match std::fs::read_to_string(&source_path) {
        Ok(raw) => serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}),
    };
    if !parsed.is_object() {
        parsed = serde_json::json!({});
    }
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        normalize_channel_identifier_for_openclaw_config(&payload.channel_type);
    let normalized_account = payload.account_id.trim();
    if normalized_channel.is_empty() || normalized_account.is_empty() {
        return Err("channelType 与 accountId 不能为空".to_string());
    }
    let binding_account = if is_default_channel_account_id(normalized_account) {
        None
    } else {
        Some(normalized_account)
    };

    upsert_channel_binding(
        root,
        &normalized_channel,
        binding_account,
        payload.agent_id.as_deref(),
    );
    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn delete_openclaw_channel_account_config(
    payload: OpenClawChannelAccountPayload,
) -> Result<(), String> {
    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        normalize_channel_identifier_for_openclaw_config(&payload.channel_type);
    let normalized_account = payload.account_id.trim().to_string();
    if normalized_channel.is_empty() || normalized_account.is_empty() {
        return Err("channelType 与 accountId 不能为空".to_string());
    }

    let mut removed = false;
    if let Some(channels_obj) = root.get_mut("channels").and_then(Value::as_object_mut) {
        let _ = merge_channel_alias_sections(channels_obj, &normalized_channel)?;
        let mut remove_channel = false;
        if let Some(section_obj) = channels_obj
            .get_mut(&normalized_channel)
            .and_then(Value::as_object_mut)
        {
            migrate_legacy_channel_section_to_accounts(section_obj);
            let default_account_id_before = section_obj
                .get("defaultAccount")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("")
                .to_string();
            if let Some(accounts_obj) = section_obj
                .get_mut("accounts")
                .and_then(Value::as_object_mut)
            {
                if accounts_obj.remove(&normalized_account).is_some() {
                    removed = true;
                    if accounts_obj.is_empty() {
                        remove_channel = true;
                    } else {
                        let default_matches_removed = !default_account_id_before.is_empty()
                            && default_account_id_before.eq_ignore_ascii_case(&normalized_account);
                        if default_matches_removed {
                            let mut next_default_ids =
                                accounts_obj.keys().cloned().collect::<Vec<_>>();
                            next_default_ids.sort();
                            if let Some(next_default) = next_default_ids.first() {
                                section_obj.insert(
                                    "defaultAccount".to_string(),
                                    Value::String(next_default.to_string()),
                                );
                            }
                        }
                        mirror_default_account_to_channel_section(section_obj);
                    }
                }
            }
        }
        if remove_channel {
            channels_obj.remove(&normalized_channel);
        }
    }

    if !removed {
        return Ok(());
    }

    if is_default_channel_account_id(&normalized_account) {
        upsert_channel_binding(root, &normalized_channel, None, None);
    }
    upsert_channel_binding(root, &normalized_channel, Some(&normalized_account), None);
    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn rename_openclaw_channel_account(
    payload: OpenClawChannelAccountRenamePayload,
) -> Result<(), String> {
    let source_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&source_path)
        .map_err(|error| format!("读取 openclaw.json 失败: {error}"))?;
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel =
        normalize_channel_identifier_for_openclaw_config(&payload.channel_type);
    let normalized_account = payload.account_id.trim().to_string();
    let normalized_name = payload.name.trim().to_string();
    if normalized_channel.is_empty() || normalized_account.is_empty() {
        return Err("channelType 与 accountId 不能为空".to_string());
    }
    if normalized_name.is_empty() {
        return Err("名称不能为空".to_string());
    }

    let channels_obj = root
        .get_mut("channels")
        .and_then(Value::as_object_mut)
        .ok_or("channels 不存在或格式错误")?;
    let _ = merge_channel_alias_sections(channels_obj, &normalized_channel)?;
    let section_obj = channels_obj
        .get_mut(&normalized_channel)
        .and_then(Value::as_object_mut)
        .ok_or("未找到对应频道配置")?;
    migrate_legacy_channel_section_to_accounts(section_obj);
    let accounts_obj = section_obj
        .get_mut("accounts")
        .and_then(Value::as_object_mut)
        .ok_or("频道账号配置缺失")?;
    let account_obj = accounts_obj
        .get_mut(&normalized_account)
        .and_then(Value::as_object_mut)
        .ok_or("未找到对应账号配置")?;

    account_obj.insert("name".to_string(), Value::String(normalized_name));
    mirror_default_account_to_channel_section(section_obj);
    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn delete_openclaw_channel_config(channel_type: String) -> Result<(), String> {
    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let normalized_channel = normalize_channel_identifier_for_openclaw_config(&channel_type);
    if normalized_channel.is_empty() {
        return Err("channelType 不能为空".to_string());
    }

    if let Some(channels_obj) = root.get_mut("channels").and_then(Value::as_object_mut) {
        let aliases = resolve_channel_config_verification_aliases(&normalized_channel);
        let keys_to_remove = channels_obj
            .keys()
            .cloned()
            .filter(|key| aliases.contains(&normalize_channel_identifier(key)))
            .collect::<Vec<_>>();
        for key in keys_to_remove {
            channels_obj.remove(&key);
        }
    }
    clear_all_channel_bindings(root, &normalized_channel);
    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn save_openclaw_provider_base_url(provider_id: String, base_url: String) -> Result<(), String> {
    let provider_id = provider_id.trim();
    if provider_id.is_empty() {
        return Err("providerId 不能为空".to_string());
    }
    let base_url = base_url.trim();
    if base_url.is_empty() {
        return Err("baseUrl 不能为空".to_string());
    }
    let normalized_base_url = normalize_local_proxy_base_url_for_persist(base_url);

    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
    let providers = root
        .entry("models")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .and_then(|models| {
            Some(
                models
                    .entry("providers")
                    .or_insert_with(|| serde_json::json!({}))
                    .as_object_mut()?,
            )
        })
        .ok_or("openclaw.json 的 models.providers 不是对象")?;

    let target_key = providers
        .keys()
        .find(|key| key.eq_ignore_ascii_case(provider_id))
        .cloned()
        .unwrap_or_else(|| provider_id.to_string());

    let provider = providers
        .entry(target_key)
        .or_insert_with(|| serde_json::json!({}));
    let provider_obj = provider.as_object_mut().ok_or("provider 配置不是对象")?;
    provider_obj.insert("baseUrl".to_string(), Value::String(normalized_base_url));

    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn delete_openclaw_provider_config(provider_id: String) -> Result<(), String> {
    let normalized_provider_id = normalize_provider_id(provider_id.trim());
    if normalized_provider_id.is_empty() {
        return Err("providerId 不能为空".to_string());
    }

    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;
    let providers = root
        .entry("models")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .and_then(|models| {
            Some(
                models
                    .entry("providers")
                    .or_insert_with(|| serde_json::json!({}))
                    .as_object_mut()?,
            )
        })
        .ok_or("openclaw.json 的 models.providers 不是对象")?;

    let target_key = providers
        .keys()
        .find(|key| key.eq_ignore_ascii_case(&normalized_provider_id))
        .cloned()
        .ok_or_else(|| format!("未找到 providerId 为 {} 的模型配置。", provider_id.trim()))?;
    providers.remove(&target_key);

    write_openclaw_config_value(&source_path, &parsed)
}

#[tauri::command]
fn save_openclaw_provider_config(config: OpenClawProviderConfigPayload) -> Result<(), String> {
    let normalized_provider_id = normalize_provider_id(config.provider_id.trim());
    if normalized_provider_id.is_empty() {
        return Err("providerId 不能为空".to_string());
    }

    let base_url = config.base_url.trim();
    if base_url.is_empty() {
        return Err("baseUrl 不能为空".to_string());
    }
    let normalized_base_url = normalize_local_proxy_base_url_for_persist(base_url);
    let _requested_provider_name = config.provider_name.as_deref().unwrap_or("").trim();
    let requested_model = config.model.as_deref().unwrap_or("").trim().to_string();

    let requested_api_kind = config
        .api_kind
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let requested_protocol = config
        .protocol
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let api_key = config.api_key.trim().to_string();

    let source_path = resolve_openclaw_config_path();
    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "{}".to_string(),
        Err(error) => return Err(format!("无法读取 openclaw.json: {error}")),
    };
    let mut parsed: Value =
        serde_json::from_str(&raw).map_err(|error| format!("openclaw.json 解析失败: {error}"))?;
    let root = parsed
        .as_object_mut()
        .ok_or("openclaw.json 根节点不是对象")?;

    let target_key = {
        let providers = root
            .entry("models")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .and_then(|models| {
                Some(
                    models
                        .entry("providers")
                        .or_insert_with(|| serde_json::json!({}))
                        .as_object_mut()?,
                )
            })
            .ok_or("openclaw.json 的 models.providers 不是对象")?;

        let target_key = providers
            .keys()
            .find(|key| key.eq_ignore_ascii_case(&normalized_provider_id))
            .cloned()
            .unwrap_or_else(|| normalized_provider_id.to_string());
        let provider = providers
            .entry(target_key.clone())
            .or_insert_with(|| serde_json::json!({}));
        let provider_obj = provider.as_object_mut().ok_or("provider 配置不是对象")?;

        let existing_api_kind = provider_obj
            .get("api")
            .and_then(Value::as_str)
            .and_then(|value| {
                let normalized = value.trim().to_ascii_lowercase();
                if normalized == "openai-responses" {
                    Some("openai-responses")
                } else if normalized == "anthropic-messages" {
                    Some("anthropic-messages")
                } else if normalized == "openai-completions" {
                    Some("openai-completions")
                } else {
                    None
                }
            });
        let api_kind = if requested_api_kind == "openai-responses" {
            "openai-responses"
        } else if requested_api_kind == "anthropic-messages" {
            "anthropic-messages"
        } else if requested_api_kind == "openai-completions" {
            "openai-completions"
        } else if requested_protocol == "anthropic" {
            "anthropic-messages"
        } else if requested_protocol == "openai" {
            "openai-completions"
        } else if let Some(existing_kind) = existing_api_kind {
            existing_kind
        } else {
            "openai-completions"
        };

        provider_obj.remove("name");
        provider_obj.insert("api".to_string(), Value::String(api_kind.to_string()));
        provider_obj.insert("baseUrl".to_string(), Value::String(normalized_base_url));
        provider_obj.insert("apiKey".to_string(), Value::String(api_key));
        if !matches!(provider_obj.get("models"), Some(Value::Array(_))) {
            provider_obj.insert("models".to_string(), Value::Array(Vec::new()));
        }

        if !requested_model.is_empty() {
            let existing_models = provider_obj
                .get("models")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut next_models = Vec::new();
            let mut seen_model_ids = std::collections::HashSet::new();
            for item in existing_models {
                let Some(model_id) = item
                    .as_object()
                    .and_then(|obj| obj.get("id"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                else {
                    continue;
                };
                if seen_model_ids.insert(model_id.to_string()) {
                    next_models.push(item);
                }
            }
            if seen_model_ids.insert(requested_model.clone()) {
                next_models.push(serde_json::json!({
                    "id": requested_model.clone(),
                    "name": requested_model.clone()
                }));
            }
            provider_obj.insert("models".to_string(), Value::Array(next_models));
        }

        target_key
    };

    if !requested_model.is_empty() {
        if let Some(models_obj) = root.get_mut("models").and_then(Value::as_object_mut) {
            models_obj.remove("default");
        }

        if !matches!(root.get("agents"), Some(Value::Object(_))) {
            root.insert("agents".to_string(), serde_json::json!({}));
        }
        let agents_obj = root
            .get_mut("agents")
            .and_then(Value::as_object_mut)
            .ok_or("openclaw.json 的 agents 不是对象")?;
        if !matches!(agents_obj.get("defaults"), Some(Value::Object(_))) {
            agents_obj.insert("defaults".to_string(), serde_json::json!({}));
        }
        let defaults_obj = agents_obj
            .get_mut("defaults")
            .and_then(Value::as_object_mut)
            .ok_or("openclaw.json 的 agents.defaults 不是对象")?;
        let existing_fallbacks = defaults_obj
            .get("model")
            .and_then(Value::as_object)
            .and_then(|obj| obj.get("fallbacks"))
            .cloned();
        let fallbacks = match existing_fallbacks {
            Some(Value::Array(items)) => Value::Array(items),
            _ => Value::Array(Vec::new()),
        };
        let primary_model_ref = format!("{}/{}", target_key, requested_model);
        defaults_obj.insert(
            "model".to_string(),
            serde_json::json!({
                "primary": primary_model_ref,
                "fallbacks": fallbacks
            }),
        );
    }

    write_openclaw_config_value(&source_path, &parsed)
}

fn load_staff_snapshot_blocking() -> Result<StaffSnapshotResponse, String> {
    let source_path = resolve_openclaw_config_path();
    let mission_statement = load_staff_mission_statement();
    let scheduled_agents = load_scheduled_agents();

    let raw = match std::fs::read_to_string(&source_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(StaffSnapshotResponse {
                mission_statement,
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 未找到，当前没有可读取的员工配置。".to_string(),
                members: Vec::new(),
            })
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(_) => {
            return Ok(StaffSnapshotResponse {
                mission_statement,
                source_path: source_path.display().to_string(),
                detail: "openclaw.json 解析失败，当前无法读取员工配置。".to_string(),
                members: Vec::new(),
            })
        }
    };

    let root = value_as_object(&parsed);
    let agents_root = root
        .and_then(|obj| obj.get("agents"))
        .and_then(value_as_object);
    let defaults = agents_root
        .and_then(|obj| obj.get("defaults"))
        .and_then(value_as_object);
    let default_model = defaults
        .and_then(|obj| obj.get("model"))
        .and_then(read_string_or_primary)
        .unwrap_or("未标注");
    let default_tools_profile = root
        .and_then(|obj| obj.get("tools"))
        .and_then(value_as_object)
        .and_then(|tools| tools.get("profile"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("default");

    let channels_by_agent = root.map(resolve_channels_from_bindings).unwrap_or_default();

    let mut members = Vec::new();
    if let Some(list) = agents_root
        .and_then(|obj| obj.get("list"))
        .and_then(Value::as_array)
    {
        for item in list {
            let Some(obj) = value_as_object(item) else {
                continue;
            };
            let agent_id = obj
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("");
            if agent_id.is_empty() {
                continue;
            }

            let display_name = obj
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(if agent_id.eq_ignore_ascii_case("main") {
                    MAIN_STAFF_DISPLAY_NAME
                } else {
                    agent_id
                });
            let display_name = if agent_id.eq_ignore_ascii_case("main") {
                MAIN_STAFF_DISPLAY_NAME
            } else {
                display_name
            };
            let channel = channels_by_agent.get(agent_id).cloned().unwrap_or_default();
            let model = obj
                .get("model")
                .and_then(read_string_or_primary)
                .unwrap_or(default_model);
            let workspace = obj
                .get("workspace")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("未标注");
            let tools_profile = obj
                .get("tools")
                .and_then(value_as_object)
                .and_then(|tools| tools.get("profile"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(default_tools_profile);
            let tools_enabled_count = root
                .map(|root_obj| {
                    let (_, _, allowed_ids) =
                        openclaw_resolve_tools_from_config(root_obj, Some(obj));
                    allowed_ids.len()
                })
                .unwrap_or_else(|| openclaw_profile_tool_ids(tools_profile).len());
            let runtime_summary = load_runtime_session_summary(agent_id);
            let status_label = runtime_summary
                .latest_updated_at_ms
                .map(derive_status_label)
                .unwrap_or_else(|| "待命".to_string());
            let (current_work, recent_output) = runtime_summary
                .latest_session_file
                .as_deref()
                .map(load_recent_activity_from_session_file)
                .unwrap_or((None, None));
            let effective_model = if model == "未标注" {
                runtime_summary.latest_model.as_deref().unwrap_or(model)
            } else {
                model
            };

            members.push(StaffMemberSnapshot {
                agent_id: agent_id.to_string(),
                display_name: display_name.to_string(),
                role_label: humanize_agent_role(agent_id),
                channel,
                model: effective_model.to_string(),
                workspace: workspace.to_string(),
                tools_profile: tools_profile.to_string(),
                tools_enabled_count,
                status_label,
                current_work_label: "正在处理什么".to_string(),
                current_work: current_work.unwrap_or_else(|| "当前无实时任务".to_string()),
                recent_output: recent_output.unwrap_or_else(|| "最近暂无产出。".to_string()),
                scheduled_label: if scheduled_agents.contains(agent_id) {
                    "已排班".to_string()
                } else {
                    "未排班".to_string()
                },
            });
        }
    }

    members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    if !members.is_empty()
        && !members
            .iter()
            .any(|member| member.agent_id.eq_ignore_ascii_case("main"))
    {
        members.push(build_main_staff_snapshot(
            &scheduled_agents,
            &channels_by_agent,
            default_model,
            default_tools_profile,
        ));
        members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    }

    if members.is_empty() {
        let mut runtime_members = load_staff_from_runtime_dirs(
            &scheduled_agents,
            &channels_by_agent,
            default_model,
            default_tools_profile,
        )?;
        if !runtime_members
            .iter()
            .any(|member| member.agent_id.eq_ignore_ascii_case("main"))
        {
            runtime_members.push(build_main_staff_snapshot(
                &scheduled_agents,
                &channels_by_agent,
                default_model,
                default_tools_profile,
            ));
            runtime_members.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
        }
        if !runtime_members.is_empty() {
            return Ok(StaffSnapshotResponse {
                mission_statement,
                source_path: source_path.display().to_string(),
                detail: format!(
                    "openclaw.json 中 agents.list 为空，已回退为运行时员工目录，共读取 {} 名员工。",
                    runtime_members.len()
                ),
                members: runtime_members,
            });
        }
    }

    Ok(StaffSnapshotResponse {
        mission_statement,
        source_path: source_path.display().to_string(),
        detail: format!("已从 openclaw.json 读取 {} 名员工。", members.len()),
        members,
    })
}

#[tauri::command]
async fn load_staff_snapshot() -> Result<StaffSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_staff_snapshot_blocking)
        .await
        .map_err(|error| format!("读取员工快照任务失败：{error}"))?
}

fn is_openai_compatible_endpoint(endpoint: &str) -> bool {
    let normalized = endpoint.trim_end_matches('/').to_ascii_lowercase();
    normalized.ends_with("/v1/chat/completions") || normalized.ends_with("/chat/completions")
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

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

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

fn resolve_lobster_backup_root() -> PathBuf {
    let home = resolve_openclaw_home_path();
    home.parent()
        .map(|parent| parent.join(".openclaw-backups"))
        .unwrap_or_else(|| PathBuf::from(".openclaw-backups"))
}

fn metadata_modified_at_ms(metadata: &std::fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn collect_dir_size_bytes(path: &Path) -> u64 {
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

fn copy_directory_recursive(source: &Path, target: &Path) -> Result<(), String> {
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

fn resolve_preinstalled_skills_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    for root in resolve_resource_root_candidates() {
        candidates.push(root.join("preinstalled-skills"));
        candidates.push(root.join("resources").join("preinstalled-skills"));
    }

    candidates.into_iter().find(|path| path.is_dir())
}

fn load_preinstalled_skill_auto_enable_map(
    source_root: &Path,
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
    let target_root = resolve_openclaw_home_path().join("skills");
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

    let config_path = resolve_openclaw_config_path();
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
            serde_json::from_str::<Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
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
        let has_explicit_enabled = entry.get("enabled").and_then(Value::as_bool).is_some();
        if !has_explicit_enabled {
            *entry = serde_json::json!({ "enabled": true });
            auto_enabled_count += 1;
        }
    }

    write_openclaw_config_value(&config_path, &parsed)?;

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

fn collect_lobster_backups() -> Vec<LobsterBackupItem> {
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
        backups.push(LobsterBackupItem {
            name,
            path: path.display().to_string(),
            created_at_ms: metadata_modified_at_ms(&metadata),
            size_bytes: collect_dir_size_bytes(&path),
        });
    }

    backups.sort_by(|left, right| right.created_at_ms.cmp(&left.created_at_ms));
    backups
}

fn find_command_paths(command_name: &str) -> Vec<String> {
    #[cfg(target_os = "windows")]
    let output = {
        let mut command = Command::new("where");
        suppress_windows_command_window(&mut command);
        match command.arg(command_name).output() {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        }
    };

    #[cfg(not(target_os = "windows"))]
    let output = {
        let mut with_all = Command::new("which");
        suppress_windows_command_window(&mut with_all);
        if let Ok(value) = with_all.arg("-a").arg(command_name).output() {
            if value.status.success() {
                value
            } else {
                let mut fallback = Command::new("which");
                suppress_windows_command_window(&mut fallback);
                match fallback.arg(command_name).output() {
                    Ok(value) => value,
                    Err(_) => return Vec::new(),
                }
            }
        } else {
            let mut fallback = Command::new("which");
            suppress_windows_command_window(&mut fallback);
            match fallback.arg(command_name).output() {
                Ok(value) => value,
                Err(_) => return Vec::new(),
            }
        }
    };

    if !output.status.success() {
        return Vec::new();
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output_paths = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
    {
        if line.is_empty() {
            continue;
        }
        if dedup.insert(line.to_string()) {
            output_paths.push(line.to_string());
        }
    }
    output_paths
}

fn resolve_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_resource_root_candidates() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(path) = APP_RESOURCE_DIR.get() {
        roots.push(path.clone());
        roots.push(path.join("resources"));
        roots.push(path.join("src-tauri").join("resources"));
    }

    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources"));

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            roots.push(exe_dir.join("resources"));
            roots.push(exe_dir.join("../resources"));
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for root in roots {
        let key = root.display().to_string();
        if dedup.insert(key) {
            output.push(root);
        }
    }

    output
}

fn resolve_openclaw_runtime_dir() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("OPENCLAW_DIR") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.join("openclaw.mjs").exists() && candidate.join("package.json").exists() {
                return Some(candidate);
            }
        }
    }

    let project_root = resolve_project_root();
    let mut candidates = Vec::new();
    for root in resolve_resource_root_candidates() {
        candidates.push(root.join("openclaw"));
        candidates.push(root.join("resources").join("openclaw"));
        candidates.push(root.join("build").join("openclaw"));
        candidates.push(root.join("resources").join("build").join("openclaw"));
    }
    candidates.push(project_root.join("build").join("openclaw"));
    candidates.push(project_root.join("node_modules").join("openclaw"));

    candidates.into_iter().find(|candidate| {
        candidate.join("openclaw.mjs").exists() && candidate.join("package.json").exists()
    })
}

fn resolve_openclaw_cli_wrapper_source() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    for root in resolve_resource_root_candidates() {
        #[cfg(target_os = "windows")]
        {
            candidates.push(root.join("cli").join("openclaw.cmd"));
            candidates.push(root.join("cli").join("win32").join("openclaw.cmd"));
            candidates.push(root.join("resources").join("cli").join("openclaw.cmd"));
            candidates.push(
                root.join("resources")
                    .join("cli")
                    .join("win32")
                    .join("openclaw.cmd"),
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            candidates.push(root.join("cli").join("openclaw"));
            candidates.push(root.join("cli").join("posix").join("openclaw"));
            candidates.push(root.join("resources").join("cli").join("openclaw"));
            candidates.push(
                root.join("resources")
                    .join("cli")
                    .join("posix")
                    .join("openclaw"),
            );
        }
    }

    candidates
        .into_iter()
        .find(|path| path.exists() && path.is_file())
}

fn prepend_global_openclaw_cli_to_command_path(command: &mut Command) -> Option<String> {
    let cli_candidate = collect_openclaw_cli_command_candidates().into_iter().next()?;
    let cli_dir = cli_candidate.parent()?.to_path_buf();
    let preferred_dir = normalize_windows_path_for_child_process(&cli_dir);

    let mut path_entries = vec![preferred_dir.clone()];
    if let Some(existing_path) = std::env::var_os("PATH") {
        path_entries.extend(std::env::split_paths(&existing_path));
    }

    let joined_path = std::env::join_paths(path_entries).ok()?;
    command.env("PATH", joined_path);
    Some(preferred_dir.display().to_string())
}

fn collect_node_binary_candidates() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(explicit) = std::env::var("OPENCLAW_NODE_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    for path in find_command_paths("node") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for candidate in candidates {
        let key = candidate.display().to_string();
        if dedup.insert(key) {
            output.push(candidate);
        }
    }
    output
}

fn resolve_node_binary_path() -> Option<PathBuf> {
    collect_node_binary_candidates().into_iter().next()
}

const OPENCLAW_MIN_NODE_MAJOR: u32 = 22;
const OPENCLAW_MIN_NODE_MINOR: u32 = 16;

fn parse_node_version_component(raw: &str) -> Option<u32> {
    let digits = raw
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn parse_node_version_triplet(raw: &str) -> Option<(u32, u32, u32)> {
    let cleaned = raw.trim().trim_start_matches('v');
    let mut parts = cleaned.split('.');
    let major = parts.next().and_then(parse_node_version_component)?;
    let minor = parts
        .next()
        .and_then(parse_node_version_component)
        .unwrap_or(0);
    let patch = parts
        .next()
        .and_then(parse_node_version_component)
        .unwrap_or(0);
    Some((major, minor, patch))
}

fn openclaw_required_node_version_label() -> String {
    format!("{OPENCLAW_MIN_NODE_MAJOR}.{OPENCLAW_MIN_NODE_MINOR}.0")
}

fn check_node_version_supported(raw: &str) -> bool {
    let Some((major, minor, _patch)) = parse_node_version_triplet(raw) else {
        return false;
    };
    major > OPENCLAW_MIN_NODE_MAJOR
        || (major == OPENCLAW_MIN_NODE_MAJOR && minor >= OPENCLAW_MIN_NODE_MINOR)
}

fn read_node_binary_version(node_path: &Path) -> Result<String, String> {
    let normalized_node_path = normalize_windows_path_for_child_process(node_path);
    let mut command = Command::new(&normalized_node_path);
    suppress_windows_command_window(&mut command);
    let output = command.arg("-v").output().map_err(|error| {
        format!(
            "执行 Node 版本检查失败（{}）: {error}",
            normalized_node_path.display()
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!(
                "Node 版本检查失败（{}，exit: {}）。",
                node_path.display(),
                output.status.code().unwrap_or(-1)
            )
        } else {
            format!("Node 版本检查失败（{}）：{stderr}", node_path.display())
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let normalized = stdout.trim().trim_start_matches('v').to_string();
    if normalized.is_empty() {
        return Err(format!(
            "无法解析 Node 版本（{} 输出为空）。",
            node_path.display()
        ));
    }

    Ok(normalized)
}

fn resolve_openclaw_node_runtime() -> Result<(PathBuf, String), String> {
    let candidates = collect_node_binary_candidates();
    if candidates.is_empty() {
        return Err("未找到 Node.js 可执行文件（可通过 OPENCLAW_NODE_PATH 指定）。".to_string());
    }

    let mut diagnostics: Vec<String> = Vec::new();
    let mut inspected_versions: Vec<String> = Vec::new();
    let mut highest_supported_candidate: Option<(PathBuf, String, (u32, u32, u32))> = None;
    let mut highest_unsupported_candidate: Option<(PathBuf, String, (u32, u32, u32))> = None;

    for node in candidates {
        match read_node_binary_version(&node) {
            Ok(version) => {
                if check_node_version_supported(&version) {
                    let parsed = parse_node_version_triplet(&version).unwrap_or((0, 0, 0));
                    match &highest_supported_candidate {
                        Some((_best_path, _best_version, best_triplet))
                            if parsed <= *best_triplet => {}
                        _ => {
                            highest_supported_candidate = Some((node, version, parsed));
                        }
                    }
                    continue;
                }

                inspected_versions.push(format!("{}（{}）", version, node.display()));

                let parsed = parse_node_version_triplet(&version).unwrap_or((0, 0, 0));
                match &highest_unsupported_candidate {
                    Some((_best_path, _best_version, best_triplet)) if parsed <= *best_triplet => {}
                    _ => {
                        highest_unsupported_candidate = Some((node, version, parsed));
                    }
                }
            }
            Err(error) => diagnostics.push(error),
        }
    }

    if let Some((node, version, _)) = highest_supported_candidate {
        return Ok((node, version));
    }

    if let Some((node, version, _)) = highest_unsupported_candidate {
        let inspected = if inspected_versions.is_empty() {
            String::new()
        } else {
            format!(" 已检测到候选 Node：{}。", inspected_versions.join("；"))
        };
        let diagnostics_detail = if diagnostics.is_empty() {
            String::new()
        } else {
            format!(" 其他候选检查异常：{}。", diagnostics.join("；"))
        };
        return Err(format!(
            "OpenClaw 运行条件不满足：Node 版本过低（当前 {}，路径 {}），要求 >= {}。请升级 Node 后重试，或通过 OPENCLAW_NODE_PATH 指向符合要求的 Node。{}{}",
            version,
            node.display(),
            openclaw_required_node_version_label(),
            inspected,
            diagnostics_detail
        ));
    }

    Err(format!(
        "未找到可用 Node.js 运行时。{}",
        if diagnostics.is_empty() {
            "请安装 Node，或通过 OPENCLAW_NODE_PATH 指向可执行文件。".to_string()
        } else {
            diagnostics.join("；")
        }
    ))
}

fn read_json_version_field(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed = serde_json::from_str::<Value>(&raw).ok()?;
    parsed
        .get("version")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn build_openclaw_command_display(node: &Path, entry: &Path, args: &[&str]) -> String {
    let mut parts = vec![node.display().to_string(), entry.display().to_string()];
    parts.extend(args.iter().map(|arg| arg.to_string()));
    parts.join(" ")
}

fn apply_openclaw_child_process_env(command: &mut Command) {
    let openclaw_state_dir =
        normalize_windows_path_for_child_process(&resolve_openclaw_home_path());
    let openclaw_config = normalize_windows_path_for_child_process(&resolve_openclaw_config_path());
    command
        .env("OPENCLAW_NO_RESPAWN", "1")
        .env("OPENCLAW_EMBEDDED_IN", "DragonClaw")
        .env("OPENCLAW_STATE_DIR", openclaw_state_dir.clone())
        .env("CLAWDBOT_STATE_DIR", openclaw_state_dir)
        .env("OPENCLAW_CONFIG_PATH", openclaw_config);
    command.env_remove("OPENCLAW_HOME");

    if let Some(token) = resolve_openclaw_gateway_token() {
        command.env("OPENCLAW_GATEWAY_TOKEN", token);
    }
}

fn parse_dotted_numeric_version(raw: &str) -> Option<Vec<u32>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut segments = Vec::new();
    for segment in trimmed.split('.') {
        if segment.is_empty() {
            return None;
        }
        let parsed = segment.parse::<u32>().ok()?;
        segments.push(parsed);
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

fn compare_dotted_numeric_versions(left: &str, right: &str) -> Option<std::cmp::Ordering> {
    let left_segments = parse_dotted_numeric_version(left)?;
    let right_segments = parse_dotted_numeric_version(right)?;
    let max_len = left_segments.len().max(right_segments.len());
    for idx in 0..max_len {
        let left_value = *left_segments.get(idx).unwrap_or(&0);
        let right_value = *right_segments.get(idx).unwrap_or(&0);
        match left_value.cmp(&right_value) {
            std::cmp::Ordering::Equal => {}
            value => return Some(value),
        }
    }
    Some(std::cmp::Ordering::Equal)
}

fn is_openclaw_official_version_newer(official: &str, bundled: &str) -> bool {
    match compare_dotted_numeric_versions(official, bundled) {
        Some(std::cmp::Ordering::Greater) => true,
        Some(_) => false,
        None => official.trim() != bundled.trim(),
    }
}

fn fetch_openclaw_latest_official_version() -> Result<String, String> {
    tauri::async_runtime::block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .map_err(|error| format!("创建官网版本检查客户端失败: {error}"))?;
        let response = client
            .get("https://registry.npmjs.org/openclaw/latest")
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| format!("访问官网版本接口失败: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("官网版本接口返回异常状态：{status}"));
        }
        let payload = response
            .json::<Value>()
            .await
            .map_err(|error| format!("解析官网版本响应失败: {error}"))?;
        let version = payload
            .get("version")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or("官网版本响应未包含有效 version 字段。".to_string())?;
        Ok(version.to_string())
    })
}

fn read_bundled_openclaw_runtime_version() -> Option<String> {
    let runtime_dir = resolve_openclaw_runtime_dir()?;
    read_json_version_field(&runtime_dir.join("package.json"))
}

fn collect_npm_command_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::<PathBuf>::new();

    if let Ok(explicit) = std::env::var("OPENCLAW_NPM_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    if let Ok((node_path, _)) = resolve_openclaw_node_runtime() {
        if let Some(node_dir) = node_path.parent() {
            #[cfg(target_os = "windows")]
            let local_npm_candidates = [node_dir.join("npm.cmd"), node_dir.join("npm.exe"), node_dir.join("npm")];
            #[cfg(not(target_os = "windows"))]
            let local_npm_candidates = [node_dir.join("npm")];

            for candidate in local_npm_candidates {
                if candidate.exists() {
                    candidates.push(candidate);
                }
            }
        }
    }

    for path in find_command_paths("npm") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for candidate in candidates {
        let key = candidate.display().to_string();
        if dedup.insert(key) {
            output.push(candidate);
        }
    }
    output
}

fn run_openclaw_cli_via_npm_exec(
    package_spec: &str,
    args: &[String],
) -> Result<(String, std::process::Output), String> {
    let npm_candidates = collect_npm_command_candidates();
    if npm_candidates.is_empty() {
        return Err("未找到 npm 可执行文件（请确认 PATH 或 OPENCLAW_NPM_PATH）。".to_string());
    }

    let args_text = args.join(" ");
    let mut launch_errors = Vec::new();
    for npm_raw in npm_candidates {
        let npm = normalize_windows_path_for_child_process(&npm_raw);
        let mut command = Command::new(&npm);
        suppress_windows_command_window(&mut command);
        command
            .arg("exec")
            .arg("--yes")
            .arg(package_spec)
            .arg("--")
            .args(args)
            .current_dir(resolve_project_root());
        apply_openclaw_child_process_env(&mut command);

        let command_display = format!(
            "{} exec --yes {} -- {}",
            npm.display(),
            package_spec,
            args_text
        );
        match command.output() {
            Ok(output) => return Ok((command_display, output)),
            Err(error) => launch_errors.push(format!("{}: {}", npm.display(), error)),
        }
    }

    Err(format!(
        "调用 npm exec 失败（{}）。",
        launch_errors.join("；")
    ))
}

fn collect_openclaw_cli_command_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::<PathBuf>::new();

    if let Ok(explicit) = std::env::var("OPENCLAW_BIN_PATH") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            let candidate = PathBuf::from(trimmed);
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    if let Some(home_parent) = resolve_default_openclaw_home_path().parent() {
        let petclaw_node_root = home_parent.join(".petclaw").join("node");
        #[cfg(target_os = "windows")]
        let petclaw_candidates = vec![
            petclaw_node_root.join("bin").join("openclaw.cmd"),
            petclaw_node_root.join("bin").join("openclaw.exe"),
            petclaw_node_root.join("openclaw.cmd"),
            petclaw_node_root.join("openclaw.exe"),
        ];
        #[cfg(not(target_os = "windows"))]
        let petclaw_candidates = vec![petclaw_node_root.join("bin").join("openclaw")];

        for candidate in petclaw_candidates {
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    for path in find_command_paths("openclaw") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }
    #[cfg(target_os = "windows")]
    for path in find_command_paths("openclaw.cmd") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    let mut dedup = std::collections::HashSet::new();
    let mut output = Vec::new();
    for candidate in candidates {
        let key = candidate.display().to_string();
        if dedup.insert(key) {
            output.push(candidate);
        }
    }
    output
}

fn run_openclaw_cli_via_global_command(
    args: &[&str],
) -> Result<(String, std::process::Output), String> {
    let cli_candidates = collect_openclaw_cli_command_candidates();
    if cli_candidates.is_empty() {
        return Err(
            "未找到全局 OpenClaw CLI（openclaw）。请先安装到系统环境并确保 PATH 可见。"
                .to_string(),
        );
    }

    let mut launch_errors = Vec::new();
    for cli_raw in cli_candidates {
        let cli = normalize_windows_path_for_child_process(&cli_raw);
        let mut command = Command::new(&cli);
        suppress_windows_command_window(&mut command);
        command.args(args).current_dir(resolve_project_root());
        apply_openclaw_child_process_env(&mut command);
        let command_display = format!("{} {}", cli.display(), args.join(" "));
        match command.output() {
            Ok(output) => return Ok((command_display, output)),
            Err(error) => launch_errors.push(format!("{}: {}", cli.display(), error)),
        }
    }

    Err(format!(
        "调用全局 OpenClaw CLI 失败（{}）。",
        launch_errors.join("；")
    ))
}

fn run_openclaw_cli_output(args: &[&str]) -> Result<(String, std::process::Output), String> {
    run_openclaw_cli_via_global_command(args)
}

fn install_openclaw_cli_wrapper() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        let wrapper = resolve_openclaw_cli_wrapper_source()
            .ok_or("未找到内置 OpenClaw CLI 包装器（openclaw.cmd）。".to_string())?;
        let cli_dir = wrapper
            .parent()
            .ok_or("无法解析 CLI 包装器目录。".to_string())?;
        let helper_path = cli_dir.join("update-user-path.ps1");
        if !helper_path.exists() {
            return Ok(Some(cli_dir.display().to_string()));
        }

        let power_shell = std::env::var("SystemRoot")
            .map(|root| {
                PathBuf::from(root)
                    .join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0")
                    .join("powershell.exe")
            })
            .unwrap_or_else(|_| PathBuf::from("powershell.exe"));

        let mut command = Command::new(power_shell);
        suppress_windows_command_window(&mut command);
        let output = command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                helper_path.to_string_lossy().as_ref(),
                "-Action",
                "add",
                "-CliDir",
                cli_dir.to_string_lossy().as_ref(),
            ])
            .output()
            .map_err(|error| format!("更新 Windows PATH 失败: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                format!(
                    "更新 Windows PATH 失败（exit: {}）。",
                    output.status.code().unwrap_or(-1)
                )
            } else {
                format!("更新 Windows PATH 失败：{stderr}")
            });
        }

        return Ok(Some(cli_dir.display().to_string()));
    }

    #[cfg(not(target_os = "windows"))]
    {
        let source = resolve_openclaw_cli_wrapper_source()
            .ok_or("未找到内置 OpenClaw CLI 包装器。".to_string())?;
        let home = std::env::var("HOME").map_err(|_| "无法读取 HOME 环境变量。".to_string())?;
        let target_dir = PathBuf::from(home).join(".local").join("bin");
        std::fs::create_dir_all(&target_dir)
            .map_err(|error| format!("创建 CLI 目录失败（{}）: {error}", target_dir.display()))?;
        let target_path = target_dir.join("openclaw");

        if target_path.exists() {
            let _ = std::fs::remove_file(&target_path);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::{symlink, PermissionsExt};
            symlink(&source, &target_path).map_err(|error| {
                format!(
                    "创建 CLI 软链接失败（{} -> {}）: {error}",
                    target_path.display(),
                    source.display()
                )
            })?;
            if let Ok(metadata) = std::fs::metadata(&source) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&source, perms);
            }
        }

        Ok(Some(target_path.display().to_string()))
    }
}

fn bootstrap_openclaw_runtime(install_cli: bool) -> Result<Vec<String>, String> {
    let cli_candidates = collect_openclaw_cli_command_candidates();
    let Some(cli_path) = cli_candidates.first() else {
        return Err(format!(
            "未找到全局 OpenClaw CLI（openclaw）。请先安装并确保 PATH 可见。{}",
            official_openclaw_install_hint_for_platform()
        ));
    };
    let openclaw_home = resolve_openclaw_home_path();

    let mut notes = vec![
        format!("home={}", openclaw_home.display()),
        format!("config={}", resolve_openclaw_config_path().display()),
        format!("cli={}", cli_path.display()),
    ];

    match sanitize_openclaw_channel_schema() {
        Ok(Some(detail)) => notes.push(format!("channel_schema_sanitized={detail}")),
        Ok(None) => {}
        Err(error) => notes.push(format!("channel_schema_sanitize_error={error}")),
    }
    match sanitize_openclaw_plugin_load_paths() {
        Ok(Some(detail)) => notes.push(format!("plugin_load_paths_sanitized={detail}")),
        Ok(None) => {}
        Err(error) => notes.push(format!("plugin_load_paths_sanitize_error={error}")),
    }
    match ensure_openclaw_chat_completions_endpoint_enabled_outcome() {
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

    match run_openclaw_cli_output(&["--version"]) {
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
                format!("全局 OpenClaw CLI 自检失败（{}）：{}", command_display, stderr)
            });
        }
        Err(error) => return Err(error),
    }

    if install_cli {
        notes.push("cli_mode=global-only".to_string());
    }

    Ok(notes)
}

fn detect_openclaw_installation() -> (bool, Option<String>, Option<String>, String) {
    let cli_candidates = collect_openclaw_cli_command_candidates();
    let binary = cli_candidates.first().map(|path| path.display().to_string());
    if binary.is_none() {
        return (
            false,
            None,
            None,
            format!(
                "未找到全局 OpenClaw CLI（openclaw）。请先安装并确保 PATH 可见。{}",
                official_openclaw_install_hint_for_platform()
            ),
        );
    }

    match run_openclaw_cli_output(&["--version"]) {
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

struct OfficialOnboardOutcome {
    success: bool,
    degraded: bool,
    command: String,
    detail: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
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
    let (_command_display, output) = run_openclaw_cli_output(&arg_refs)?;
    Ok(OfficialOnboardCommandRun {
        command_redacted,
        output,
        strategy_note: Some("使用全局 OpenClaw CLI 执行安装。".to_string()),
    })
}

fn parse_json_object_from_line_tail(line: &str) -> Option<Value> {
    extract_last_json_object_from_text(line)
}

fn extract_last_json_object_from_text(raw: &str) -> Option<Value> {
    let cleaned = strip_ansi_escape_sequences(raw).replace('\r', "");
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

fn extract_last_json_object_from_streams(stdout: &str, stderr: &str) -> Option<Value> {
    let stderr_cleaned = strip_ansi_escape_sequences(stderr).replace('\r', "");
    let stdout_cleaned = strip_ansi_escape_sequences(stdout).replace('\r', "");

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

#[cfg(target_os = "windows")]
fn extract_last_onboard_json(stdout: &str, stderr: &str) -> Option<Value> {
    extract_last_json_object_from_streams(stdout, stderr)
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

fn run_openclaw_official_silent_onboard_once() -> OfficialOnboardOutcome {
    let token = match resolve_openclaw_gateway_token_for_onboard() {
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
    let preferred_gateway_port = resolve_openclaw_gateway_port();
    let gateway_port = match find_available_loopback_port(preferred_gateway_port) {
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
    let workspace = resolve_workspace_main_root();
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

struct GatewayBootstrapOutcome {
    success: bool,
    command: String,
    detail: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

struct GatewayDaemonEnsureOutcome {
    success: bool,
    command: String,
    detail: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
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
    let primary_path = resolve_openclaw_config_path();
    if let Some(port) = load_openclaw_gateway_port_from_path(&primary_path) {
        return Some(port);
    }

    let fallback_path = resolve_default_openclaw_config_path();
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

fn normalize_local_openclaw_chat_endpoint(raw: &str) -> String {
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

fn should_try_enable_chat_completions_endpoint(endpoint: &str) -> bool {
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

fn resolve_default_openclaw_api_url() -> Option<String> {
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

fn resolve_openclaw_gateway_port() -> u16 {
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

fn find_available_loopback_port(preferred_port: u16) -> Option<u16> {
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

fn wait_for_loopback_port_listening(port: u16, attempts: usize, interval_ms: u64) -> bool {
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

fn gateway_cli_text_indicates_non_effective_success(text: &str) -> bool {
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

fn gateway_cli_step_effective_success(output: &std::process::Output) -> bool {
    if !output.status.success() {
        return false;
    }
    !gateway_cli_text_indicates_non_effective_success(&gateway_cli_output_text(output))
}

fn append_labeled_output_section(target: &mut Vec<String>, label: &str, text: &str) {
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        target.push(format!("[{label}]\n{trimmed}"));
    }
}

fn gateway_status_payload_indicates_running(payload: &Value) -> bool {
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

fn gateway_status_payload_summary(payload: &Value) -> String {
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

fn gateway_status_text_indicates_running(text: &str) -> bool {
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

fn is_gateway_health_probe_online(probe: &GatewayHealthResponse) -> bool {
    probe.status.trim().eq_ignore_ascii_case("online")
}

fn summarize_gateway_health_probe(probe: Option<&GatewayHealthResponse>) -> String {
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

fn check_openclaw_gateway_health_fallback_blocking() -> Option<GatewayHealthResponse> {
    tauri::async_runtime::block_on(async { check_openclaw_gateway(None).await.ok() })
}

fn run_openclaw_gateway_daemon_ensure_once() -> GatewayDaemonEnsureOutcome {
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
    let (install_command, install_output) = match run_openclaw_cli_output(&install_refs) {
        Ok(value) => value,
        Err(error) => {
            return GatewayDaemonEnsureOutcome {
                success: false,
                command: "openclaw gateway install --runtime node --port <port> --json".to_string(),
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
    let (start_command, start_output) = match run_openclaw_cli_output(&start_refs) {
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
    let (status_command, status_output) = match run_openclaw_cli_output(&status_refs) {
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
    let status_payload = extract_last_json_object_from_streams(&status_stdout, &status_stderr);
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

fn run_openclaw_gateway_bootstrap_once() -> GatewayBootstrapOutcome {
    let gateway_port = resolve_openclaw_gateway_port();

    let (command_display, output) = match run_openclaw_cli_output(&["gateway", "restart", "--json"])
    {
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

fn run_lobster_install_action() -> LobsterActionResult {
    let started_at = std::time::Instant::now();
    match bootstrap_openclaw_runtime(true) {
        Ok(mut notes) => {
            let official_onboard = run_openclaw_official_silent_onboard_once();
            notes.push(format!(
                "official_onboard: success={}, command={}",
                official_onboard.success, official_onboard.command
            ));
            notes.push(format!(
                "official_onboard_detail={}",
                official_onboard.detail
            ));

            let endpoint_config = if official_onboard.success {
                ensure_openclaw_chat_completions_endpoint_enabled_outcome()
            } else {
                ChatCompletionsEndpointEnableOutcome::default()
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
                run_openclaw_gateway_daemon_ensure_once()
            } else {
                GatewayDaemonEnsureOutcome {
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

            LobsterActionResult {
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
        Err(error) => LobsterActionResult {
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

fn run_lobster_upgrade_action() -> LobsterActionResult {
    let started_at = std::time::Instant::now();
    match bootstrap_openclaw_runtime(true) {
        Ok(mut notes) => {
            let official_onboard = run_openclaw_official_silent_onboard_once();
            notes.push(format!(
                "official_onboard: success={}, command={}",
                official_onboard.success, official_onboard.command
            ));
            notes.push(format!(
                "official_onboard_detail={}",
                official_onboard.detail
            ));

            let endpoint_config = if official_onboard.success {
                ensure_openclaw_chat_completions_endpoint_enabled_outcome()
            } else {
                ChatCompletionsEndpointEnableOutcome::default()
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
                run_openclaw_gateway_daemon_ensure_once()
            } else {
                GatewayDaemonEnsureOutcome {
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

            LobsterActionResult {
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
        Err(error) => LobsterActionResult {
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

fn run_lobster_restart_gateway_action() -> LobsterActionResult {
    let started_at = std::time::Instant::now();
    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return LobsterActionResult {
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

    let gateway_bootstrap = run_openclaw_gateway_bootstrap_once();
    LobsterActionResult {
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

fn run_lobster_start_gateway_action() -> LobsterActionResult {
    let started_at = std::time::Instant::now();
    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return LobsterActionResult {
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

    let gateway_daemon = run_openclaw_gateway_daemon_ensure_once();
    LobsterActionResult {
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

fn run_lobster_pause_gateway_action() -> LobsterActionResult {
    let started_at = std::time::Instant::now();
    if let Err(error) = bootstrap_openclaw_runtime(false) {
        return LobsterActionResult {
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

    match run_openclaw_cli_output(&["gateway", "stop", "--json"]) {
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
            let port_still_ready =
                wait_for_loopback_port_listening(resolve_openclaw_gateway_port(), 6, 180);
            let success = !port_still_ready && (output.status.success() || likely_already_stopped);

            LobsterActionResult {
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
        Err(error) => LobsterActionResult {
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

fn run_lobster_backup_action() -> LobsterActionResult {
    let started_at = std::time::Instant::now();
    let openclaw_home = resolve_openclaw_home_path();
    if !openclaw_home.exists() {
        return LobsterActionResult {
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
        return LobsterActionResult {
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

    let backup_name = format!("openclaw-backup-{}", current_timestamp_millis());
    let backup_path = backup_root.join(&backup_name);
    if let Err(error) = copy_directory_recursive(&openclaw_home, &backup_path) {
        return LobsterActionResult {
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
    LobsterActionResult {
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

fn run_lobster_restore_action(backup_path: Option<String>) -> LobsterActionResult {
    let started_at = std::time::Instant::now();
    let selected_backup = backup_path
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        })
        .or_else(|| {
            collect_lobster_backups()
                .first()
                .map(|item| PathBuf::from(&item.path))
        });

    let Some(selected_backup) = selected_backup else {
        return LobsterActionResult {
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
        return LobsterActionResult {
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

    let openclaw_home = resolve_openclaw_home_path();
    let restore_parent = openclaw_home
        .parent()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| PathBuf::from("."));
    let home_name = openclaw_home
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "openclaw".to_string());
    let restore_stamp = current_timestamp_millis();
    let stage_path = restore_parent.join(format!(".{home_name}.restore-stage-{restore_stamp}"));
    let old_backup_path = restore_parent.join(format!(".{home_name}.pre-restore-{restore_stamp}"));

    if stage_path.exists() {
        let _ = std::fs::remove_dir_all(&stage_path);
    }

    if let Err(error) = copy_directory_recursive(&selected_backup, &stage_path) {
        return LobsterActionResult {
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
            return LobsterActionResult {
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
        return LobsterActionResult {
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

    LobsterActionResult {
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
    let should_set_openclaw_scopes =
        protocol != "anthropic" && should_try_enable_chat_completions_endpoint(&target_url);

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

#[cfg(target_os = "windows")]
fn center_window_on_current_monitor(window: &tauri::WebviewWindow) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };

    let Ok(window_size) = window.outer_size() else {
        return;
    };
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();

    let centered_x =
        monitor_position.x + ((monitor_size.width as i32 - window_size.width as i32).max(0) / 2);
    let centered_y =
        monitor_position.y + ((monitor_size.height as i32 - window_size.height as i32).max(0) / 2);

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
        centered_x, centered_y,
    )));
}

#[tauri::command]
fn start_main_window_drag(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    window.start_dragging().map_err(|error| error.to_string())
}

#[tauri::command]
fn toggle_main_window_maximize(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    let is_maximized = window.is_maximized().map_err(|error| error.to_string())?;
    if is_maximized {
        window.unmaximize().map_err(|error| error.to_string())?;
    } else {
        window.maximize().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn toggle_main_window_visibility(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let is_visible = window.is_visible().unwrap_or(true);
    if is_visible {
        let _ = window.hide();
        return;
    }

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn open_main_chat_panel(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit("dragonclaw://chat-open", "main");
}

#[cfg(target_os = "windows")]
fn reinforce_main_window_overlay(app: tauri::AppHandle) {
    thread::spawn(move || {
        for delay_ms in [300u64] {
            thread::sleep(Duration::from_millis(delay_ms));

            let Some(window) = app.get_webview_window("main") else {
                return;
            };

            let _ = window.set_decorations(false);
            let _ = window.set_shadow(false);
            let _ = window.set_always_on_top(true);
            let _ = window.set_resizable(false);
            center_window_on_current_monitor(&window);

            unsafe {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
                use windows::Win32::UI::Controls::MARGINS;
                use windows::Win32::UI::WindowsAndMessaging::*;

                if let Ok(raw) = window.hwnd() {
                    let hwnd = HWND(raw.0 as *mut _);

                    let style = GetWindowLongW(hwnd, GWL_STYLE);
                    SetWindowLongW(
                        hwnd,
                        GWL_STYLE,
                        style & !(WS_CAPTION.0 as i32) & !(WS_THICKFRAME.0 as i32),
                    );

                    // 扩展玻璃区域到整个客户区，实现真正的透明背景
                    let margins = MARGINS {
                        cxLeftWidth: -1,
                        cxRightWidth: -1,
                        cyTopHeight: -1,
                        cyBottomHeight: -1,
                    };
                    let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

                    let _ = SetWindowPos(
                        hwnd,
                        None,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                    );
                }
            }

            let _ = window.with_webview(|webview| unsafe {
                use webview2_com::Microsoft::Web::WebView2::Win32::*;
                use windows::core::Interface;

                let controller = webview.controller();
                if let Ok(c2) = controller.cast::<ICoreWebView2Controller2>() {
                    let _ = c2.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {
                        A: 0,
                        R: 0,
                        G: 0,
                        B: 0,
                    });
                }
            });
        }
    });
}

fn load_agent_context_messages(agent_id: &str) -> Vec<OpenClawMessage> {
    let config_path = resolve_openclaw_config_path();
    let configured_workspace = std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|parsed| {
            parsed
                .get("agents")
                .and_then(Value::as_object)
                .and_then(|agents| agents.get("list"))
                .and_then(Value::as_array)
                .and_then(|list| {
                    list.iter().find(|item| {
                        item.as_object()
                            .and_then(|obj| obj.get("id").and_then(Value::as_str))
                            .map(|id| id.eq_ignore_ascii_case(agent_id))
                            .unwrap_or(false)
                    })
                })
                .and_then(|item| {
                    item.as_object()
                        .and_then(|obj| obj.get("workspace").and_then(Value::as_str))
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                })
        });
    let workspace_root =
        resolve_workspace_root_for_agent(agent_id, configured_workspace.as_deref());

    let context_files = [
        "SOUL.md",
        "IDENTITY.md",
        "MEMORY.md",
        "BOOTSTRAP.md",
        "HEARTBEAT.md",
        "AGENTS.md",
    ];

    let mut sections = Vec::new();
    for file_name in context_files {
        let path = workspace_root.join(file_name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                let tag = file_name.trim_end_matches(".md");
                sections.push(format!("<{tag}>\n{trimmed}\n</{tag}>"));
            }
        }
    }

    if sections.is_empty() {
        return Vec::new();
    }

    vec![OpenClawMessage {
        role: "system".to_string(),
        content: sections.join("\n\n"),
    }]
}

fn resolve_agent_model_from_config(agent_id: &str) -> Option<String> {
    let config_path = resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(config_path).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let agents = parsed.get("agents")?.as_object()?;
    let list = agents.get("list")?.as_array()?;
    let agent = list.iter().find(|item| {
        item.as_object()
            .and_then(|obj| obj.get("id").and_then(Value::as_str))
            .map(|id| id.eq_ignore_ascii_case(agent_id))
            .unwrap_or(false)
    })?;
    let model = agent
        .as_object()?
        .get("model")
        .and_then(read_string_or_primary)
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    model.or_else(|| {
        agents
            .get("defaults")
            .and_then(Value::as_object)
            .and_then(|d| d.get("model"))
            .and_then(read_string_or_primary)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    })
}

#[tauri::command]
async fn openclaw_chat(
    messages: Vec<OpenClawMessage>,
    endpoint: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    protocol: Option<String>,
    agent_id: Option<String>,
    session_key: Option<String>,
) -> Result<OpenClawResponse, String> {
    let effective_agent_id = agent_id.as_deref().map(str::trim).filter(|v| !v.is_empty());

    let mut final_messages = Vec::new();
    if let Some(aid) = effective_agent_id {
        final_messages.extend(load_agent_context_messages(aid));
    }
    final_messages.extend(messages);

    let endpoint = endpoint
        .filter(|value| !value.trim().is_empty())
        .or_else(resolve_default_openclaw_api_url)
        .ok_or_else(|| "未设置可用的聊天接口地址。".to_string())?;
    let request_protocol = protocol
        .unwrap_or_else(|| "openai".to_string())
        .to_lowercase();
    let endpoint = if request_protocol == "openai" {
        normalize_local_openclaw_chat_endpoint(&endpoint)
    } else {
        endpoint
    };
    let should_try_enable_chat_completions =
        request_protocol == "openai" && should_try_enable_chat_completions_endpoint(&endpoint);
    let mut chat_completions_enable_error: Option<String> = None;
    if should_try_enable_chat_completions {
        if let Err(error) = ensure_openclaw_chat_completions_endpoint_enabled() {
            chat_completions_enable_error = Some(error);
        }
    }
    let is_openai_compatible = is_openai_compatible_endpoint(&endpoint);
    let gateway_token = resolve_openclaw_gateway_token();
    let api_key = api_key.filter(|value| !value.trim().is_empty());
    let session_key = session_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let agent_id_owned = effective_agent_id.map(|s| s.to_string());

    let mut model = model
        .filter(|value| !value.trim().is_empty())
        .or_else(|| effective_agent_id.and_then(resolve_agent_model_from_config))
        .or_else(|| std::env::var("OPENCLAW_MODEL").ok());
    if request_protocol == "openai" && should_try_enable_chat_completions {
        let has_openclaw_model = model
            .as_deref()
            .map(str::trim)
            .map(|value| {
                let lower = value.to_ascii_lowercase();
                lower == "openclaw" || lower.starts_with("openclaw/")
            })
            .unwrap_or(false);
        if !has_openclaw_model {
            model = Some(
                agent_id_owned
                    .as_deref()
                    .map(|aid| format!("openclaw/{aid}"))
                    .unwrap_or_else(|| "openclaw".to_string()),
            );
        }
    }

    let client = reqwest::Client::new();
    let mut request = client
        .post(&endpoint)
        .header(CONTENT_TYPE, "application/json");

    if let Some(aid) = agent_id_owned.as_deref() {
        request = request.header("X-OpenClaw-Agent-Id", aid);
    }
    if request_protocol == "openai" && should_try_enable_chat_completions {
        // OpenClaw gateway HTTP API (2026.3.x) requires explicit operator scopes
        // via x-openclaw-scopes for guarded methods such as chat.send.
        request = request.header("x-openclaw-scopes", "operator.write");
        if let Some(session_key) = session_key.as_deref() {
            request = request.header("x-openclaw-session-key", session_key);
        }
    }

    if request_protocol == "anthropic" {
        if let Some(api_key) = api_key.as_deref().filter(|value| !value.trim().is_empty()) {
            request = request.header("x-api-key", api_key);
        }
        request = request.header("anthropic-version", "2023-06-01");
    } else if let Some(api_key) = api_key
        .as_deref()
        .filter(|api_key| !api_key.trim().is_empty())
    {
        request = request.header(AUTHORIZATION, format!("Bearer {api_key}"));
    } else if let Some(token) = gateway_token
        .as_deref()
        .filter(|token| !token.trim().is_empty())
    {
        request = request.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    request = if request_protocol == "anthropic" {
        let model = model.ok_or_else(|| "Anthropic 协议需要模型配置。".to_string())?;
        let system = final_messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        let anthropic_messages = final_messages
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
            system: if system.is_empty() {
                None
            } else {
                Some(system)
            },
            messages: anthropic_messages,
        })
    } else if is_openai_compatible {
        request.json(&OpenAiChatRequest {
            model,
            messages: final_messages,
            agent_id: agent_id_owned.clone(),
        })
    } else {
        request.json(&OpenClawRequest {
            messages: final_messages,
            agent_id: agent_id_owned.clone(),
        })
    };

    let response = request
        .send()
        .await
        .map_err(|error| format!("请求 OpenClaw 失败: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let mut body = response
            .text()
            .await
            .unwrap_or_else(|_| "未返回错误详情".to_string());
        if status.as_u16() == 404
            && request_protocol == "openai"
            && should_try_enable_chat_completions
        {
            let hint = if let Some(error) = chat_completions_enable_error.as_deref() {
                format!(
                    "提示：已尝试自动启用 gateway.http.endpoints.chatCompletions.enabled，但失败：{error}"
                )
            } else {
                "提示：请确认 openclaw.json 中 gateway.http.endpoints.chatCompletions.enabled=true。"
                    .to_string()
            };
            if body.trim().is_empty() {
                body = hint;
            } else {
                body = format!("{body}\n{hint}");
            }
        }
        return Err(format!(
            "OpenClaw 返回错误状态 {status}（endpoint: {endpoint}）: {body}"
        ));
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

fn load_lobster_snapshot_blocking() -> Result<LobsterSnapshotResponse, String> {
    let (installed, version, binary, detail) = detect_openclaw_installation();
    let openclaw_home = resolve_openclaw_home_path();
    let backup_dir = resolve_lobster_backup_root();
    Ok(LobsterSnapshotResponse {
        openclaw_installed: installed,
        openclaw_version: version,
        openclaw_binary: binary,
        openclaw_home: openclaw_home.display().to_string(),
        backup_dir: backup_dir.display().to_string(),
        detail,
        backups: collect_lobster_backups(),
        install_wizard_open_every_launch: read_env_bool(
            "DRAGONCLAW_INSTALL_WIZARD_OPEN_EVERY_LAUNCH",
            false,
        ),
    })
}

#[tauri::command]
async fn load_lobster_snapshot() -> Result<LobsterSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_lobster_snapshot_blocking)
        .await
        .map_err(|error| format!("读取安装快照任务失败：{error}"))?
}

fn check_openclaw_runtime_status_blocking() -> Result<OpenClawRuntimeStatusResponse, String> {
    let gateway_port = Some(resolve_openclaw_gateway_port());
    let sanitize_notice = match sanitize_openclaw_models_provider_schema() {
        Ok(value) => value,
        Err(error) => Some(error),
    };
    let (installed, version, _binary, installation_detail) = detect_openclaw_installation();

    if !installed {
        return Ok(OpenClawRuntimeStatusResponse {
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
        return Ok(OpenClawRuntimeStatusResponse {
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

    let (command_display, output) = match run_openclaw_cli_output(&["gateway", "status", "--json"])
    {
        Ok(value) => value,
        Err(error) => {
            return Ok(OpenClawRuntimeStatusResponse {
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
    let status_effective = gateway_cli_step_effective_success(&output);
    let status_payload = extract_last_json_object_from_streams(&stdout, &stderr);
    let status_running = if let Some(payload) = status_payload.as_ref() {
        gateway_status_payload_indicates_running(payload)
    } else {
        gateway_status_text_indicates_running(&merged_output)
    };
    let port_ready = wait_for_loopback_port_listening(resolve_openclaw_gateway_port(), 4, 180);
    let gateway_probe = if port_ready && (!status_effective || !status_running) {
        check_openclaw_gateway_health_fallback_blocking()
    } else {
        None
    };
    let gateway_probe_online = gateway_probe
        .as_ref()
        .map(is_gateway_health_probe_online)
        .unwrap_or(false);
    let fallback_online = port_ready && gateway_probe_online;
    let healthy = (status_effective && status_running && port_ready) || fallback_online;

    let status_summary = status_payload
        .as_ref()
        .map(gateway_status_payload_summary)
        .unwrap_or_else(|| "未解析到 JSON 状态对象".to_string());
    let probe_summary = summarize_gateway_health_probe(gateway_probe.as_ref());
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

    Ok(OpenClawRuntimeStatusResponse {
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

#[tauri::command]
async fn check_openclaw_runtime_status() -> Result<OpenClawRuntimeStatusResponse, String> {
    tauri::async_runtime::spawn_blocking(check_openclaw_runtime_status_blocking)
        .await
        .map_err(|error| format!("读取运行状态任务失败：{error}"))?
}

#[tauri::command]
fn load_lobster_install_guide() -> Result<LobsterInstallGuideResponse, String> {
    let mut checks: Vec<LobsterInstallCheckItem> = Vec::new();

    let global_cli = collect_openclaw_cli_command_candidates().into_iter().next();
    checks.push(LobsterInstallCheckItem {
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

    let node_check = match resolve_openclaw_node_runtime() {
        Ok((path, version)) => LobsterInstallCheckItem {
            id: "nodejs".to_string(),
            title: "Node.js 执行器".to_string(),
            status: "success".to_string(),
            detail: format!(
                "已找到 Node 可执行文件：{}（版本 {}）",
                path.display(),
                version
            ),
        },
        Err(error) => LobsterInstallCheckItem {
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
    checks.push(LobsterInstallCheckItem {
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

    checks.push(LobsterInstallCheckItem {
        id: "official-onboard".to_string(),
        title: "官方静默安装命令".to_string(),
        status: "success".to_string(),
        detail: format!(
            "将通过全局 openclaw CLI 执行（不使用内置运行时）。命令参数：openclaw onboard --non-interactive --accept-risk --mode local --flow quickstart --auth-choice skip --gateway-auth token --gateway-token *** --gateway-port {} --gateway-bind loopback --install-daemon --workspace {} --json",
            resolve_openclaw_gateway_port(),
            resolve_workspace_main_root().display()
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

    Ok(LobsterInstallGuideResponse {
        os: os.to_string(),
        ready,
        checks,
    })
}

fn run_lobster_action_blocking(
    action: String,
    backup_path: Option<String>,
) -> Result<LobsterActionResult, String> {
    let normalized = action.trim().to_ascii_lowercase();
    match sanitize_openclaw_models_provider_schema() {
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
                LobsterActionResult {
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
                match run_openclaw_cli_output(&["doctor", "--fix", "--yes", "--non-interactive"]) {
                    Ok((command_display, output)) => LobsterActionResult {
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
                    Err(error) => LobsterActionResult {
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

#[tauri::command]
async fn run_lobster_action(
    action: String,
    backup_path: Option<String>,
) -> Result<LobsterActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || run_lobster_action_blocking(action, backup_path))
        .await
        .map_err(|error| format!("龙虾操作任务执行失败：{error}"))?
}

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
async fn send_openclaw_channel_message(
    payload: OpenClawChannelMessageSendPayload,
) -> Result<OpenClawChannelMessageSendResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_openclaw_channel_message_send_blocking(payload)
    })
    .await
    .map_err(|error| format!("频道消息发送任务执行失败：{error}"))?
}

#[tauri::command]
async fn append_openclaw_channel_mirror_failure_log(
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
async fn install_openclaw_channel_plugin(
    channel_type: String,
) -> Result<OpenClawChannelPluginInstallResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_openclaw_channel_plugin_install_blocking(channel_type)
    })
    .await
    .map_err(|error| format!("频道插件安装任务执行失败：{error}"))
}

#[tauri::command]
async fn check_openclaw_gateway(endpoint: Option<String>) -> Result<GatewayHealthResponse, String> {
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

fn is_whatsapp_ascii_qr_line(raw: &str) -> bool {
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

fn strip_ansi_escape_sequences(raw: &str) -> String {
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

fn update_openclaw_channel_qr_binding_session_from_line(
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
fn start_openclaw_channel_qr_binding(
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
            "当前仅支持微信（weixin）、企业微信（wecom）和 WhatsApp（whatsapp）二维码绑定。".to_string(),
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
        "whatsapp" => {
            Some("openclaw plugins install @openclaw/whatsapp && openclaw plugins enable whatsapp".to_string())
        }
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
                    "未找到全局 OpenClaw CLI 路径，当前将使用系统 PATH 中的 openclaw。"
                        .to_string(),
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
fn poll_openclaw_channel_qr_binding(
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
fn clear_openclaw_channel_qr_binding_session(session_id: String) -> Result<(), String> {
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
async fn request_feishu_openclaw_qr() -> Result<FeishuOnboardingQrResponse, String> {
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
async fn poll_feishu_openclaw_qr_result(
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

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("仅支持打开 http 或 https 链接。".to_string());
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(trimmed);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        suppress_windows_command_window(&mut command);
        command.args(["/C", "start", "", trimmed]);
        command
    };

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(trimmed);
        command
    };

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("打开外部浏览器失败：{error}"))
}

#[tauri::command]
fn open_local_path_in_folder(path: String) -> Result<String, String> {
    let normalized = path.trim();
    if normalized.is_empty() {
        return Err("本地路径不能为空。".to_string());
    }

    let expanded = expand_home_path(normalized);
    let resolved = if expanded.exists() {
        std::fs::canonicalize(&expanded).unwrap_or(expanded)
    } else {
        expanded
    };
    if !resolved.exists() {
        return Err(format!("目标路径不存在：{}", resolved.display()));
    }

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        if resolved.is_file() {
            command.arg("-R").arg(&resolved);
        } else {
            command.arg(&resolved);
        }
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        suppress_windows_command_window(&mut command);
        if resolved.is_file() {
            command.arg("/select,").arg(&resolved);
        } else {
            command.arg(&resolved);
        }
        command
    };

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        let target_dir = if resolved.is_dir() {
            resolved.clone()
        } else {
            resolved
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| resolved.clone())
        };
        command.arg(target_dir);
        command
    };

    command
        .spawn()
        .map_err(|error| format!("打开文件夹失败：{error}"))?;
    Ok(resolved.display().to_string())
}

#[tauri::command]
fn show_system_notification(
    app: tauri::AppHandle,
    title: String,
    body: Option<String>,
) -> Result<(), String> {
    let normalized_title = title.trim().to_string();
    if normalized_title.is_empty() {
        return Err("通知标题不能为空。".to_string());
    }

    let normalized_body = body
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    show_platform_system_notification(&app, &normalized_title, normalized_body.as_deref())
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

fn extract_dashboard_url_from_text(text: &str) -> Option<String> {
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
fn build_openclaw_control_ui_url() -> Result<String, String> {
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
async fn open_openclaw_control_ui() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(open_openclaw_control_ui_blocking)
        .await
        .map_err(|error| format!("打开 OpenClaw 控制台任务执行失败：{error}"))?
}

#[tauri::command]
async fn send_sms_code(phone: String) -> Result<SmsSendResponse, String> {
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
fn verify_sms_code(phone: String, code: String) -> Result<SmsVerifyResponse, String> {
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

fn normalize_skill_market_install_version(raw: Option<String>) -> Option<String> {
    let trimmed = raw
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_string();

    let mut chars = trimmed.chars();
    if let Some(head) = chars.next() {
        if (head == 'v' || head == 'V')
            && chars
                .clone()
                .next()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
        {
            let normalized = chars.collect::<String>().trim().to_string();
            if !normalized.is_empty() {
                return Some(normalized);
            }
        }
    }

    Some(trimmed)
}

fn trim_skill_market_output(raw: &str, max_chars: usize) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    for (index, ch) in trimmed.chars().enumerate() {
        if index >= max_chars {
            output.push('…');
            break;
        }
        output.push(ch);
    }
    output
}

fn resolve_skill_market_clawhub_install_root() -> PathBuf {
    resolve_openclaw_home_path()
        .join("tools")
        .join("clawhub-cli")
}

fn resolve_skill_market_clawhub_binary_path() -> PathBuf {
    let mut path = resolve_skill_market_clawhub_install_root()
        .join("node_modules")
        .join(".bin");
    #[cfg(target_os = "windows")]
    {
        path = path.join("clawhub.cmd");
    }
    #[cfg(not(target_os = "windows"))]
    {
        path = path.join("clawhub");
    }
    path
}

fn stringify_command_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    format!("{}\n{}", stdout.trim(), stderr.trim())
        .trim()
        .to_string()
}

fn resolve_global_clawhub_path() -> Option<PathBuf> {
    find_command_paths("clawhub")
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn install_clawhub_cli_for_skill_market() -> Result<bool, String> {
    let local_bin = resolve_skill_market_clawhub_binary_path();
    if local_bin.exists() {
        return Ok(false);
    }
    if resolve_global_clawhub_path().is_some() {
        return Ok(false);
    }

    let install_root = resolve_skill_market_clawhub_install_root();
    std::fs::create_dir_all(&install_root).map_err(|error| {
        format!(
            "创建 ClawHub CLI 目录失败（{}）：{error}",
            install_root.display()
        )
    })?;

    let npm_path = find_command_paths("npm")
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            "未找到 npm，无法自动安装 ClawHub CLI。请先安装 Node.js/npm。".to_string()
        })?;

    let mut command = Command::new(&npm_path);
    suppress_windows_command_window(&mut command);
    command
        .arg("install")
        .arg("--prefix")
        .arg(&install_root)
        .arg("--no-audit")
        .arg("--no-fund")
        .arg("clawhub@latest")
        .env("npm_config_update_notifier", "false")
        .env("npm_config_fund", "false")
        .env("npm_config_audit", "false");

    let command_display = format!(
        "{} install --prefix {} --no-audit --no-fund clawhub@latest",
        npm_path.display(),
        install_root.display()
    );
    let output = command
        .output()
        .map_err(|error| format!("自动安装 ClawHub CLI 失败（{command_display}）：{error}"))?;

    if !output.status.success() {
        let detail = trim_skill_market_output(&stringify_command_output(&output), 1200);
        return Err(if detail.is_empty() {
            format!(
                "自动安装 ClawHub CLI 失败（{command_display}，exit: {}）。",
                output.status.code().unwrap_or(-1)
            )
        } else {
            format!("自动安装 ClawHub CLI 失败（{command_display}）：{detail}")
        });
    }

    if local_bin.exists() || resolve_global_clawhub_path().is_some() {
        Ok(true)
    } else {
        Err(format!(
            "ClawHub CLI 安装已执行，但未找到可执行文件。预期路径：{}",
            local_bin.display()
        ))
    }
}

fn resolve_clawhub_command_for_skill_market() -> Result<(PathBuf, bool), String> {
    let installed_now = install_clawhub_cli_for_skill_market()?;
    let local_bin = resolve_skill_market_clawhub_binary_path();
    if local_bin.exists() {
        return Ok((local_bin, installed_now));
    }
    if let Some(global) = resolve_global_clawhub_path() {
        return Ok((global, installed_now));
    }
    Err("未找到 ClawHub CLI，可尝试在终端手动执行 `npm i -g clawhub` 后重试。".to_string())
}

fn install_skill_market_skill_blocking(
    skill_slug: String,
    version: Option<String>,
) -> Result<String, String> {
    let normalized_slug = skill_slug.trim().to_string();
    if normalized_slug.is_empty() {
        return Err("技能 slug 不能为空。".to_string());
    }
    if normalized_slug
        .chars()
        .any(|ch| ch.is_whitespace() || ch.is_control())
    {
        return Err("技能 slug 格式无效，请检查后重试。".to_string());
    }

    let normalized_version = normalize_skill_market_install_version(version);
    let workspace = resolve_workspace_main_root();
    std::fs::create_dir_all(&workspace).map_err(|error| {
        format!(
            "创建 OpenClaw 主工作区失败（{}）：{error}",
            workspace.display()
        )
    })?;

    let workspace_arg = normalize_windows_path_for_child_process(&workspace)
        .display()
        .to_string();
    let (clawhub_path, cli_installed_now) = resolve_clawhub_command_for_skill_market()?;

    let mut command = Command::new(&clawhub_path);
    suppress_windows_command_window(&mut command);
    command
        .arg("install")
        .arg(&normalized_slug)
        .arg("--workdir")
        .arg(&workspace_arg)
        .arg("--no-input")
        .current_dir(&workspace);
    if let Some(version_value) = normalized_version.as_deref() {
        command.arg("--version").arg(version_value);
    }

    let version_display = normalized_version
        .as_deref()
        .map(|value| format!(" --version {value}"))
        .unwrap_or_default();
    let command_display = format!(
        "{} install {} --workdir {} --no-input{}",
        clawhub_path.display(),
        normalized_slug,
        workspace_arg,
        version_display
    );
    let output = command
        .output()
        .map_err(|error| format!("执行 ClawHub 安装命令失败（{command_display}）：{error}"))?;
    let merged_detail = stringify_command_output(&output);

    if output.status.success() {
        let version_hint = normalized_version
            .as_deref()
            .map(|value| format!("（版本 {value}）"))
            .unwrap_or_default();
        let cli_hint = if cli_installed_now {
            "已自动完成 ClawHub CLI 首次安装。"
        } else {
            ""
        };
        return Ok(format!(
            "{}技能「{}」{}安装成功。目录：{}/skills。请开启新会话以确保能力刷新。",
            cli_hint, normalized_slug, version_hint, workspace_arg
        ));
    }

    let merged_lower = merged_detail.to_ascii_lowercase();
    if merged_lower.contains("already exists")
        || merged_lower.contains("already installed")
        || merged_lower.contains("has been installed")
    {
        return Ok(format!(
            "技能「{}」已存在于 {}/skills，无需重复安装。",
            normalized_slug, workspace_arg
        ));
    }

    let exit_code = output.status.code().unwrap_or(-1);
    let clipped_detail = trim_skill_market_output(&merged_detail, 1200);
    Err(if clipped_detail.is_empty() {
        format!("技能安装失败（{command_display}，exit: {exit_code}）。")
    } else {
        format!("技能安装失败（{command_display}）：{clipped_detail}")
    })
}

#[tauri::command]
async fn install_skill_market_skill(
    skill_slug: String,
    version: Option<String>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        install_skill_market_skill_blocking(skill_slug, version)
    })
    .await
    .map_err(|error| format!("技能安装任务执行失败：{error}"))?
}

async fn fetch_skill_market_json(url: &str) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| format!("创建技能市场客户端失败: {error}"))?;

    let response = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("技能市场请求失败: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let detail = response.text().await.unwrap_or_default();
        return Err(if detail.trim().is_empty() {
            format!("技能市场请求失败（{status}）")
        } else {
            format!("技能市场请求失败（{status}）：{detail}")
        });
    }

    response
        .json::<Value>()
        .await
        .map_err(|error| format!("技能市场响应解析失败: {error}"))
}

#[tauri::command]
async fn load_skill_market_top() -> Result<Value, String> {
    fetch_skill_market_json("https://lightmake.site/api/skills/top").await
}

#[tauri::command]
async fn load_skill_market_by_category(
    page: Option<u32>,
    page_size: Option<u32>,
    sort_by: Option<String>,
    order: Option<String>,
    category: String,
) -> Result<Value, String> {
    let category = category.trim().to_string();

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(24).clamp(1, 100);
    let sort_by = sort_by
        .unwrap_or_else(|| "score".to_string())
        .trim()
        .to_string();
    let order = order
        .unwrap_or_else(|| "desc".to_string())
        .trim()
        .to_string();

    let mut url = reqwest::Url::parse("https://lightmake.site/api/skills")
        .map_err(|error| format!("技能市场地址解析失败: {error}"))?;
    url.query_pairs_mut()
        .append_pair("page", &page.to_string())
        .append_pair("pageSize", &page_size.to_string())
        .append_pair("sortBy", &sort_by)
        .append_pair("order", &order);
    if !category.is_empty() {
        url.query_pairs_mut().append_pair("category", &category);
    }

    fetch_skill_market_json(url.as_str()).await
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn extract_last_json_object_from_streams_prefers_latest_line_object() {
        let stdout = "progress\n{\"ok\":true,\"service\":{\"loaded\":true}}\n";
        let stderr = "warn: step\nprefix {\"ok\":false,\"result\":\"not-loaded\"}\n";
        let payload = extract_last_json_object_from_streams(stdout, stderr).unwrap();
        assert_eq!(payload.get("ok").and_then(Value::as_bool), Some(false));
    }

    #[test]
    fn extract_last_json_object_from_streams_strips_ansi_and_noise() {
        let stdout = "\u{1b}[35m[plugins]\u{1b}[39m init\n";
        let stderr = "prefix \u{1b}[36m{\"ok\":true,\"service\":{\"loaded\":true,\"runtime\":{\"status\":\"running\"}}}\u{1b}[39m tail";
        let payload = extract_last_json_object_from_streams(stdout, stderr).unwrap();
        assert_eq!(payload.get("ok").and_then(Value::as_bool), Some(true));
        assert_eq!(
            payload
                .get("service")
                .and_then(Value::as_object)
                .and_then(|service| service.get("loaded"))
                .and_then(Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn extract_last_json_object_from_streams_supports_multiline_payload() {
        let stdout = "progress line\n";
        let stderr =
            "boot logs\n{\n  \"ok\": true,\n  \"service\": { \"loaded\": true }\n}\nmore logs";
        let payload = extract_last_json_object_from_streams(stdout, stderr).unwrap();
        assert_eq!(payload.get("ok").and_then(Value::as_bool), Some(true));
    }

    #[test]
    fn dotted_version_compare_detects_newer_official_version() {
        assert!(is_openclaw_official_version_newer("2026.3.28", "2026.3.13"));
        assert!(!is_openclaw_official_version_newer("2026.3.13", "2026.3.28"));
        assert!(!is_openclaw_official_version_newer("2026.3.28", "2026.3.28"));
    }

    #[test]
    fn dotted_version_compare_handles_length_differences() {
        assert!(is_openclaw_official_version_newer("2026.4", "2026.3.99"));
        assert!(!is_openclaw_official_version_newer("2026.3", "2026.3.0"));
    }

    #[test]
    fn detects_whatsapp_ascii_qr_line() {
        assert!(is_whatsapp_ascii_qr_line("██▀▀▄█"));
        assert!(is_whatsapp_ascii_qr_line(" ▄▄▄ █ "));
        assert!(!is_whatsapp_ascii_qr_line("Waiting for WhatsApp connection..."));
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_openclaw_env();

    #[cfg(target_os = "windows")]
    std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "0x00000000");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(
            GlobalShortcutBuilder::new()
                .with_shortcuts(["Ctrl+`", "Alt+`", "Alt+1"])
                .expect("failed to configure global shortcuts")
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let shortcut_text = shortcut.to_string().to_ascii_lowercase();
                        let is_open_chat_shortcut = shortcut_text.ends_with("+1")
                            && (shortcut_text.contains("alt") || shortcut_text.contains("option"));
                        if is_open_chat_shortcut {
                            open_main_chat_panel(app);
                            return;
                        }
                        toggle_main_window_visibility(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            quit_app,
            start_main_window_drag,
            toggle_main_window_maximize,
            openclaw_chat,
            sync_local_proxy,
            load_lobster_snapshot,
            check_openclaw_runtime_status,
            load_lobster_install_guide,
            run_lobster_action,
            check_openclaw_gateway,
            read_local_audio_file,
            read_local_media_file,
            persist_chat_attachment_data_url,
            load_openclaw_platforms_snapshot,
            load_openclaw_channel_accounts_snapshot,
            load_openclaw_channel_form_values,
            save_openclaw_channel_config,
            install_openclaw_channel_plugin,
            send_openclaw_channel_message,
            append_openclaw_channel_mirror_failure_log,
            save_openclaw_channel_binding,
            delete_openclaw_channel_account_config,
            rename_openclaw_channel_account,
            delete_openclaw_channel_config,
            save_openclaw_provider_base_url,
            delete_openclaw_provider_config,
            save_openclaw_provider_config,
            load_openclaw_message_logs,
            load_openclaw_agent_sessions_snapshot,
            load_openclaw_agent_session_history,
            load_staff_snapshot,
            load_task_snapshot,
            set_task_enabled,
            delete_task,
            load_memory_file_snapshot,
            load_document_file_snapshot,
            load_openclaw_resource_snapshot,
            load_openclaw_skills_list,
            save_openclaw_skill_enabled,
            load_openclaw_tools_list,
            save_openclaw_tools_config,
            save_openclaw_agent_model,
            save_source_file,
            install_role_workflow_agent,
            remove_role_workflow_agent,
            start_openclaw_channel_qr_binding,
            poll_openclaw_channel_qr_binding,
            clear_openclaw_channel_qr_binding_session,
            request_feishu_openclaw_qr,
            poll_feishu_openclaw_qr_result,
            open_external_url,
            open_local_path_in_folder,
            show_system_notification,
            build_openclaw_control_ui_url,
            open_openclaw_control_ui,
            send_sms_code,
            verify_sms_code,
            load_skill_market_top,
            load_skill_market_by_category,
            install_skill_market_skill
        ])
        .setup(|app| {
            if let Ok(resource_dir) = app.path().resource_dir() {
                set_app_resource_dir(resource_dir);
            }

            std::thread::spawn(|| {
                if let Err(error) = bootstrap_openclaw_runtime(true) {
                    eprintln!("[dragonclaw] openclaw bootstrap skipped: {error}");
                }
            });

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_autostart::init(
                    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                    None::<Vec<&str>>,
                ))
                .map_err(|error| error.to_string())?;

            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon().cloned() {
                    if let Err(error) = window.set_icon(icon) {
                        eprintln!("[dragonclaw] failed to refresh main window icon: {error}");
                    }
                }
                let _ = window.set_decorations(false);
                let _ = window.set_always_on_top(false);
                let _ = window.set_shadow(false);
                let _ = window.set_skip_taskbar(false);
                let _ = window.set_resizable(true);
                let _ = window.set_maximizable(true);
                let _ = window.set_minimizable(true);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
