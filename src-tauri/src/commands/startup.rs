use crate::log;
use crate::util::{powershell, powershell_json, as_array, s, ActionResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub location: String,
    pub enabled: bool,
}

const LIST_SCRIPT: &str = r#"
$approvedPaths = @(
 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run',
 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32',
 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder',
 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run',
 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run32',
 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\StartupFolder'
)
function Get-Enabled($name){
  foreach($p in $approvedPaths){
    try{
      $item = Get-ItemProperty -Path $p -ErrorAction Stop
      $val = $item.$name
      if($val -is [byte[]] -and $val.Length -ge 1){
        if($val[0] -band 1){ return $false } else { return $true }
      }
    } catch {}
  }
  return $true
}
@(Get-CimInstance Win32_StartupCommand -ErrorAction SilentlyContinue | ForEach-Object {
  [pscustomobject]@{
    name = [string]$_.Name
    command = [string]$_.Command
    location = [string]$_.Location
    enabled = [bool](Get-Enabled $_.Name)
  }
}) | ConvertTo-Json -Depth 3
"#;

#[tauri::command]
pub async fn list_startup() -> Vec<StartupItem> {
    tauri::async_runtime::spawn_blocking(list_startup_impl)
        .await
        .unwrap_or_default()
}

pub fn list_startup_impl() -> Vec<StartupItem> {
    let json = powershell_json(LIST_SCRIPT);
    as_array(json)
        .into_iter()
        .map(|v| StartupItem {
            name: s(&v, "name"),
            command: s(&v, "command"),
            location: s(&v, "location"),
            enabled: v.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true),
        })
        .filter(|i| !i.name.is_empty())
        .collect()
}

/// Deriva a chave StartupApproved correta a partir da location reportada.
fn approved_key(location: &str) -> String {
    let up = location.to_uppercase();
    let hive = if up.contains("HKLM") || up.contains("LOCAL_MACHINE") {
        "HKLM"
    } else {
        "HKCU"
    };
    let leaf = if up.contains("STARTUP") && !up.contains("\\RUN") {
        "StartupFolder"
    } else {
        "Run"
    };
    format!("{hive}:\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\{leaf}")
}

#[tauri::command]
pub async fn set_startup(
    name: String,
    location: String,
    enabled: bool,
    dry_run: bool,
) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || set_startup_impl(name, location, enabled, dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn set_startup_impl(
    name: String,
    location: String,
    enabled: bool,
    dry_run: bool,
) -> ActionResult {
    let key = approved_key(&location);
    let state = if enabled { "habilitar" } else { "desabilitar" };
    if dry_run {
        return ActionResult::dry(format!("Iria {state} `{name}` em {key}."));
    }

    // Task Manager usa: byte[0]=2 (habilitado) / 3 (desabilitado).
    let first = if enabled { 2 } else { 3 };
    let safe_name = name.replace('\'', "''");
    let script = format!(
        r#"$bytes = [byte[]]({first},0,0,0,0,0,0,0,0,0,0,0)
$key = '{key}'
if(-not (Test-Path $key)){{ New-Item -Path $key -Force | Out-Null }}
New-ItemProperty -Path $key -Name '{safe_name}' -Value $bytes -PropertyType Binary -Force | Out-Null"#
    );

    let result = match powershell(&script) {
        Ok(o) if o.success => {
            ActionResult::ok(format!("`{name}` {}.", if enabled { "habilitado" } else { "desabilitado" }))
        }
        Ok(o) => ActionResult::fail("Falha ao alterar item de inicialização.", o.combined()),
        Err(e) => ActionResult::fail("Falha ao executar PowerShell.", e.to_string()),
    };
    log::record("set_startup", result.ok, &result.message, "user");
    result
}
