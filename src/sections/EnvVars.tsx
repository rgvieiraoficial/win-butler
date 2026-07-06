import { useState } from "react";
import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import { Card, Button, DryRunToggle, Empty, Badge, Spinner } from "../components/ui";
import { useRunAction } from "../hooks/useRunAction";

type Scope = "user" | "machine";

export default function EnvVars() {
  const [dry, setDry] = useState(true);
  const [scope, setScope] = useState<Scope>("user");
  const [onlySuspicious, setOnlySuspicious] = useState(false);
  const { execute, busy } = useRunAction();
  const { data, loading, reload } = useAsync(
    () => api.listEnvVars(scope),
    [scope]
  );

  const list = (data ?? [])
    .filter((v) => !onlySuspicious || v.suspicious)
    .sort((a, b) => Number(b.suspicious) - Number(a.suspicious));

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">
            Variáveis de ambiente
          </h1>
          <p className="text-xs text-zinc-500">
            Variáveis de usuário e sistema · tokens destacados
          </p>
        </div>
        <DryRunToggle value={dry} onChange={setDry} />
      </header>

      <div className="flex items-center gap-2">
        <div className="flex rounded-lg border border-zinc-800 p-0.5 text-xs">
          {(["user", "machine"] as Scope[]).map((s) => (
            <button
              key={s}
              onClick={() => setScope(s)}
              className={`rounded-md px-3 py-1 transition ${
                scope === s
                  ? "bg-accent/15 text-accent"
                  : "text-zinc-400 hover:text-zinc-200"
              }`}
            >
              {s === "user" ? "Usuário" : "Sistema"}
            </button>
          ))}
        </div>
        <label className="flex cursor-pointer items-center gap-1.5 text-xs text-zinc-400">
          <input
            type="checkbox"
            checked={onlySuspicious}
            onChange={(e) => setOnlySuspicious(e.target.checked)}
            className="accent-accent"
          />
          Só suspeitas
        </label>
        <Button variant="ghost" onClick={reload} loading={loading}>
          Atualizar
        </Button>
      </div>

      <Card>
        {loading ? (
          <div className="flex items-center gap-2 text-xs text-zinc-500">
            <Spinner /> lendo variáveis…
          </div>
        ) : list.length > 0 ? (
          <ul className="space-y-1.5">
            {list.map((v) => (
              <li
                key={v.name}
                className={`flex items-center justify-between gap-3 rounded-lg border px-3 py-2 ${
                  v.suspicious
                    ? "border-amber-500/30 bg-amber-500/5"
                    : "border-zinc-800 bg-zinc-950/40"
                }`}
              >
                <div className="min-w-0 selectable">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-medium text-zinc-100">
                      {v.name}
                    </span>
                    {v.suspicious && <Badge tone="warn">possível token</Badge>}
                  </div>
                  <div
                    className="truncate text-[11px] text-zinc-500"
                    title={v.value}
                  >
                    {v.suspicious ? mask(v.value) : v.value || "(vazio)"}
                  </div>
                </div>
                <Button
                  variant={dry ? "ghost" : "danger"}
                  loading={busy === v.name}
                  onClick={() =>
                    execute(v.name, {
                      confirm: dry
                        ? undefined
                        : {
                            title: `Remover ${v.name}?`,
                            description: `A variável de ambiente de ${
                              scope === "user" ? "usuário" : "sistema"
                            } será removida permanentemente. Programas que a usam podem parar de funcionar.`,
                            details: [`${scope}: ${v.name}`],
                            confirmLabel: "Remover",
                            danger: true,
                          },
                      run: () => api.removeEnvVar(scope, v.name, dry),
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
          <Empty>Nenhuma variável para exibir.</Empty>
        )}
      </Card>
    </div>
  );
}

function mask(v: string): string {
  if (v.length <= 8) return "•".repeat(v.length);
  return `${v.slice(0, 4)}${"•".repeat(6)}${v.slice(-4)}`;
}
