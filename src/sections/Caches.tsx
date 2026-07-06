import { useState } from "react";
import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import {
  Card,
  Button,
  DryRunToggle,
  Empty,
  Badge,
  Spinner,
  OutputPanel,
} from "../components/ui";
import { useRunAction } from "../hooks/useRunAction";
import { bytes } from "../lib/format";

export default function Caches() {
  const [dry, setDry] = useState(true);
  // Saída do último resultado de cada ação (por id do gerenciador, + "all").
  const [out, setOut] = useState<Record<string, string>>({});
  const { execute, busy } = useRunAction();
  const caches = useAsync(() => api.listDevCaches(), []);

  // Atualizar recarrega a lista e limpa toda a saída de terminal mostrada.
  function refresh() {
    setOut({});
    caches.reload();
  }

  const list = caches.data ?? [];
  const available = list.filter((c) => c.available);

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">
            Caches de desenvolvimento
          </h1>
          <p className="text-xs text-zinc-500">
            Cache de gerenciadores de pacote (npm, Yarn, pnpm, pip, NuGet, Cargo)
          </p>
        </div>
        <DryRunToggle value={dry} onChange={setDry} />
      </header>

      <Card
        title="Limpar todos os disponíveis"
        subtitle="Roda a limpeza de cada gerenciador encontrado na máquina, um por um."
        right={
          <Button variant="ghost" onClick={refresh} loading={caches.loading}>
            Atualizar
          </Button>
        }
      >
        <Button
          variant={dry ? "primary" : "danger"}
          loading={busy === "all"}
          disabled={available.length === 0}
          onClick={() =>
            execute("all", {
              confirm: dry
                ? undefined
                : {
                    title: "Limpar todos os caches?",
                    description:
                      "Vai limpar o cache de cada gerenciador encontrado. Os pacotes são baixados de novo na próxima instalação — nenhum projeto é afetado, só fica mais lento da próxima vez.",
                    details: available.map((c) => c.name),
                    confirmLabel: "Limpar tudo",
                    danger: true,
                  },
              run: () => api.cleanAllDevCaches(dry),
              onDone: (r) => {
                setOut((o) => ({ ...o, all: r.output }));
                caches.reload();
              },
            })
          }
        >
          {dry ? "Simular limpeza geral" : "Limpar tudo"}
        </Button>
        <OutputPanel output={out.all} />
      </Card>

      {caches.loading ? (
        <div className="flex items-center gap-2 text-xs text-zinc-500">
          <Spinner /> lendo caches…
        </div>
      ) : available.length === 0 ? (
        <Empty>Nenhum gerenciador de pacote com cache encontrado na máquina.</Empty>
      ) : (
        available.map((c) => (
          <Card
            key={c.id}
            title={c.name}
            subtitle={c.path}
            right={<Badge tone="accent">{bytes(c.sizeBytes)}</Badge>}
          >
            <Button
              variant={dry ? "primary" : "danger"}
              loading={busy === c.id}
              onClick={() =>
                execute(c.id, {
                  confirm: dry
                    ? undefined
                    : {
                        title: `Limpar o cache do ${c.name}?`,
                        description:
                          "Os pacotes em cache são apagados. Eles voltam a ser baixados na próxima instalação — nenhum projeto quebra.",
                        details: [c.path],
                        confirmLabel: "Limpar",
                        danger: true,
                      },
                  run: () => api.cleanDevCache(c.id, dry),
                  onDone: (r) => {
                    setOut((o) => ({ ...o, [c.id]: r.output }));
                    caches.reload();
                  },
                })
              }
            >
              {dry ? "Simular" : "Limpar"}
            </Button>
            <OutputPanel output={out[c.id]} />
          </Card>
        ))
      )}
    </div>
  );
}
