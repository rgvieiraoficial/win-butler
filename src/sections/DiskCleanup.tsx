import { useState } from "react";
import { api } from "../lib/api";
import { Card, Button, DryRunToggle } from "../components/ui";
import { useRunAction } from "../hooks/useRunAction";

export default function DiskCleanup() {
  const [dry, setDry] = useState(true);
  const { execute, busy } = useRunAction();

  return (
    <div className="space-y-5">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-100">
            Limpeza de disco
          </h1>
          <p className="text-xs text-zinc-500">
            Limpeza de arquivos temporários e lixeira
          </p>
        </div>
        <DryRunToggle value={dry} onChange={setDry} />
      </header>

      <Card
        title="Disk Cleanup (cleanmgr)"
        subtitle="Executa o perfil salvo em /sageset:1. Configure o perfil uma vez com: cleanmgr /sageset:1"
      >
        <Button
          variant="primary"
          loading={busy === "cleanmgr"}
          onClick={() =>
            execute("cleanmgr", {
              confirm: dry
                ? undefined
                : {
                    title: "Executar limpeza de disco?",
                    description:
                      "Vai rodar `cleanmgr /sagerun:1`, removendo os itens marcados no perfil 1 (temporários, cache, miniaturas, etc.).",
                    details: ["cleanmgr /sagerun:1"],
                    confirmLabel: "Limpar",
                  },
              run: () => api.diskCleanup(dry),
            })
          }
        >
          {dry ? "Simular limpeza" : "Executar limpeza"}
        </Button>
      </Card>

      <Card
        title="Esvaziar lixeira"
        subtitle="Remove permanentemente todos os itens da Lixeira."
      >
        <Button
          variant={dry ? "primary" : "danger"}
          loading={busy === "recycle"}
          onClick={() =>
            execute("recycle", {
              confirm: dry
                ? undefined
                : {
                    title: "Esvaziar a lixeira?",
                    description:
                      "Todos os itens na Lixeira serão apagados permanentemente. Não há como desfazer.",
                    details: ["Clear-RecycleBin -Force"],
                    confirmLabel: "Esvaziar",
                    danger: true,
                  },
              run: () => api.emptyRecycleBin(dry),
            })
          }
        >
          {dry ? "Simular" : "Esvaziar lixeira"}
        </Button>
      </Card>
    </div>
  );
}
