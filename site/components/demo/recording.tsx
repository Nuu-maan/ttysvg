/* eslint-disable @next/next/no-img-element */
import { cn } from "@/lib/utils";

export function Recording({
  src,
  width,
  height,
  alt,
  className,
  eager = false,
}: {
  src: string;
  width: number;
  height: number;
  alt: string;
  className?: string;
  eager?: boolean;
}) {
  return (
    <img
      src={src}
      width={width}
      height={height}
      alt={alt}
      loading={eager ? "eager" : "lazy"}
      decoding="async"
      className={cn("h-auto w-full rounded-xl bg-card", className)}
    />
  );
}
