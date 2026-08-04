import type { ReactNode } from "react";

import { Container } from "@/components/layout/container";
import { cn } from "@/lib/utils";

type SectionProps = {
  id?: string;
  eyebrow?: string;
  title: string;
  lead?: ReactNode;
  className?: string;
  children: ReactNode;
};

export function Section({
  id,
  eyebrow,
  title,
  lead,
  className,
  children,
}: SectionProps) {
  const headingId = id ? `${id}-title` : undefined;

  return (
    <section
      id={id}
      aria-labelledby={headingId}
      className={cn("scroll-mt-20 border-t border-line py-20 sm:py-28", className)}
    >
      <Container>
        <header className="max-w-2xl">
          {eyebrow ? (
            <p className="mb-3 font-mono text-xs tracking-[0.14em] text-muted uppercase">
              {eyebrow}
            </p>
          ) : null}
          <h2
            id={headingId}
            className="font-mono text-2xl font-medium tracking-[-0.02em] sm:text-3xl"
          >
            {title}
          </h2>
          {lead ? (
            <p className="mt-4 text-[0.9375rem] leading-relaxed text-dim sm:text-base">
              {lead}
            </p>
          ) : null}
        </header>
        <div className="mt-10 sm:mt-12">{children}</div>
      </Container>
    </section>
  );
}
