import defaultMdxComponents from "fumadocs-ui/mdx";
import type { MDXComponents } from "mdx/types";

import { Recording } from "@/components/demo/recording";

export function getMDXComponents(components?: MDXComponents) {
  return {
    ...defaultMdxComponents,
    Recording,
    ...components,
  } satisfies MDXComponents;
}

export const useMDXComponents = getMDXComponents;

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}
