use crate::commands::docker::find_docker_vhdx;
use crate::commands::schedule::list_scheduled_impl;
use crate::log;
use crate::util::{as_array, powershell, powershell_json, run, s};
use chrono::DateTime;
use serde::Serialize;
use std::collections::BTreeSet;
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub mount: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
}

/// Uso de memória (RAM) do sistema, em bytes.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

/// Processador: nome, uso atual (%), num. de nucleos logicos e temperatura (se der pra ler).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub name: String,
    pub usage: f32,
    pub cores: usize,
    pub temp_c: Option<f32>,
}

/// Placa de video: nome e, quando disponivel (NVIDIA), memoria e temperatura.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub temp_c: Option<f32>,
    pub memory_bytes: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub disks: Vec<DiskInfo>,
    pub memory: MemInfo,
    pub cpu: CpuInfo,
    pub gpus: Vec<GpuInfo>,
    pub docker_vhdx_path: Option<String>,
    pub docker_vhdx_bytes: Option<u64>,
    pub bitlocker_status: Option<String>,
    pub bitlocker_raw: Option<String>,
    pub last_cleanup: Option<String>,
    pub next_cleanup: Option<String>,
}

const CLEANUP_ACTIONS: [&str; 4] =
    ["disk_cleanup", "empty_recycle_bin", "docker_prune", "docker_compact"];

fn collect_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = vec![];
    for d in &disks {
        let total = d.total_space();
        if total == 0 {
            continue;
        }
        let mount = d.mount_point().to_string_lossy().to_string();
        if !seen.insert(mount.clone()) {
            continue;
        }
        let free = d.available_space();
        out.push(DiskInfo {
            mount: mount.trim_end_matches('\\').to_string(),
            total_bytes: total,
            free_bytes: free,
            used_bytes: total.saturating_sub(free),
        });
    }
    out
}

/// Dados que mudam a todo momento — usados pelo auto-refresh do Dashboard.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStats {
    pub cpu_usage: f32,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
}

/// Leitura rapida de CPU e RAM. Reusa um `System` vivo (guardado no estado do app),
/// entao o uso de CPU sai da diferenca desde a ultima chamada — sem `sleep`, bem leve.
#[tauri::command]
pub fn live_stats(sys: tauri::State<'_, std::sync::Mutex<System>>) -> LiveStats {
    let mut sys = match sys.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(), // segue mesmo se um lock anterior tiver dado panic
    };
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    LiveStats {
        cpu_usage: sys.global_cpu_usage(),
        mem_used_bytes: sys.used_memory(),
        mem_total_bytes: sys.total_memory(),
    }
}

fn collect_cpu() -> CpuInfo {
    // Uso de CPU exige duas leituras com um intervalo entre elas.
    let mut sys = System::new();
    sys.refresh_cpu_all();
    std::thread::sleep(std::time::Duration::from_millis(250));
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    let name = cpus
        .first()
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_default();
    CpuInfo {
        name,
        usage: sys.global_cpu_usage(),
        cores: cpus.len(),
        temp_c: cpu_temp(),
    }
}

/// Temperatura da CPU via WMI (zona termica ACPI). Muitas placas nao expoem -> None.
fn cpu_temp() -> Option<f32> {
    let out = powershell(
        "$t = Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty CurrentTemperature; if ($t) { [math]::Round(($t/10 - 273.15),1) }",
    )
    .ok()?;
    out.stdout
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|v| *v > 0.0 && *v < 150.0)
}

fn collect_gpus() -> Vec<GpuInfo> {
    // Nomes das placas fisicas via WMI. Filtra por barramento PCI para descartar
    // adaptadores virtuais (ex: "MS Idd Device", telas espelhadas/virtuais).
    let script = "@(Get-CimInstance Win32_VideoController -ErrorAction SilentlyContinue | Where-Object { $_.PNPDeviceID -like 'PCI\\*' } | ForEach-Object { [pscustomobject]@{ name = [string]$_.Name } }) | ConvertTo-Json -Depth 3";
    let mut gpus: Vec<GpuInfo> = as_array(powershell_json(script))
        .into_iter()
        .map(|v| GpuInfo {
            name: s(&v, "name"),
            temp_c: None,
            memory_bytes: None,
        })
        .filter(|g| !g.name.is_empty())
        .collect();

    // Enriquecer com nvidia-smi (temperatura e memoria reais), se existir placa NVIDIA.
    if let Ok(o) = run(
        "nvidia-smi",
        &[
            "--query-gpu=name,temperature.gpu,memory.total",
            "--format=csv,noheader,nounits",
        ],
    ) {
        if o.success {
            for line in o.stdout.lines() {
                let cols: Vec<&str> = line.split(',').map(|c| c.trim()).collect();
                if cols.len() < 3 {
                    continue;
                }
                let (nm, temp, mem_mb) = (
                    cols[0],
                    cols[1].parse::<f32>().ok(),
                    cols[2].parse::<u64>().ok(),
                );
                let mem_bytes = mem_mb.map(|m| m * 1024 * 1024);
                match gpus
                    .iter_mut()
                    .find(|g| g.name.contains(nm) || nm.contains(g.name.as_str()))
                {
                    Some(g) => {
                        g.temp_c = temp;
                        g.memory_bytes = mem_bytes;
                    }
                    None => gpus.push(GpuInfo {
                        name: nm.to_string(),
                        temp_c: temp,
                        memory_bytes: mem_bytes,
                    }),
                }
            }
        }
    }
    gpus
}

fn collect_memory() -> MemInfo {
    let mut sys = System::new();
    sys.refresh_memory();
    MemInfo {
        total_bytes: sys.total_memory(),
        used_bytes: sys.used_memory(),
        available_bytes: sys.available_memory(),
    }
}

fn bitlocker() -> (Option<String>, Option<String>) {
    // Preferimos o PowerShell: ele forca saida em UTF-8, evitando o mojibake que o
    // manage-bde produz em Windows PT-BR (acento de "Proteção" quebra o parsing).
    // ProtectionStatus vira "On" / "Off" / "Unknown" (nome do enum, sempre em ingles).
    if let Ok(o) = powershell("(Get-BitLockerVolume -MountPoint 'C:').ProtectionStatus.ToString()") {
        let status = o.stdout.trim().to_string();
        if o.success && !status.is_empty() {
            return (Some(status), Some(o.combined()));
        }
    }

    // Fallback: manage-bde. A saida vem em codepage OEM, entao o acento de "Proteção"
    // corrompe; por isso o filtro usa "prote" (sem o ç, que sobrevive a corrupcao) e "protection".
    let out = match run("manage-bde", &["-status", "C:"]) {
        Ok(o) if o.success => o,
        _ => return (None, None),
    };
    let raw = out.stdout.clone();
    let status = raw
        .lines()
        .find(|l| {
            let low = l.to_lowercase();
            (low.contains("protection") || low.contains("prote")) && l.contains(':')
        })
        .and_then(|l| l.split_once(':').map(|(_, v)| v.trim().to_string()));
    (status, Some(raw))
}

fn last_cleanup() -> Option<String> {
    log::read(500)
        .into_iter()
        .filter(|e| e.ok && CLEANUP_ACTIONS.contains(&e.action.as_str()))
        .map(|e| e.timestamp)
        .last()
}

fn next_cleanup() -> Option<String> {
    let mut earliest: Option<(DateTime<chrono::FixedOffset>, String)> = None;
    for t in list_scheduled_impl() {
        if let Some(nr) = t.next_run.as_ref() {
            if let Ok(dt) = DateTime::parse_from_rfc3339(nr) {
                match &earliest {
                    Some((cur, _)) if *cur <= dt => {}
                    _ => earliest = Some((dt, nr.clone())),
                }
            }
        }
    }
    earliest.map(|(_, s)| s)
}

#[tauri::command]
pub async fn get_dashboard() -> Dashboard {
    tauri::async_runtime::spawn_blocking(get_dashboard_impl)
        .await
        .unwrap_or_default()
}

pub fn get_dashboard_impl() -> Dashboard {
    let (docker_vhdx_path, docker_vhdx_bytes) = match find_docker_vhdx() {
        Some((p, size)) => (Some(p.to_string_lossy().to_string()), Some(size)),
        None => (None, None),
    };
    let (bitlocker_status, bitlocker_raw) = bitlocker();

    Dashboard {
        disks: collect_disks(),
        memory: collect_memory(),
        cpu: collect_cpu(),
        gpus: collect_gpus(),
        docker_vhdx_path,
        docker_vhdx_bytes,
        bitlocker_status,
        bitlocker_raw,
        last_cleanup: last_cleanup(),
        next_cleanup: next_cleanup(),
    }
}
