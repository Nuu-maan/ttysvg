import { Section } from "@/components/layout/section";

const alternatives = [
  {
    tool: "asciinema",
    reason:
      "Unix only. There is no Windows recorder, because it depends on a Unix pty.",
  },
  {
    tool: "VHS",
    reason:
      "Requires ttyd and ffmpeg. The Windows path is unofficial and breaks easily.",
  },
  {
    tool: "svg-term-cli",
    reason: "Unmaintained, and it needs an asciinema recording as its input.",
  },
  {
    tool: "Screen capture to GIF",
    reason:
      "Multi megabyte, blurry when scaled, wrong colors in dark mode, and a single typo means recording the whole thing again.",
  },
];

export function Comparison() {
  return (
    <Section
      id="why"
      eyebrow="Why this exists"
      title="A command line tool is judged by the demo at the top of its README"
      lead="If you develop on Windows, there was no good way to make one."
    >
      <dl className="grid gap-px overflow-hidden rounded-xl bg-line shadow-[var(--shadow-border)] sm:grid-cols-2">
        {alternatives.map((item) => (
          <div key={item.tool} className="bg-canvas p-6">
            <dt className="font-mono text-sm text-ink">{item.tool}</dt>
            <dd className="mt-2 text-sm leading-relaxed text-dim">
              {item.reason}
            </dd>
          </div>
        ))}
      </dl>

      <p className="mt-8 max-w-2xl text-[0.9375rem] leading-relaxed text-dim">
        ttysvg talks to the Windows pseudo console directly. The same code runs
        on macOS and Linux through a normal pty, so a project can use one tool
        everywhere.
      </p>
    </Section>
  );
}
