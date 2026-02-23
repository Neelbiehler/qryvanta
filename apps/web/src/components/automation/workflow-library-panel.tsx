"use client";

import { useMemo, useState } from "react";
import Link from "next/link";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Select,
  StatusBadge,
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
  if (status === "succeeded") {
    return "succeeded";
  }

  if (status === "dead_lettered" || status === "failed") {
    return "failed";
  }

  if (
    status === "running" ||
    status === "pending" ||
    status === "leased" ||
    status === "queued"
  ) {
    return "running";
  }

  return "other";
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
      if (category === "failed") {
        current.failedCount += 1;
      } else if (category === "succeeded") {
        current.succeededCount += 1;
      } else if (category === "running") {
        current.runningCount += 1;
      }

      const runTimestamp = Number.isFinite(Date.parse(run.started_at))
        ? Date.parse(run.started_at)
        : 0;
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
        const haystack = `${workflow.display_name} ${workflow.logical_name} ${workflow.description ?? ""}`
          .toLowerCase();
        if (!haystack.includes(normalizedQuery)) {
          return false;
        }
      }

      if (enabledFilter === "enabled" && !workflow.is_enabled) {
        return false;
      }

      if (enabledFilter === "disabled" && workflow.is_enabled) {
        return false;
      }

      if (triggerFilter !== "all" && workflow.trigger_type !== triggerFilter) {
        return false;
      }

      const metrics = metricsByWorkflow.get(workflow.logical_name) ?? DEFAULT_METRICS;
      if (lastRunFilter !== "all" && metrics.latestRunCategory !== lastRunFilter) {
        return false;
      }

      return true;
    });

    const sorted = [...base];
    sorted.sort((left, right) => {
      const leftMetrics = metricsByWorkflow.get(left.logical_name) ?? DEFAULT_METRICS;
      const rightMetrics = metricsByWorkflow.get(right.logical_name) ?? DEFAULT_METRICS;

      if (sortMode === "last_run_desc") {
        return rightMetrics.latestRunTimestamp - leftMetrics.latestRunTimestamp;
      }

      if (sortMode === "name_asc") {
        return left.display_name.localeCompare(right.display_name);
      }

      if (sortMode === "name_desc") {
        return right.display_name.localeCompare(left.display_name);
      }

      if (sortMode === "failures_desc") {
        const byFailures = rightMetrics.failedCount - leftMetrics.failedCount;
        return byFailures !== 0
          ? byFailures
          : left.display_name.localeCompare(right.display_name);
      }

      const byRuns = rightMetrics.runCount - leftMetrics.runCount;
      return byRuns !== 0
        ? byRuns
        : left.display_name.localeCompare(right.display_name);
    });

    return sorted;
  }, [
    enabledFilter,
    lastRunFilter,
    metricsByWorkflow,
    normalizedQuery,
    sortMode,
    triggerFilter,
    workflows,
  ]);

  const visibleRunSummary = useMemo(() => {
    return filteredWorkflows.reduce(
      (accumulator, workflow) => {
        const metrics = metricsByWorkflow.get(workflow.logical_name) ?? DEFAULT_METRICS;
        accumulator.totalRuns += metrics.runCount;
        accumulator.failedRuns += metrics.failedCount;
        accumulator.runningRuns += metrics.runningCount;
        accumulator.succeededRuns += metrics.succeededCount;
        return accumulator;
      },
      {
        totalRuns: 0,
        failedRuns: 0,
        runningRuns: 0,
        succeededRuns: 0,
      },
    );
  }, [filteredWorkflows, metricsByWorkflow]);

  const triggerOptions = useMemo(() => {
    const values = new Set(workflows.map((workflow) => workflow.trigger_type));
    return Array.from(values).sort();
  }, [workflows]);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Workflow Library</CardTitle>
          <CardDescription>
            Browse workflows, filter operational health, and open dedicated edit or history workspaces.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-2">
          <StatusBadge tone="neutral">
            Showing {filteredWorkflows.length} / {workflows.length}
          </StatusBadge>
          <StatusBadge tone="neutral">Runs {visibleRunSummary.totalRuns}</StatusBadge>
          <StatusBadge tone="neutral">Succeeded {visibleRunSummary.succeededRuns}</StatusBadge>
          <StatusBadge tone="neutral">Running {visibleRunSummary.runningRuns}</StatusBadge>
          <StatusBadge tone="neutral">Failed {visibleRunSummary.failedRuns}</StatusBadge>
          <Link
            href="/maker/automation/new/edit"
            className={cn(buttonVariants({ variant: "default", size: "sm" }), "ml-auto")}
          >
            New Workflow
          </Link>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Filters</CardTitle>
          <CardDescription>
            Narrow workflows by lifecycle state, trigger type, and recent run outcomes.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-2 md:grid-cols-5">
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search name or logical name"
          />
          <Select
            value={enabledFilter}
            onChange={(event) => setEnabledFilter(event.target.value as EnabledFilter)}
          >
            <option value="all">All states</option>
            <option value="enabled">Enabled</option>
            <option value="disabled">Disabled</option>
          </Select>
          <Select
            value={triggerFilter}
            onChange={(event) => setTriggerFilter(event.target.value)}
          >
            <option value="all">All triggers</option>
            {triggerOptions.map((triggerOption) => (
              <option key={triggerOption} value={triggerOption}>
                {triggerOption}
              </option>
            ))}
          </Select>
          <Select
            value={lastRunFilter}
            onChange={(event) => setLastRunFilter(event.target.value as LastRunFilter)}
          >
            <option value="all">Any last run</option>
            <option value="none">No runs</option>
            <option value="succeeded">Succeeded</option>
            <option value="running">Running</option>
            <option value="failed">Failed</option>
            <option value="other">Other</option>
          </Select>
          <div className="flex gap-2">
            <Select
              value={sortMode}
              onChange={(event) => setSortMode(event.target.value as SortMode)}
            >
              <option value="last_run_desc">Sort: Last run</option>
              <option value="name_asc">Sort: Name A-Z</option>
              <option value="name_desc">Sort: Name Z-A</option>
              <option value="failures_desc">Sort: Failures</option>
              <option value="runs_desc">Sort: Run volume</option>
            </Select>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => {
                setQuery("");
                setEnabledFilter("all");
                setTriggerFilter("all");
                setLastRunFilter("all");
                setSortMode("last_run_desc");
              }}
            >
              Reset
            </Button>
          </div>
        </CardContent>
      </Card>

      {filteredWorkflows.length > 0 ? (
        <div className="grid gap-4 lg:grid-cols-2">
          {filteredWorkflows.map((workflow) => {
            const metrics = metricsByWorkflow.get(workflow.logical_name) ?? DEFAULT_METRICS;
            const workflowHrefSafe = encodeURIComponent(workflow.logical_name);

            return (
              <Card key={workflow.logical_name}>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <span>{workflow.display_name}</span>
                    <StatusBadge tone="neutral">
                      {workflow.is_enabled ? "Enabled" : "Disabled"}
                    </StatusBadge>
                  </CardTitle>
                  <CardDescription>
                    <span className="font-mono text-xs">{workflow.logical_name}</span>
                    {workflow.description ? ` - ${workflow.description}` : ""}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  <div className="flex flex-wrap gap-2 text-xs text-zinc-600">
                    <StatusBadge tone="neutral">
                      Trigger {workflow.trigger_type}
                    </StatusBadge>
                    <StatusBadge tone="neutral">Runs {metrics.runCount}</StatusBadge>
                    <StatusBadge tone="neutral">
                      Failures {metrics.failedCount}
                    </StatusBadge>
                    <StatusBadge tone="neutral">
                      Max attempts {workflow.max_attempts}
                    </StatusBadge>
                  </div>

                  {metrics.latestRun ? (
                    <p className="text-xs text-zinc-600">
                      Last run: <span className="font-mono">{metrics.latestRun.run_id}</span> (
                      {metrics.latestRun.status}) at{" "}
                      {formatUtcDateTime(metrics.latestRun.started_at)}
                    </p>
                  ) : (
                    <p className="text-xs text-zinc-500">No runs yet for this workflow.</p>
                  )}

                  <div className="flex gap-2">
                    <Link
                      href={`/maker/automation/${workflowHrefSafe}/edit`}
                      className={cn(buttonVariants({ size: "sm" }))}
                    >
                      Edit Workflow
                    </Link>
                    <Link
                      href={`/maker/automation/${workflowHrefSafe}/history`}
                      className={cn(buttonVariants({ size: "sm", variant: "outline" }))}
                    >
                      View History
                    </Link>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>No workflows match these filters</CardTitle>
            <CardDescription>
              Reset filters or create a new workflow definition.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setQuery("");
                setEnabledFilter("all");
                setTriggerFilter("all");
                setLastRunFilter("all");
                setSortMode("last_run_desc");
              }}
            >
              Reset Filters
            </Button>
            <Link href="/maker/automation/new/edit" className={cn(buttonVariants())}>
              Create Workflow
            </Link>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
