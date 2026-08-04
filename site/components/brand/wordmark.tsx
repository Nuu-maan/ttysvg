import { Logo } from "@/components/brand/logo";
import { cn } from "@/lib/utils";

type WordmarkProps = {
  className?: string;
  logoClassName?: string;
};

export function Wordmark({ className, logoClassName }: WordmarkProps) {
  return (
    <span className={cn("inline-flex items-center gap-2", className)}>
      <Logo className={cn("size-5", logoClassName)} />
      <span className="font-mono text-[0.9375rem] font-medium tracking-[-0.02em] select-none">
        tty<span className="text-accent">svg</span>
      </span>
    </span>
  );
}
