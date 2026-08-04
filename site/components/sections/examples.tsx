import { ExampleTabs } from "@/components/demo/example-tabs";
import { Section } from "@/components/layout/section";

export function Examples() {
  return (
    <Section
      id="examples"
      eyebrow="Examples"
      title="Write the demo as a script, regenerate it whenever your output changes"
      lead="Recording by hand is fine once. The problem comes later, when your output changes and every demo in the repo is out of date. Each tape below is a real file in the repository, and each recording is what running that exact file produced."
    >
      <ExampleTabs />
    </Section>
  );
}
