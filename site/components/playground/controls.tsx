"use client";

import { ThemeChips } from "@/components/demo/theme-chips";
import { Group, Range, TextField, Toggle } from "@/components/playground/field";
import type { RenderOptions } from "@/lib/playground";

export function Controls({
  options,
  onChange,
  disabled,
}: {
  options: RenderOptions;
  onChange: (patch: Partial<RenderOptions>) => void;
  disabled: boolean;
}) {
  return (
    <div
      aria-busy={disabled}
      className="space-y-7 rounded-xl bg-surface p-5 shadow-[var(--shadow-border)]"
    >
      <Group label="Theme">
        <ThemeChips
          value={options.theme}
          onChange={(theme) => onChange({ theme })}
          label="Render theme"
        />
      </Group>

      <Group label="Window">
        <Toggle
          label="window chrome"
          checked={options.window}
          onChange={(window) => onChange({ window })}
        />
        <TextField
          label="title"
          value={options.title}
          placeholder="untitled"
          onChange={(title) => onChange({ title })}
        />
      </Group>

      <Group label="Type">
        <Range
          label="font-size"
          value={options.font_size}
          display={`${options.font_size}px`}
          min={10}
          max={22}
          step={1}
          onChange={(font_size) => onChange({ font_size })}
        />
        <Range
          label="padding"
          value={options.padding}
          display={`${options.padding}px`}
          min={0}
          max={48}
          step={2}
          onChange={(padding) => onChange({ padding })}
        />
      </Group>

      <Group label="Timing">
        <Range
          label="speed"
          value={options.speed}
          display={`${options.speed.toFixed(2)}x`}
          min={0.25}
          max={4}
          step={0.25}
          onChange={(speed) => onChange({ speed })}
        />
        <Range
          label="trim-idle"
          value={options.trim_idle_ms}
          display={
            options.trim_idle_ms === 0 ? "off" : `${options.trim_idle_ms}ms`
          }
          min={0}
          max={2000}
          step={100}
          onChange={(trim_idle_ms) => onChange({ trim_idle_ms })}
        />
        <Range
          label="tail"
          value={options.tail_ms}
          display={`${(options.tail_ms / 1000).toFixed(1)}s`}
          min={0}
          max={6000}
          step={250}
          onChange={(tail_ms) => onChange({ tail_ms })}
        />
        <Toggle
          label="loop forever"
          checked={options.loop_forever}
          onChange={(loop_forever) => onChange({ loop_forever })}
        />
      </Group>
    </div>
  );
}
