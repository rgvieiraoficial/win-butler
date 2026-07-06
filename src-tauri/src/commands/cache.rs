use crate::log;
use crate::util::{powershell, ActionResult};
use serde::Serialize;
use std::path::PathBuf;

/// Um cache de gerenciador de pacotes que o app sabe limpar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevCache {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: Option<u64>,
    pub available: bool,
}

/// Como limpar o cache de um gerenciador.
enum CleanKind {
    /// Comando nativo do gerenciador (roda via PowerShell, resolve os atalhos .cmd).
    Cmd(&'static str),
    /// Sem comando nativo: apaga estas subpastas (relativas à pasta descoberta).
    RemoveDirs(&'static [&'static str]),
}

struct Spec {
    id: &'static str,
    name: &'static str,
    /// Comando que imprime a pasta do cache. Vazio = usa só o fallback.
    dir_cmd: &'static str,
    /// Pasta padrão caso o comando acima falhe: (variável de ambiente, caminho relativo).
    fallback: Option<(&'static str, &'static str)>,
    clean: CleanKind,
}

const SPECS: &[Spec] = &[
    Spec {
        id: "npm",
        name: "npm",
        dir_cmd: "npm config get cache",
        fallback: Some(("LOCALAPPDATA", "npm-cache")),
        clean: CleanKind::Cmd("npm cache clean --force"),
    },
    Spec {
        id: "yarn",
        name: "Yarn",
        dir_cmd: "yarn cache dir",
        fallback: Some(("LOCALAPPDATA", "Yarn\\Cache")),
        clean: CleanKind::Cmd("yarn cache clean"),
    },
    Spec {
        id: "pnpm",
        name: "pnpm",
        dir_cmd: "pnpm store path",
        fallback: Some(("LOCALAPPDATA", "pnpm-store")),
        clean: CleanKind::Cmd("pnpm store prune"),
    },
    Spec {
        id: "pip",
        name: "pip",
        dir_cmd: "pip cache dir",
        fallback: Some(("LOCALAPPDATA", "pip\\Cache")),
        clean: CleanKind::Cmd("pip cache purge"),
    },
    Spec {
        id: "nuget",
        name: "NuGet (.NET)",
        // O comando de limpeza apaga http-cache/temp/plugins também; o tamanho mostrado
        // é só o global-packages (a maior parte).
        dir_cmd: "",
        fallback: Some(("USERPROFILE", ".nuget\\packages")),
        clean: CleanKind::Cmd("dotnet nuget locals all --clear"),
    },
    Spec {
        id: "cargo",
        name: "Cargo (Rust)",
        // Cargo não tem comando de limpeza embutido; apagamos os downloads
        // (registry\cache e registry\src) — são baixados de novo quando precisar.
        dir_cmd: "",
        fallback: Some(("USERPROFILE", ".cargo\\registry")),
        clean: CleanKind::RemoveDirs(&["cache", "src"]),
    },
];

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

/// Soma recursiva do tamanho de uma pasta, em bytes. Ignora o que não conseguir ler.
fn dir_size(p: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(p) {
        for e in entries.flatten() {
            match e.file_type() {
                Ok(ft) if ft.is_dir() => total += dir_size(&e.path()),
                Ok(ft) if ft.is_file() => {
                    if let Ok(md) = e.metadata() {
                        total += md.len();
                    }
                }
                _ => {}
            }
        }
    }
    total
}

fn fallback_path(s: &Spec) -> Option<PathBuf> {
    s.fallback
        .and_then(|(var, rel)| std::env::var(var).ok().map(|base| PathBuf::from(base).join(rel)))
}

/// Descobre a pasta real do cache: tenta o comando do gerenciador, cai no fallback.
/// Só devolve caminho que existe de fato.
fn resolve_path(s: &Spec) -> Option<PathBuf> {
    if !s.dir_cmd.is_empty() {
        if let Ok(o) = powershell(s.dir_cmd) {
            if o.success {
                let p = o.stdout.trim();
                if !p.is_empty() {
                    let pb = PathBuf::from(p);
                    if pb.exists() {
                        return Some(pb);
                    }
                }
            }
        }
    }
    fallback_path(s).filter(|p| p.exists())
}

/// Limpa um gerenciador já com a pasta resolvida (evita descobrir de novo).
fn clean_one(spec: &Spec, path: Option<PathBuf>, dry_run: bool) -> ActionResult {
    let label = spec.name;
    let cmd_desc = match &spec.clean {
        CleanKind::Cmd(c) => c.to_string(),
        CleanKind::RemoveDirs(subs) => format!("apagar {}", subs.join(", ")),
    };

    if dry_run {
        let size = path.as_ref().map(|p| dir_size(p)).unwrap_or(0);
        return ActionResult::dry(format!(
            "{label}: rodaria `{cmd_desc}`. Cache atual (aprox.): {}.",
            human(size as i64)
        ));
    }

    let path = match path {
        Some(p) => p,
        None => return ActionResult::ok(format!("{label}: nenhum cache encontrado.")),
    };

    let before = dir_size(&path);
    let (success, out_text) = match &spec.clean {
        CleanKind::Cmd(c) => match powershell(c) {
            Ok(o) => (o.success, o.combined()),
            Err(e) => (false, e.to_string()),
        },
        CleanKind::RemoveDirs(subs) => {
            let mut errs = vec![];
            for sub in *subs {
                let target = path.join(sub);
                if target.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&target) {
                        errs.push(format!("{}: {e}", target.display()));
                    }
                }
            }
            if errs.is_empty() {
                (true, format!("Removido: {}", subs.join(", ")))
            } else {
                (false, errs.join("\n"))
            }
        }
    };
    let after = dir_size(&path);
    let freed = before as i64 - after as i64;

    if success {
        ActionResult::ok(format!(
            "{label}: limpo. Espaço recuperado (aprox.): {}.",
            human(freed.max(0))
        ))
        .with_output(out_text)
    } else {
        ActionResult::fail(format!("{label}: falha ao limpar."), out_text)
    }
}

#[tauri::command]
pub async fn list_dev_caches() -> Vec<DevCache> {
    tauri::async_runtime::spawn_blocking(list_dev_caches_impl)
        .await
        .unwrap_or_default()
}

pub fn list_dev_caches_impl() -> Vec<DevCache> {
    SPECS
        .iter()
        .map(|s| {
            let path = resolve_path(s);
            DevCache {
                id: s.id.into(),
                name: s.name.into(),
                path: path.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
                size_bytes: path.as_ref().map(|p| dir_size(p)),
                available: path.is_some(),
            }
        })
        .collect()
}

#[tauri::command]
pub async fn clean_dev_cache(manager: String, dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || clean_dev_cache_impl(manager, dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn clean_dev_cache_impl(manager: String, dry_run: bool) -> ActionResult {
    let spec = match SPECS.iter().find(|s| s.id == manager) {
        Some(s) => s,
        None => return ActionResult::fail(format!("Gerenciador desconhecido: {manager}"), ""),
    };
    let path = resolve_path(spec);
    let result = clean_one(spec, path, dry_run);
    log::record(
        &format!("clean_cache_{}", spec.id),
        result.ok,
        &result.message,
        "user",
    );
    result
}

#[tauri::command]
pub async fn clean_all_dev_caches(dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || clean_all_dev_caches_impl(dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

/// Limpa todos os caches disponíveis. Usado pelo botão "Limpar todos" e pela rotina agendada.
pub fn clean_all_dev_caches_impl(dry_run: bool) -> ActionResult {
    let mut lines = vec![];
    let mut ok_count = 0;
    let mut fail_count = 0;

    for s in SPECS {
        let path = resolve_path(s);
        if path.is_none() {
            continue; // gerenciador sem cache na máquina: ignora
        }
        let r = clean_one(s, path, dry_run);
        if r.ok {
            ok_count += 1;
        } else {
            fail_count += 1;
        }
        lines.push(r.message.clone());
    }

    let result = if dry_run {
        ActionResult::dry(format!("Simularia limpeza de {ok_count} cache(s) de gerenciadores."))
            .with_output(lines.join("\n"))
    } else if fail_count > 0 {
        ActionResult::fail(
            format!("Caches limpos: {ok_count}. Falhas: {fail_count}."),
            lines.join("\n"),
        )
    } else {
        ActionResult::ok(format!("Caches limpos: {ok_count}.")).with_output(lines.join("\n"))
    };
    log::record("clean_dev_caches", result.ok, &result.message, "user");
    result
}
