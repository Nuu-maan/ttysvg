import { Section } from "@/components/layout/section";
import { cn } from "@/lib/utils";

const formats = [
  {
    flag: "--webp",
    kb: 34,
    note: "True color, and the smallest by a wide margin",
    best: true,
  },
  { flag: "--apng", kb: 96, note: "True color, only the changed part of each frame is stored" },
  { flag: "SVG", kb: 424, note: "Vector text, light and dark in one file" },
  { flag: "--gif", kb: 900, note: "Crushed to 256 colors, and the format everything accepts" },
];

const max = Math.max(...formats.map((f) => f.kb));

export function Formats() {
  return (
    <Section
      id="formats"
      eyebrow="Output formats"
      title="An SVG is right for a README and wrong nearly everywhere else"
      lead="Social sites, chat apps and issue trackers mostly reject it. Six other formats cover that, and all of them work on all three commands."
    >
      <ul className="space-y-5">
        {formats.map((format) => (
          <li key={format.flag}>
            <div className="flex items-baseline justify-between gap-4">
              <span
                className={cn(
                  "font-mono text-sm",
                  format.best ? "text-accent" : "text-ink",
                )}
              >
                {format.flag}
              </span>
              <span className="font-mono text-sm text-dim tabular-nums">
                {format.kb} KB
              </span>
            </div>
            <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-surface">
              <div
                style={{ width: `${(format.kb / max) * 100}%` }}
                className={cn(
                  "h-full rounded-full",
                  format.best ? "bg-accent" : "bg-line-strong",
                )}
              />
            </div>
            <p className="mt-2 text-sm text-muted">{format.note}</p>
          </li>
        ))}
      </ul>

      <p className="mt-10 max-w-2xl text-[0.9375rem] leading-relaxed text-dim">
        Pick WebP unless something in the chain refuses it. The same recording is
        usually an order of magnitude smaller as WebP than as GIF, and it keeps
        every color instead of being crushed to a palette of 256. GIF remains the
        safe answer for anywhere that has not caught up, notably X.
      </p>
    </Section>
  );
}
