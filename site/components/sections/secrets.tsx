import { Section } from "@/components/layout/section";
import { TapeSource } from "@/components/demo/tape-source";

const flags = `ttysvg record --sanitize \\
  --redact "sk-[A-Za-z0-9]+" \\
  --redact "ghp_[A-Za-z0-9]+" \\
  --out demo.svg -- ./deploy`;

const directives = `sanitize on
redact "sk-[A-Za-z0-9]+"
redact "ghp_[A-Za-z0-9]+"`;

export function Secrets() {
  return (
    <Section
      id="secrets"
      eyebrow="Recording something real"
      title="A recording is a publishing format"
      lead="Whatever was on screen ends up in a file that goes into a README, and a capture stores it as plain readable text. Two flags exist so that does not become a problem."
    >
      <div className="grid items-start gap-6 lg:grid-cols-2">
        <div>
          <h3 className="font-mono text-sm text-ink">On the command line</h3>
          <TapeSource tape={flags} className="mt-3" />
        </div>
        <div>
          <h3 className="font-mono text-sm text-ink">Or in the tape</h3>
          <TapeSource tape={directives} className="mt-3" />
        </div>
      </div>

      <div className="mt-8 grid max-w-4xl gap-6 sm:grid-cols-2">
        <p className="text-sm leading-relaxed text-dim">
          <span className="font-mono text-ink">--sanitize</span> handles the
          boring case with no regex. It rewrites your home directory to a tilde
          and replaces your username and hostname, which covers most of what
          makes people delete a demo and record it again.
        </p>
        <p className="text-sm leading-relaxed text-dim">
          Masking runs the moment a frame is captured, so a secret is never
          written to the SVG or to a saved capture. It covers the command line
          itself, and it keeps the original character count so the layout does
          not move.
        </p>
      </div>
    </Section>
  );
}
