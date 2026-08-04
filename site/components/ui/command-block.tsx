import { CopyButton } from "@/components/ui/copy-button";
import { cn } from "@/lib/utils";

export function CommandBlock({
  lines,
  className,
}: {
  lines: readonly string[];
  className?: string;
}) {
  const text = lines.join("\n");

  return (
    <div
      className={cn(
        "flex items-start gap-3 rounded-xl bg-surface p-2 pl-4 shadow-[var(--shadow-border)]",
        className,
      )}
    >
      <pre className="min-w-0 flex-1 overflow-x-auto py-1.5 font-mono text-[0.8125rem] leading-6 sm:text-sm">
        <code>
          {lines.map((line) => (
            <span key={line} className="block">
              <span aria-hidden className="mr-2.5 text-muted select-none">
                $
              </span>
              {line}
            </span>
          ))}
        </code>
      </pre>
      <CopyButton value={text} label="Copy install commands" />
    </div>
  );
}
