import { copyFileSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const svgDir = resolve(here, "../../docs/examples");
const tapeDir = resolve(here, "../../examples");
const publicDir = resolve(here, "../public/demos");
const outFile = resolve(here, "../lib/recordings.ts");

const META = {
  banner: {
    title: "A README banner for a command line tool",
    blurb:
      "The most common case. A fixed size, a clean prompt with no machine name in it, and a title bar so it reads as a terminal rather than a screenshot of text.",
    note: "The wait on Usage is doing the real work. It holds until the help text is actually on screen, so the recording cannot end early on a slow machine.",
  },
  tui: {
    title: "A full screen TUI",
    blurb:
      "Anything that draws a whole screen, moves the cursor around and redraws in place. This is the case a naive recorder gets wrong.",
    note: "sanitize on matters here. System dashboards print your username and hostname by default, and this is exactly the kind of recording that ends up in a README.",
  },
  build: {
    title: "A long build or test run, sped up",
    blurb:
      "Nobody wants to watch a full compile at real speed, but cutting it loses the point. Speed up the playback and clamp the dead air.",
    note: "speed and trim-idle are different tools. One plays the whole thing faster, the other caps every individual gap, and they are worth setting independently.",
  },
  prompt: {
    title: "An interactive prompt or wizard",
    blurb:
      "Scaffolding tools, npm init, anything that asks questions and waits. The point is to answer the question only after it is on screen.",
    note: "A fixed sleep is the single most common reason a scripted demo desynchronizes. Wait on the question text instead.",
  },
  repl: {
    title: "A REPL session",
    blurb:
      "Language demos and library tutorials, where the output of one line motivates the next. A slower type delay makes it read like a person thinking.",
    note: "The shell is the REPL itself, with no wrapper around it.",
  },
  secrets: {
    title: "A deploy, or anything holding credentials",
    blurb:
      "The case where getting it wrong publishes a key. This recording printed a real looking API key, a token and a home directory. None of them survived into the file.",
    note: "Masking runs the moment a frame is captured, so the secret is never written to the SVG or to a saved capture, and the mask keeps the original character count so nothing shifts.",
  },
  error: {
    title: "Showing an error before the fix",
    blurb:
      "The most useful recordings in a bug report or a tutorial show the failure first. Two commands in one take, with a pause long enough to read the message.",
    note: "The long tail holds the final state before the loop restarts, so the resolution stays on screen long enough to register.",
  },
  git: {
    title: "A git workflow",
    blurb:
      "Several short commands where the interesting part is the sequence rather than any single output.",
    note: "--no-pager is not optional. A pager waiting for a keypress inside a recording looks like a hang.",
  },
  bash: {
    title: "Linux and macOS",
    blurb:
      "Same tool, different shell line. Everything after the terminal parser is platform independent, so the rest of the tape is unchanged.",
    note: "This one was recorded through bash on the same Windows machine as the others, which is the shortest proof that the shell is the only thing that changes.",
  },
};

const ORDER = Object.keys(META);

mkdirSync(publicDir, { recursive: true });

const available = new Set(
  readdirSync(svgDir)
    .filter((f) => f.endsWith(".svg"))
    .map((f) => f.replace(/\.svg$/, "")),
);

const recordings = ORDER.filter((slug) => available.has(slug)).map((slug) => {
  const svgPath = join(svgDir, `${slug}.svg`);
  const svg = readFileSync(svgPath, "utf8");
  const viewBox = svg.match(/viewBox="([\d.\s-]+)"/)?.[1];
  if (!viewBox) throw new Error(`${slug}.svg has no viewBox`);
  const [, , width, height] = viewBox.split(/\s+/).map(Number);

  copyFileSync(svgPath, join(publicDir, `${slug}.svg`));

  const tape = readFileSync(join(tapeDir, `${slug}.tape`), "utf8").trimEnd();

  return {
    slug,
    ...META[slug],
    src: `/demos/${slug}.svg`,
    width: Math.round(width),
    height: Math.round(height),
    tape,
  };
});

function copyStandalone(slug) {
  const svgPath = join(svgDir, `${slug}.svg`);
  const svg = readFileSync(svgPath, "utf8");
  const viewBox = svg.match(/viewBox="([\d.\s-]+)"/)?.[1];
  if (!viewBox) throw new Error(`${slug}.svg has no viewBox`);
  const [, , width, height] = viewBox.split(/\s+/).map(Number);
  copyFileSync(svgPath, join(publicDir, `${slug}.svg`));
  return { src: `/demos/${slug}.svg`, width: Math.round(width), height: Math.round(height) };
}

const wizard = copyStandalone("wizard");

const banner = `// Generated by scripts/build-recordings.mjs from ../examples and ../docs/examples.

`;

const types = `export type Recording = {
  slug: string;
  title: string;
  blurb: string;
  note: string;
  src: string;
  width: number;
  height: number;
  tape: string;
};

`;

writeFileSync(
  outFile,
  `${banner}${types}export const RECORDINGS: Recording[] = ${JSON.stringify(recordings, null, 2)};

export const WIZARD = ${JSON.stringify(wizard, null, 2)};
`,
);

console.log(
  `recordings: wrote ${recordings.length} plus the wizard, and copied their SVGs to public/demos`,
);
