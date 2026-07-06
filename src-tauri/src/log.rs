use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub action: String,
    pub ok: bool,
    pub message: String,
    pub source: String,
}

/// %LOCALAPPDATA%\WinButler
pub fn dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("WinButler")
}

fn log_file() -> PathBuf {
    dir().join("winbutler.log")
}

/// Registra uma ação no log local (JSONL). Nunca entra em pânico.
pub fn record(action: &str, ok: bool, message: &str, source: &str) {
    let entry = LogEntry {
        timestamp: Local::now().to_rfc3339(),
        action: action.to_string(),
        ok,
        message: message.to_string(),
        source: source.to_string(),
    };
    let _ = create_dir_all(dir());
    if let Ok(line) = serde_json::to_string(&entry) {
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file())
        {
            let _ = writeln!(f, "{line}");
        }
    }
}

/// Lê as últimas `limit` entradas do log (mais antigas primeiro).
pub fn read(limit: usize) -> Vec<LogEntry> {
    let content = match std::fs::read_to_string(log_file()) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut entries: Vec<LogEntry> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    if entries.len() > limit {
        entries = entries.split_off(entries.len() - limit);
    }
    entries
}
