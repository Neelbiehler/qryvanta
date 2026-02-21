"use client";

import { useState } from "react";

import { Button, Input, Label } from "@qryvanta/ui";

import {
  apiFetch,
  type AuditLogEntryResponse,
  type AuditPurgeResultResponse,
  type UpdateAuditRetentionPolicyRequest,
} from "@/lib/api";

type AuditControlsPanelProps = {
  queryString: string;
  retentionDays: number | null;
};

export function AuditControlsPanel({
  queryString,
  retentionDays,
}: AuditControlsPanelProps) {
  const [retentionDaysValue, setRetentionDaysValue] = useState(
    retentionDays ? String(retentionDays) : "",
  );
  const [lastPurgeResult, setLastPurgeResult] =
    useState<AuditPurgeResultResponse | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [isUpdatingRetention, setIsUpdatingRetention] = useState(false);
  const [isPurging, setIsPurging] = useState(false);

  async function handleExport() {
    setErrorMessage(null);
    setIsExporting(true);

    try {
      const response = await apiFetch(
        `/api/security/audit-log/export?${queryString}`,
      );
      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to export audit log.");
        return;
      }

      const entries = (await response.json()) as AuditLogEntryResponse[];
      const blob = new Blob([JSON.stringify(entries, null, 2)], {
        type: "application/json",
      });
      const downloadUrl = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = downloadUrl;
      link.download = `audit-log-${new Date().toISOString()}.json`;
      link.click();
      URL.revokeObjectURL(downloadUrl);
    } catch {
      setErrorMessage("Unable to export audit log.");
    } finally {
      setIsExporting(false);
    }
  }

  async function handleRetentionSave() {
    setErrorMessage(null);
    const parsedRetentionDays = Number.parseInt(retentionDaysValue, 10);
    if (Number.isNaN(parsedRetentionDays) || parsedRetentionDays <= 0) {
      setErrorMessage("Retention days must be a positive number.");
      return;
    }

    setIsUpdatingRetention(true);
    try {
      const payload: UpdateAuditRetentionPolicyRequest = {
        retention_days: parsedRetentionDays,
      };

      const response = await apiFetch("/api/security/audit-retention-policy", {
        method: "PUT",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(
          payload.message ?? "Unable to update retention policy.",
        );
      }
    } catch {
      setErrorMessage("Unable to update retention policy.");
    } finally {
      setIsUpdatingRetention(false);
    }
  }

  async function handlePurge() {
    setErrorMessage(null);
    setLastPurgeResult(null);
    setIsPurging(true);

    try {
      const response = await apiFetch("/api/security/audit-log/purge", {
        method: "POST",
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to purge audit entries.");
        return;
      }

      const result = (await response.json()) as AuditPurgeResultResponse;
      setLastPurgeResult(result);
    } catch {
      setErrorMessage("Unable to purge audit entries.");
    } finally {
      setIsPurging(false);
    }
  }

  return (
    <div className="space-y-3 rounded-md border border-emerald-100 bg-emerald-50/50 p-3">
      <div className="flex flex-wrap gap-2">
        <Button disabled={isExporting} onClick={handleExport} type="button">
          {isExporting ? "Exporting..." : "Export Filtered Results"}
        </Button>
      </div>

      {retentionDays !== null ? (
        <div className="grid gap-3 md:grid-cols-3 md:items-end">
          <div className="space-y-2">
            <Label htmlFor="audit_retention_days">Retention days</Label>
            <Input
              id="audit_retention_days"
              type="number"
              value={retentionDaysValue}
              onChange={(event) => setRetentionDaysValue(event.target.value)}
            />
          </div>
          <Button
            disabled={isUpdatingRetention}
            onClick={handleRetentionSave}
            type="button"
            variant="outline"
          >
            {isUpdatingRetention ? "Saving..." : "Save Retention"}
          </Button>
          <Button
            disabled={isPurging}
            onClick={handlePurge}
            type="button"
            variant="outline"
          >
            {isPurging ? "Purging..." : "Purge Old Entries"}
          </Button>
        </div>
      ) : (
        <p className="text-sm text-zinc-600">
          Audit retention and purge controls require role management permission.
        </p>
      )}

      {lastPurgeResult ? (
        <p className="text-sm text-zinc-700">
          Purged {lastPurgeResult.deleted_count} entries older than{" "}
          {lastPurgeResult.retention_days} day(s).
        </p>
      ) : null}

      {errorMessage ? (
        <p className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {errorMessage}
        </p>
      ) : null}
    </div>
  );
}
