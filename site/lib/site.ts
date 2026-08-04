export const site = {
  name: "ttysvg",
  tagline: "Record your terminal. Get one file.",
  description:
    "Record your terminal into a single animated SVG that stays sharp, follows the reader's light or dark theme, and drops straight into a README. Works on Windows.",
  url: "https://ttysvg.dev",
  repo: "https://github.com/Nuu-maan/ttysvg",
  author: "Nuu-maan",
} as const;

export const install = [
  "git clone https://github.com/Nuu-maan/ttysvg",
  "cd ttysvg",
  "cargo install --path .",
] as const;

export const nav = [
  { href: "/docs", label: "Docs" },
  { href: "/#playground", label: "Playground" },
  { href: "/#formats", label: "Formats" },
  { href: "/#examples", label: "Examples" },
] as const;
