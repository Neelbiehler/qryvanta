import nextConfig from "../next.config.mjs";

const headerEntries = await nextConfig.headers();
const globalRule = headerEntries.find((entry) => entry.source === "/:path*");

if (!globalRule) {
  throw new Error("Missing global security header rule for /:path*");
}

const headerMap = new Map(
  globalRule.headers.map((header) => [header.key, header.value]),
);

const requiredHeaders = new Map([
  ["X-Content-Type-Options", "nosniff"],
  ["X-Frame-Options", "DENY"],
  ["Referrer-Policy", "strict-origin-when-cross-origin"],
  [
    "Content-Security-Policy",
    "frame-ancestors 'none'; object-src 'none'; base-uri 'self'",
  ],
]);

for (const [key, expected] of requiredHeaders) {
  const actual = headerMap.get(key);
  if (actual !== expected) {
    throw new Error(`Expected ${key}=${expected}, received ${actual ?? "missing"}`);
  }
}

if (!headerMap.has("Permissions-Policy")) {
  throw new Error("Missing Permissions-Policy header");
}

console.log("security header config verified");
