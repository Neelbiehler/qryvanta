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

import type { AppResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type WorkerAppLibraryPanelProps = {
  apps: AppResponse[];
};

type SortMode = "display_asc" | "display_desc" | "logical_asc" | "logical_desc";

export function WorkerAppLibraryPanel({ apps }: WorkerAppLibraryPanelProps) {
  const [query, setQuery] = useState("");
  const [sortMode, setSortMode] = useState<SortMode>("display_asc");

  const normalizedQuery = query.trim().toLowerCase();

  const filteredApps = useMemo(() => {
    const matchedApps = apps.filter((app) => {
      if (!normalizedQuery) {
        return true;
      }

      const haystack = `${app.display_name} ${app.logical_name} ${app.description ?? ""}`.toLowerCase();
      return haystack.includes(normalizedQuery);
    });

    const sorted = [...matchedApps];
    sorted.sort((left, right) => {
      if (sortMode === "display_asc") {
        return left.display_name.localeCompare(right.display_name);
      }
      if (sortMode === "display_desc") {
        return right.display_name.localeCompare(left.display_name);
      }
      if (sortMode === "logical_asc") {
        return left.logical_name.localeCompare(right.logical_name);
      }
      return right.logical_name.localeCompare(left.logical_name);
    });

    return sorted;
  }, [apps, normalizedQuery, sortMode]);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Dynamics-style App Hub</CardTitle>
          <CardDescription>
            Open business apps, navigate to entity areas, and execute daily operations.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-2">
          <StatusBadge tone="neutral">
            Showing {filteredApps.length} / {apps.length}
          </StatusBadge>
          <StatusBadge tone="neutral">Worker surface</StatusBadge>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Filter and Sort</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-2 md:grid-cols-[1fr_240px_auto]">
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search app name or description"
          />
          <Select
            value={sortMode}
            onChange={(event) => setSortMode(event.target.value as SortMode)}
          >
            <option value="display_asc">Sort: Name A-Z</option>
            <option value="display_desc">Sort: Name Z-A</option>
            <option value="logical_asc">Sort: Logical A-Z</option>
            <option value="logical_desc">Sort: Logical Z-A</option>
          </Select>
          <Button
            type="button"
            variant="outline"
            onClick={() => {
              setQuery("");
              setSortMode("display_asc");
            }}
          >
            Reset
          </Button>
        </CardContent>
      </Card>

      {filteredApps.length > 0 ? (
        <div className="grid gap-4 lg:grid-cols-2">
          {filteredApps.map((app) => {
            const encodedLogicalName = encodeURIComponent(app.logical_name);

            return (
              <Card key={app.logical_name}>
                <CardHeader>
                  <CardTitle>{app.display_name}</CardTitle>
                  <CardDescription>
                    <span className="font-mono text-xs">{app.logical_name}</span>
                    {app.description ? ` - ${app.description}` : ""}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Link
                    href={`/worker/apps/${encodedLogicalName}`}
                    className={cn(buttonVariants({ size: "sm" }))}
                  >
                    Open Workspace
                  </Link>
                </CardContent>
              </Card>
            );
          })}
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>No apps match current filters</CardTitle>
            <CardDescription>Try a broader search query.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setQuery("");
                setSortMode("display_asc");
              }}
            >
              Reset Filters
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
