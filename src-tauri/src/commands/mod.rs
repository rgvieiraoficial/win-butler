pub mod cache;
pub mod creds;
pub mod dashboard;
pub mod detect;
pub mod disk;
pub mod docker;
pub mod env;
pub mod projects;
pub mod schedule;
pub mod startup;

use crate::log;
use crate::util::ActionResult;

/// Executa uma rotina pelo nome (usado pelo modo headless do Agendador).
/// Sempre executa de verdade (dry_run = false) e registra origem "scheduled".
pub fn run_routine(name: &str) -> ActionResult {
    let result = match name {
        "disk_cleanup" => disk::disk_cleanup_impl(false),
        "empty_recycle_bin" => disk::empty_recycle_bin_impl(false),
        "docker_prune" => docker::docker_prune_impl(false),
        "docker_compact" => docker::docker_compact_vhdx_impl(false),
        "clean_dev_caches" => cache::clean_all_dev_caches_impl(false),
        other => ActionResult::fail(format!("Rotina desconhecida: {other}"), ""),
    };
    log::record(name, result.ok, &result.message, "scheduled");
    result
}
