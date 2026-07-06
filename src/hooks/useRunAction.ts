import { useState } from "react";
import { useConfirm } from "../components/ConfirmDialog";
import type { ActionResult } from "../lib/types";

interface RunOptions {
  // Confirmação: se omitido, executa direto (usado para dry run / não destrutivo).
  confirm?: {
    title: string;
    description: string;
    details?: string[];
    confirmLabel?: string;
    danger?: boolean;
  };
  run: () => Promise<ActionResult>;
  onDone?: (r: ActionResult) => void;
}

// Padroniza: confirmar → executar → notificar → recarregar.
export function useRunAction() {
  const { confirm, notify } = useConfirm();
  const [busy, setBusy] = useState<string | null>(null);

  async function execute(key: string, opts: RunOptions) {
    if (opts.confirm) {
      const ok = await confirm(opts.confirm);
      if (!ok) return;
    }
    setBusy(key);
    try {
      const result = await opts.run();
      notify(result);
      opts.onDone?.(result);
    } catch (e) {
      notify({ ok: false, message: String(e) });
    } finally {
      setBusy(null);
    }
  }

  return { execute, busy };
}
