use crate::log;
use crate::util::{run, powershell, ActionResult};
use sysinfo::Disks;

/// Espaço livre do volume C: em bytes.
fn c_free_bytes() -> u64 {
    let disks = Disks::new_with_refreshed_list();
    for d in &disks {
        let mp = d.mount_point().to_string_lossy();
        if mp.starts_with("C:") || mp.starts_with("C:\\") {
            return d.available_space();
        }
    }
    0
}

fn human(bytes: i64) -> String {
    let neg = bytes < 0;
    let mut v = bytes.unsigned_abs() as f64;
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut i = 0;
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{}{:.1} {}", if neg { "-" } else { "" }, v, units[i])
}

#[tauri::command]
pub async fn disk_cleanup(dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || disk_cleanup_impl(dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn disk_cleanup_impl(dry_run: bool) -> ActionResult {
    if dry_run {
        return ActionResult::dry(
            "Rodaria `cleanmgr /sagerun:1` com o perfil de limpeza salvo (configurável via `cleanmgr /sageset:1`).",
        );
    }

    let before = c_free_bytes();
    let out = run("cleanmgr", &["/sagerun:1"]);
    let after = c_free_bytes();
    let freed = after as i64 - before as i64;

    let result = match out {
        Ok(o) if o.success || o.code.is_none() => ActionResult::ok(format!(
            "Limpeza iniciada. Espaço recuperado (aprox.): {}. Obs.: cleanmgr roda em background; o valor pode subestimar.",
            human(freed.max(0))
        ))
        .with_output(o.combined()),
        Ok(o) => ActionResult::fail("cleanmgr retornou erro.", o.combined()),
        Err(e) => ActionResult::fail("Falha ao executar cleanmgr.", e.to_string()),
    };
    log::record("disk_cleanup", result.ok, &result.message, "user");
    result
}

#[tauri::command]
pub async fn empty_recycle_bin(dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || empty_recycle_bin_impl(dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn empty_recycle_bin_impl(dry_run: bool) -> ActionResult {
    if dry_run {
        return ActionResult::dry("Esvaziaria a Lixeira de todos os drives (`Clear-RecycleBin -Force`).");
    }

    let before = c_free_bytes();
    let out = powershell("Clear-RecycleBin -Force -ErrorAction Stop");
    let after = c_free_bytes();
    let freed = after as i64 - before as i64;

    let result = match out {
        Ok(o) if o.success => ActionResult::ok(format!(
            "Lixeira esvaziada. Espaço recuperado (aprox.): {}.",
            human(freed.max(0))
        )),
        // "lixeira já vazia" volta como erro não fatal no PS.
        Ok(o) if o.combined().to_lowercase().contains("empty") => {
            ActionResult::ok("A Lixeira já estava vazia.")
        }
        Ok(o) => ActionResult::fail("Não foi possível esvaziar a Lixeira.", o.combined()),
        Err(e) => ActionResult::fail("Falha ao esvaziar a Lixeira.", e.to_string()),
    };
    log::record("empty_recycle_bin", result.ok, &result.message, "user");
    result
}
