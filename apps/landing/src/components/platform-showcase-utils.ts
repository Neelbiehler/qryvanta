import type { CSSProperties } from "react";

export function frameStyle(
  index: number,
  accent: string,
  glow: string,
): CSSProperties {
  return {
    "--frame-index": index,
    "--frame-accent": accent,
    "--frame-glow": glow,
  } as CSSProperties;
}

export function revealStyle(delayMs: number): CSSProperties {
  return {
    "--reveal-delay": `${delayMs}ms`,
  } as CSSProperties;
}

export function wordStyle(index: number): CSSProperties {
  return {
    "--word-index": index,
  } as CSSProperties;
}

export function cx(...parts: Array<string | null | false | undefined>) {
  return parts.filter(Boolean).join(" ");
}
