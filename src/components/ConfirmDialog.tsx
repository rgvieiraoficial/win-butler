import React, {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
} from "react";
import type { ActionResult } from "../lib/types";
import { Button } from "./ui";

interface ConfirmOptions {
  title: string;
  description: string;
  details?: string[];
  confirmLabel?: string;
  danger?: boolean;
}

interface Toast {
  id: number;
  ok: boolean;
  dryRun?: boolean;
  message: string;
}

interface Ctx {
  confirm: (opts: ConfirmOptions) => Promise<boolean>;
  notify: (r: ActionResult | { ok: boolean; message: string }) => void;
}

const ConfirmCtx = createContext<Ctx | null>(null);

export function useConfirm(): Ctx {
  const ctx = useContext(ConfirmCtx);
  if (!ctx) throw new Error("useConfirm fora do ConfirmProvider");
  return ctx;
}

export function ConfirmProvider({ children }: { children: React.ReactNode }) {
  const [pending, setPending] = useState<ConfirmOptions | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const resolver = useRef<((v: boolean) => void) | null>(null);
  const idSeq = useRef(0);

  const confirm = useCallback((opts: ConfirmOptions) => {
    setPending(opts);
    return new Promise<boolean>((resolve) => {
      resolver.current = resolve;
    });
  }, []);

  const close = (result: boolean) => {
    resolver.current?.(result);
    resolver.current = null;
    setPending(null);
  };

  const notify = useCallback(
    (r: ActionResult | { ok: boolean; message: string }) => {
      const id = ++idSeq.current;
      const dryRun = "dryRun" in r ? r.dryRun : false;
      setToasts((t) => [...t, { id, ok: r.ok, dryRun, message: r.message }]);
      setTimeout(() => {
        setToasts((t) => t.filter((x) => x.id !== id));
      }, 6000);
    },
    []
  );

  return (
    <ConfirmCtx.Provider value={{ confirm, notify }}>
      {children}

      {pending && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
          <div className="w-full max-w-md rounded-xl border border-zinc-800 bg-zinc-900 p-5 shadow-2xl">
            <h2 className="text-base font-semibold text-zinc-100">
              {pending.title}
            </h2>
            <p className="mt-2 text-sm text-zinc-400">{pending.description}</p>
            {pending.details && pending.details.length > 0 && (
              <ul className="mt-3 space-y-1 rounded-lg border border-zinc-800 bg-zinc-950/60 p-3 text-xs text-zinc-300 selectable">
                {pending.details.map((d, i) => (
                  <li key={i} className="flex gap-2">
                    <span className="text-accent">›</span>
                    <code className="break-all">{d}</code>
                  </li>
                ))}
              </ul>
            )}
            <div className="mt-5 flex justify-end gap-2">
              <Button variant="ghost" onClick={() => close(false)}>
                Cancelar
              </Button>
              <Button
                variant={pending.danger ? "danger" : "primary"}
                onClick={() => close(true)}
              >
                {pending.confirmLabel ?? "Confirmar"}
              </Button>
            </div>
          </div>
        </div>
      )}

      <div className="pointer-events-none fixed bottom-4 right-4 z-50 flex w-80 flex-col gap-2">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={`pointer-events-auto rounded-lg border p-3 text-xs shadow-lg selectable ${
              t.dryRun
                ? "border-accent/40 bg-accent/10 text-accent"
                : t.ok
                ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-200"
                : "border-red-500/40 bg-red-500/10 text-red-200"
            }`}
          >
            <div className="mb-0.5 font-semibold">
              {t.dryRun ? "Dry run" : t.ok ? "Concluído" : "Falha"}
            </div>
            {t.message}
          </div>
        ))}
      </div>
    </ConfirmCtx.Provider>
  );
}
