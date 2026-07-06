use serde::Serialize;
use serde_json::Value;
use std::os::windows::process::CommandExt;
use std::process::Command;

/// Impede que janelas de console pisquem ao rodar comandos externos.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Resultado padronizado de qualquer ação exposta ao frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub ok: bool,
    pub dry_run: bool,
    pub message: String,
    pub output: String,
    pub details: Option<Value>,
}

impl ActionResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            dry_run: false,
            message: message.into(),
            output: String::new(),
            details: None,
        }
    }

    pub fn dry(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            dry_run: true,
            message: message.into(),
            output: String::new(),
            details: None,
        }
    }

    pub fn fail(message: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            ok: false,
            dry_run: false,
            message: message.into(),
            output: output.into(),
            details: None,
        }
    }

    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        self.output = output.into();
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

pub struct Output {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
}

impl Output {
    /// stdout + stderr concatenados, úteis para exibir/registrar.
    pub fn combined(&self) -> String {
        let mut s = self.stdout.trim_end().to_string();
        if !self.stderr.trim().is_empty() {
            if !s.is_empty() {
                s.push('\n');
            }
            s.push_str(self.stderr.trim_end());
        }
        s
    }
}

/// Executa um programa capturando saída, sem abrir janela de console.
pub fn run(program: &str, args: &[&str]) -> std::io::Result<Output> {
    let out = Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()?;
    Ok(Output {
        success: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code(),
    })
}

/// Roda um script PowerShell forçando saída UTF-8 (evita mojibake).
pub fn powershell(script: &str) -> std::io::Result<Output> {
    let wrapped = format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $ProgressPreference='SilentlyContinue'; {script}"
    );
    run(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped,
        ],
    )
}

/// Roda um script PowerShell que emite JSON e retorna já parseado.
/// Garante array mesmo quando o PS serializa um único objeto.
pub fn powershell_json(script: &str) -> Value {
    match powershell(script) {
        Ok(o) if o.success => {
            let trimmed = o.stdout.trim();
            if trimmed.is_empty() {
                Value::Array(vec![])
            } else {
                serde_json::from_str(trimmed).unwrap_or(Value::Null)
            }
        }
        _ => Value::Null,
    }
}

/// Normaliza um valor JSON para vetor (PS retorna objeto único sem array).
pub fn as_array(v: Value) -> Vec<Value> {
    match v {
        Value::Array(a) => a,
        Value::Null => vec![],
        other => vec![other],
    }
}

pub fn s(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(|x| x.to_string())
        .unwrap_or_default()
}
