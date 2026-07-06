import React from "react";

export function Card({
  title,
  subtitle,
  right,
  children,
  className = "",
}: {
  title?: React.ReactNode;
  subtitle?: React.ReactNode;
  right?: React.ReactNode;
  children?: React.ReactNode;
  className?: string;
}) {
  return (
    <div
      className={`rounded-xl border border-zinc-800 bg-zinc-900/60 p-4 ${className}`}
    >
      {(title || right) && (
        <div className="mb-3 flex items-start justify-between gap-3">
          <div>
            {title && (
              <h3 className="text-sm font-medium text-zinc-100">{title}</h3>
            )}
            {subtitle && (
              <p className="mt-0.5 text-xs text-zinc-500">{subtitle}</p>
            )}
          </div>
          {right}
        </div>
      )}
      {children}
    </div>
  );
}

type BtnVariant = "primary" | "ghost" | "danger" | "subtle";

export function Button({
  variant = "subtle",
  loading,
  className = "",
  children,
  ...rest
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: BtnVariant;
  loading?: boolean;
}) {
  const styles: Record<BtnVariant, string> = {
    primary:
      "bg-accent/15 text-accent border-accent/30 hover:bg-accent/25",
    danger:
      "bg-red-500/10 text-red-300 border-red-500/30 hover:bg-red-500/20",
    ghost:
      "bg-transparent text-zinc-300 border-zinc-700 hover:bg-zinc-800",
    subtle:
      "bg-zinc-800 text-zinc-200 border-zinc-700 hover:bg-zinc-700",
  };
  return (
    <button
      {...rest}
      disabled={rest.disabled || loading}
      className={`inline-flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-medium transition disabled:cursor-not-allowed disabled:opacity-50 ${styles[variant]} ${className}`}
    >
      {loading && <Spinner />}
      {children}
    </button>
  );
}

export function Spinner() {
  return (
    <span className="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent" />
  );
}

export function Badge({
  tone = "neutral",
  children,
}: {
  tone?: "neutral" | "good" | "warn" | "bad" | "accent";
  children: React.ReactNode;
}) {
  const tones = {
    neutral: "bg-zinc-800 text-zinc-300",
    good: "bg-emerald-500/15 text-emerald-300",
    warn: "bg-amber-500/15 text-amber-300",
    bad: "bg-red-500/15 text-red-300",
    accent: "bg-accent/15 text-accent",
  };
  return (
    <span
      className={`inline-flex items-center rounded-md px-1.5 py-0.5 text-[11px] font-medium ${tones[tone]}`}
    >
      {children}
    </span>
  );
}

export function Toggle({
  checked,
  onChange,
  disabled,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative h-5 w-9 shrink-0 rounded-full transition disabled:opacity-40 ${
        checked ? "bg-accent/70" : "bg-zinc-700"
      }`}
    >
      <span
        className={`absolute top-0.5 h-4 w-4 rounded-full bg-white transition-all ${
          checked ? "left-4" : "left-0.5"
        }`}
      />
    </button>
  );
}

export function Empty({ children }: { children: React.ReactNode }) {
  return (
    <div className="rounded-lg border border-dashed border-zinc-800 p-6 text-center text-xs text-zinc-500">
      {children}
    </div>
  );
}

// Mostra a saída de texto de um comando (ex: docker system df, diskpart).
// Não renderiza nada se não houver saída.
export function OutputPanel({
  output,
  title = "Saída",
}: {
  output?: string | null;
  title?: string;
}) {
  if (!output || !output.trim()) return null;
  return (
    <div className="mt-3">
      <div className="mb-1 text-[11px] font-medium text-zinc-500">{title}</div>
      <pre className="max-h-60 overflow-auto whitespace-pre-wrap break-all rounded-lg border border-zinc-800 bg-zinc-950/60 p-3 text-[11px] leading-relaxed text-zinc-300 selectable">
        {output.trim()}
      </pre>
    </div>
  );
}

export function DryRunToggle({
  value,
  onChange,
}: {
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="inline-flex cursor-pointer items-center gap-2 text-xs text-zinc-400">
      <Toggle checked={value} onChange={onChange} />
      <span>Dry run {value ? "(simular)" : "(executar)"}</span>
    </label>
  );
}
