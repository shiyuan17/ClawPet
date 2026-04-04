#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::{Mutex, OnceLock};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(test)]
struct TestHomeGuard {
    temp_root: PathBuf,
    previous_home: Option<String>,
    previous_userprofile: Option<String>,
    previous_homedrive: Option<String>,
    previous_homepath: Option<String>,
}

#[cfg(test)]
impl TestHomeGuard {
    fn new(prefix: &str) -> Self {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let temp_root =
            std::env::temp_dir().join(format!("{prefix}-{}-{unique_suffix}", std::process::id()));
        std::fs::create_dir_all(&temp_root).expect("create temp home root");

        let previous_home = std::env::var("HOME").ok();
        let previous_userprofile = std::env::var("USERPROFILE").ok();
        let previous_homedrive = std::env::var("HOMEDRIVE").ok();
        let previous_homepath = std::env::var("HOMEPATH").ok();

        std::env::set_var("HOME", &temp_root);
        std::env::set_var("USERPROFILE", &temp_root);
        std::env::remove_var("HOMEDRIVE");
        std::env::remove_var("HOMEPATH");

        Self {
            temp_root,
            previous_home,
            previous_userprofile,
            previous_homedrive,
            previous_homepath,
        }
    }
}

#[cfg(test)]
impl Drop for TestHomeGuard {
    fn drop(&mut self) {
        restore_env_var("HOME", &self.previous_home);
        restore_env_var("USERPROFILE", &self.previous_userprofile);
        restore_env_var("HOMEDRIVE", &self.previous_homedrive);
        restore_env_var("HOMEPATH", &self.previous_homepath);
        let _ = std::fs::remove_dir_all(&self.temp_root);
    }
}

#[cfg(test)]
fn restore_env_var(key: &str, previous: &Option<String>) {
    if let Some(value) = previous {
        std::env::set_var(key, value);
    } else {
        std::env::remove_var(key);
    }
}

#[cfg(test)]
pub(crate) fn with_temp_openclaw_home<T>(
    prefix: &str,
    initial_config: Option<Value>,
    test: impl FnOnce(&Path) -> T,
) -> T {
    let _env_guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let home_guard = TestHomeGuard::new(prefix);
    let config_path = crate::resolve_openclaw_config_path();

    if let Some(config) = initial_config {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).expect("create openclaw config parent");
        }
        let config_text = serde_json::to_string_pretty(&config).expect("serialize openclaw config");
        std::fs::write(&config_path, config_text).expect("write openclaw config");
    }

    let output = test(&home_guard.temp_root);
    drop(home_guard);
    output
}

#[cfg(test)]
pub(crate) fn read_temp_openclaw_config_json() -> Value {
    let config_path = crate::resolve_openclaw_config_path();
    let raw = std::fs::read_to_string(&config_path).expect("read openclaw config");
    serde_json::from_str(&raw).expect("parse openclaw config")
}
