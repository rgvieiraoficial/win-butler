use crate::util::run;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Capabilities {
    pub docker: bool,
    pub wsl: bool,
    pub bitlocker: bool,
    pub cleanmgr: bool,
}

fn works(program: &str, args: &[&str]) -> bool {
    run(program, args).map(|o| o.success).unwrap_or(false)
}

#[tauri::command]
pub async fn detect_capabilities() -> Capabilities {
    tauri::async_runtime::spawn_blocking(detect_capabilities_impl)
        .await
        .unwrap_or_default()
}

pub fn detect_capabilities_impl() -> Capabilities {
    Capabilities {
        docker: works("docker", &["--version"]),
        wsl: works("wsl", &["--status"]) || works("wsl", &["-l", "-q"]),
        bitlocker: works("manage-bde", &["-status", "C:"]),
        // cleanmgr acompanha o Windows.
        cleanmgr: true,
    }
}
