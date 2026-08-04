import { Recording } from "@/components/demo/recording";
import { Section } from "@/components/layout/section";
import { WIZARD } from "@/lib/recordings";

const steps = [
  { step: "preset", detail: "Size, padding, font size, window chrome and timing, in one choice." },
  { step: "command", detail: "The lines to run, in order, each typed into your shell." },
  { step: "theme", detail: "Arrow through the themes with a sample frame beside them." },
  { step: "details", detail: "Output path, title bar, sanitizing, and how long to hold." },
  { step: "record", detail: "A summary, then enter. You watch the session run for real." },
];

export function Wizard() {
  return (
    <Section
      id="wizard"
      eyebrow="The short way in"
      title="Run it with no arguments and answer five questions"
      lead="It asks the two things only you can answer, the command to record and how it should look, and fills in everything else."
    >
      <Recording
        src={WIZARD.src}
        width={WIZARD.width}
        height={WIZARD.height}
        alt="The ttysvg interactive mode, from the first prompt through to the finished recording"
      />

      <ol className="mt-10 grid gap-px overflow-hidden rounded-xl bg-line shadow-[var(--shadow-border)] sm:grid-cols-2 lg:grid-cols-5">
        {steps.map((item, i) => (
          <li key={item.step} className="bg-canvas p-5">
            <p className="font-mono text-xs text-muted tabular-nums">
              {String(i + 1).padStart(2, "0")}
            </p>
            <p className="mt-2 font-mono text-sm text-ink">{item.step}</p>
            <p className="mt-1.5 text-sm leading-relaxed text-dim">{item.detail}</p>
          </li>
        ))}
      </ol>

      <p className="mt-8 max-w-2xl text-[0.9375rem] leading-relaxed text-dim">
        It writes two files, not one. Next to the SVG you get the tape that
        produces it, so you leave with something repeatable without having had to
        learn the tape format first.
      </p>
    </Section>
  );
}
