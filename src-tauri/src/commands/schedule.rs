use crate::log;
use crate::util::{run, powershell_json, as_array, s, ActionResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    pub name: String,
    pub state: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub last_result: Option<i64>,
}

const VALID_ROUTINES: [&str; 5] = [
    "disk_cleanup",
    "empty_recycle_bin",
    "docker_prune",
    "docker_compact",
    "clean_dev_caches",
];

const LIST_SCRIPT: &str = r#"
@(Get-ScheduledTask -TaskPath '\WinButler\*' -ErrorAction SilentlyContinue | ForEach-Object {
  $i = $_ | Get-ScheduledTaskInfo
  [pscustomobject]@{
    name = [string]$_.TaskName
    state = [string]$_.State
    lastRun = $(if($i.LastRunTime){ $i.LastRunTime.ToString('o') } else { $null })
    nextRun = $(if($i.NextRunTime){ $i.NextRunTime.ToString('o') } else { $null })
    lastResult = $i.LastTaskResult
  }
}) | ConvertTo-Json -Depth 3
"#;

#[tauri::command]
pub async fn list_scheduled() -> Vec<ScheduledTask> {
    tauri::async_runtime::spawn_blocking(list_scheduled_impl)
        .await
        .unwrap_or_default()
}

pub fn list_scheduled_impl() -> Vec<ScheduledTask> {
    let json = powershell_json(LIST_SCRIPT);
    as_array(json)
        .into_iter()
        .map(|v| ScheduledTask {
            name: s(&v, "name"),
            state: s(&v, "state"),
            last_run: v.get("lastRun").and_then(|x| x.as_str()).map(String::from),
            next_run: v.get("nextRun").and_then(|x| x.as_str()).map(String::from),
            last_result: v.get("lastResult").and_then(|x| x.as_i64()),
        })
        .filter(|t| !t.name.is_empty())
        .collect()
}

#[tauri::command]
pub async fn schedule_routine(
    routine: String,
    frequency: String,
    time: String,
    day: Option<String>,
    dry_run: bool,
) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || {
        schedule_routine_impl(routine, frequency, time, day, dry_run)
    })
    .await
    .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn schedule_routine_impl(
    routine: String,
    frequency: String,
    time: String,
    day: Option<String>,
    dry_run: bool,
) -> ActionResult {
    if !VALID_ROUTINES.contains(&routine.as_str()) {
        return ActionResult::fail(format!("Rotina inválida: {routine}"), "");
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(e) => return ActionResult::fail("Não foi possível localizar o executável.", e.to_string()),
    };

    let task_name = format!("WinButler\\{routine}");
    let tr = format!("\"{exe}\" --run {routine}");

    // Monta os argumentos de agendamento por frequência.
    let sched = frequency.to_uppercase();
    let mut args: Vec<String> = vec![
        "/create".into(),
        "/tn".into(),
        task_name.clone(),
        "/tr".into(),
        tr.clone(),
        "/sc".into(),
        sched.clone(),
        "/st".into(),
        time.clone(),
        "/rl".into(),
        "HIGHEST".into(),
        "/f".into(),
    ];
    if let Some(d) = day.as_ref() {
        if sched == "WEEKLY" || sched == "MONTHLY" {
            args.push("/d".into());
            args.push(d.clone());
        }
    }

    let desc = format!(
        "{routine} · {frequency} · {time}{}",
        day.as_ref().map(|d| format!(" · {d}")).unwrap_or_default()
    );

    if dry_run {
        return ActionResult::dry(format!("Criaria a tarefa `{task_name}` ({desc})."))
            .with_output(format!("schtasks {}", args.join(" ")));
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let result = match run("schtasks", &arg_refs) {
        Ok(o) if o.success => ActionResult::ok(format!("Rotina agendada: {desc}."))
            .with_output(o.combined()),
        Ok(o) => ActionResult::fail("Falha ao criar tarefa agendada.", o.combined()),
        Err(e) => ActionResult::fail("Falha ao executar schtasks.", e.to_string()),
    };
    log::record("schedule_routine", result.ok, &result.message, "user");
    result
}

#[tauri::command]
pub async fn remove_scheduled(name: String, dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || remove_scheduled_impl(name, dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn remove_scheduled_impl(name: String, dry_run: bool) -> ActionResult {
    // Normaliza para o caminho completo sob WinButler.
    let leaf = name.trim_start_matches('\\');
    let full = if leaf.starts_with("WinButler\\") {
        leaf.to_string()
    } else {
        format!("WinButler\\{leaf}")
    };

    if dry_run {
        return ActionResult::dry(format!("Removeria a tarefa `{full}`."));
    }

    let result = match run("schtasks", &["/delete", "/tn", &full, "/f"]) {
        Ok(o) if o.success => ActionResult::ok(format!("Tarefa `{full}` removida.")),
        Ok(o) => ActionResult::fail("Falha ao remover tarefa.", o.combined()),
        Err(e) => ActionResult::fail("Falha ao executar schtasks.", e.to_string()),
    };
    log::record("remove_scheduled", result.ok, &result.message, "user");
    result
}
