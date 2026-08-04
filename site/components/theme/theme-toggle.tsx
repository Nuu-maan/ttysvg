"use client";

import { Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";

const swap =
  "absolute transition-[opacity,filter,scale] duration-300 [transition-timing-function:cubic-bezier(0.2,0,0,1)]";

export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();

  return (
    <button
      type="button"
      aria-label="Toggle between light and dark"
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
      className="relative grid size-9 place-items-center rounded-lg text-dim transition-[color,background-color,scale] duration-150 ease-out hover:bg-surface hover:text-ink active:scale-[0.96] after:absolute after:size-11"
    >
      <Sun
        className={`${swap} size-[1.05rem] scale-25 opacity-0 blur-[4px] dark:scale-100 dark:opacity-100 dark:blur-0`}
        aria-hidden
      />
      <Moon
        className={`${swap} size-[1.05rem] scale-100 opacity-100 blur-0 dark:scale-25 dark:opacity-0 dark:blur-[4px]`}
        aria-hidden
      />
    </button>
  );
}
