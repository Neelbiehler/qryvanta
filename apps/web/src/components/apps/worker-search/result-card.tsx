import { Button, StatusBadge } from "@qryvanta/ui";
import { GitCompare } from "lucide-react";

import type { ParsedResult, SearchHit } from "./types";
import {
  collectPrimaryFields,
  collectSecondaryFields,
  formatValue,
  humanizeFieldName,
  mapStatusTone,
} from "./utils";

type SearchResultCardProps = {
  hit: SearchHit;
  parsed: ParsedResult;
  rank: number;
  compared: boolean;
  onToggleCompare: () => void;
  onTrackOpen: () => void;
};

export function SearchResultCard({
  hit,
  parsed,
  rank,
  compared,
  onToggleCompare,
  onTrackOpen,
}: SearchResultCardProps) {
  const primary = collectPrimaryFields(parsed);
  const secondary = collectSecondaryFields(parsed, primary);
  const statusTone = mapStatusTone(parsed.byKey.status);

  return (
    <article className="rounded-xl border border-zinc-200 bg-gradient-to-b from-white to-zinc-50 p-4 shadow-sm">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-base font-semibold text-zinc-900">{hit.title}</p>
          <p className="mt-1 text-xs text-zinc-500">{hit.connector_type}</p>
        </div>
        <div className="flex flex-col items-end gap-1">
          <StatusBadge tone="neutral">#{rank}</StatusBadge>
          <StatusBadge tone="info">{(hit.score * 100).toFixed(0)}% match</StatusBadge>
        </div>
      </div>

      <div className="mt-3 flex flex-wrap gap-2">
        {parsed.byKey.status ? <StatusBadge tone={statusTone}>{formatValue("status", parsed.byKey.status)}</StatusBadge> : null}
        {parsed.byKey.due_date ? <StatusBadge tone="neutral">Due {formatValue("due_date", parsed.byKey.due_date)}</StatusBadge> : null}
        {parsed.byKey.total_amount ? <StatusBadge tone="success">{formatValue("total_amount", parsed.byKey.total_amount)}</StatusBadge> : null}
        {!parsed.byKey.total_amount && parsed.byKey.amount ? <StatusBadge tone="success">{formatValue("amount", parsed.byKey.amount)}</StatusBadge> : null}
      </div>

      {primary.length > 0 ? (
        <div className="mt-3 grid gap-2 rounded-lg border border-zinc-200 bg-white p-3 text-sm md:grid-cols-2">
          {primary.map((field) => (
            <div key={field.key} className="min-w-0">
              <p className="text-[11px] uppercase tracking-wide text-zinc-500">{humanizeFieldName(field.key)}</p>
              <p className="truncate font-medium text-zinc-800">{formatValue(field.key, field.value)}</p>
            </div>
          ))}
        </div>
      ) : null}

      {secondary.length > 0 ? (
        <details className="mt-3 rounded-lg border border-zinc-200 bg-white p-3 text-xs text-zinc-600">
          <summary className="cursor-pointer font-medium text-zinc-800">More fields ({secondary.length})</summary>
          <div className="mt-2 grid gap-2 md:grid-cols-2">
            {secondary.map((field) => (
              <div key={field.key}>
                <span className="text-zinc-500">{humanizeFieldName(field.key)}:</span>{" "}
                <span>{formatValue(field.key, field.value)}</span>
              </div>
            ))}
          </div>
        </details>
      ) : null}

      <div className="mt-3 flex flex-wrap items-center gap-2">
        <Button type="button" size="sm" variant={compared ? "default" : "outline"} onClick={onToggleCompare}>
          <GitCompare className="h-4 w-4" />
          {compared ? "Selected" : "Compare"}
        </Button>
        {hit.url && hit.url !== "about:blank" ? (
          <a
            href={hit.url}
            target="_blank"
            rel="noreferrer"
            onClick={onTrackOpen}
            className="inline-flex text-xs font-medium text-sky-700 hover:text-sky-900"
          >
            Open source
          </a>
        ) : null}
      </div>
    </article>
  );
}
