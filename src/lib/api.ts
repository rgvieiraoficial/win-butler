import { invoke } from "@tauri-apps/api/core";
import type {
  ActionResult,
  Capabilities,
  Credential,
  Dashboard,
  DevCache,
  LiveStats,
  ScanResult,
  EnvVar,
  Frequency,
  LogEntry,
  Routine,
  ScheduledTask,
  StartupItem,
} from "./types";

// Camada tipada sobre os comandos Rust. Toda ação destrutiva aceita `dryRun`.
export const api = {
  detectCapabilities: () => invoke<Capabilities>("detect_capabilities"),
  getDashboard: () => invoke<Dashboard>("get_dashboard"),
  liveStats: () => invoke<LiveStats>("live_stats"),

  // Disco
  diskCleanup: (dryRun: boolean) =>
    invoke<ActionResult>("disk_cleanup", { dryRun }),
  emptyRecycleBin: (dryRun: boolean) =>
    invoke<ActionResult>("empty_recycle_bin", { dryRun }),

  // Caches de gerenciadores de pacote (npm, yarn, pnpm, pip, nuget, cargo)
  listDevCaches: () => invoke<DevCache[]>("list_dev_caches"),
  cleanDevCache: (manager: string, dryRun: boolean) =>
    invoke<ActionResult>("clean_dev_cache", { manager, dryRun }),
  cleanAllDevCaches: (dryRun: boolean) =>
    invoke<ActionResult>("clean_all_dev_caches", { dryRun }),

  // node_modules de projetos
  scanNodeModules: (root: string) =>
    invoke<ScanResult>("scan_node_modules", { root }),
  deleteNodeModules: (paths: string[], dryRun: boolean) =>
    invoke<ActionResult>("delete_node_modules", { paths, dryRun }),

  // Docker
  dockerPrune: (dryRun: boolean) =>
    invoke<ActionResult>("docker_prune", { dryRun }),
  dockerCompactVhdx: (dryRun: boolean) =>
    invoke<ActionResult>("docker_compact_vhdx", { dryRun }),
  listDockerTokens: () => invoke<string[]>("list_docker_tokens"),

  // Credenciais
  listCredentials: () => invoke<Credential[]>("list_credentials"),
  removeCredential: (target: string, dryRun: boolean) =>
    invoke<ActionResult>("remove_credential", { target, dryRun }),

  // Variáveis de ambiente
  listEnvVars: (scope: "user" | "machine") =>
    invoke<EnvVar[]>("list_env_vars", { scope }),
  removeEnvVar: (scope: "user" | "machine", name: string, dryRun: boolean) =>
    invoke<ActionResult>("remove_env_var", { scope, name, dryRun }),

  // Inicialização
  listStartup: () => invoke<StartupItem[]>("list_startup"),
  setStartup: (
    name: string,
    location: string,
    enabled: boolean,
    dryRun: boolean
  ) => invoke<ActionResult>("set_startup", { name, location, enabled, dryRun }),

  // Agendamento
  listScheduled: () => invoke<ScheduledTask[]>("list_scheduled"),
  scheduleRoutine: (
    routine: Routine,
    frequency: Frequency,
    time: string,
    day: string | null,
    dryRun: boolean
  ) =>
    invoke<ActionResult>("schedule_routine", {
      routine,
      frequency,
      time,
      day,
      dryRun,
    }),
  removeScheduled: (name: string, dryRun: boolean) =>
    invoke<ActionResult>("remove_scheduled", { name, dryRun }),

  // Logs
  readLogs: () => invoke<LogEntry[]>("read_logs"),
};
