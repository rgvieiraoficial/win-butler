use crate::log;
use crate::util::{run, ActionResult};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Um node_modules encontrado na varredura, com dados pra decidir se apaga.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeModulesEntry {
    pub project_path: String,
    pub node_modules_path: String,
    pub size_bytes: u64,
    /// Data do último toque no projeto (último commit git, ou arquivo mais novo). ISO 8601.
    pub last_touched: Option<String>,
    /// Dias parado desde o último toque.
    pub days_idle: Option<i64>,
    pub has_package_json: bool,
    pub has_lockfile: bool,
    pub has_git: bool,
    /// Tem pasta `patches/` (patch-package): apagar pode perder ajustes manuais.
    pub has_patches: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub root: String,
    pub error: Option<String>,
    pub entries: Vec<NodeModulesEntry>,
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

/// Soma recursiva do tamanho de uma pasta, em bytes. Ignora o que não conseguir ler.
fn dir_size(p: &Path) -> u64 {
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

/// Acha todos os node_modules abaixo de `dir`, sem entrar dentro deles nem no `.git`.
fn find_node_modules(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let ft = match e.file_type() {
            Ok(f) => f,
            Err(_) => continue,
        };
        if !ft.is_dir() {
            continue; // ignora links simbólicos e arquivos (evita loop e lixo)
        }
        let name = e.file_name();
        let name = name.to_string_lossy();
        if name.eq_ignore_ascii_case("node_modules") {
            out.push(e.path()); // achou: registra e NÃO entra dentro
        } else if name == ".git" {
            // pula: pesado e sem node_modules
        } else {
            find_node_modules(&e.path(), out);
        }
    }
}

fn to_iso(t: SystemTime) -> String {
    let dt: DateTime<Utc> = t.into();
    dt.to_rfc3339()
}

/// Data do arquivo mais novo do projeto, ignorando node_modules e .git.
fn newest_mtime(dir: &Path, best: &mut Option<SystemTime>) {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let ft = match e.file_type() {
            Ok(f) => f,
            Err(_) => continue,
        };
        let name = e.file_name();
        let name = name.to_string_lossy();
        if ft.is_dir() {
            if name.eq_ignore_ascii_case("node_modules") || name == ".git" {
                continue;
            }
            newest_mtime(&e.path(), best);
        } else if let Ok(md) = e.metadata() {
            if let Ok(m) = md.modified() {
                if best.map(|b| m > b).unwrap_or(true) {
                    *best = Some(m);
                }
            }
        }
    }
}

/// Último toque no projeto: usa o último commit git; senão o arquivo mais novo.
fn last_touched(project: &Path, has_git: bool) -> Option<String> {
    if has_git {
        if let Ok(o) = run(
            "git",
            &["-C", &project.to_string_lossy(), "log", "-1", "--format=%cI"],
        ) {
            if o.success {
                let t = o.stdout.trim().to_string();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
    }
    let mut best = None;
    newest_mtime(project, &mut best);
    best.map(to_iso)
}

fn days_since(iso: &str) -> Option<i64> {
    let dt = DateTime::parse_from_rfc3339(iso).ok()?;
    Some((Utc::now() - dt.with_timezone(&Utc)).num_days())
}

fn build_entry(nm: &Path) -> NodeModulesEntry {
    let project = nm.parent().unwrap_or(nm).to_path_buf();
    let has_git = project.join(".git").exists();
    let has_lockfile = ["package-lock.json", "yarn.lock", "pnpm-lock.yaml", "bun.lockb"]
        .iter()
        .any(|f| project.join(f).exists());
    let last = last_touched(&project, has_git);
    let days = last.as_deref().and_then(days_since);
    NodeModulesEntry {
        project_path: project.to_string_lossy().to_string(),
        node_modules_path: nm.to_string_lossy().to_string(),
        size_bytes: dir_size(nm),
        last_touched: last,
        days_idle: days,
        has_package_json: project.join("package.json").exists(),
        has_lockfile,
        has_git,
        has_patches: project.join("patches").is_dir(),
    }
}

#[tauri::command]
pub async fn scan_node_modules(root: String) -> ScanResult {
    tauri::async_runtime::spawn_blocking(move || scan_node_modules_impl(root))
        .await
        .unwrap_or_else(|_| ScanResult {
            root: String::new(),
            error: Some("Varredura interrompida.".into()),
            entries: vec![],
        })
}

pub fn scan_node_modules_impl(root: String) -> ScanResult {
    let root_path = PathBuf::from(&root);
    if root.trim().is_empty() || !root_path.is_dir() {
        return ScanResult {
            root,
            error: Some("Pasta não encontrada. Confira o caminho.".into()),
            entries: vec![],
        };
    }

    let mut dirs = vec![];
    find_node_modules(&root_path, &mut dirs);

    let mut entries: Vec<NodeModulesEntry> = dirs.iter().map(|p| build_entry(p)).collect();
    // Maior primeiro: o que mais libera espaço fica no topo.
    entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    ScanResult {
        root,
        error: None,
        entries,
    }
}

#[tauri::command]
pub async fn delete_node_modules(paths: Vec<String>, dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || delete_node_modules_impl(paths, dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn delete_node_modules_impl(paths: Vec<String>, dry_run: bool) -> ActionResult {
    if paths.is_empty() {
        return ActionResult::fail("Nada selecionado.", "");
    }

    if dry_run {
        let total: u64 = paths.iter().map(|p| dir_size(Path::new(p))).sum();
        return ActionResult::dry(format!(
            "Apagaria {} pasta(s) node_modules, liberando (aprox.): {}.",
            paths.len(),
            human(total as i64)
        ))
        .with_output(paths.join("\n"));
    }

    let mut freed = 0i64;
    let mut ok_count = 0;
    let mut errs = vec![];

    for p in &paths {
        let path = Path::new(p);
        // Trava de segurança: só apaga se o caminho realmente termina em node_modules.
        let is_nm = path
            .file_name()
            .map(|n| n.to_string_lossy().eq_ignore_ascii_case("node_modules"))
            .unwrap_or(false);
        if !is_nm {
            errs.push(format!("{p}: ignorado (não é node_modules)."));
            continue;
        }
        if !path.exists() {
            continue; // já sumiu
        }

        let before = dir_size(path);
        // rmdir /s /q aguenta melhor caminho longo e pasta grande que o remove_dir_all.
        match run("cmd", &["/c", "rmdir", "/s", "/q", p]) {
            Ok(o) if o.success && !path.exists() => {
                freed += before as i64;
                ok_count += 1;
            }
            Ok(o) => errs.push(format!(
                "{p}: falhou (talvez travado por um editor/servidor aberto). {}",
                o.combined()
            )),
            Err(e) => errs.push(format!("{p}: {e}")),
        }
    }

    let result = if errs.is_empty() {
        ActionResult::ok(format!(
            "{ok_count} pasta(s) apagada(s). Espaço recuperado (aprox.): {}.",
            human(freed.max(0))
        ))
    } else {
        ActionResult::fail(
            format!(
                "Apagadas: {ok_count}. Falhas: {}. Espaço recuperado (aprox.): {}.",
                errs.len(),
                human(freed.max(0))
            ),
            errs.join("\n"),
        )
    };
    log::record("delete_node_modules", result.ok, &result.message, "user");
    result
}
