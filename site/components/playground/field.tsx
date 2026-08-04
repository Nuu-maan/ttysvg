"use client";

import { useId } from "react";

import { cn } from "@/lib/utils";

export function Range({
  label,
  value,
  display,
  min,
  max,
  step,
  onChange,
}: {
  label: string;
  value: number;
  display: string;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
}) {
  const id = useId();

  return (
    <div>
      <div className="flex items-baseline justify-between gap-3">
        <label htmlFor={id} className="font-mono text-xs text-dim">
          {label}
        </label>
        <span className="font-mono text-xs text-ink tabular-nums">
          {display}
        </span>
      </div>
      <input
        id={id}
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="mt-2 h-5 w-full cursor-pointer accent-accent"
      />
    </div>
  );
}

export function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className="group flex w-full items-center justify-between gap-3 rounded-lg py-1.5 text-left transition-[scale] duration-150 ease-out active:scale-[0.98]"
    >
      <span className="font-mono text-xs text-dim group-hover:text-ink">
        {label}
      </span>
      <span
        aria-hidden
        className={cn(
          "flex h-5 w-9 shrink-0 items-center rounded-full p-0.5 transition-colors duration-150 ease-out",
          checked ? "bg-accent" : "bg-line-strong",
        )}
      >
        <span
          className={cn(
            "size-4 rounded-full bg-canvas transition-[translate] duration-150 ease-out",
            checked && "translate-x-4",
          )}
        />
      </span>
    </button>
  );
}

export function TextField({
  label,
  value,
  placeholder,
  onChange,
}: {
  label: string;
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
}) {
  const id = useId();

  return (
    <div>
      <label htmlFor={id} className="font-mono text-xs text-dim">
        {label}
      </label>
      <input
        id={id}
        type="text"
        value={value}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        className="mt-2 h-9 w-full rounded-lg bg-canvas px-3 font-mono text-xs text-ink shadow-[var(--shadow-border)] placeholder:text-muted"
      />
    </div>
  );
}

export function Group({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <fieldset>
      <legend className="mb-3 font-mono text-[0.6875rem] tracking-[0.14em] text-muted uppercase">
        {label}
      </legend>
      <div className="space-y-4">{children}</div>
    </fieldset>
  );
}
