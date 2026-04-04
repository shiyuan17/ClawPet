use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct OpenClawMessage {
    pub(crate) role: String,
    pub(crate) content: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct OpenClawRequest {
    pub(crate) messages: Vec<OpenClawMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) agent_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct OpenClawHttpResponse {
    pub(crate) text: Option<String>,
    pub(crate) content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct OpenAiMessage {
    pub(crate) content: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct OpenAiChoice {
    pub(crate) message: Option<OpenAiMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct OpenAiUsage {
    pub(crate) prompt_tokens: Option<u32>,
    pub(crate) completion_tokens: Option<u32>,
    pub(crate) total_tokens: Option<u32>,
    pub(crate) cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct OpenAiChatResponse {
    pub(crate) choices: Option<Vec<OpenAiChoice>>,
    pub(crate) usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct AnthropicUsage {
    pub(crate) input_tokens: Option<u32>,
    pub(crate) output_tokens: Option<u32>,
    pub(crate) cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct OpenClawResponse {
    pub(crate) text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) usage: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GatewayHealthResponse {
    pub(crate) status: String,
    pub(crate) checked_url: Option<String>,
    pub(crate) detail: Option<String>,
    pub(crate) latency_ms: Option<u128>,
    pub(crate) gateway_port: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeishuOnboardingQrResponse {
    pub(crate) qr_url: String,
    pub(crate) user_code: String,
    pub(crate) device_code: String,
    pub(crate) poll_interval_seconds: u64,
    pub(crate) expires_in_seconds: u64,
    pub(crate) expires_at_ms: u128,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FeishuOnboardingInitResponse {
    #[serde(default)]
    pub(crate) supported_auth_methods: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FeishuOnboardingBeginResponse {
    pub(crate) device_code: Option<String>,
    pub(crate) verification_uri_complete: Option<String>,
    pub(crate) user_code: Option<String>,
    pub(crate) expire_in: Option<u64>,
    pub(crate) interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FeishuOnboardingPollUserInfo {
    pub(crate) tenant_brand: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FeishuOnboardingPollRawResponse {
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    pub(crate) user_info: Option<FeishuOnboardingPollUserInfo>,
    pub(crate) error: Option<String>,
    pub(crate) error_description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeishuOnboardingPollResponse {
    pub(crate) status: String,
    pub(crate) message: Option<String>,
    pub(crate) app_id: Option<String>,
    pub(crate) app_secret: Option<String>,
    pub(crate) tenant_brand: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelQrBindingSessionSnapshot {
    pub(crate) session_id: String,
    pub(crate) channel_type: String,
    pub(crate) status: String,
    pub(crate) qr_url: Option<String>,
    pub(crate) qr_ascii: Option<String>,
    pub(crate) detail: Option<String>,
    pub(crate) logs: Vec<String>,
    pub(crate) started_at_ms: u128,
    pub(crate) updated_at_ms: u128,
}

#[derive(Debug, Clone)]
pub(crate) struct OpenClawChannelQrBindingSessionState {
    pub(crate) session_id: String,
    pub(crate) channel_type: String,
    pub(crate) status: String,
    pub(crate) qr_url: Option<String>,
    pub(crate) qr_ascii: Option<String>,
    pub(crate) qr_ascii_collecting: bool,
    pub(crate) qr_ascii_buffer: Vec<String>,
    pub(crate) detail: Option<String>,
    pub(crate) logs: Vec<String>,
    pub(crate) started_at_ms: u128,
    pub(crate) updated_at_ms: u128,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LobsterBackupItem {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) created_at_ms: u128,
    pub(crate) size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LobsterSnapshotResponse {
    pub(crate) openclaw_installed: bool,
    pub(crate) openclaw_version: Option<String>,
    pub(crate) openclaw_binary: Option<String>,
    pub(crate) openclaw_home: String,
    pub(crate) backup_dir: String,
    pub(crate) detail: String,
    pub(crate) backups: Vec<LobsterBackupItem>,
    pub(crate) install_wizard_open_every_launch: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawRuntimeStatusResponse {
    pub(crate) installed: bool,
    pub(crate) healthy: bool,
    pub(crate) status: String,
    pub(crate) command: String,
    pub(crate) detail: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) gateway_port: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LobsterActionResult {
    pub(crate) action: String,
    pub(crate) command: String,
    pub(crate) success: bool,
    pub(crate) detail: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) duration_ms: u128,
    pub(crate) backup_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelPluginInstallResult {
    pub(crate) channel_type: String,
    pub(crate) plugin_id: Option<String>,
    pub(crate) plugin_spec: Option<String>,
    pub(crate) command: String,
    pub(crate) success: bool,
    pub(crate) detail: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) duration_ms: u128,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelMessageSendResult {
    pub(crate) channel_type: String,
    pub(crate) account_id: String,
    pub(crate) target: String,
    pub(crate) command: String,
    pub(crate) success: bool,
    pub(crate) detail: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) duration_ms: u128,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LobsterInstallCheckItem {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LobsterInstallGuideResponse {
    pub(crate) os: String,
    pub(crate) ready: bool,
    pub(crate) checks: Vec<LobsterInstallCheckItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StaffMemberSnapshot {
    pub(crate) agent_id: String,
    pub(crate) display_name: String,
    pub(crate) role_label: String,
    pub(crate) channel: String,
    pub(crate) model: String,
    pub(crate) workspace: String,
    pub(crate) tools_profile: String,
    pub(crate) tools_enabled_count: usize,
    pub(crate) status_label: String,
    pub(crate) current_work_label: String,
    pub(crate) current_work: String,
    pub(crate) recent_output: String,
    pub(crate) scheduled_label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StaffSnapshotResponse {
    pub(crate) mission_statement: String,
    pub(crate) source_path: String,
    pub(crate) detail: String,
    pub(crate) members: Vec<StaffMemberSnapshot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskSnapshotItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) agent_id: String,
    pub(crate) session_target: String,
    pub(crate) enabled: bool,
    pub(crate) delete_after_run: bool,
    pub(crate) status_kind: String,
    pub(crate) status_label: String,
    pub(crate) summary: String,
    pub(crate) next_run_at_ms: Option<i64>,
    pub(crate) created_at_ms: Option<i64>,
    pub(crate) updated_at_ms: Option<i64>,
    pub(crate) schedule_kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskSnapshotResponse {
    pub(crate) source_path: String,
    pub(crate) detail: String,
    pub(crate) jobs: Vec<TaskSnapshotItem>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceFileSnapshotItem {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) content: String,
    pub(crate) source_path: String,
    pub(crate) relative_path: String,
    pub(crate) facet_key: String,
    pub(crate) facet_label: String,
    pub(crate) category: String,
    pub(crate) updated_at_ms: i64,
    pub(crate) exists: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceFileSnapshotResponse {
    pub(crate) source_path: String,
    pub(crate) detail: String,
    pub(crate) items: Vec<SourceFileSnapshotItem>,
}

/// 已安装技能项：来自 ~/.openclaw/skills 与 workspace/skills，对应 openclaw 技能信息（非文档编辑）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawSkillListItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) enabled: bool,
    pub(crate) relative_path: String,
    pub(crate) source_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawSkillsListResponse {
    pub(crate) source_path: String,
    /// 内置技能（来自 openclaw.json skills.allowBundled + entries）
    pub(crate) built_in: Vec<OpenClawSkillListItem>,
    /// 安装技能（来自 ~/.openclaw/skills 与 workspace/skills 下的 SKILL.md）
    pub(crate) installed: Vec<OpenClawSkillListItem>,
}

/// 已配置工具项：来自 openclaw tools.profile / allow/deny，对应已安装工具信息（非 TOOLS.md 编辑）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawToolListItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) category: String,
    pub(crate) enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawToolsListResponse {
    pub(crate) profile: String,
    pub(crate) profile_label: String,
    pub(crate) tools: Vec<OpenClawToolListItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawPlatformSnapshotItem {
    pub(crate) id: String,
    pub(crate) provider_id: String,
    pub(crate) name: String,
    pub(crate) protocol: String,
    pub(crate) base_url: String,
    pub(crate) path_prefix: String,
    pub(crate) api_path: String,
    pub(crate) api_key: String,
    pub(crate) model: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawPlatformSnapshotResponse {
    pub(crate) source_path: String,
    pub(crate) detail: String,
    pub(crate) platforms: Vec<OpenClawPlatformSnapshotItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelAccountSnapshotItem {
    pub(crate) account_id: String,
    pub(crate) name: String,
    pub(crate) configured: bool,
    pub(crate) status: String,
    pub(crate) is_default: bool,
    pub(crate) agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelGroupSnapshotItem {
    pub(crate) channel_type: String,
    pub(crate) default_account_id: String,
    pub(crate) status: String,
    pub(crate) accounts: Vec<OpenClawChannelAccountSnapshotItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelAccountsSnapshotResponse {
    pub(crate) source_path: String,
    pub(crate) detail: String,
    pub(crate) channels: Vec<OpenClawChannelGroupSnapshotItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelConfigPayload {
    pub(crate) channel_type: String,
    pub(crate) account_id: Option<String>,
    pub(crate) config: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelBindingPayload {
    pub(crate) channel_type: String,
    pub(crate) account_id: String,
    pub(crate) agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelMessageSendPayload {
    pub(crate) channel_type: String,
    pub(crate) account_id: Option<String>,
    pub(crate) target: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelMirrorFailureLogPayload {
    pub(crate) channel_type: String,
    pub(crate) account_id: String,
    pub(crate) target: String,
    pub(crate) message_preview: String,
    pub(crate) error_detail: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelAccountPayload {
    pub(crate) channel_type: String,
    pub(crate) account_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawChannelAccountRenamePayload {
    pub(crate) channel_type: String,
    pub(crate) account_id: String,
    pub(crate) name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawMessageLogItem {
    pub(crate) id: String,
    pub(crate) session_id: String,
    pub(crate) platform_id: String,
    pub(crate) platform_name: String,
    pub(crate) protocol: String,
    pub(crate) method: String,
    pub(crate) endpoint: String,
    pub(crate) base_url: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) request_body: String,
    pub(crate) response_status: u16,
    pub(crate) response_body: String,
    pub(crate) stream_summary: Option<String>,
    pub(crate) duration: u32,
    pub(crate) first_token_time: Option<u32>,
    pub(crate) error: Option<String>,
    pub(crate) prompt_tokens: Option<u32>,
    pub(crate) completion_tokens: Option<u32>,
    pub(crate) total_tokens: Option<u32>,
    pub(crate) cache_read_input_tokens: Option<u32>,
    pub(crate) created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawMessageLogResponse {
    pub(crate) detail: String,
    pub(crate) logs: Vec<OpenClawMessageLogItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawAgentSessionSnapshotItem {
    pub(crate) session_key: String,
    pub(crate) session_target: String,
    pub(crate) session_id: String,
    pub(crate) updated_at_ms: i64,
    pub(crate) message_count: usize,
    pub(crate) preview: String,
    pub(crate) last_channel: Option<String>,
    pub(crate) chat_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawAgentSessionsSnapshotResponse {
    pub(crate) detail: String,
    pub(crate) sessions: Vec<OpenClawAgentSessionSnapshotItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawAgentSessionHistoryMessage {
    pub(crate) id: String,
    pub(crate) role: String,
    pub(crate) text: String,
    pub(crate) created_at_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawAgentSessionHistoryResponse {
    pub(crate) detail: String,
    pub(crate) session_key: String,
    pub(crate) messages: Vec<OpenClawAgentSessionHistoryMessage>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingToolCall {
    pub(crate) tool_name: String,
    pub(crate) arguments: String,
    pub(crate) created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AudioFilePayload {
    pub(crate) data_url: String,
    pub(crate) mime_type: String,
    pub(crate) file_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalMediaFilePayload {
    pub(crate) data_url: String,
    pub(crate) mime_type: String,
    pub(crate) file_name: String,
    pub(crate) byte_length: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct EditableScope {
    pub(crate) facet_key: String,
    pub(crate) facet_label: String,
    pub(crate) workspace_root: PathBuf,
}

#[derive(Debug, Serialize)]
pub(crate) struct OpenAiChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) model: Option<String>,
    pub(crate) messages: Vec<OpenClawMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AnthropicMessage {
    pub(crate) role: String,
    pub(crate) content: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AnthropicRequest {
    pub(crate) model: String,
    pub(crate) max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) system: Option<String>,
    pub(crate) messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub(crate) block_type: Option<String>,
    pub(crate) text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct AnthropicResponse {
    pub(crate) content: Option<Vec<AnthropicContentBlock>>,
    pub(crate) usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenClawProviderConfigPayload {
    pub(crate) provider_id: String,
    pub(crate) provider_name: Option<String>,
    pub(crate) protocol: Option<String>,
    pub(crate) api_kind: Option<String>,
    pub(crate) base_url: String,
    pub(crate) model: Option<String>,
    pub(crate) api_key: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SmsCodeRecord {
    pub(crate) code: String,
    pub(crate) expires_at_ms: u128,
    pub(crate) last_sent_at_ms: u128,
}

#[derive(Debug, Clone)]
pub(crate) struct AliyunSmsConfig {
    pub(crate) access_key_id: String,
    pub(crate) access_key_secret: String,
    pub(crate) sign_name: String,
    pub(crate) template_code: String,
    pub(crate) endpoint: String,
    pub(crate) region_id: String,
    pub(crate) code_ttl_seconds: u64,
    pub(crate) cooldown_seconds: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SmsSendResponse {
    pub(crate) detail: String,
    pub(crate) cooldown_seconds: u64,
    pub(crate) expires_in_seconds: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SmsVerifyResponse {
    pub(crate) detail: String,
    pub(crate) session_token: String,
}
