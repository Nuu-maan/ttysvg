import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

import { Wordmark } from "@/components/brand/wordmark";
import { site } from "@/lib/site";

export function baseOptions(): BaseLayoutProps {
  return {
    githubUrl: site.repo,
    nav: {
      title: <Wordmark />,
      url: "/",
    },
    links: [
      { text: "Home", url: "/", active: "none" },
      { text: "Examples", url: "/#examples", active: "none" },
    ],
  };
}
