import { ArrowRight } from "lucide-react";

import { Container } from "@/components/layout/container";
import { ButtonLink } from "@/components/ui/button";
import { CommandBlock } from "@/components/ui/command-block";
import { install, site } from "@/lib/site";

export function GetStarted() {
  return (
    <section
      id="install"
      aria-labelledby="install-title"
      className="scroll-mt-20 border-t border-line py-20 sm:py-28"
    >
      <Container>
        <div className="mx-auto max-w-xl text-center">
          <h2
            id="install-title"
            className="font-mono text-2xl font-medium tracking-[-0.02em] sm:text-3xl"
          >
            Make the demo your README deserves
          </h2>
          <p className="mt-4 text-[0.9375rem] leading-relaxed text-dim sm:text-base">
            You need Rust. On Windows you also need version 1809 or newer, which
            is when the pseudo console arrived.
          </p>

          <CommandBlock lines={install} className="mt-8 text-left" />

          <div className="mt-8 flex flex-wrap justify-center gap-3">
            <ButtonLink href="/docs/getting-started/install">
              Read the docs
              <ArrowRight className="size-4" aria-hidden />
            </ButtonLink>
            <ButtonLink href={site.repo} target="_blank" rel="noreferrer" variant="secondary">
              Star on GitHub
            </ButtonLink>
          </div>
        </div>
      </Container>
    </section>
  );
}
