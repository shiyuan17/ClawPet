use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

#[derive(Default)]
pub(crate) struct LocalProxyState {
    pub(crate) stop_signal: Option<Arc<AtomicBool>>,
    pub(crate) handle: Option<JoinHandle<()>>,
}

static LOCAL_PROXY_STATE: OnceLock<Mutex<LocalProxyState>> = OnceLock::new();
static APP_RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();
static SMS_CODE_STORE: OnceLock<Mutex<HashMap<String, crate::SmsCodeRecord>>> = OnceLock::new();
static OPENCLAW_CHANNEL_QR_BINDING_SESSIONS: OnceLock<
    Mutex<HashMap<String, Arc<Mutex<crate::OpenClawChannelQrBindingSessionState>>>>,
> = OnceLock::new();

pub(crate) fn local_proxy_state() -> &'static Mutex<LocalProxyState> {
    LOCAL_PROXY_STATE.get_or_init(|| Mutex::new(LocalProxyState::default()))
}

pub(crate) fn sms_code_store() -> &'static Mutex<HashMap<String, crate::SmsCodeRecord>> {
    SMS_CODE_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn openclaw_channel_qr_binding_sessions(
) -> &'static Mutex<HashMap<String, Arc<Mutex<crate::OpenClawChannelQrBindingSessionState>>>> {
    OPENCLAW_CHANNEL_QR_BINDING_SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn set_app_resource_dir(path: PathBuf) {
    let _ = APP_RESOURCE_DIR.set(path);
}

pub(crate) fn app_resource_dir() -> Option<PathBuf> {
    APP_RESOURCE_DIR.get().cloned()
}
