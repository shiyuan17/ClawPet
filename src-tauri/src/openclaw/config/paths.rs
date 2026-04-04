use std::path::PathBuf;

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

pub(crate) fn resolve_default_openclaw_home_path() -> PathBuf {
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

pub(crate) fn resolve_default_openclaw_config_path() -> PathBuf {
    resolve_default_openclaw_home_path().join("openclaw.json")
}

pub(crate) fn resolve_openclaw_home_path() -> PathBuf {
    resolve_default_openclaw_home_path()
}

pub(crate) fn resolve_openclaw_config_path() -> PathBuf {
    resolve_openclaw_home_path().join("openclaw.json")
}

pub(crate) fn resolve_workspace_main_root() -> PathBuf {
    resolve_openclaw_home_path().join("workspace-main")
}

pub(crate) fn resolve_workspace_agents_root() -> PathBuf {
    resolve_openclaw_home_path().join("workspaces")
}

pub(crate) fn expand_home_path(raw: &str) -> PathBuf {
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

pub(crate) fn resolve_workspace_root_for_agent(
    agent_id: &str,
    configured_workspace: Option<&str>,
) -> PathBuf {
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
