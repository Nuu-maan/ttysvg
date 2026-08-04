import { Playground } from "@/components/playground/playground";
import { Section } from "@/components/layout/section";

export function PlaygroundSection() {
  return (
    <Section
      id="playground"
      eyebrow="Playground"
      title="Change the render, not the recording"
      lead="This is ttysvg compiled to WebAssembly, running in your browser on a real capture. Every control below is a flag you would pass on the command line, and the file it produces is the file you would get. Drop in a capture of your own if you have one."
    >
      <Playground />
    </Section>
  );
}
