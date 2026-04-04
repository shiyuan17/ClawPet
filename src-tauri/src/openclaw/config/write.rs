use serde_json::Value;
use std::path::{Path, PathBuf};

use super::paths::resolve_default_openclaw_config_path;

pub(crate) fn write_openclaw_config_value(
    source_path: &Path,
    parsed: &Value,
) -> Result<(), String> {
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

pub(crate) fn set_openclaw_runtime_paths(config_path: &Path) {
    std::env::set_var("OPENCLAW_CONFIG_PATH", config_path);
    if let Some(home_path) = config_path.parent() {
        std::env::set_var("OPENCLAW_HOME", home_path);
    }
}
