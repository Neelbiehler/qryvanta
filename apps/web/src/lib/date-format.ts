export function formatUtcDateTime(value: string): string {
  const timestamp = Date.parse(value);
  if (!Number.isFinite(timestamp)) {
    return value;
  }

  const iso = new Date(timestamp).toISOString();
  return `${iso.slice(0, 19)} UTC`;
}
