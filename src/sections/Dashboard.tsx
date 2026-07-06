import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import { Card, Badge, Button, Spinner } from "../components/ui";
import { bytes, pct, dateTime } from "../lib/format";
import type { Capabilities, LiveStats } from "../lib/types";

export default function Dashboard({
  caps,
  onNavigate,
}: {
  caps: Capabilities | null;
  onNavigate: (k: "disk" | "docker" | "scheduler") => void;
}) {
  const { data, loading, error, reload } = useAsync(
    () => api.getDashboard(),
    []
  );

  // Auto-refresh leve de CPU e RAM enquanto o Dashboard está aberto.
  const [live, setLive] = useState<LiveStats | null>(null);
  useEffect(() => {
    let alive = true;
    const tick = () =>
      api
        .liveStats()
        .then((s) => alive && setLive(s))
        .catch(() => {});
    tick();
    const id = setInterval(tick, 1500);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  // Valores ao vivo com fallback pro carregamento inicial.
  const cpuUsage = live?.cpuUsage ?? data?.cpu.usage ?? 0;
  const memUsed = live?.memUsedBytes ?? data?.memory.usedBytes ?? 0;
  const memTotal = live?.memTotalBytes ?? data?.memory.totalBytes ?? 0;

  const bl = data?.bitlockerStatus?.toLowerCase() ?? "";
  const blTone = bl.includes("on") || bl.includes("ativ")
    ? "good"
    : bl.includes("off") || bl.includes("desativ")
    ? "warn"
    : "neutral";

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">Dashboard</h1>
          <p className="text-xs text-zinc-500">Resumo do sistema</p>
        </div>
        <Button variant="ghost" onClick={reload} loading={loading}>
          Atualizar
        </Button>
      </header>

      {error && (
        <Card className="border-red-500/30">
          <p className="text-xs text-red-300">{error}</p>
        </Card>
      )}

      {loading && !data ? (
        <div className="flex items-center gap-2 text-xs text-zinc-500">
          <Spinner /> coletando informações…
        </div>
      ) : (
        <>
          <div className="grid grid-cols-2 gap-4">
            {data?.disks.map((d) => (
              <Card key={d.mount} title={`Disco ${d.mount}`}>
                <div className="mb-2 flex items-end justify-between">
                  <span className="text-2xl font-light text-zinc-100">
                    {bytes(d.freeBytes)}
                  </span>
                  <span className="text-xs text-zinc-500">
                    livres de {bytes(d.totalBytes)}
                  </span>
                </div>
                <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                  <div
                    className="h-full rounded-full bg-accent/70"
                    style={{ width: `${pct(d.usedBytes, d.totalBytes)}%` }}
                  />
                </div>
                <div className="mt-1 text-[11px] text-zinc-500">
                  {pct(d.usedBytes, d.totalBytes)}% em uso
                </div>
              </Card>
            ))}

            <Card title="Memória (RAM)">
              <div className="mb-2 flex items-end justify-between">
                <span className="text-2xl font-light text-zinc-100">
                  {bytes(memUsed)}
                </span>
                <span className="text-xs text-zinc-500">
                  em uso de {bytes(memTotal)}
                </span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                <div
                  className="h-full rounded-full bg-accent/70"
                  style={{ width: `${pct(memUsed, memTotal)}%` }}
                />
              </div>
              <div className="mt-1 text-[11px] text-zinc-500">
                {pct(memUsed, memTotal)}% em uso ·{" "}
                {bytes(Math.max(memTotal - memUsed, 0))} livre
              </div>
            </Card>

            <Card title="Processador (CPU)">
              <div className="mb-2 flex items-end justify-between">
                <span className="text-2xl font-light text-zinc-100">
                  {cpuUsage.toFixed(0)}%
                </span>
                <span className="text-xs text-zinc-500">
                  {data?.cpu.tempC != null
                    ? `${data.cpu.tempC}°C`
                    : "temp. indisponível"}
                </span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-zinc-800">
                <div
                  className="h-full rounded-full bg-accent/70 transition-all duration-500"
                  style={{ width: `${Math.min(cpuUsage, 100)}%` }}
                />
              </div>
              <div
                className="mt-1 truncate text-[11px] text-zinc-500"
                title={data?.cpu.name ?? ""}
              >
                {data?.cpu.name || "—"} · {data?.cpu.cores ?? 0} núcleos
              </div>
            </Card>

            {(data?.gpus ?? []).map((g) => (
              <Card key={g.name} title="Placa de vídeo (GPU)">
                <div className="mb-1 flex items-end justify-between gap-2">
                  <span
                    className="truncate text-sm font-light text-zinc-100"
                    title={g.name}
                  >
                    {g.name}
                  </span>
                  <span className="shrink-0 text-xs text-zinc-500">
                    {g.tempC != null ? `${g.tempC}°C` : "—"}
                  </span>
                </div>
                <div className="text-[11px] text-zinc-500">
                  {g.memoryBytes != null
                    ? `${bytes(g.memoryBytes)} de memória`
                    : "temperatura e memória indisponíveis (só em placas NVIDIA, via nvidia-smi)"}
                </div>
              </Card>
            ))}

            <Card
              title="BitLocker"
              right={<Badge tone={blTone as never}>{data?.bitlockerStatus ?? "—"}</Badge>}
            >
              <p className="text-xs text-zinc-500">
                {caps?.bitlocker
                  ? "Status via manage-bde"
                  : "manage-bde indisponível"}
              </p>
            </Card>

            {caps?.docker && (
              <Card
                title="Docker (vhdx)"
                right={
                  <Button variant="ghost" onClick={() => onNavigate("docker")}>
                    Gerenciar
                  </Button>
                }
              >
                <div className="text-2xl font-light text-zinc-100">
                  {bytes(data?.dockerVhdxBytes ?? null)}
                </div>
                <div
                  className="mt-1 truncate text-[11px] text-zinc-500 selectable"
                  title={data?.dockerVhdxPath ?? ""}
                >
                  {data?.dockerVhdxPath ?? "vhdx não encontrado"}
                </div>
              </Card>
            )}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <Card title="Última limpeza">
              <div className="text-sm text-zinc-200">
                {dateTime(data?.lastCleanup)}
              </div>
              <Button
                className="mt-3"
                variant="primary"
                onClick={() => onNavigate("disk")}
              >
                Limpar agora
              </Button>
            </Card>
            <Card title="Próxima limpeza agendada">
              <div className="text-sm text-zinc-200">
                {dateTime(data?.nextCleanup)}
              </div>
              <Button
                className="mt-3"
                variant="ghost"
                onClick={() => onNavigate("scheduler")}
              >
                Ver agendamentos
              </Button>
            </Card>
          </div>
        </>
      )}
    </div>
  );
}
