// Tipos espelham os structs serializados pelo backend Rust (serde camelCase).

export interface Capabilities {
  docker: boolean;
  wsl: boolean;
  bitlocker: boolean;
  cleanmgr: boolean;
}

export interface ActionResult {
  ok: boolean;
  dryRun: boolean;
  message: string;
  output: string;
  details: unknown | null;
}

export interface DiskInfo {
  mount: string;
  totalBytes: number;
  freeBytes: number;
  usedBytes: number;
}

export interface MemInfo {
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
}

export interface CpuInfo {
  name: string;
  usage: number;
  cores: number;
  tempC: number | null;
}

export interface GpuInfo {
  name: string;
  tempC: number | null;
  memoryBytes: number | null;
}

export interface LiveStats {
  cpuUsage: number;
  memUsedBytes: number;
  memTotalBytes: number;
}

export interface Dashboard {
  disks: DiskInfo[];
  memory: MemInfo;
  cpu: CpuInfo;
  gpus: GpuInfo[];
  dockerVhdxPath: string | null;
  dockerVhdxBytes: number | null;
  bitlockerStatus: string | null;
  bitlockerRaw: string | null;
  lastCleanup: string | null;
  nextCleanup: string | null;
}

export interface DevCache {
  id: string;
  name: string;
  path: string;
  sizeBytes: number | null;
  available: boolean;
}

export interface NodeModulesEntry {
  projectPath: string;
  nodeModulesPath: string;
  sizeBytes: number;
  lastTouched: string | null;
  daysIdle: number | null;
  hasPackageJson: boolean;
  hasLockfile: boolean;
  hasGit: boolean;
  hasPatches: boolean;
}

export interface ScanResult {
  root: string;
  error: string | null;
  entries: NodeModulesEntry[];
}

export interface Credential {
  target: string;
  type: string;
  user: string;
}

export interface EnvVar {
  name: string;
  value: string;
  scope: "user" | "machine";
  suspicious: boolean;
}

export interface StartupItem {
  name: string;
  command: string;
  location: string;
  enabled: boolean;
}

export interface ScheduledTask {
  name: string;
  state: string;
  lastRun: string | null;
  nextRun: string | null;
  lastResult: number | null;
}

export interface LogEntry {
  timestamp: string;
  action: string;
  ok: boolean;
  message: string;
  source: string;
}

export type Routine =
  | "disk_cleanup"
  | "empty_recycle_bin"
  | "docker_prune"
  | "docker_compact"
  | "clean_dev_caches";

export type Frequency = "daily" | "weekly" | "monthly";
