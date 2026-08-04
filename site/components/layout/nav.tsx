import Link from "next/link";

import { GithubMark } from "@/components/brand/github-mark";
import { Wordmark } from "@/components/brand/wordmark";
import { Container } from "@/components/layout/container";
import { ThemeToggle } from "@/components/theme/theme-toggle";
import { nav, site } from "@/lib/site";

export function Nav() {
  return (
    <header className="sticky top-0 z-50 border-b border-line bg-canvas/80 backdrop-blur-md">
      <Container>
        <div className="flex h-14 items-center justify-between gap-4">
          <Link
            href="/"
            className="rounded-md text-ink transition-opacity duration-150 hover:opacity-70"
            aria-label={`${site.name}, home`}
          >
            <Wordmark />
          </Link>

          <div className="flex items-center gap-1">
            <nav aria-label="Sections" className="hidden items-center sm:flex">
              {nav.map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className="rounded-lg px-3 py-2 text-sm text-dim transition-colors duration-150 hover:bg-surface hover:text-ink"
                >
                  {item.label}
                </Link>
              ))}
            </nav>

            <Link
              href="/docs"
              className="rounded-lg px-3 py-2 text-sm text-dim transition-colors duration-150 hover:bg-surface hover:text-ink sm:hidden"
            >
              Docs
            </Link>

            <a
              href={site.repo}
              target="_blank"
              rel="noreferrer"
              aria-label={`${site.name} on GitHub`}
              className="relative grid size-9 place-items-center rounded-lg text-dim transition-[color,background-color,scale] duration-150 ease-out hover:bg-surface hover:text-ink active:scale-[0.96] after:absolute after:size-11"
            >
              <GithubMark className="size-[1.05rem]" />
            </a>

            <ThemeToggle />
          </div>
        </div>
      </Container>
    </header>
  );
}
