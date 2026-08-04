import { readFileSync, writeFileSync } from "node:fs";

const SVG = "public/playground/default.svg";
const CAPTURE = "public/playground/capture.json";
const OUT = "lib/playground-default.ts";

const svg = readFileSync(SVG, "utf8");
const box = svg.match(/viewBox="0 0 ([\d.]+) ([\d.]+)"/);
if (!box) {
  throw new Error(`no viewBox in ${SVG}`);
}

const capture = JSON.parse(readFileSync(CAPTURE, "utf8"));
const bytes = Buffer.byteLength(svg);
const width = Number(box[1]);
const height = Number(box[2]);

writeFileSync(
  OUT,
  `export const DEFAULT_SRC = "/playground/default.svg";
export const DEFAULT_WIDTH = ${width};
export const DEFAULT_HEIGHT = ${height};
export const DEFAULT_BYTES = ${bytes};
export const CAPTURE_COMMAND = ${JSON.stringify(capture.command.join(" "))};
export const CAPTURE_SHOTS = ${capture.shots.length};
`,
);

console.log(
  `playground: ${width}x${height}, ${(bytes / 1024).toFixed(1)} KB, ${capture.shots.length} frames`,
);
