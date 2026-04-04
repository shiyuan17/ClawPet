use std::path::{Path, PathBuf};

fn load_env_file(path: &Path) {
    if path.exists() {
        let _ = dotenvy::from_path(path);
    }
}

pub(crate) fn read_env_bool(name: &str, default: bool) -> bool {
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

pub(crate) fn read_env_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
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

pub(crate) fn load_openclaw_env() {
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
