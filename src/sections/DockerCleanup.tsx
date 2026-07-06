import { useState } from "react";
import { api } from "../lib/api";
import { useAsync } from "../hooks/useAsync";
import {
  Card,
  Button,
  DryRunToggle,
  Empty,
  Badge,
  OutputPanel,
} from "../components/ui";
import { useRunAction } from "../hooks/useRunAction";
import { bytes } from "../lib/format";

export default function DockerCleanup() {
  const [dry, setDry] = useState(true);
  // Guarda a saída do último resultado de cada ação, para exibir no card.
  const [out, setOut] = useState<Record<string, string>>({});
  const { execute, busy } = useRunAction();
  const dash = useAsync(() => api.getDashboard(), []);
  const tokens = useAsync(() => api.listDockerTokens(), []);

  // Atualizar recarrega os dados e limpa toda a saída de terminal mostrada.
  function refresh() {
    setOut({});
    dash.reload();
    tokens.reload();
  }

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">Docker</h1>
          <p className="text-xs text-zinc-500">
            Prune de recursos e compactação do vhdx (WSL2)
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            onClick={refresh}
            loading={dash.loading || tokens.loading}
          >
            Atualizar
          </Button>
          <DryRunToggle value={dry} onChange={setDry} />
        </div>
      </header>

      <Card
        title="System prune"
        subtitle="Remove imagens, containers, redes e volumes não usados."
      >
        <Button
          variant={dry ? "primary" : "danger"}
          loading={busy === "prune"}
          onClick={() =>
            execute("prune", {
              confirm: dry
                ? undefined
                : {
                    title: "Executar docker system prune?",
                    description:
                      "Ação agressiva e sem volta. Remove: containers parados; TODAS as imagens sem container ativo; redes sem uso; e volumes sem uso — incluindo volumes NOMEADOS. Isso pode apagar dados (ex: banco de um projeto que não está rodando agora).",
                    details: ["docker system prune -a --volumes -f"],
                    confirmLabel: "Fazer prune",
                    danger: true,
                  },
              run: () => api.dockerPrune(dry),
              onDone: (r) => {
                setOut((o) => ({ ...o, prune: r.output }));
                dash.reload();
              },
            })
          }
        >
          {dry ? "Simular prune" : "Executar prune"}
        </Button>
        <OutputPanel output={out.prune} />
      </Card>

      <Card
        title="Compactar vhdx"
        subtitle="Encerra o WSL e compacta o disco virtual, liberando espaço no host."
        right={
          <span className="text-xs text-zinc-500">
            Atual: {bytes(dash.data?.dockerVhdxBytes ?? null)}
          </span>
        }
      >
        <div
          className="mb-3 truncate text-[11px] text-zinc-500 selectable"
          title={dash.data?.dockerVhdxPath ?? ""}
        >
          {dash.data?.dockerVhdxPath ?? "vhdx não localizado"}
        </div>
        <Button
          variant="primary"
          loading={busy === "compact"}
          onClick={() =>
            execute("compact", {
              confirm: dry
                ? undefined
                : {
                    title: "Compactar o vhdx do Docker?",
                    description:
                      "O WSL será encerrado (wsl --shutdown) — feche o Docker Desktop antes. Em seguida o disco virtual é compactado via diskpart. Pode levar minutos.",
                    details: [
                      "wsl --shutdown",
                      "diskpart » compact vdisk",
                    ],
                    confirmLabel: "Compactar",
                  },
              run: () => api.dockerCompactVhdx(dry),
              onDone: (r) => {
                setOut((o) => ({ ...o, compact: r.output }));
                dash.reload();
              },
            })
          }
        >
          {dry ? "Simular compactação" : "Compactar agora"}
        </Button>
        <OutputPanel output={out.compact} />
      </Card>

      <Card title="Tokens Docker" subtitle="Registros de auth em ~/.docker/config.json">
        {tokens.data && tokens.data.length > 0 ? (
          <ul className="space-y-1">
            {tokens.data.map((t) => (
              <li
                key={t}
                className="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-950/40 px-3 py-2 text-xs selectable"
              >
                <span className="text-zinc-300">{t}</span>
                <Badge tone="accent">auth</Badge>
              </li>
            ))}
          </ul>
        ) : (
          <Empty>Nenhum registro de auth encontrado.</Empty>
        )}
      </Card>
    </div>
  );
}
