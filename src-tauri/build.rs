#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;

fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rerun-if-changed=src/macos_notifications.m");
        println!("cargo:rerun-if-changed=src/macos_dev_notifier.applescript");
        cc::Build::new()
            .file("src/macos_notifications.m")
            .flag("-fobjc-arc")
            .compile("dragonclaw_macos_notifications");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=UserNotifications");

        let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR must be set"));
        let notifier_app_path = out_dir.join("DragonClaw Dev Notifier.app");
        let status = Command::new("/usr/bin/osacompile")
            .arg("-o")
            .arg(&notifier_app_path)
            .arg("src/macos_dev_notifier.applescript")
            .status()
            .expect("failed to launch osacompile for DragonClaw dev notifier");
        if !status.success() {
            panic!("osacompile failed for DragonClaw dev notifier: {status}");
        }
        println!(
            "cargo:rustc-env=DRAGONCLAW_DEV_NOTIFIER_APP={}",
            notifier_app_path.display()
        );
    }

    tauri_build::build()
}
