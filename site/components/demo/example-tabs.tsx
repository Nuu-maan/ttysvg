"use client";

import { useRef, useState } from "react";

import { Recording } from "@/components/demo/recording";
import { TapeSource } from "@/components/demo/tape-source";
import { RECORDINGS } from "@/lib/recordings";
import { cn } from "@/lib/utils";

const STEP: Record<string, number> = {
  ArrowRight: 1,
  ArrowDown: 1,
  ArrowLeft: -1,
  ArrowUp: -1,
};

export function ExampleTabs() {
  const [index, setIndex] = useState(0);
  const tabs = useRef<(HTMLButtonElement | null)[]>([]);
  const active = RECORDINGS[index];

  function move(next: number) {
    const wrapped = (next + RECORDINGS.length) % RECORDINGS.length;
    setIndex(wrapped);
    tabs.current[wrapped]?.focus();
  }

  function onKeyDown(event: React.KeyboardEvent, i: number) {
    const step = STEP[event.key];
    if (step) {
      event.preventDefault();
      move(i + step);
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      move(0);
    }
    if (event.key === "End") {
      event.preventDefault();
      move(RECORDINGS.length - 1);
    }
  }

  return (
    <div>
      <div className="scroll-subtle -mx-5 overflow-x-auto px-5 py-1 sm:-mx-8 sm:px-8">
        <div
          role="tablist"
          aria-label="Example recordings"
          className="flex w-max gap-1 rounded-xl bg-surface p-1 shadow-[var(--shadow-border)]"
        >
          {RECORDINGS.map((recording, i) => (
            <button
              key={recording.slug}
              ref={(node) => {
                tabs.current[i] = node;
              }}
              type="button"
              role="tab"
              id={`tab-${recording.slug}`}
              aria-controls={`panel-${recording.slug}`}
              aria-selected={i === index}
              tabIndex={i === index ? 0 : -1}
              onClick={() => setIndex(i)}
              onKeyDown={(event) => onKeyDown(event, i)}
              className={cn(
                "rounded-lg px-3 py-1.5 font-mono text-xs whitespace-nowrap transition-[background-color,color,box-shadow,scale] duration-150 ease-out active:scale-[0.96]",
                i === index
                  ? "bg-card text-ink shadow-[var(--shadow-border)]"
                  : "text-dim hover:text-ink",
              )}
            >
              {recording.slug}
            </button>
          ))}
        </div>
      </div>

      <div
        role="tabpanel"
        id={`panel-${active.slug}`}
        aria-labelledby={`tab-${active.slug}`}
        tabIndex={0}
        className="mt-8 rounded-sm"
      >
        <h3 className="font-mono text-base text-ink">{active.title}</h3>
        <p className="mt-2 max-w-2xl text-sm leading-relaxed text-dim">
          {active.blurb}
        </p>

        <div className="mt-6 grid items-start gap-6 lg:grid-cols-[minmax(0,5fr)_minmax(0,2fr)]">
          <Recording
            key={active.slug}
            src={active.src}
            width={active.width}
            height={active.height}
            alt={`Recording produced by ${active.slug}.tape`}
          />
          <TapeSource
            tape={active.tape}
            className="max-h-[28rem] lg:max-h-[32rem] lg:p-4 lg:text-xs"
          />
        </div>

        <p className="mt-6 max-w-2xl text-sm leading-relaxed text-muted">
          {active.note}
        </p>
      </div>
    </div>
  );
}
