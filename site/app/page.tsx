import { Footer } from "@/components/layout/footer";
import { Nav } from "@/components/layout/nav";
import { Comparison } from "@/components/sections/comparison";
import { Examples } from "@/components/sections/examples";
import { Formats } from "@/components/sections/formats";
import { GetStarted } from "@/components/sections/get-started";
import { Hero } from "@/components/sections/hero";
import { PlaygroundSection } from "@/components/sections/playground";
import { Secrets } from "@/components/sections/secrets";
import { Wizard } from "@/components/sections/wizard";

export default function Home() {
  return (
    <>
      <a
        href="#content"
        className="sr-only focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:z-100 focus:rounded-lg focus:bg-surface focus:px-4 focus:py-2 focus:text-sm focus:shadow-[var(--shadow-border)]"
      >
        Skip to content
      </a>
      <Nav />
      <main id="content" className="flex-1">
        <Hero />
        <PlaygroundSection />
        <Comparison />
        <Wizard />
        <Examples />
        <Formats />
        <Secrets />
        <GetStarted />
      </main>
      <Footer />
    </>
  );
}
