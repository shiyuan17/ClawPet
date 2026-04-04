use std::time::{SystemTime, UNIX_EPOCH};
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};
mod app;
mod openclaw;

use app::bootstrap::setup_app;
use app::notifications::show_system_notification;
use app::shell::{open_external_url, open_local_path_in_folder};
use app::window::{
    open_main_chat_panel, quit_app, start_main_window_drag, toggle_main_window_maximize,
    toggle_main_window_visibility,
};
use openclaw::agents::{load_agent_context_messages, resolve_agent_model_from_config};
use openclaw::channel_config::*;
use openclaw::channel_onboarding::{
    build_openclaw_control_ui_url, clear_openclaw_channel_qr_binding_session,
    open_openclaw_control_ui, poll_feishu_openclaw_qr_result, poll_openclaw_channel_qr_binding,
    request_feishu_openclaw_qr, start_openclaw_channel_qr_binding,
};
pub(crate) use openclaw::channel_runtime::check_openclaw_gateway_internal;
use openclaw::channel_runtime::{
    append_openclaw_channel_mirror_failure_log, check_openclaw_gateway,
    install_openclaw_channel_plugin, send_openclaw_channel_message,
};
use openclaw::chat_api::openclaw_chat;
use openclaw::config::env::{load_openclaw_env, read_env_bool, read_env_u64};
pub(crate) use openclaw::config::paths::{
    expand_home_path, resolve_default_openclaw_config_path, resolve_default_openclaw_home_path,
    resolve_openclaw_config_path, resolve_openclaw_home_path, resolve_workspace_agents_root,
    resolve_workspace_main_root, resolve_workspace_root_for_agent,
};
use openclaw::config::schema::{
    ensure_openclaw_chat_completions_endpoint_enabled, sanitize_openclaw_channel_schema,
    sanitize_openclaw_models_provider_schema, sanitize_openclaw_plugin_load_paths,
};
pub(crate) use openclaw::config::write::write_openclaw_config_value;
use openclaw::gateway::{
    check_openclaw_gateway_health_fallback_blocking, gateway_cli_step_effective_success,
    gateway_status_payload_indicates_running, gateway_status_payload_summary,
    gateway_status_text_indicates_running, is_gateway_health_probe_online,
    normalize_local_openclaw_chat_endpoint, resolve_default_openclaw_api_url,
    resolve_openclaw_gateway_port, run_openclaw_gateway_bootstrap_once,
    should_try_enable_chat_completions_endpoint, summarize_gateway_health_probe,
};
use openclaw::lobster::{
    check_openclaw_runtime_status_blocking, load_lobster_install_guide_blocking,
    load_lobster_snapshot_blocking, run_lobster_action_blocking,
};
use openclaw::platform_channels::{
    delete_openclaw_channel_account_config, delete_openclaw_channel_config,
    load_openclaw_channel_accounts_snapshot, load_openclaw_channel_form_values,
    rename_openclaw_channel_account, save_openclaw_channel_binding, save_openclaw_channel_config,
};
use openclaw::platform_providers::{
    delete_openclaw_provider_config, load_openclaw_platforms_snapshot,
    save_openclaw_provider_base_url, save_openclaw_provider_config,
};
use openclaw::proxy::sync_local_proxy;
use openclaw::resources::{
    load_document_file_snapshot, load_memory_file_snapshot, load_openclaw_message_logs,
    load_openclaw_resource_snapshot, load_openclaw_skills_list, load_openclaw_tools_list,
    persist_chat_attachment_data_url, read_local_audio_file, read_local_media_file,
    save_openclaw_agent_model, save_openclaw_skill_enabled, save_openclaw_tools_config,
    save_source_file,
};
use openclaw::role_workflow::{install_role_workflow_agent, remove_role_workflow_agent};
use openclaw::runtime::{
    collect_openclaw_cli_command_candidates, prepend_global_openclaw_cli_to_command_path,
    resolve_openclaw_node_runtime, run_openclaw_cli_output,
};
pub(crate) use openclaw::runtime::{
    find_command_paths, resolve_project_root, resolve_resource_root_candidates,
};
use openclaw::skill_market::{
    install_skill_market_skill, load_skill_market_by_category, load_skill_market_top,
};
use openclaw::sms_auth::{send_sms_code, verify_sms_code};
use openclaw::staff_runtime::{
    delete_task, ensure_agents_list_has_main, load_openclaw_agent_session_history,
    load_openclaw_agent_sessions_snapshot, load_staff_snapshot, load_task_snapshot,
    set_task_enabled,
};
use openclaw::state::{app_resource_dir, openclaw_channel_qr_binding_sessions, sms_code_store};

pub(crate) use openclaw::channel_onboarding::strip_ansi_escape_sequences;
use openclaw::models::*;
pub(crate) use openclaw::resources::{
    openclaw_profile_tool_ids, openclaw_resolve_tools_from_config,
};
pub(crate) use openclaw::utils::{
    bytes_to_lower_hex, load_staff_mission_statement, normalize_windows_path_for_child_process,
    official_openclaw_install_hint_for_platform, read_string_or_primary,
    resolve_openclaw_gateway_token, resolve_openclaw_gateway_token_for_onboard,
    suppress_windows_command_window, value_as_object,
};

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[tauri::command]
async fn load_lobster_snapshot() -> Result<LobsterSnapshotResponse, String> {
    tauri::async_runtime::spawn_blocking(load_lobster_snapshot_blocking)
        .await
        .map_err(|error| format!("读取安装快照任务失败：{error}"))?
}

#[tauri::command]
async fn check_openclaw_runtime_status() -> Result<OpenClawRuntimeStatusResponse, String> {
    tauri::async_runtime::spawn_blocking(check_openclaw_runtime_status_blocking)
        .await
        .map_err(|error| format!("读取运行状态任务失败：{error}"))?
}

#[tauri::command]
fn load_lobster_install_guide() -> Result<LobsterInstallGuideResponse, String> {
    load_lobster_install_guide_blocking()
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
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
