import { useMemo, useState } from "react";
import { api } from "./lib/api";
import { useAsync } from "./hooks/useAsync";
import { Spinner } from "./components/ui";
import Dashboard from "./sections/Dashboard";
import DiskCleanup from "./sections/DiskCleanup";
import Caches from "./sections/Caches";
import NodeModules from "./sections/NodeModules";
import DockerCleanup from "./sections/DockerCleanup";
import Credentials from "./sections/Credentials";
import EnvVars from "./sections/EnvVars";
import Startup from "./sections/Startup";
import Scheduler from "./sections/Scheduler";
import Logs from "./sections/Logs";

type Key =
  | "dashboard"
  | "disk"
  | "caches"
  | "node_modules"
  | "docker"
  | "credentials"
  | "env"
  | "startup"
  | "scheduler"
  | "logs";

const NAV: { key: Key; label: string; icon: string; needs?: "docker" }[] = [
  { key: "dashboard", label: "Dashboard", icon: "◧" },
  { key: "disk", label: "Limpeza de disco", icon: "🧹" },
  { key: "caches", label: "Caches de dev", icon: "📦" },
  { key: "node_modules", label: "node_modules", icon: "🗂" },
  { key: "docker", label: "Docker", icon: "🐳", needs: "docker" },
  { key: "credentials", label: "Credenciais", icon: "🔑" },
  { key: "env", label: "Variáveis de ambiente", icon: "⚙" },
  { key: "startup", label: "Inicialização", icon: "⏻" },
  { key: "scheduler", label: "Agendamento", icon: "🗓" },
  { key: "logs", label: "Log de ações", icon: "🧾" },
];

export default function App() {
  const [active, setActive] = useState<Key>("dashboard");
  const { data: caps, loading } = useAsync(() => api.detectCapabilities(), []);

  const nav = useMemo(
    () => NAV.filter((n) => !n.needs || (caps && caps[n.needs])),
    [caps]
  );

  return (
    <div className="flex h-full">
      <aside className="flex w-56 shrink-0 flex-col border-r border-zinc-800 bg-zinc-900/40">
        <div className="flex items-center gap-2 px-4 py-4">
          <span className="text-lg text-accent">❖</span>
          <div>
            <div className="text-sm font-semibold text-zinc-100">WinButler</div>
            <div className="text-[10px] text-zinc-500">
              Manutenção do Windows
            </div>
          </div>
        </div>

        <nav className="flex-1 space-y-0.5 px-2">
          {loading && (
            <div className="flex items-center gap-2 px-3 py-2 text-xs text-zinc-500">
              <Spinner /> detectando ambiente…
            </div>
          )}
          {nav.map((n) => (
            <button
              key={n.key}
              onClick={() => setActive(n.key)}
              className={`flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-left text-xs transition ${
                active === n.key
                  ? "bg-accent/15 text-accent"
                  : "text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
              }`}
            >
              <span className="w-4 text-center">{n.icon}</span>
              {n.label}
            </button>
          ))}
        </nav>

        <div className="px-4 py-3 text-[10px] leading-relaxed text-zinc-600">
          Requer privilégios de administrador.
          <br />
          Sem telemetria · v0.1.0
        </div>
      </aside>

      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto max-w-4xl p-6">
          {active === "dashboard" && (
            <Dashboard caps={caps} onNavigate={(k) => setActive(k)} />
          )}
          {active === "disk" && <DiskCleanup />}
          {active === "caches" && <Caches />}
          {active === "node_modules" && <NodeModules />}
          {active === "docker" && <DockerCleanup />}
          {active === "credentials" && <Credentials />}
          {active === "env" && <EnvVars />}
          {active === "startup" && <Startup />}
          {active === "scheduler" && <Scheduler caps={caps} />}
          {active === "logs" && <Logs />}
        </div>
      </main>
    </div>
  );
}
