"use client";

import { Check, Copy } from "lucide-react";
import { useEffect, useRef, useState } from "react";

import { cn } from "@/lib/utils";

const swap =
  "absolute size-4 transition-[opacity,filter,scale] duration-300 [transition-timing-function:cubic-bezier(0.2,0,0,1)]";

export function CopyButton({
  value,
  label = "Copy to clipboard",
  className,
}: {
  value: string;
  label?: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => () => clearTimeout(timer.current), []);

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      return;
    }
    setCopied(true);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setCopied(false), 1600);
  }

  return (
    <button
      type="button"
      onClick={copy}
      aria-label={label}
      className={cn(
        "relative grid size-8 shrink-0 place-items-center rounded-md text-muted transition-[color,background-color,scale] duration-150 ease-out hover:bg-canvas hover:text-ink active:scale-[0.96] after:absolute after:size-11",
        className,
      )}
    >
      <Check
        aria-hidden
        className={cn(
          swap,
          "text-accent",
          copied ? "scale-100 opacity-100 blur-0" : "scale-25 opacity-0 blur-[4px]",
        )}
      />
      <Copy
        aria-hidden
        className={cn(
          swap,
          copied ? "scale-25 opacity-0 blur-[4px]" : "scale-100 opacity-100 blur-0",
        )}
      />
      <span aria-live="polite" className="sr-only">
        {copied ? "Copied" : ""}
      </span>
    </button>
  );
}
