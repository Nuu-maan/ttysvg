"use client";

import { useMemo, useState, type ReactNode } from "react";

import { ThemeChips } from "@/components/demo/theme-chips";
import { HERO_ASPECT } from "@/lib/hero-svg";
import { DEFAULT_THEME, THEMES, paletteVars, type Theme } from "@/lib/themes";

const STAGE_ID = "ttysvg-stage";

function declarations(vars: Record<string, string>): string {
  return Object.entries(vars)
    .map(([name, value]) => `${name}:${value}`)
    .join(";");
}

function stageCss(theme: Theme): string {
  const dark = `#${STAGE_ID}{${declarations(paletteVars(theme.dark))}}`;
  if (!theme.light) return dark;
  const light = `[data-theme="light"] #${STAGE_ID}{${declarations(paletteVars(theme.light))}}`;
  return `${dark}${light}`;
}

export function ThemeStage({ children }: { children: ReactNode }) {
  const [slug, setSlug] = useState(DEFAULT_THEME);

  const active = useMemo(
    () => THEMES.find((t) => t.slug === slug) ?? THEMES[0],
    [slug],
  );

  return (
    <div>
      <style dangerouslySetInnerHTML={{ __html: stageCss(active) }} />

      <div
        id={STAGE_ID}
        style={{ aspectRatio: HERO_ASPECT }}
        className="overflow-hidden rounded-xl shadow-[var(--shadow-border)]"
      >
        {children}
      </div>

      <ThemeChips
        value={active.slug}
        onChange={setSlug}
        label="Preview theme"
        className="mt-6"
      />

      <p className="mt-4 text-sm text-muted">
        Nothing is re-recorded. The theme is eighteen CSS variables, which is
        exactly how{" "}
        <code className="font-mono text-[0.8125rem] text-dim">
          ttysvg render --theme
        </code>{" "}
        does it.
      </p>
    </div>
  );
}
