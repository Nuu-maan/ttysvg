import Link from "next/link";

import { GithubMark } from "@/components/brand/github-mark";
import { Wordmark } from "@/components/brand/wordmark";
import { Container } from "@/components/layout/container";
import { site } from "@/lib/site";

const links = [
  { href: "/docs", label: "Documentation" },
  { href: "/docs/getting-started/install", label: "Install" },
  { href: "/#examples", label: "Examples" },
  { href: `${site.repo}/issues`, label: "Report a bug", external: true },
];

export function Footer() {
  return (
    <footer className="border-t border-line py-12">
      <Container>
        <div className="flex flex-col gap-8 sm:flex-row sm:items-start sm:justify-between">
          <div className="max-w-xs">
            <Wordmark />
            <p className="mt-3 text-sm leading-relaxed text-muted">
              A terminal recording that is a single file. MIT licensed, and it
              works on Windows.
            </p>
          </div>

          <nav aria-label="Footer" className="flex flex-col gap-2.5 text-sm">
            {links.map((link) =>
              link.external ? (
                <a
                  key={link.href}
                  href={link.href}
                  target="_blank"
                  rel="noreferrer"
                  className="w-fit rounded-sm text-dim transition-colors duration-150 hover:text-ink"
                >
                  {link.label}
                </a>
              ) : (
                <Link
                  key={link.href}
                  href={link.href}
                  className="w-fit rounded-sm text-dim transition-colors duration-150 hover:text-ink"
                >
                  {link.label}
                </Link>
              ),
            )}
          </nav>
        </div>

        <div className="mt-10 flex flex-col gap-4 border-t border-line pt-6 text-sm text-muted sm:flex-row sm:items-center sm:justify-between">
          <p>
            Built by{" "}
            <a
              href={site.repo}
              target="_blank"
              rel="noreferrer"
              className="rounded-sm text-dim transition-colors duration-150 hover:text-ink"
            >
              {site.author}
            </a>
          </p>
          <a
            href={site.repo}
            target="_blank"
            rel="noreferrer"
            className="flex w-fit items-center gap-2 rounded-sm transition-colors duration-150 hover:text-ink"
          >
            <GithubMark className="size-4" />
            <span>Source</span>
          </a>
        </div>
      </Container>
    </footer>
  );
}
