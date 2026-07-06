import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import { Card, Button, Empty, Badge, Spinner } from "../components/ui";
import { dateTime } from "../lib/format";

export default function Logs() {
  const { data, loading, reload } = useAsync(() => api.readLogs(), []);
  const entries = (data ?? []).slice().reverse();

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">Log de ações</h1>
          <p className="text-xs text-zinc-500">
            Registro local de todas as operações (%LOCALAPPDATA%\WinButler)
          </p>
        </div>
        <Button variant="ghost" onClick={reload} loading={loading}>
          Atualizar
        </Button>
      </header>

      <Card>
        {loading ? (
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <Spinner /> lendo log…
          </div>
        ) : entries.length > 0 ? (
          <ul className="max-h-[calc(100vh-220px)] space-y-1 overflow-y-auto pr-1 selectable">
            {entries.map((e, i) => (
              <li
                key={i}
                className="flex items-start gap-3 rounded-lg border border-zinc-800 bg-zinc-950/40 px-3 py-2 text-xs"
              >
                <span className="mt-0.5 shrink-0">
                  <Badge tone={e.ok ? "good" : "bad"}>
                    {e.ok ? "ok" : "erro"}
                  </Badge>
                </span>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-zinc-200">{e.action}</span>
                    <Badge>{e.source}</Badge>
                    <span className="text-[10px] text-zinc-500">
                      {dateTime(e.timestamp)}
                    </span>
                  </div>
                  <div className="text-zinc-400">{e.message}</div>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <Empty>Nenhuma ação registrada ainda.</Empty>
        )}
      </Card>
    </div>
  );
}
