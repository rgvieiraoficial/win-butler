mod commands;
mod log;
mod util;

use commands::*;

/// Retorna o log local (últimas 500 ações).
#[tauri::command]
fn read_logs() -> Vec<log::LogEntry> {
    log::read(500)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Modo headless para o Agendador de Tarefas: `WinButler.exe --run <rotina>`.
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--run") {
        let routine = args.get(pos + 1).cloned().unwrap_or_default();
        let res = commands::run_routine(&routine);
        std::process::exit(if res.ok { 0 } else { 1 });
    }

    // System vivo para o auto-refresh (medir CPU% entre leituras, sem sleep).
    let mut live_sys = sysinfo::System::new();
    live_sys.refresh_cpu_all();
    live_sys.refresh_memory();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(std::sync::Mutex::new(live_sys))
        .invoke_handler(tauri::generate_handler![
            read_logs,
            detect::detect_capabilities,
            dashboard::get_dashboard,
            dashboard::live_stats,
            disk::disk_cleanup,
            disk::empty_recycle_bin,
            cache::list_dev_caches,
            cache::clean_dev_cache,
            cache::clean_all_dev_caches,
            projects::scan_node_modules,
            projects::delete_node_modules,
            docker::docker_prune,
            docker::docker_compact_vhdx,
            docker::list_docker_tokens,
            creds::list_credentials,
            creds::remove_credential,
            env::list_env_vars,
            env::remove_env_var,
            startup::list_startup,
            startup::set_startup,
            schedule::list_scheduled,
            schedule::schedule_routine,
            schedule::remove_scheduled,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o WinButler");
}
