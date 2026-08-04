import { cn } from "@/lib/utils";

function Line({ text }: { text: string }) {
  if (!text.trim()) return <span className="block h-[1.5em]" />;

  if (text.trimStart().startsWith("#")) {
    return (
      <span className="block pl-[2.5ch] -indent-[2.5ch] text-muted">
        {text}
      </span>
    );
  }

  const match = text.match(/^(\s*)(\S+)([\s\S]*)$/);
  if (!match) {
    return (
      <span className="block pl-[2.5ch] -indent-[2.5ch] text-dim">{text}</span>
    );
  }

  const [, indent, directive, rest] = match;
  return (
    <span className="block pl-[2.5ch] -indent-[2.5ch]">
      {indent}
      <span className="text-ink">{directive}</span>
      <span className="text-dim">{rest}</span>
    </span>
  );
}

export function TapeSource({
  tape,
  className,
}: {
  tape: string;
  className?: string;
}) {
  return (
    <pre
      tabIndex={0}
      role="region"
      aria-label="Tape source"
      className={cn(
        "scroll-subtle overflow-x-hidden overflow-y-auto overscroll-contain rounded-xl bg-surface p-5 font-mono text-xs leading-6 whitespace-pre-wrap shadow-[var(--shadow-border)] sm:text-[0.8125rem]",
        className,
      )}
    >
      <code>
        {tape.split("\n").map((line, i) => (
          <Line key={i} text={line} />
        ))}
      </code>
    </pre>
  );
}
