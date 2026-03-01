"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { CheckCircle, Circle, Clock, XCircle, Loader, AlertCircle, ChevronRight } from "lucide-react";

import {
  Button,
  Input,
  Select,
  buttonVariants,
} from "@qryvanta/ui";

import type { WorkflowResponse, WorkflowRunResponse } from "@/lib/api";
import { formatUtcDateTime } from "@/lib/date-format";
import { cn } from "@/lib/utils";

type WorkflowLibraryPanelProps = {
  workflows: WorkflowResponse[];
  runs: WorkflowRunResponse[];
};

type EnabledFilter = "all" | "enabled" | "disabled";
type LastRunFilter = "all" | "none" | "succeeded" | "running" | "failed" | "other";
type SortMode =
  | "last_run_desc"
  | "name_asc"
  | "name_desc"
  | "failures_desc"
  | "runs_desc";

type LastRunCategory = Exclude<LastRunFilter, "all">;

type WorkflowMetrics = {
  runCount: number;
  failedCount: number;
  succeededCount: number;
  runningCount: number;
  latestRun: WorkflowRunResponse | null;
  latestRunTimestamp: number;
  latestRunCategory: LastRunCategory;
};

const DEFAULT_METRICS: WorkflowMetrics = {
  runCount: 0,
  failedCount: 0,
  succeededCount: 0,
  runningCount: 0,
  latestRun: null,
  latestRunTimestamp: 0,
  latestRunCategory: "none",
};

function classifyRunCategory(status: string): LastRunCategory {
  if (status === "succeeded") return "succeeded";
  if (status === "dead_lettered" || status === "failed") return "failed";
  if (status === "running" || status === "pending" || status === "leased" || status === "queued") return "running";
  return "other";
}

function RunStatusIcon({ category }: { category: LastRunCategory }) {
  if (category === "succeeded") return <CheckCircle className="size-3.5 text-emerald-500" />;
  if (category === "failed") return <XCircle className="size-3.5 text-red-500" />;
  if (category === "running") return <Loader className="size-3.5 animate-spin text-blue-500" />;
  if (category === "none") return <Circle className="size-3.5 text-zinc-300" />;
  return <AlertCircle className="size-3.5 text-amber-500" />;
}

export function WorkflowLibraryPanel({ workflows, runs }: WorkflowLibraryPanelProps) {
  const [query, setQuery] = useState("");
  const [enabledFilter, setEnabledFilter] = useState<EnabledFilter>("all");
  const [triggerFilter, setTriggerFilter] = useState("all");
  const [lastRunFilter, setLastRunFilter] = useState<LastRunFilter>("all");
  const [sortMode, setSortMode] = useState<SortMode>("last_run_desc");

  const normalizedQuery = query.trim().toLowerCase();

  const metricsByWorkflow = useMemo(() => {
    const result = new Map<string, WorkflowMetrics>();

    for (const workflow of workflows) {
      result.set(workflow.logical_name, { ...DEFAULT_METRICS });
    }

    for (const run of runs) {
      const current = result.get(run.workflow_logical_name) ?? { ...DEFAULT_METRICS };
      const category = classifyRunCategory(run.status);

      current.runCount += 1;
      if (category === "failed") current.failedCount += 1;
      else if (category === "succeeded") current.succeededCount += 1;
      else if (category === "running") current.runningCount += 1;

      const runTimestamp = Number.isFinite(Date.parse(run.started_at)) ? Date.parse(run.started_at) : 0;
      if (runTimestamp >= current.latestRunTimestamp) {
        current.latestRun = run;
        current.latestRunTimestamp = runTimestamp;
        current.latestRunCategory = category;
      }

      result.set(run.workflow_logical_name, current);
    }

    return result;
  }, [runs, workflows]);

  const filteredWorkflows = useMemo(() => {
    const base = workflows.filter((workflow) => {
      if (normalizedQuery.length > 0) {
        const haystack = `${workflow.display_name} ${workflow.logical_name} ${workflow.description ?? ""}`.toLowerCase();
        if (!haystack.includes(normalizedQuery)) return false;
      }

      if (enabledFilter === "enabled" && !workflow.is_enabled) return false;
      if (enabledFilter === "disabled" && workflow.is_enabled) return false;
      if (triggerFilter !== "all" && workflow.trigger_type !== triggerFilter) return false;

      const metrics = metricsByWorkflow.get(workflow.logical_name) ?? DEFAULT_METRICS;
      if (lastRunFilter !== "all" && metrics.latestRunCategory !== lastRunFilter) return false;

      return true;
    });

    const sorted = [...base];
    sorted.sort((left, right) => {
      const lm = metricsByWorkflow.get(left.logical_name) ?? DEFAULT_METRICS;
      const rm = metricsByWorkflow.get(right.logical_name) ?? DEFAULT_METRICS;

      if (sortMode === "last_run_desc") return rm.latestRunTimestamp - lm.latestRunTimestamp;
      if (sortMode === "name_asc") return left.display_name.localeCompare(right.display_name);
      if (sortMode === "name_desc") return right.display_name.localeCompare(left.display_name);
      if (sortMode === "failures_desc") {
        const d = rm.failedCount - lm.failedCount;
        return d !== 0 ? d : left.display_name.localeCompare(right.display_name);
      }
      const d = rm.runCount - lm.runCount;
      return d !== 0 ? d : left.display_name.localeCompare(right.display_name);
    });

    return sorted;
  }, [enabledFilter, lastRunFilter, metricsByWorkflow, normalizedQuery, sortMode, triggerFilter, workflows]);

  const triggerOptions = useMemo(() => {
    const values = new Set(workflows.map((w) => w.trigger_type));
    return Array.from(values).sort();
  }, [workflows]);

  const totalFailures = useMemo(() => {
    return filteredWorkflows.reduce((sum, w) => {
      return sum + (metricsByWorkflow.get(w.logical_name)?.failedCount ?? 0);
    }, 0);
  }, [filteredWorkflows, metricsByWorkflow]);

  function resetFilters() {
    setQuery("");
    setEnabledFilter("all");
    setTriggerFilter("all");
    setLastRunFilter("all");
    setSortMode("last_run_desc");
  }

  return (
    <div className="space-y-4">
      {/* Header bar */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-zinc-900">Automation</h1>
          <p className="text-sm text-zinc-500">
            {filteredWorkflows.length} of {workflows.length} flows
            {totalFailures > 0 && (
              <span className="ml-2 font-medium text-red-600">{totalFailures} failure{totalFailures !== 1 ? "s" : ""}</span>
            )}
          </p>
        </div>
        <Link
          href="/maker/automation/new/edit"
          className={cn(buttonVariants({ variant: "default", size: "sm" }))}
        >
          New Workflow
        </Link>
      </div>

      {/* Filter bar */}
      <div className="flex flex-wrap items-center gap-2 rounded-lg border border-zinc-200 bg-white p-3">
        <Input
          className="h-8 w-48 text-xs"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search flows..."
        />
        <Select
          value={enabledFilter}
          onChange={(e) => setEnabledFilter(e.target.value as EnabledFilter)}
        >
          <option value="all">All states</option>
          <option value="enabled">Enabled</option>
          <option value="disabled">Disabled</option>
        </Select>
        <Select
          value={triggerFilter}
          onChange={(e) => setTriggerFilter(e.target.value)}
        >
          <option value="all">All triggers</option>
          {triggerOptions.map((t) => (
            <option key={t} value={t}>{t}</option>
          ))}
        </Select>
        <Select
          value={lastRunFilter}
          onChange={(e) => setLastRunFilter(e.target.value as LastRunFilter)}
        >
          <option value="all">Any last run</option>
          <option value="none">No runs</option>
          <option value="succeeded">Succeeded</option>
          <option value="running">Running</option>
          <option value="failed">Failed</option>
          <option value="other">Other</option>
        </Select>
        <Select
          value={sortMode}
          onChange={(e) => setSortMode(e.target.value as SortMode)}
        >
          <option value="last_run_desc">Last run</option>
          <option value="name_asc">Name A-Z</option>
          <option value="name_desc">Name Z-A</option>
          <option value="failures_desc">Most failures</option>
          <option value="runs_desc">Most runs</option>
        </Select>
        <Button type="button" variant="outline" size="sm" onClick={resetFilters}>
          Reset
        </Button>
      </div>

      {/* Workflows list */}
      {filteredWorkflows.length > 0 ? (
        <div className="overflow-hidden rounded-lg border border-zinc-200 bg-white">
          {/* Column headers */}
          <div className="grid grid-cols-[1fr_auto_auto_auto_auto] items-center gap-x-4 border-b border-zinc-100 bg-zinc-50 px-4 py-2">
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Flow</p>
            <p className="w-24 text-right text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Last Run</p>
            <p className="w-20 text-right text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Runs</p>
            <p className="w-20 text-right text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Failures</p>
            <p className="w-28 text-right text-[11px] font-semibold uppercase tracking-wide text-zinc-500">Actions</p>
          </div>

          {filteredWorkflows.map((workflow, index) => {
            const metrics = metricsByWorkflow.get(workflow.logical_name) ?? DEFAULT_METRICS;
            const workflowHrefSafe = encodeURIComponent(workflow.logical_name);
            const isLast = index === filteredWorkflows.length - 1;

            return (
              <div
                key={workflow.logical_name}
                className={cn(
                  "grid grid-cols-[1fr_auto_auto_auto_auto] items-center gap-x-4 px-4 py-3 transition hover:bg-zinc-50",
                  !isLast && "border-b border-zinc-100",
                )}
              >
                {/* Flow info */}
                <div className="flex min-w-0 items-center gap-3">
                  <RunStatusIcon category={metrics.latestRunCategory} />
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="truncate text-sm font-medium text-zinc-900">{workflow.display_name}</p>
                      <span className={cn(
                        "shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-semibold",
                        workflow.is_enabled
                          ? "bg-emerald-100 text-emerald-700"
                          : "bg-zinc-100 text-zinc-500",
                      )}>
                        {workflow.is_enabled ? "On" : "Off"}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-[11px] text-zinc-400">{workflow.logical_name}</span>
                      <span className="text-[11px] text-zinc-400">·</span>
                      <span className="text-[11px] text-zinc-400">{workflow.trigger_type}</span>
                    </div>
                  </div>
                </div>

                {/* Last run time */}
                <div className="w-24 text-right">
                  {metrics.latestRun ? (
                    <span className="text-xs text-zinc-500">
                      {formatUtcDateTime(metrics.latestRun.started_at).slice(0, 16)}
                    </span>
                  ) : (
                    <span className="flex items-center justify-end gap-1 text-xs text-zinc-300">
                      <Clock className="size-3" />
                      Never
                    </span>
                  )}
                </div>

                {/* Run count */}
                <div className="w-20 text-right">
                  <span className="text-xs tabular-nums text-zinc-600">{metrics.runCount}</span>
                </div>

                {/* Failure count */}
                <div className="w-20 text-right">
                  <span className={cn(
                    "text-xs tabular-nums",
                    metrics.failedCount > 0 ? "font-semibold text-red-600" : "text-zinc-400",
                  )}>
                    {metrics.failedCount}
                  </span>
                </div>

                {/* Actions */}
                <div className="flex w-28 items-center justify-end gap-1">
                  <Link
                    href={`/maker/automation/${workflowHrefSafe}/edit`}
                    className={cn(buttonVariants({ size: "sm", variant: "outline" }), "h-7 px-2 text-xs")}
                  >
                    Edit
                  </Link>
                  <Link
                    href={`/maker/automation/${workflowHrefSafe}/history`}
                    className={cn(buttonVariants({ size: "sm", variant: "outline" }), "h-7 px-2 text-xs")}
                  >
                    <ChevronRight className="size-3.5" />
                  </Link>
                </div>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="rounded-lg border border-zinc-200 bg-white px-6 py-10 text-center">
          <p className="text-sm font-medium text-zinc-600">No flows match these filters</p>
          <p className="mt-1 text-xs text-zinc-400">Reset filters or create a new workflow.</p>
          <div className="mt-4 flex justify-center gap-2">
            <Button type="button" variant="outline" size="sm" onClick={resetFilters}>
              Reset Filters
            </Button>
            <Link href="/maker/automation/new/edit" className={cn(buttonVariants({ size: "sm" }))}>
              Create Workflow
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}
