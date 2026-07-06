use crate::log;
use crate::util::{powershell, powershell_json, as_array, s, ActionResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub scope: String,
    pub suspicious: bool,
}

fn ps_scope(scope: &str) -> &'static str {
    if scope == "machine" {
        "Machine"
    } else {
        "User"
    }
}

/// Heurística: parece token/chave/segredo?
fn looks_suspicious(name: &str, value: &str) -> bool {
    let n = name.to_uppercase();
    const NEEDLES: [&str; 11] = [
        "TOKEN", "SECRET", "PASSWORD", "PWD", "APIKEY", "API_KEY", "CREDENTIAL",
        "AUTH", "ACCESS_KEY", "PRIVATE", "_KEY",
    ];
    if NEEDLES.iter().any(|k| n.contains(k)) {
        return true;
    }
    // Valor longo, sem espaços, com cara de base64/hex.
    let v = value.trim();
    if v.len() >= 24 && !v.contains(' ') && !v.contains('\\') && !v.contains('/') {
        let alnum_ratio = v
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || "+=/_-".contains(*c))
            .count() as f64
            / v.len() as f64;
        if alnum_ratio > 0.95 {
            return true;
        }
    }
    false
}

#[tauri::command]
pub async fn list_env_vars(scope: String) -> Vec<EnvVar> {
    tauri::async_runtime::spawn_blocking(move || list_env_vars_impl(scope))
        .await
        .unwrap_or_default()
}

pub fn list_env_vars_impl(scope: String) -> Vec<EnvVar> {
    let target = ps_scope(&scope);
    let script = format!(
        "@([Environment]::GetEnvironmentVariables('{target}').GetEnumerator() | ForEach-Object {{ [pscustomobject]@{{ name = [string]$_.Key; value = [string]$_.Value }} }}) | Sort-Object name | ConvertTo-Json -Depth 3"
    );
    let json = powershell_json(&script);
    as_array(json)
        .into_iter()
        .map(|v| {
            let name = s(&v, "name");
            let value = s(&v, "value");
            let suspicious = looks_suspicious(&name, &value);
            EnvVar {
                name,
                value,
                scope: scope.clone(),
                suspicious,
            }
        })
        .filter(|e| !e.name.is_empty())
        .collect()
}

#[tauri::command]
pub async fn remove_env_var(scope: String, name: String, dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || remove_env_var_impl(scope, name, dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn remove_env_var_impl(scope: String, name: String, dry_run: bool) -> ActionResult {
    let target = ps_scope(&scope);
    if dry_run {
        return ActionResult::dry(format!(
            "Removeria a variável de {scope} `{name}` (SetEnvironmentVariable = null)."
        ));
    }
    let safe = name.replace('\'', "''");
    let script = format!("[Environment]::SetEnvironmentVariable('{safe}', $null, '{target}')");
    let result = match powershell(&script) {
        Ok(o) if o.success => ActionResult::ok(format!("Variável `{name}` removida.")),
        Ok(o) => ActionResult::fail("Falha ao remover variável.", o.combined()),
        Err(e) => ActionResult::fail("Falha ao executar PowerShell.", e.to_string()),
    };
    log::record("remove_env_var", result.ok, &result.message, "user");
    result
}
