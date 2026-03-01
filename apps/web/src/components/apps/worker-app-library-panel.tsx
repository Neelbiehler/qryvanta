"use client";

import * as React from "react";
import Link from "next/link";
import { useEffect, useMemo, useRef, useState } from "react";
import { AppWindow, ArrowRight, Clock, Search, X } from "lucide-react";

import {
  Button,
  EmptyState,
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

type SortMode = "display_asc" | "display_desc";

type RecentEntry = {
  logical_name: string;
  display_name: string;
  opened_at: number;
};

const RECENT_KEY = "worker_recent_apps_v1";
const MAX_RECENT = 5;

// Derived from the design system's status badge tone palette
const AVATAR_PALETTE = [
  { bg: "bg-emerald-100", text: "text-emerald-700", ring: "ring-emerald-200" },
  { bg: "bg-sky-100", text: "text-sky-700", ring: "ring-sky-200" },
  { bg: "bg-amber-100", text: "text-amber-700", ring: "ring-amber-200" },
  { bg: "bg-red-100", text: "text-red-700", ring: "ring-red-200" },
  { bg: "bg-zinc-100", text: "text-zinc-600", ring: "ring-zinc-200" },
] as const;

function getAvatarColor(logicalName: string) {
  let h = 0;
  for (let i = 0; i < logicalName.length; i++) {
    h = (h * 31 + logicalName.charCodeAt(i)) | 0;
  }
  return AVATAR_PALETTE[Math.abs(h) % AVATAR_PALETTE.length]!;
}

function getInitials(displayName: string): string {
  const words = displayName.trim().split(/\s+/);
  if (words.length === 1) return (words[0]?.[0] ?? "A").toUpperCase();
  return ((words[0]?.[0] ?? "") + (words[1]?.[0] ?? "")).toUpperCase();
}

export function WorkerAppLibraryPanel({ apps }: WorkerAppLibraryPanelProps) {
  const [query, setQuery] = useState("");
  const [sortMode, setSortMode] = useState<SortMode>("display_asc");
  const [recentEntries, setRecentEntries] = useState<RecentEntry[]>([]);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    try {
      const raw = localStorage.getItem(RECENT_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as unknown;
      if (Array.isArray(parsed)) {
        setRecentEntries(parsed as RecentEntry[]);
      }
    } catch {
      // ignore storage errors
    }
  }, []);

  function recordOpen(app: AppResponse) {
    setRecentEntries((prev) => {
      const next: RecentEntry[] = [
        { logical_name: app.logical_name, display_name: app.display_name, opened_at: Date.now() },
        ...prev.filter((r) => r.logical_name !== app.logical_name),
      ].slice(0, MAX_RECENT);
      try {
        localStorage.setItem(RECENT_KEY, JSON.stringify(next));
      } catch {
        // ignore storage errors
      }
      return next;
    });
  }

  const normalizedQuery = query.trim().toLowerCase();

  const filteredApps = useMemo(
    () =>
      apps
        .filter((app) => {
          if (!normalizedQuery) return true;
          return `${app.display_name} ${app.logical_name} ${app.description ?? ""}`
            .toLowerCase()
            .includes(normalizedQuery);
        })
        .sort((a, b) =>
          sortMode === "display_asc"
            ? a.display_name.localeCompare(b.display_name)
            : b.display_name.localeCompare(a.display_name),
        ),
    [apps, normalizedQuery, sortMode],
  );

  const recentApps = useMemo(
    () =>
      recentEntries
        .map((r) => apps.find((a) => a.logical_name === r.logical_name) ?? null)
        .filter((a): a is AppResponse => a !== null)
        .slice(0, MAX_RECENT),
    [apps, recentEntries],
  );

  const hasFilter = Boolean(query || sortMode !== "display_asc");
  const isSearching = normalizedQuery.length > 0;

  return (
    <div className="h-full overflow-y-auto bg-zinc-50">
      {/* Page header */}
      <div className="border-b border-emerald-100 bg-white">
        <div className="mx-auto max-w-5xl px-6 py-5">
          <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-emerald-600">
            Worker Portal
          </p>
          <div className="mt-1 flex flex-wrap items-end justify-between gap-3">
            <div>
              <h1 className="text-xl font-semibold text-zinc-900">My Apps</h1>
              <p className="mt-0.5 text-sm text-zinc-500">
                {apps.length === 0
                  ? "No apps are assigned to your account."
                  : `${apps.length} app${apps.length !== 1 ? "s" : ""} available`}
              </p>
            </div>
            <div className="flex items-center gap-2">
              {apps.length > 0 ? (
                <StatusBadge tone="success" dot>
                  {apps.length} assigned
                </StatusBadge>
              ) : null}
              <Link href="/worker/apps" className={buttonVariants({ size: "sm", variant: "outline" })}>
                Refresh
              </Link>
            </div>
          </div>
        </div>
      </div>

      <div className="mx-auto max-w-5xl space-y-6 px-6 py-6">
        {/* Search + sort */}
        <div className="flex flex-wrap items-center gap-2">
          <div className="relative min-w-[200px] flex-1">
            <Search
              aria-hidden="true"
              className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-zinc-400"
            />
            <Input
              ref={searchRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search apps…"
              autoComplete="off"
              spellCheck={false}
              aria-label="Search apps"
              className="h-9 pl-8 pr-8"
            />
            {query ? (
              <button
                type="button"
                aria-label="Clear search"
                className="absolute right-2 top-1/2 -translate-y-1/2 rounded p-0.5 text-zinc-400 hover:text-zinc-700"
                onClick={() => {
                  setQuery("");
                  searchRef.current?.focus();
                }}
              >
                <X aria-hidden="true" className="h-3.5 w-3.5" />
              </button>
            ) : null}
          </div>

          <Select
            value={sortMode}
            onChange={(e) => setSortMode(e.target.value as SortMode)}
            aria-label="Sort apps"
            className="h-9 w-36"
          >
            <option value="display_asc">Name A → Z</option>
            <option value="display_desc">Name Z → A</option>
          </Select>

          {hasFilter ? (
            <Button
              type="button"
              variant="outline"
              size="sm"
              className="h-9"
              onClick={() => {
                setQuery("");
                setSortMode("display_asc");
              }}
            >
              Reset
            </Button>
          ) : null}

          <StatusBadge tone="neutral">
            {filteredApps.length} / {apps.length}
          </StatusBadge>
        </div>

        {/* Recently opened */}
        {recentApps.length > 0 && !isSearching ? (
          <section aria-label="Recently opened apps">
            <div className="mb-2.5 flex items-center gap-1.5">
              <Clock aria-hidden="true" className="h-3.5 w-3.5 text-zinc-400" />
              <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
                Recently Opened
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              {recentApps.map((app) => {
                const c = getAvatarColor(app.logical_name);
                return (
                  <Link
                    key={app.logical_name}
                    href={`/worker/apps/${encodeURIComponent(app.logical_name)}`}
                    onClick={() => recordOpen(app)}
                    className="group flex items-center gap-2 rounded-lg border border-zinc-200 bg-white px-3 py-2 shadow-sm transition-shadow hover:border-emerald-200 hover:shadow-md"
                  >
                    <span
                      className={cn(
                        "flex h-6 w-6 shrink-0 items-center justify-center rounded text-[10px] font-bold ring-1",
                        c.bg,
                        c.text,
                        c.ring,
                      )}
                    >
                      {getInitials(app.display_name)}
                    </span>
                    <span className="text-xs font-medium text-zinc-700 group-hover:text-emerald-700">
                      {app.display_name}
                    </span>
                  </Link>
                );
              })}
            </div>
          </section>
        ) : null}

        {/* App grid or empty state */}
        {filteredApps.length > 0 ? (
          <section aria-label="All apps">
            {recentApps.length > 0 && !isSearching ? (
              <p className="mb-3 text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
                All Apps
              </p>
            ) : null}
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {filteredApps.map((app) => (
                <AppTile
                  key={app.logical_name}
                  app={app}
                  href={`/worker/apps/${encodeURIComponent(app.logical_name)}`}
                  onOpen={() => recordOpen(app)}
                />
              ))}
            </div>
          </section>
        ) : (
          <EmptyState
            icon={<AppWindow aria-hidden="true" />}
            title={isSearching ? "No matching apps" : "No apps assigned"}
            description={
              isSearching
                ? `No apps match "${query}".`
                : "Contact your administrator to be assigned to an app."
            }
            action={
              isSearching ? (
                <Button type="button" variant="outline" size="sm" onClick={() => setQuery("")}>
                  Clear search
                </Button>
              ) : undefined
            }
          />
        )}
      </div>
    </div>
  );
}

type AppTileProps = {
  app: AppResponse;
  href: string;
  onOpen: () => void;
};

function AppTile({ app, href, onOpen }: AppTileProps) {
  const c = getAvatarColor(app.logical_name);

  return (
    <Link
      href={href}
      onClick={onOpen}
      className="group flex flex-col rounded-xl border border-zinc-200 bg-white shadow-sm transition-all hover:border-emerald-200 hover:shadow-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500"
    >
      <div className="flex flex-1 flex-col gap-3 p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-center gap-3">
            <span
              className={cn(
                "flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-sm font-bold ring-1",
                c.bg,
                c.text,
                c.ring,
              )}
            >
              {getInitials(app.display_name)}
            </span>
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold text-zinc-900 group-hover:text-emerald-800">
                {app.display_name}
              </p>
              <p className="truncate font-mono text-[10px] text-zinc-400">{app.logical_name}</p>
            </div>
          </div>
          <ArrowRight
            aria-hidden="true"
            className="mt-1 h-4 w-4 shrink-0 text-zinc-300 transition-transform group-hover:translate-x-0.5 group-hover:text-emerald-500"
          />
        </div>

        <p
          className={cn(
            "text-xs leading-relaxed line-clamp-2",
            app.description ? "text-zinc-500" : "italic text-zinc-300",
          )}
        >
          {app.description ?? "No description available."}
        </p>
      </div>
    </Link>
  );
}
