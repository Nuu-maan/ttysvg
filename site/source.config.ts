import { defineConfig } from "fumadocs-mdx/config";

const tape = {
  name: "tape",
  scopeName: "source.tape",
  repository: {},
  patterns: [
    { match: "#.*$", name: "comment.line.number-sign.tape" },
    {
      match:
        "^\\s*(output|theme|width|height|font-size|font|padding|advance|line-height|shell|window|title|loop|trim-idle|tail|speed|type-delay|redact|sanitize)\\b",
      name: "keyword.other.directive.tape",
    },
    {
      match:
        "^\\s*(type|wait|sleep|enter|tab|backspace|escape|space|up|down|left|right|ctrl)\\b",
      name: "keyword.control.action.tape",
    },
    {
      begin: '"',
      end: '"',
      name: "string.quoted.double.tape",
      patterns: [{ match: "\\\\.", name: "constant.character.escape.tape" }],
    },
    { match: "\\b\\d+(?:\\.\\d+)?(?:ms|s|m)\\b", name: "constant.numeric.tape" },
    { match: "\\b(?:on|off)\\b", name: "constant.language.boolean.tape" },
    { match: "\\b\\d+(?:\\.\\d+)?\\b", name: "constant.numeric.tape" },
  ],
};

export default defineConfig({
  mdxOptions: {
    rehypeCodeOptions: {
      langs: ["bash", "toml", "json", "html", tape],
      themes: { light: "github-light", dark: "github-dark-dimmed" },
    },
  },
});
