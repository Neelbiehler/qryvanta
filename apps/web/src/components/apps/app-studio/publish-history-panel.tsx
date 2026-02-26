import { useMemo, useState } from "react";

import { Button, StatusBadge } from "@qryvanta/ui";

type PublishHistoryEntry = {
  runId: string;
  runAt: string;
  subject: string;
  requestedEntities: number;
  requestedApps: number;
  requestedEntityLogicalNames: string[];
  requestedAppLogicalNames: string[];
  publishedEntities: string[];
  validatedApps: string[];
  issueCount: number;
  isPublishable: boolean;
};

type PublishHistoryPanelProps = {
  publishHistory: PublishHistoryEntry[];
  onLoadSelection: (entityLogicalNames: string[], appLogicalNames: string[]) => void;
};

export function PublishHistoryPanel({
  publishHistory,
  onLoadSelection,
}: PublishHistoryPanelProps) {
  const [selectedHistoryRunId, setSelectedHistoryRunId] = useState<string | null>(null);

  const selectedHistoryEntry = useMemo(
    () =>
      publishHistory.find((entry) => entry.runId === selectedHistoryRunId) ??
      publishHistory[0] ??
      null,
    [publishHistory, selectedHistoryRunId],
  );

  if (publishHistory.length === 0) {
    return null;
  }

  return (
    <div className="rounded-md border border-zinc-200 bg-zinc-50 p-3">
      <p className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600">
        Publish History
      </p>
      <div className="mt-2 space-y-2">
        {publishHistory.map((entry) => (
          <button
            key={entry.runId}
            type="button"
            className={`w-full rounded border px-2 py-2 text-left text-xs ${
              selectedHistoryEntry?.runId === entry.runId
                ? "border-zinc-800 bg-white"
                : "border-zinc-200 bg-white/70"
            }`}
            onClick={() => setSelectedHistoryRunId(entry.runId)}
          >
            <div className="flex items-center justify-between gap-2">
              <span className="font-semibold text-zinc-800">
                {new Date(entry.runAt).toLocaleString()}
              </span>
              <StatusBadge tone={entry.isPublishable ? "success" : "warning"}>
                {entry.isPublishable ? "publishable" : "blocked"}
              </StatusBadge>
            </div>
            <p className="mt-1 text-zinc-600">
              {entry.requestedEntities} requested entities / {entry.requestedApps} requested apps -{" "}
              {entry.issueCount} issues - by {entry.subject}
            </p>
          </button>
        ))}
      </div>

      {selectedHistoryEntry ? (
        <div className="mt-3 rounded-md border border-zinc-200 bg-white p-3 text-xs text-zinc-700">
          <p className="font-semibold text-zinc-800">Run {selectedHistoryEntry.runId}</p>
          <p className="mt-1">
            Requested entities: {selectedHistoryEntry.requestedEntityLogicalNames.join(", ") || "none"}
          </p>
          <p className="mt-1">
            Requested apps: {selectedHistoryEntry.requestedAppLogicalNames.join(", ") || "none"}
          </p>
          <p className="mt-1">
            Published entities: {selectedHistoryEntry.publishedEntities.join(", ") || "none"}
          </p>
          <p className="mt-1">
            Validated apps: {selectedHistoryEntry.validatedApps.join(", ") || "none"}
          </p>
          <div className="mt-2 grid gap-2 md:grid-cols-2">
            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={() =>
                onLoadSelection(
                  selectedHistoryEntry.requestedEntityLogicalNames,
                  selectedHistoryEntry.requestedAppLogicalNames,
                )
              }
            >
              Load Requested Selection
            </Button>
            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={() =>
                onLoadSelection(
                  selectedHistoryEntry.publishedEntities,
                  selectedHistoryEntry.validatedApps,
                )
              }
              disabled={
                selectedHistoryEntry.publishedEntities.length === 0 &&
                selectedHistoryEntry.validatedApps.length === 0
              }
            >
              Load Published Selection
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
