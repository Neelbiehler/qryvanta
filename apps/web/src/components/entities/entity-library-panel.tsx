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

import type { EntityResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type EntityLibraryPanelProps = {
  entities: EntityResponse[];
};

type SortMode = "display_asc" | "display_desc" | "logical_asc" | "logical_desc";

export function EntityLibraryPanel({ entities }: EntityLibraryPanelProps) {
  const [query, setQuery] = useState("");
  const [sortMode, setSortMode] = useState<SortMode>("display_asc");

  const normalizedQuery = query.trim().toLowerCase();

  const filteredEntities = useMemo(() => {
    const matched = entities.filter((entity) => {
      if (!normalizedQuery) {
        return true;
      }

      const haystack = `${entity.display_name} ${entity.logical_name}`.toLowerCase();
      return haystack.includes(normalizedQuery);
    });

    const sorted = [...matched];
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
  }, [entities, normalizedQuery, sortMode]);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Entity Designer Library</CardTitle>
          <CardDescription>
            Build metadata, publish schema versions, and open runtime workbenches.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-2">
          <StatusBadge tone="neutral">
            Showing {filteredEntities.length} / {entities.length}
          </StatusBadge>
          <StatusBadge tone="neutral">Model-driven surface</StatusBadge>
          <Link
            href="/maker/entities/new"
            className={cn(buttonVariants({ variant: "default", size: "sm" }), "ml-auto")}
          >
            New Entity
          </Link>
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
            placeholder="Search display or logical name"
          />
          <Select
            value={sortMode}
            onChange={(event) => setSortMode(event.target.value as SortMode)}
          >
            <option value="display_asc">Sort: Display A-Z</option>
            <option value="display_desc">Sort: Display Z-A</option>
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

      {filteredEntities.length > 0 ? (
        <div className="grid gap-4 lg:grid-cols-2">
          {filteredEntities.map((entity) => {
            const encodedLogicalName = encodeURIComponent(entity.logical_name);

            return (
              <Card key={entity.logical_name}>
                <CardHeader>
                  <CardTitle>{entity.display_name}</CardTitle>
                  <CardDescription>
                    <span className="font-mono text-xs">{entity.logical_name}</span>
                  </CardDescription>
                </CardHeader>
                <CardContent className="flex gap-2">
                  <Link
                    href={`/maker/entities/${encodedLogicalName}`}
                    className={cn(buttonVariants({ size: "sm" }))}
                  >
                    Open Builder
                  </Link>
                  <Link
                    href={`/maker/entities/${encodedLogicalName}`}
                    className={cn(buttonVariants({ size: "sm", variant: "outline" }))}
                  >
                    Open Runtime
                  </Link>
                </CardContent>
              </Card>
            );
          })}
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>No entities match current filters</CardTitle>
            <CardDescription>Reset filters or create a new entity.</CardDescription>
          </CardHeader>
          <CardContent className="flex gap-2">
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
            <Link href="/maker/entities/new" className={cn(buttonVariants())}>
              Create Entity
            </Link>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
