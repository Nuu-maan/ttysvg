import { ThemeStage } from "@/components/demo/theme-stage";
import { HERO_SVG } from "@/lib/hero-svg";

export function HeroDemo() {
  return (
    <ThemeStage>
      <div
        role="img"
        aria-label="A terminal recording of ttysvg recording the glowfetch system dashboard"
        dangerouslySetInnerHTML={{ __html: HERO_SVG }}
      />
    </ThemeStage>
  );
}
