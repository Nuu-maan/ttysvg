export type RenderOptions = {
  theme: string;
  title: string;
  window: boolean;
  padding: number;
  font_size: number;
  speed: number;
  trim_idle_ms: number;
  tail_ms: number;
  loop_forever: boolean;
};

export type CaptureInfo = {
  cols: number;
  rows: number;
  shots: number;
  duration_ms: number;
  command: string[];
  theme: string;
  title: string;
  window: boolean;
  padding: number;
  font_size: number;
  advance: number;
  line_height: number;
  speed: number;
  trim_idle_ms: number;
  tail_ms: number;
  loop_forever: boolean;
};

export type Engine = {
  sample: string;
  render(capture: string, options: RenderOptions): string;
  inspect(capture: string): CaptureInfo;
};

type Module = {
  default(init: { module_or_path: string }): Promise<unknown>;
  render(capture: string, patch: string): string;
  inspect(capture: string): string;
};

const WASM_JS = "/wasm/ttysvg_wasm.js";
const WASM_BINARY = "/wasm/ttysvg_wasm_bg.wasm";
const SAMPLE = "/playground/capture.json";

const importModule = new Function("url", "return import(url)") as (
  url: string,
) => Promise<Module>;

let pending: Promise<Engine> | null = null;

export function loadEngine(): Promise<Engine> {
  pending ??= boot();
  return pending;
}

async function boot(): Promise<Engine> {
  const [wasm, response] = await Promise.all([
    importModule(WASM_JS),
    fetch(SAMPLE),
  ]);

  if (!response.ok) {
    throw new Error(`sample capture: ${response.status}`);
  }

  const [sample] = await Promise.all([
    response.text(),
    wasm.default({ module_or_path: WASM_BINARY }),
  ]);

  return {
    sample,
    render: (capture, options) => wasm.render(capture, JSON.stringify(options)),
    inspect: (capture) => JSON.parse(wasm.inspect(capture)) as CaptureInfo,
  };
}

export function optionsFrom(info: CaptureInfo): RenderOptions {
  return {
    theme: info.theme,
    title: info.title,
    window: info.window,
    padding: info.padding,
    font_size: info.font_size,
    speed: info.speed,
    trim_idle_ms: info.trim_idle_ms,
    tail_ms: info.tail_ms,
    loop_forever: info.loop_forever,
  };
}

const CHROME_HEIGHT = 34;

export function dimensions(info: CaptureInfo, options: RenderOptions) {
  const advance = info.advance > 0 ? info.advance : options.font_size * 0.6;
  const lineHeight =
    info.line_height > 0 ? info.line_height : options.font_size * 1.32;
  return {
    width: info.cols * advance + options.padding * 2,
    height:
      info.rows * lineHeight +
      options.padding * 2 +
      (options.window ? CHROME_HEIGHT : 0),
  };
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  return `${(kb / 1024).toFixed(2)} MB`;
}
