use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

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
    crate::resolve_openclaw_home_path()
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
    crate::find_command_paths("clawhub")
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

    let npm_path = crate::find_command_paths("npm")
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            "未找到 npm，无法自动安装 ClawHub CLI。请先安装 Node.js/npm。".to_string()
        })?;

    let mut command = Command::new(&npm_path);
    crate::suppress_windows_command_window(&mut command);
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
    let workspace = crate::resolve_workspace_main_root();
    std::fs::create_dir_all(&workspace).map_err(|error| {
        format!(
            "创建 OpenClaw 主工作区失败（{}）：{error}",
            workspace.display()
        )
    })?;

    let workspace_arg = crate::normalize_windows_path_for_child_process(&workspace)
        .display()
        .to_string();
    let (clawhub_path, cli_installed_now) = resolve_clawhub_command_for_skill_market()?;

    let mut command = Command::new(&clawhub_path);
    crate::suppress_windows_command_window(&mut command);
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
pub(crate) async fn install_skill_market_skill(
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
pub(crate) async fn load_skill_market_top() -> Result<Value, String> {
    fetch_skill_market_json("https://lightmake.site/api/skills/top").await
}

#[tauri::command]
pub(crate) async fn load_skill_market_by_category(
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
