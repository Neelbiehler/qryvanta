import type { Dispatch, SetStateAction } from "react";

import type { FacetSuggestion, ParsedField, ParsedResult, SearchHitView } from "./types";

export const QUERY_EXAMPLES = [
  "which invoices are unpaid",
  "deals closing this month",
  "contacts in negotiation accounts",
];

export const HISTORY_STORAGE_KEY = "qryvanta.worker.search.history";

export function inferGroupLabel(parsed: ParsedResult): string {
  const entity = parsed.byKey.entity;
  if (entity) {
    return entity;
  }
  if (parsed.byKey.invoice_number) {
    return "invoice";
  }
  if (parsed.byKey.stage) {
    return "pipeline";
  }
  if (parsed.byKey.email || parsed.byKey.job_title) {
    return "contacts";
  }
  return "other";
}

export function parseResultText(text: string): ParsedResult {
  const fields: ParsedField[] = [];
  const byKey: Record<string, string> = {};
  const regex = /([\p{L}\p{N}_.-]+):\s*([^:]+?)(?=\s+[\p{L}\p{N}_.-]+:\s|$)/gu;

  for (const match of text.matchAll(regex)) {
    const key = match[1]?.trim();
    const value = match[2]?.trim();
    if (!key || !value) {
      continue;
    }
    fields.push({ key, value });
    byKey[key.toLowerCase()] = value;
  }

  if (fields.length === 0 && text.trim()) {
    return {
      fields: [{ key: "content", value: text.trim() }],
      byKey,
    };
  }

  return { fields, byKey };
}

export function buildFacetSuggestions(hitViews: SearchHitView[]): FacetSuggestion[] {
  const allowedKeys = new Set(["status", "stage", "industry", "priority", "category"]);
  const counts = new Map<string, FacetSuggestion>();

  for (const { parsed } of hitViews) {
    for (const [key, value] of Object.entries(parsed.byKey)) {
      if (!allowedKeys.has(key)) {
        continue;
      }
      if (!value || value.length > 24) {
        continue;
      }
      const mapKey = `${key}::${value.toLowerCase()}`;
      const existing = counts.get(mapKey);
      if (existing) {
        existing.count += 1;
      } else {
        counts.set(mapKey, { key, value, count: 1 });
      }
    }
  }

  return [...counts.values()].sort((a, b) => b.count - a.count).slice(0, 8);
}

export function collectPrimaryFields(parsed: ParsedResult): ParsedField[] {
  const preferred = [
    "invoice_number",
    "status",
    "due_date",
    "total_amount",
    "amount",
    "stage",
    "name",
    "subject",
    "display_name",
    "email",
    "job_title",
  ];

  const byKey = parsed.fields.reduce<Record<string, ParsedField>>((acc, field) => {
    acc[field.key.toLowerCase()] = field;
    return acc;
  }, {});

  const picked: ParsedField[] = [];
  for (const key of preferred) {
    const field = byKey[key];
    if (field && !picked.some((item) => item.key === field.key)) {
      picked.push(field);
    }
  }

  for (const field of parsed.fields) {
    if (picked.length >= 6) {
      break;
    }
    if (picked.some((item) => item.key === field.key)) {
      continue;
    }
    if (field.key.toLowerCase().endsWith("_id")) {
      continue;
    }
    picked.push(field);
  }

  return picked.slice(0, 6);
}

export function collectSecondaryFields(parsed: ParsedResult, primary: ParsedField[]): ParsedField[] {
  const primaryKeys = new Set(primary.map((field) => field.key));
  return parsed.fields.filter((field) => !primaryKeys.has(field.key)).slice(0, 12);
}

export function humanizeFieldName(key: string): string {
  return key.replaceAll("_", " ").replaceAll(".", " ").replace(/\s+/g, " ").trim();
}

export function formatValue(key: string, value: string): string {
  const normalizedKey = key.toLowerCase();

  if (value === "-") {
    return value;
  }

  if (
    (normalizedKey.includes("amount") || normalizedKey.includes("revenue")) &&
    /^\d+(\.\d+)?$/.test(value)
  ) {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency: "USD",
      maximumFractionDigits: 0,
    }).format(Number(value));
  }

  if ((normalizedKey.includes("date") || normalizedKey.includes("_at")) && /^\d{4}-\d{2}-\d{2}/.test(value)) {
    const date = new Date(value);
    if (!Number.isNaN(date.getTime())) {
      return date.toLocaleDateString();
    }
  }

  if (normalizedKey.endsWith("_id") && value.length > 18) {
    return `${value.slice(0, 8)}...${value.slice(-6)}`;
  }

  return value;
}

export function mapStatusTone(status: string | undefined): "success" | "warning" | "critical" | "neutral" {
  const normalized = status?.trim().toLowerCase();
  if (!normalized) {
    return "neutral";
  }
  if (["paid", "completed", "resolved", "closed", "active"].includes(normalized)) {
    return "success";
  }
  if (["draft", "pending", "queued", "open", "sent"].includes(normalized)) {
    return "warning";
  }
  if (["overdue", "failed", "rejected", "blocked", "cancelled"].includes(normalized)) {
    return "critical";
  }
  return "neutral";
}

export function saveQueryToHistory(query: string, setRecentQueries: Dispatch<SetStateAction<string[]>>) {
  const normalized = query.trim();
  if (!normalized) {
    return;
  }

  setRecentQueries((current) => {
    const next = [normalized, ...current.filter((value) => value !== normalized)].slice(0, 8);
    try {
      localStorage.setItem(HISTORY_STORAGE_KEY, JSON.stringify(next));
    } catch {
      // no-op
    }
    return next;
  });
}
