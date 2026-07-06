use crate::log;
use crate::util::{run, ActionResult};
use std::path::PathBuf;

/// Procura o vhdx do Docker/WSL2 em locais conhecidos e retorna (caminho, tamanho).
pub fn find_docker_vhdx() -> Option<(PathBuf, u64)> {
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let base = PathBuf::from(&local).join("Docker").join("wsl");

    // Candidatos diretos mais comuns (Docker Desktop com WSL2).
    let candidates = [
        base.join("disk").join("docker_data.vhdx"),
        base.join("data").join("ext4.vhdx"),
        base.join("main").join("ext4.vhdx"),
    ];
    for c in candidates {
        if let Ok(meta) = std::fs::metadata(&c) {
            return Some((c, meta.len()));
        }
    }

    // Varredura rasa por qualquer *.vhdx sob %LOCALAPPDATA%\Docker\wsl.
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Ok(inner) = std::fs::read_dir(&p) {
                    for f in inner.flatten() {
                        let fp = f.path();
                        if fp.extension().map(|e| e == "vhdx").unwrap_or(false) {
                            let size = f.metadata().map(|m| m.len()).unwrap_or(0);
                            return Some((fp, size));
                        }
                    }
                }
            }
        }
    }
    None
}

#[tauri::command]
pub async fn docker_prune(dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || docker_prune_impl(dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn docker_prune_impl(dry_run: bool) -> ActionResult {
    if dry_run {
        return match run("docker", &["system", "df"]) {
            Ok(o) => ActionResult::dry(
                "Rodaria `docker system prune -a --volumes -f`. Uso atual abaixo (docker system df).",
            )
            .with_output(o.combined()),
            Err(e) => ActionResult::dry("Rodaria `docker system prune -a --volumes -f`.")
                .with_output(e.to_string()),
        };
    }

    let result = match run(
        "docker",
        &["system", "prune", "-a", "--volumes", "-f"],
    ) {
        Ok(o) if o.success => {
            ActionResult::ok("Prune concluído.").with_output(o.combined())
        }
        Ok(o) => ActionResult::fail("docker prune falhou.", o.combined()),
        Err(e) => ActionResult::fail("Docker não disponível.", e.to_string()),
    };
    log::record("docker_prune", result.ok, &result.message, "user");
    result
}

#[tauri::command]
pub async fn docker_compact_vhdx(dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || docker_compact_vhdx_impl(dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn docker_compact_vhdx_impl(dry_run: bool) -> ActionResult {
    let vhdx = match find_docker_vhdx() {
        Some((p, _)) => p,
        None => {
            return ActionResult::fail("vhdx do Docker não encontrado.", "");
        }
    };
    let path_str = vhdx.to_string_lossy().to_string();
    let size_before = std::fs::metadata(&vhdx).map(|m| m.len()).unwrap_or(0);

    if dry_run {
        return ActionResult::dry(format!(
            "Encerraria o WSL (`wsl --shutdown`) e compactaria via diskpart:\n{path_str}"
        ));
    }

    // 1) Encerra o WSL para liberar o disco virtual.
    if let Ok(o) = run("wsl", &["--shutdown"]) {
        if !o.success {
            // segue mesmo assim; diskpart falhará se ainda estiver em uso.
        }
    }

    // 2) Script diskpart temporário.
    let script = format!(
        "select vdisk file=\"{path_str}\"\r\nattach vdisk readonly\r\ncompact vdisk\r\ndetach vdisk\r\nexit\r\n"
    );
    let script_path = crate::log::dir().join("compact.dpt");
    let _ = std::fs::create_dir_all(crate::log::dir());
    if std::fs::write(&script_path, &script).is_err() {
        return ActionResult::fail("Não foi possível criar script diskpart.", "");
    }

    let out = run("diskpart", &["/s", &script_path.to_string_lossy()]);
    let _ = std::fs::remove_file(&script_path);
    let size_after = std::fs::metadata(&vhdx).map(|m| m.len()).unwrap_or(size_before);
    let freed = size_before as i64 - size_after as i64;

    let result = match out {
        Ok(o) if o.success => ActionResult::ok(format!(
            "vhdx compactado. Antes: {} · Depois: {} · Liberado: {}",
            mb(size_before),
            mb(size_after),
            mb(freed.max(0) as u64)
        ))
        .with_output(o.combined())
        .with_details(serde_json::json!({
            "before": size_before, "after": size_after, "freed": freed.max(0)
        })),
        Ok(o) => ActionResult::fail(
            "diskpart falhou (feche o Docker Desktop e tente novamente).",
            o.combined(),
        ),
        Err(e) => ActionResult::fail("Falha ao rodar diskpart.", e.to_string()),
    };
    log::record("docker_compact", result.ok, &result.message, "user");
    result
}

fn mb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
}

/// Lê os registros de auth do ~/.docker/config.json.
#[tauri::command]
pub fn list_docker_tokens() -> Vec<String> {
    let home = match std::env::var("USERPROFILE") {
        Ok(h) => h,
        Err(_) => return vec![],
    };
    let cfg = PathBuf::from(home).join(".docker").join("config.json");
    let content = match std::fs::read_to_string(cfg) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    json.get("auths")
        .and_then(|a| a.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}
