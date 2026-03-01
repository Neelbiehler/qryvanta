import Link from "next/link";
import { ArrowRight, Database, Layers, LayoutDashboard } from "lucide-react";

import { StatusBadge } from "@qryvanta/ui";

import type {
  SitemapDashboardNavigationItem,
  SitemapNavigationItem,
} from "@/components/apps/workspace-entity/helpers";
import type { AppSitemapResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

// Avatar palette derived from the design system status badge tones
const AVATAR_PALETTE = [
  { bg: "bg-emerald-100", text: "text-emerald-700", ring: "ring-emerald-200" },
  { bg: "bg-sky-100", text: "text-sky-700", ring: "ring-sky-200" },
  { bg: "bg-amber-100", text: "text-amber-700", ring: "ring-amber-200" },
  { bg: "bg-red-100", text: "text-red-700", ring: "ring-red-200" },
  { bg: "bg-zinc-100", text: "text-zinc-600", ring: "ring-zinc-200" },
] as const;

function getAvatarColor(key: string) {
  let h = 0;
  for (let i = 0; i < key.length; i++) {
    h = (h * 31 + key.charCodeAt(i)) | 0;
  }
  return AVATAR_PALETTE[Math.abs(h) % AVATAR_PALETTE.length]!;
}

function getInitials(displayName: string): string {
  const words = displayName.trim().split(/\s+/);
  if (words.length === 1) return (words[0]?.[0] ?? "A").toUpperCase();
  return ((words[0]?.[0] ?? "") + (words[1]?.[0] ?? "")).toUpperCase();
}

type WorkerAppHomePanelProps = {
  appLogicalName: string;
  sitemap: AppSitemapResponse;
  entityItems: SitemapNavigationItem[];
  dashboardItems: SitemapDashboardNavigationItem[];
};

export function WorkerAppHomePanel({
  appLogicalName,
  sitemap,
  entityItems,
  dashboardItems,
}: WorkerAppHomePanelProps) {
  const sortedAreas = [...sitemap.areas].sort((a, b) => a.position - b.position);

  return (
    <div className="space-y-6 p-4">
      {/* Summary header */}
      <div className="rounded-xl border border-emerald-100 bg-white p-5 shadow-sm">
        <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-emerald-600">
          Worker Portal
        </p>
        <h2 className="mt-1 text-xl font-semibold text-zinc-900">{appLogicalName}</h2>
        <p className="mt-0.5 font-mono text-xs text-zinc-400">
          /worker/apps/{appLogicalName}
        </p>
        <div className="mt-3 flex flex-wrap gap-2">
          {entityItems.length > 0 ? (
            <StatusBadge tone="success" dot>
              {entityItems.length} workspace{entityItems.length !== 1 ? "s" : ""}
            </StatusBadge>
          ) : null}
          {dashboardItems.length > 0 ? (
            <StatusBadge tone="info" dot>
              {dashboardItems.length} dashboard{dashboardItems.length !== 1 ? "s" : ""}
            </StatusBadge>
          ) : null}
          <StatusBadge tone="neutral">
            {sortedAreas.length} area{sortedAreas.length !== 1 ? "s" : ""}
          </StatusBadge>
        </div>
      </div>

      {/* Entity workspaces */}
      {entityItems.length > 0 ? (
        <section>
          <div className="mb-3 flex items-center gap-1.5">
            <Database aria-hidden="true" className="h-3.5 w-3.5 text-zinc-400" />
            <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
              Workspaces
            </p>
          </div>
          <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
            {entityItems.map((item) => {
              const c = getAvatarColor(item.entity_logical_name);
              const params = new URLSearchParams();
              if (item.default_view) params.set("view", item.default_view);
              if (item.default_form) params.set("form", item.default_form);
              const suffix = params.toString() ? `?${params.toString()}` : "";
              const href = `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(item.entity_logical_name)}${suffix}`;

              return (
                <Link
                  key={item.entity_logical_name}
                  href={href}
                  className="group flex items-center gap-3 rounded-lg border border-zinc-200 bg-white p-3 shadow-sm transition-all hover:border-emerald-200 hover:shadow-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500"
                >
                  <span
                    className={cn(
                      "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-sm font-bold ring-1",
                      c.bg,
                      c.text,
                      c.ring,
                    )}
                  >
                    {getInitials(item.display_name)}
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-semibold text-zinc-900 group-hover:text-emerald-800">
                      {item.display_name}
                    </p>
                    <p className="truncate font-mono text-[10px] text-zinc-400">
                      {item.entity_logical_name}
                    </p>
                  </div>
                  <ArrowRight
                    aria-hidden="true"
                    className="h-4 w-4 shrink-0 text-zinc-300 transition-transform group-hover:translate-x-0.5 group-hover:text-emerald-500"
                  />
                </Link>
              );
            })}
          </div>
        </section>
      ) : null}

      {/* Dashboards */}
      {dashboardItems.length > 0 ? (
        <section>
          <div className="mb-3 flex items-center gap-1.5">
            <LayoutDashboard aria-hidden="true" className="h-3.5 w-3.5 text-zinc-400" />
            <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
              Dashboards
            </p>
          </div>
          <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
            {dashboardItems.map((item) => {
              const c = getAvatarColor(item.dashboard_logical_name);
              const href = `/worker/apps/${encodeURIComponent(appLogicalName)}/dashboards/${encodeURIComponent(item.dashboard_logical_name)}`;

              return (
                <Link
                  key={item.dashboard_logical_name}
                  href={href}
                  className="group flex items-center gap-3 rounded-lg border border-zinc-200 bg-white p-3 shadow-sm transition-all hover:border-sky-200 hover:shadow-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-500"
                >
                  <span
                    className={cn(
                      "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-sm font-bold ring-1",
                      c.bg,
                      c.text,
                      c.ring,
                    )}
                  >
                    {getInitials(item.display_name)}
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-semibold text-zinc-900 group-hover:text-sky-700">
                      {item.display_name}
                    </p>
                    <p className="truncate font-mono text-[10px] text-zinc-400">
                      {item.dashboard_logical_name}
                    </p>
                  </div>
                  <ArrowRight
                    aria-hidden="true"
                    className="h-4 w-4 shrink-0 text-zinc-300 transition-transform group-hover:translate-x-0.5 group-hover:text-sky-400"
                  />
                </Link>
              );
            })}
          </div>
        </section>
      ) : null}

      {/* App structure */}
      {sortedAreas.length > 0 ? (
        <section>
          <div className="mb-3 flex items-center gap-1.5">
            <Layers aria-hidden="true" className="h-3.5 w-3.5 text-zinc-400" />
            <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
              App Structure
            </p>
          </div>
          <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
            {sortedAreas.map((area) => {
              const sortedGroups = [...area.groups].sort((a, b) => a.position - b.position);
              const totalItems = sortedGroups.reduce((sum, g) => sum + g.sub_areas.length, 0);

              return (
                <div
                  key={area.logical_name}
                  className="rounded-lg border border-emerald-100 bg-emerald-50/30 p-3"
                >
                  <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">
                    {area.display_name}
                  </p>
                  <div className="mt-2 space-y-1">
                    {sortedGroups.map((group) => (
                      <div
                        key={group.logical_name}
                        className="flex items-center justify-between"
                      >
                        <p className="text-xs text-zinc-600">{group.display_name}</p>
                        <StatusBadge tone="neutral">{group.sub_areas.length}</StatusBadge>
                      </div>
                    ))}
                  </div>
                  <p className="mt-2 text-[10px] text-zinc-400">
                    {sortedGroups.length} group{sortedGroups.length !== 1 ? "s" : ""} ·{" "}
                    {totalItems} item{totalItems !== 1 ? "s" : ""}
                  </p>
                </div>
              );
            })}
          </div>
        </section>
      ) : null}
    </div>
  );
}
