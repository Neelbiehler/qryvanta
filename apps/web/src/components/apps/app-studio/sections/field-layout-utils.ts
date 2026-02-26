export function parseFieldLogicalNames(raw: string): string[] {
  return raw
    .split(",")
    .map((value) => value.trim())
    .filter((value) => value.length > 0);
}

export function serializeFieldLogicalNames(fields: string[]): string {
  return fields.join(", ");
}

export function appendUniqueField(fields: string[], logicalName: string): string[] {
  return fields.includes(logicalName) ? fields : [...fields, logicalName];
}

export function moveField(
  fields: string[],
  logicalName: string,
  direction: "up" | "down",
): string[] {
  const index = fields.indexOf(logicalName);
  if (index < 0) {
    return fields;
  }

  const targetIndex = direction === "up" ? index - 1 : index + 1;
  if (targetIndex < 0 || targetIndex >= fields.length) {
    return fields;
  }

  const next = [...fields];
  const [entry] = next.splice(index, 1);
  next.splice(targetIndex, 0, entry);
  return next;
}

export function buildNextLogicalName(
  existingLogicalNames: string[],
  prefix: string,
  startIndex: number,
): string {
  let nextIndex = startIndex;
  let candidate = `${prefix}_${nextIndex}`;

  while (existingLogicalNames.includes(candidate)) {
    nextIndex += 1;
    candidate = `${prefix}_${nextIndex}`;
  }

  return candidate;
}

export function normalizeSurfaceLogicalName(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "_")
    .replace(/[^a-z0-9_]/g, "");
}
