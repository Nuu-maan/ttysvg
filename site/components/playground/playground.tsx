"use client";

import { Download, RotateCcw, Upload } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

import { Controls } from "@/components/playground/controls";
import { Button } from "@/components/ui/button";
import {
  DEFAULT_BYTES,
  DEFAULT_HEIGHT,
  DEFAULT_SRC,
  DEFAULT_WIDTH,
} from "@/lib/playground-default";
import {
  dimensions,
  formatBytes,
  loadEngine,
  optionsFrom,
  type CaptureInfo,
  type Engine,
  type RenderOptions,
} from "@/lib/playground";

type Source = {
  capture: string;
  info: CaptureInfo;
  name: string;
};

const DEBOUNCE_MS = 120;

export function Playground() {
  const root = useRef<HTMLDivElement>(null);
  const objectUrl = useRef<string | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [engine, setEngine] = useState<Engine | null>(null);
  const [source, setSource] = useState<Source | null>(null);
  const [options, setOptions] = useState<RenderOptions | null>(null);
  const [output, setOutput] = useState<{ url: string; bytes: number } | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);

  const publish = useCallback((svg: string) => {
    const blob = new Blob([svg], { type: "image/svg+xml" });
    const url = URL.createObjectURL(blob);
    if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
    objectUrl.current = url;
    setOutput({ url, bytes: blob.size });
  }, []);

  useEffect(() => {
    const node = root.current;
    if (!node) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        observer.disconnect();
        loadEngine()
          .then((loaded) => {
            const info = loaded.inspect(loaded.sample);
            const next = optionsFrom(info);
            setEngine(loaded);
            setSource({
              capture: loaded.sample,
              info,
              name: "cargo test",
            });
            setOptions(next);
            publish(loaded.render(loaded.sample, next));
          })
          .catch((cause: unknown) => {
            setError(cause instanceof Error ? cause.message : String(cause));
          });
      },
      { rootMargin: "400px" },
    );

    observer.observe(node);
    return () => observer.disconnect();
  }, [publish]);

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
      if (objectUrl.current) URL.revokeObjectURL(objectUrl.current);
    },
    [],
  );

  function update(patch: Partial<RenderOptions>) {
    if (!engine || !source || !options) return;
    const next = { ...options, ...patch };
    setOptions(next);

    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      try {
        publish(engine.render(source.capture, next));
        setError(null);
      } catch (cause: unknown) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
    }, DEBOUNCE_MS);
  }

  async function onFile(file: File) {
    if (!engine) return;
    try {
      const capture = await file.text();
      const info = engine.inspect(capture);
      const next = optionsFrom(info);
      setSource({ capture, info, name: file.name });
      setOptions(next);
      publish(engine.render(capture, next));
      setError(null);
    } catch (cause: unknown) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  function reset() {
    if (!engine || !source) return;
    const next = optionsFrom(source.info);
    setOptions(next);
    publish(engine.render(source.capture, next));
  }

  const ready = Boolean(engine && source && options && output);
  const size =
    source && options
      ? dimensions(source.info, options)
      : { width: DEFAULT_WIDTH, height: DEFAULT_HEIGHT };

  return (
    <div
      ref={root}
      className="grid items-start gap-6 lg:grid-cols-[minmax(0,5fr)_minmax(0,2fr)]"
    >
      <div>
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={output?.url ?? DEFAULT_SRC}
          width={size.width}
          height={size.height}
          alt={
            ready
              ? `Recording rendered with the ${options?.theme} theme`
              : "A cargo test run recorded with ttysvg"
          }
          loading="lazy"
          decoding="async"
          className="h-auto w-full rounded-xl bg-card"
        />

        <div className="mt-4 flex flex-wrap items-center gap-x-4 gap-y-2 font-mono text-xs text-muted">
          <span className="text-dim tabular-nums">
            {formatBytes(output?.bytes ?? DEFAULT_BYTES)}
          </span>
          <span aria-hidden>·</span>
          <span className="tabular-nums">
            {Math.round(size.width)} × {Math.round(size.height)}
          </span>
          {source ? (
            <>
              <span aria-hidden>·</span>
              <span className="tabular-nums">
                {source.info.cols}×{source.info.rows}, {source.info.shots}{" "}
                frames
              </span>
              <span aria-hidden>·</span>
              <span>{source.name}</span>
            </>
          ) : null}
        </div>

        <div className="mt-5 flex flex-wrap items-center gap-3">
          <Button
            size="sm"
            onClick={() => {
              if (!output || !options) return;
              const link = document.createElement("a");
              link.href = output.url;
              link.download = `ttysvg-${options.theme}.svg`;
              link.click();
            }}
            disabled={!ready}
            className="disabled:opacity-50"
          >
            <Download className="size-4" aria-hidden />
            Download SVG
          </Button>

          <label className="inline-flex h-9 cursor-pointer items-center gap-2 rounded-lg bg-surface pr-3 pl-3.5 text-sm text-ink shadow-[var(--shadow-border)] transition-[box-shadow,scale] duration-150 ease-out hover:shadow-[var(--shadow-border-hover)] active:scale-[0.96]">
            <Upload className="size-4" aria-hidden />
            Use your capture
            <input
              type="file"
              accept="application/json,.json"
              className="sr-only"
              disabled={!ready}
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (file) void onFile(file);
                event.target.value = "";
              }}
            />
          </label>

          <Button
            size="sm"
            variant="ghost"
            onClick={reset}
            disabled={!ready}
            className="disabled:opacity-50"
          >
            <RotateCcw className="size-4" aria-hidden />
            Reset
          </Button>
        </div>

        <p aria-live="polite" className="mt-4 text-sm text-muted">
          {error ? (
            <span className="text-ink">{error}</span>
          ) : ready ? (
            <>
              Rendered here, by the same Rust that runs on your machine. The
              file above follows your operating system&rsquo;s light or dark
              setting, not this page&rsquo;s, because that is what it does
              inside a README.
            </>
          ) : (
            <>Loading the renderer. The controls come alive when it lands.</>
          )}
        </p>
      </div>

      {options ? (
        <Controls options={options} onChange={update} disabled={!ready} />
      ) : (
        <div
          aria-hidden
          className="h-96 rounded-xl bg-surface shadow-[var(--shadow-border)]"
        />
      )}
    </div>
  );
}
