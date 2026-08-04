import { ArrowRight } from "lucide-react";

import { GithubMark } from "@/components/brand/github-mark";
import { HeroDemo } from "@/components/demo/hero-demo";
import { Container } from "@/components/layout/container";
import { ButtonLink } from "@/components/ui/button";
import { CommandBlock } from "@/components/ui/command-block";
import { install, site } from "@/lib/site";

export function Hero() {
  return (
    <section className="pt-16 pb-20 sm:pt-24 sm:pb-28">
      <Container>
        <div className="max-w-2xl">
          <p className="mb-5 inline-flex items-center gap-2 rounded-full py-1 pr-3 pl-2.5 font-mono text-xs text-dim shadow-[var(--shadow-border)]">
            <span className="size-1.5 rounded-full bg-accent" aria-hidden />
            Works on Windows, macOS and Linux
          </p>

          <h1 className="font-mono text-[clamp(1.5rem,6.5vw,3rem)] leading-[1.1] font-medium tracking-[-0.03em]">
            Record your terminal.
            <br />
            Get one file.
          </h1>

          <p className="mt-6 text-base leading-relaxed text-dim sm:text-lg">
            {site.name} captures a session into a single animated SVG. The text
            stays real text, so it is sharp at any zoom and you can select it,
            and one file carries a light and a dark palette at once.
          </p>

          <div className="mt-8 flex flex-wrap items-center gap-3">
            <ButtonLink href="/docs/getting-started/install">
              Get started
              <ArrowRight className="size-4" aria-hidden />
            </ButtonLink>
            <ButtonLink href={site.repo} target="_blank" rel="noreferrer" variant="secondary">
              <GithubMark className="size-4" />
              View source
            </ButtonLink>
          </div>

          <CommandBlock lines={install} className="mt-8 max-w-lg" />
        </div>

        <div id="themes" className="mt-14 scroll-mt-24 sm:mt-20">
          <HeroDemo />
        </div>
      </Container>
    </section>
  );
}
