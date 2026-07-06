import { useState, type ReactNode } from "react";
import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import { Card, Button, DryRunToggle, Empty, Badge, Spinner } from "../components/ui";
import { useRunAction } from "../hooks/useRunAction";
import { dateTime } from "../lib/format";
import type { Capabilities, Frequency, Routine } from "../lib/types";

const ROUTINES: { key: Routine; label: string; needs?: "docker" }[] = [
  { key: "disk_cleanup", label: "Limpeza de disco (cleanmgr)" },
  { key: "empty_recycle_bin", label: "Esvaziar lixeira" },
  { key: "docker_prune", label: "Docker system prune", needs: "docker" },
  { key: "docker_compact", label: "Compactar vhdx do Docker", needs: "docker" },
  { key: "clean_dev_caches", label: "Limpar caches de dev (npm/yarn/…)" },
];

const WEEKDAYS = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"];

export default function Scheduler({ caps }: { caps: Capabilities | null }) {
  const [dry, setDry] = useState(true);
  const [routine, setRoutine] = useState<Routine>("disk_cleanup");
  const [freq, setFreq] = useState<Frequency>("weekly");
  const [time, setTime] = useState("03:00");
  const [day, setDay] = useState("SUN");
  const { execute, busy } = useRunAction();
  const { data, loading, reload } = useAsync(() => api.listScheduled(), []);

  const routines = ROUTINES.filter((r) => !r.needs || (caps && caps[r.needs]));
  const needsDay = freq === "weekly" || freq === "monthly";

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">Agendamento</h1>
          <p className="text-xs text-zinc-500">
            Rotinas via Agendador de Tarefas do Windows
          </p>
        </div>
        <DryRunToggle value={dry} onChange={setDry} />
      </header>

      <Card title="Nova rotina agendada">
        <div className="grid grid-cols-2 gap-3">
          <Field label="Rotina">
            <select
              className="input"
              value={routine}
              onChange={(e) => setRoutine(e.target.value as Routine)}
            >
              {routines.map((r) => (
                <option key={r.key} value={r.key}>
                  {r.label}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Frequência">
            <select
              className="input"
              value={freq}
              onChange={(e) => setFreq(e.target.value as Frequency)}
            >
              <option value="daily">Diário</option>
              <option value="weekly">Semanal</option>
              <option value="monthly">Mensal</option>
            </select>
          </Field>
          <Field label="Horário">
            <input
              type="time"
              className="input"
              value={time}
              onChange={(e) => setTime(e.target.value)}
            />
          </Field>
          {needsDay && (
            <Field label={freq === "weekly" ? "Dia da semana" : "Dia do mês"}>
              {freq === "weekly" ? (
                <select
                  className="input"
                  value={day}
                  onChange={(e) => setDay(e.target.value)}
                >
                  {WEEKDAYS.map((d) => (
                    <option key={d} value={d}>
                      {d}
                    </option>
                  ))}
                </select>
              ) : (
                <input
                  type="number"
                  min={1}
                  max={28}
                  className="input"
                  value={day}
                  onChange={(e) => setDay(e.target.value)}
                />
              )}
            </Field>
          )}
        </div>
        <div className="mt-4">
          <Button
            variant="primary"
            loading={busy === "create"}
            onClick={() =>
              execute("create", {
                confirm: dry
                  ? undefined
                  : {
                      title: "Criar tarefa agendada?",
                      description:
                        "Uma tarefa será registrada no Agendador do Windows para executar esta rotina automaticamente (com privilégios elevados).",
                      details: [
                        `${routine} · ${freq} · ${time}${
                          needsDay ? ` · ${day}` : ""
                        }`,
                      ],
                      confirmLabel: "Agendar",
                    },
                run: () =>
                  api.scheduleRoutine(
                    routine,
                    freq,
                    time,
                    needsDay ? day : null,
                    dry
                  ),
                onDone: () => reload(),
              })
            }
          >
            {dry ? "Simular agendamento" : "Agendar"}
          </Button>
        </div>
      </Card>

      <Card
        title="Rotinas agendadas"
        subtitle="Log de execuções (última execução e resultado)"
        right={
          <Button variant="ghost" onClick={reload} loading={loading}>
            Atualizar
          </Button>
        }
      >
        {loading ? (
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <Spinner /> lendo agendamentos…
          </div>
        ) : data && data.length > 0 ? (
          <ul className="space-y-1.5">
            {data.map((t) => (
              <li
                key={t.name}
                className="flex items-center justify-between gap-3 rounded-lg border border-zinc-800 bg-zinc-950/40 px-3 py-2"
              >
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-medium text-zinc-100">
                      {t.name}
                    </span>
                    <Badge tone={t.state === "Ready" ? "good" : "neutral"}>
                      {t.state}
                    </Badge>
                    {t.lastResult != null && (
                      <Badge tone={t.lastResult === 0 ? "good" : "bad"}>
                        rc={t.lastResult}
                      </Badge>
                    )}
                  </div>
                  <div className="text-[11px] text-zinc-500">
                    Última: {dateTime(t.lastRun)} · Próxima:{" "}
                    {dateTime(t.nextRun)}
                  </div>
                </div>
                <Button
                  variant={dry ? "ghost" : "danger"}
                  loading={busy === t.name}
                  onClick={() =>
                    execute(t.name, {
                      confirm: dry
                        ? undefined
                        : {
                            title: "Remover agendamento?",
                            description:
                              "A tarefa será excluída do Agendador de Tarefas do Windows.",
                            details: [t.name],
                            confirmLabel: "Remover",
                            danger: true,
                          },
                      run: () => api.removeScheduled(t.name, dry),
                      onDone: () => reload(),
                    })
                  }
                >
                  {dry ? "Simular" : "Remover"}
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <Empty>Nenhuma rotina agendada.</Empty>
        )}
      </Card>
    </div>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-[11px] text-zinc-500">{label}</span>
      {children}
    </label>
  );
}
