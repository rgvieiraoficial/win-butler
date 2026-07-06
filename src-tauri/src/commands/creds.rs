use crate::log;
use crate::util::{run, ActionResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Credential {
    pub target: String,
    #[serde(rename = "type")]
    pub cred_type: String,
    pub user: String,
}

/// Lista credenciais do Gerenciador de Credenciais via `cmdkey /list`.
/// Tolera rótulos em PT e EN (Target/Destino, Type/Tipo, User/Usuário).
#[tauri::command]
pub async fn list_credentials() -> Vec<Credential> {
    tauri::async_runtime::spawn_blocking(list_credentials_impl)
        .await
        .unwrap_or_default()
}

pub fn list_credentials_impl() -> Vec<Credential> {
    // `cmdkey` em PT-BR sai na codepage OEM (ex: "Genérico" vira "Gen�rico" ao ler como UTF-8).
    // Rodar com o console em UTF-8 (chcp 65001) faz a saída já vir correta.
    let out = match run("cmd", &["/c", "chcp 65001>nul && cmdkey /list"]) {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let mut creds: Vec<Credential> = vec![];
    let mut cur: Option<Credential> = None;

    for raw in out.stdout.lines() {
        let line = raw.trim();
        let Some((label, value)) = line.split_once(':') else {
            continue;
        };
        let label = label.trim().to_lowercase();
        let value = value.trim().to_string();

        if label.contains("target") || label.contains("destino") {
            if let Some(c) = cur.take() {
                creds.push(c);
            }
            cur = Some(Credential {
                target: value,
                cred_type: String::new(),
                user: String::new(),
            });
        } else if let Some(c) = cur.as_mut() {
            if label.contains("type") || label.contains("tipo") {
                c.cred_type = value;
            } else if label.contains("user")
                || label.contains("usuário")
                || label.contains("usuario")
            {
                c.user = value;
            }
        }
    }
    if let Some(c) = cur.take() {
        creds.push(c);
    }
    creds
}

#[tauri::command]
pub async fn remove_credential(target: String, dry_run: bool) -> ActionResult {
    tauri::async_runtime::spawn_blocking(move || remove_credential_impl(target, dry_run))
        .await
        .unwrap_or_else(|_| ActionResult::fail("Tarefa interrompida.", ""))
}

pub fn remove_credential_impl(target: String, dry_run: bool) -> ActionResult {
    if dry_run {
        return ActionResult::dry(format!("Removeria a credencial `{target}` (cmdkey /delete)."));
    }
    let arg = format!("/delete:{target}");
    let result = match run("cmdkey", &[&arg]) {
        Ok(o) if o.success => ActionResult::ok(format!("Credencial `{target}` removida.")),
        Ok(o) => ActionResult::fail("Falha ao remover credencial.", o.combined()),
        Err(e) => ActionResult::fail("Falha ao executar cmdkey.", e.to_string()),
    };
    log::record("remove_credential", result.ok, &result.message, "user");
    result
}
