"use client";

import { useRef } from "react";

import { THEMES } from "@/lib/themes";
import { cn } from "@/lib/utils";

const KEYS: Record<string, number> = {
  ArrowRight: 1,
  ArrowDown: 1,
  ArrowLeft: -1,
  ArrowUp: -1,
};

export function ThemeChips({
  value,
  onChange,
  label,
  className,
}: {
  value: string;
  onChange: (slug: string) => void;
  label: string;
  className?: string;
}) {
  const chips = useRef<(HTMLButtonElement | null)[]>([]);

  function move(index: number) {
    const next = (index + THEMES.length) % THEMES.length;
    onChange(THEMES[next].slug);
    chips.current[next]?.focus();
  }

  function onKeyDown(event: React.KeyboardEvent, index: number) {
    const step = KEYS[event.key];
    if (step) {
      event.preventDefault();
      move(index + step);
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      move(0);
    }
    if (event.key === "End") {
      event.preventDefault();
      move(THEMES.length - 1);
    }
  }

  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={cn("flex flex-wrap gap-2", className)}
    >
      {THEMES.map((theme, index) => {
        const selected = theme.slug === value;
        return (
          <button
            key={theme.slug}
            ref={(node) => {
              chips.current[index] = node;
            }}
            type="button"
            role="radio"
            aria-checked={selected}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(theme.slug)}
            onKeyDown={(event) => onKeyDown(event, index)}
            className={cn(
              "flex items-center gap-2 rounded-lg py-1.5 pr-3 pl-2 text-xs transition-[background-color,color,box-shadow,scale] duration-150 ease-out active:scale-[0.96]",
              selected
                ? "bg-surface text-ink shadow-[0_0_0_1px_var(--color-accent)]"
                : "text-dim shadow-[var(--shadow-border)] hover:text-ink hover:shadow-[var(--shadow-border-hover)]",
            )}
          >
            <span
              aria-hidden
              className="flex overflow-hidden rounded-full shadow-[var(--shadow-border)]"
            >
              {[
                theme.dark.bg,
                theme.dark.ansi[1],
                theme.dark.ansi[2],
                theme.dark.ansi[4],
              ].map((color, i) => (
                <span
                  key={i}
                  style={{ backgroundColor: color }}
                  className="size-3"
                />
              ))}
            </span>
            <span className="font-mono whitespace-nowrap">{theme.label}</span>
          </button>
        );
      })}
    </div>
  );
}
