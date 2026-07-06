import { useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../lib/api";
import {
  Card,
  Button,
  DryRunToggle,
  Empty,
  Badge,
  Spinner,
  Toggle,
  OutputPanel,
} from "../components/ui";
import { useRunAction } from "../hooks/useRunAction";
import { bytes } from "../lib/format";
import type { ScanResult } from "../lib/types";

// "parado há": traduz dias pra algo legível.
function idleLabel(d: number | null): string {
  if (d == null) return "sem data";
  if (d < 1) return "hoje";
  if (d < 30) return `parado há ${d}d`;
  const months = Math.floor(d / 30);
  if (months < 12) return `parado há ${months} mês${months > 1 ? "es" : ""}`;
  const years = Math.floor(d / 365);
  return `parado há ${years} ano${years > 1 ? "s" : ""}`;
}

export default function NodeModules() {
  const [root, setRoot] = useState("C:\\www");
  const [dry, setDry] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState<ScanResult | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [out, setOut] = useState<string>("");
  // Filtro: mostra só quem está parado há pelo menos N dias (0 = todos).
  const [minIdle, setMinIdle] = useState(0);
  const { execute, busy } = useRunAction();

  const entries = result?.entries ?? [];

  // Abre a caixa de diálogo do Windows pra escolher a pasta.
  async function browse() {
    const picked = await open({
      directory: true,
      title: "Escolha a pasta pra varrer",
      defaultPath: root || undefined,
    });
    if (typeof picked === "string") setRoot(picked);
  }

  async function scan() {
    setScanning(true);
    setOut("");
    setSelected(new Set());
    try {
      setResult(await api.scanNodeModules(root));
    } finally {
      setScanning(false);
    }
  }

  function toggle(path: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  // Só os que passam no filtro de "parado há X". Sem data (null) some quando o filtro está ativo.
  const visible = useMemo(
    () => entries.filter((e) => (e.daysIdle ?? -1) >= minIdle),
    [entries, minIdle]
  );

  const allSelected =
    visible.length > 0 && visible.every((e) => selected.has(e.nodeModulesPath));
  function toggleAll() {
    setSelected((prev) => {
      const next = new Set(prev);
      if (allSelected) visible.forEach((e) => next.delete(e.nodeModulesPath));
      else visible.forEach((e) => next.add(e.nodeModulesPath));
      return next;
    });
  }

  const selectedTotal = useMemo(
    () =>
      entries
        .filter((e) => selected.has(e.nodeModulesPath))
        .reduce((sum, e) => sum + e.sizeBytes, 0),
    [entries, selected]
  );

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">
            node_modules de projetos
          </h1>
          <p className="text-xs text-zinc-500">
            Encontra e apaga pastas node_modules pra liberar espaço
          </p>
        </div>
        <DryRunToggle value={dry} onChange={setDry} />
      </header>

      <Card
        title="Varrer uma pasta"
        subtitle="Procura todos os node_modules abaixo do caminho. Não entra dentro de node_modules nem de .git."
      >
        <div className="flex gap-2">
          <input
            className="input flex-1"
            value={root}
            placeholder="C:\www"
            onChange={(e) => setRoot(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && scan()}
          />
          <Button variant="ghost" onClick={browse}>
            Procurar…
          </Button>
          <Button variant="primary" onClick={scan} loading={scanning}>
            Varrer
          </Button>
        </div>
        {result?.error && (
          <p className="mt-2 text-xs text-red-300">{result.error}</p>
        )}
      </Card>

      {scanning ? (
        <div className="flex items-center gap-2 text-xs text-zinc-500">
          <Spinner /> varrendo (pode levar um tempo em pastas grandes)…
        </div>
      ) : result && !result.error ? (
        entries.length === 0 ? (
          <Empty>Nenhum node_modules encontrado em {result.root}.</Empty>
        ) : (
          <>
            {/* Barra fixa: some pra baixo a lista, mas a ação de apagar fica sempre à vista. */}
            <div className="sticky top-0 z-10 -mx-6 border-b border-zinc-800 bg-zinc-950/95 px-6 py-3 backdrop-blur">
              <div className="flex items-center justify-between gap-3">
                <span className="text-xs text-zinc-400">
                  {selected.size} selecionado(s) ·{" "}
                  <span className="text-zinc-200">{bytes(selectedTotal)}</span>
                </span>
                <Button
                  variant={dry ? "primary" : "danger"}
                  loading={busy === "delete"}
                  disabled={selected.size === 0}
                  onClick={() =>
                    execute("delete", {
                      confirm: dry
                        ? undefined
                        : {
                            title: `Apagar ${selected.size} node_modules?`,
                            description:
                              "As pastas marcadas serão apagadas de vez. Os projetos não quebram — rode `npm/yarn/pnpm install` pra reconstruir quando precisar. Sem lockfile, a reinstalação pode trazer versões diferentes.",
                            details: [`Libera aprox.: ${bytes(selectedTotal)}`],
                            confirmLabel: "Apagar",
                            danger: true,
                          },
                      run: () => api.deleteNodeModules([...selected], dry),
                      onDone: (r) => {
                        setOut(r.output);
                        if (!dry) scan(); // revarre pra atualizar a lista
                      },
                    })
                  }
                >
                  {dry
                    ? `Simular (${selected.size})`
                    : `Apagar ${selected.size} selecionado(s)`}
                </Button>
              </div>
              <OutputPanel output={out} />
            </div>

            <Card
              title={
                minIdle > 0
                  ? `${visible.length} de ${entries.length}`
                  : `${entries.length} encontrado(s)`
              }
              subtitle="Marque o que apagar. Apagar não quebra o projeto: `install` reconstrói."
              right={
                <div className="flex items-center gap-2">
                  <select
                    className="input"
                    value={minIdle}
                    onChange={(e) => setMinIdle(Number(e.target.value))}
                  >
                    <option value={0}>Todos</option>
                    <option value={30}>Parado 30d+</option>
                    <option value={90}>Parado 90d+</option>
                    <option value={180}>Parado 6 meses+</option>
                    <option value={365}>Parado 1 ano+</option>
                  </select>
                  <span className="whitespace-nowrap text-xs text-zinc-500">
                    {selected.size} sel. · {bytes(selectedTotal)}
                  </span>
                  <Button variant="ghost" onClick={toggleAll}>
                    {allSelected ? "Limpar" : "Todos"}
                  </Button>
                </div>
              }
            >
              {visible.length === 0 ? (
                <Empty>Nenhum projeto parado nesse tempo. Baixe o filtro.</Empty>
              ) : (
              <ul className="space-y-1.5">
                {visible.map((e) => (
                  <li
                    key={e.nodeModulesPath}
                    className="flex items-center justify-between gap-3 rounded-lg border border-zinc-800 bg-zinc-950/40 px-3 py-2"
                  >
                    <div className="flex min-w-0 items-center gap-3">
                      <Toggle
                        checked={selected.has(e.nodeModulesPath)}
                        onChange={() => toggle(e.nodeModulesPath)}
                      />
                      <div className="min-w-0">
                        <div
                          className="truncate text-xs text-zinc-200 selectable"
                          title={e.projectPath}
                        >
                          {e.projectPath}
                        </div>
                        <div className="mt-0.5 flex flex-wrap items-center gap-1.5">
                          <Badge tone={(e.daysIdle ?? 0) >= 90 ? "warn" : "neutral"}>
                            {idleLabel(e.daysIdle)}
                          </Badge>
                          {!e.hasLockfile && (
                            <Badge tone="bad">sem lockfile</Badge>
                          )}
                          {e.hasPatches && <Badge tone="warn">tem patches</Badge>}
                          {!e.hasPackageJson && (
                            <Badge tone="bad">sem package.json</Badge>
                          )}
                        </div>
                      </div>
                    </div>
                    <Badge tone="accent">{bytes(e.sizeBytes)}</Badge>
                  </li>
                ))}
              </ul>
              )}
            </Card>
          </>
        )
      ) : null}
    </div>
  );
}
