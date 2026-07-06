import { useState } from "react";
import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import { Card, Toggle, Empty, Spinner, Button, Badge } from "../components/ui";
import { useConfirm } from "../components/ConfirmDialog";

export default function Startup() {
  const { data, loading, reload } = useAsync(() => api.listStartup(), []);
  const { notify } = useConfirm();
  const [busy, setBusy] = useState<string | null>(null);

  async function toggle(name: string, location: string, next: boolean) {
    setBusy(name);
    try {
      const r = await api.setStartup(name, location, next, false);
      notify(r);
      await reload();
    } catch (e) {
      notify({ ok: false, message: String(e) });
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">
            Apps de inicialização
          </h1>
          <p className="text-xs text-zinc-500">
            Programas que iniciam com o Windows
          </p>
        </div>
        <Button variant="ghost" onClick={reload} loading={loading}>
          Atualizar
        </Button>
      </header>

      <Card>
        {loading ? (
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <Spinner /> lendo itens de inicialização…
          </div>
        ) : data && data.length > 0 ? (
          <ul className="space-y-1.5">
            {data.map((s) => (
              <li
                key={`${s.location}::${s.name}`}
                className="flex items-center justify-between gap-3 rounded-lg border border-zinc-800 bg-zinc-950/40 px-3 py-2"
              >
                <div className="min-w-0 selectable">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-medium text-zinc-100">
                      {s.name}
                    </span>
                    <Badge>{s.location}</Badge>
                  </div>
                  <div
                    className="truncate text-[11px] text-zinc-500"
                    title={s.command}
                  >
                    {s.command}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span
                    className={`text-[11px] ${
                      s.enabled ? "text-emerald-300" : "text-zinc-500"
                    }`}
                  >
                    {s.enabled ? "ativo" : "desativado"}
                  </span>
                  <Toggle
                    checked={s.enabled}
                    disabled={busy === s.name}
                    onChange={(v) => toggle(s.name, s.location, v)}
                  />
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <Empty>Nenhum item de inicialização encontrado.</Empty>
        )}
      </Card>
    </div>
  );
}
