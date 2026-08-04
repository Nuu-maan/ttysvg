import { cn } from "@/lib/utils";

type LogoProps = {
  className?: string;
  title?: string;
};

export function Logo({ className, title }: LogoProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden={title ? undefined : true}
      role={title ? "img" : undefined}
      className={cn("size-6", className)}
    >
      {title ? <title>{title}</title> : null}
      <rect x="4" y="3" width="9" height="18" rx="1.5" fill="currentColor" />
      <path
        d="M18.93 8.58 17.07 15.43"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        opacity="0.6"
      />
      <circle cx="19.5" cy="6.5" r="1.75" stroke="currentColor" strokeWidth="1.5" />
      <circle cx="16.5" cy="17.5" r="1.75" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}
