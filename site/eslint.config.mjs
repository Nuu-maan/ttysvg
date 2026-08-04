import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  // Override default ignores of eslint-config-next.
  globalIgnores([
    // Default ignores of eslint-config-next:
    ".next/**",
    "out/**",
    "build/**",
    "next-env.d.ts",
    ".source/**",
    "lib/hero-svg.ts",
    "lib/playground-default.ts",
    "lib/recordings.ts",
    "lib/themes.ts",
    "public/wasm/**",
  ]),
]);

export default eslintConfig;
