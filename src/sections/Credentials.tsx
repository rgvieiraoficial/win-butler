import { useState } from "react";
import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import { Card, Button, DryRunToggle, Empty, Badge, Spinner } from "../components/ui";
import { useRunAction } from "../hooks/useRunAction";

export default function Credentials() {
  const [dry, setDry] = useState(true);
  const { execute, busy } = useRunAction();
  const { data, loading, reload } = useAsync(() => api.listCredentials(), []);

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">Credenciais</h1>
          <p className="text-xs text-zinc-500">
            Windows Credential Manager (cmdkey)
          </p>
        </div>
        <div className="flex items-center gap-3">
          <DryRunToggle value={dry} onChange={setDry} />
          <Button variant="ghost" onClick={reload} loading={loading}>
            Atualizar
          </Button>
        </div>
      </header>

      <Card>
        {loading ? (
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <Spinner /> lendo credenciais…
          </div>
        ) : data && data.length > 0 ? (
          <ul className="space-y-1.5">
            {data.map((c) => (
              <li
                key={c.target}
                className="flex items-center justify-between gap-3 rounded-lg border border-zinc-800 bg-zinc-950/40 px-3 py-2"
              >
                <div className="min-w-0 selectable">
                  <div className="truncate text-xs text-zinc-200" title={c.target}>
                    {c.target}
                  </div>
                  <div className="text-[11px] text-zinc-500">
                    {c.user || "sem usuário"} · <Badge>{c.type}</Badge>
                  </div>
                </div>
                <Button
                  variant={dry ? "ghost" : "danger"}
                  loading={busy === c.target}
                  onClick={() =>
                    execute(c.target, {
                      confirm: dry
                        ? undefined
                        : {
                            title: "Remover credencial?",
                            description:
                              "A credencial será removida do Gerenciador de Credenciais do Windows. Apps que dependem dela pedirão login novamente.",
                            details: [`cmdkey /delete:${c.target}`],
                            confirmLabel: "Remover",
                            danger: true,
                          },
                      run: () => api.removeCredential(c.target, dry),
                      onDone: () => reload(),
                    })
                  }
                >
                  {dry ? "Simular remoção" : "Remover"}
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <Empty>Nenhuma credencial armazenada.</Empty>
        )}
      </Card>
    </div>
  );
}
