"use client";

type ProtectedEmailLinkProps = {
  localPart: readonly number[];
  domain: readonly number[];
  tld: readonly number[];
  className?: string;
  fallback?: string;
};

function decodeAsciiCodes(value: readonly number[]): string {
  return String.fromCharCode(...value);
}

export function ProtectedEmailLink({
  localPart,
  domain,
  tld,
  className,
  fallback = "Enable JavaScript to view email address.",
}: ProtectedEmailLinkProps) {
  const email = `${decodeAsciiCodes(localPart)}@${decodeAsciiCodes(domain)}.${decodeAsciiCodes(tld)}`;
  const isClient = typeof window !== "undefined";

  return (
    <a
      href={isClient ? `mailto:${email}` : undefined}
      className={className}
      suppressHydrationWarning
    >
      {isClient ? email : fallback}
    </a>
  );
}
