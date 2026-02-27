import type { Metadata } from "next";
import Link from "next/link";
import { cookies } from "next/headers";
import { ArrowRight, LayoutGrid } from "lucide-react";

import { EmptyState, StatusBadge, buttonVariants } from "@qryvanta/ui";

import { apiServerFetch, type AppResponse } from "@/lib/api";
import { requireSurfaceUser } from "@/lib/surface-access";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Worker Portal",
};

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

const MAX_HOME_APPS = 6;

export default async function WorkerHomePage() {
  const cookieHeader = (await cookies()).toString();

  const [user, appsResponse] = await Promise.all([
    requireSurfaceUser("worker"),
    apiServerFetch("/api/workspace/apps", cookieHeader),
  ]);

  const apps: AppResponse[] = appsResponse.ok
    ? ((await appsResponse.json()) as AppResponse[])
    : [];

  const displayedApps = apps.slice(0, MAX_HOME_APPS);
  const hasMore = apps.length > MAX_HOME_APPS;

  return (
    <div className="h-full overflow-y-auto bg-zinc-50">
      {/* Hero */}
      <div className="border-b border-emerald-100 bg-white">
        <div className="mx-auto max-w-4xl px-6 py-8">
          <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-emerald-600">
            Worker Portal
          </p>
          <h1 className="mt-1 text-2xl font-semibold text-zinc-900">
            Welcome back, {user.display_name}
          </h1>
          {user.email ? (
            <p className="mt-0.5 text-sm text-zinc-500">{user.email}</p>
          ) : null}
          <div className="mt-4 flex flex-wrap items-center gap-3">
            {apps.length > 0 ? (
              <StatusBadge tone="success" dot>
                {apps.length} app{apps.length !== 1 ? "s" : ""} assigned
              </StatusBadge>
            ) : (
              <StatusBadge tone="warning">No apps assigned</StatusBadge>
            )}
            <Link href="/worker/apps" className={buttonVariants({ size: "sm" })}>
              My Apps
            </Link>
          </div>
        </div>
      </div>

      <div className="mx-auto max-w-4xl space-y-6 px-6 py-6">
        {apps.length > 0 ? (
          <section>
            <div className="mb-3 flex items-center justify-between">
              <div className="flex items-center gap-1.5">
                <LayoutGrid aria-hidden="true" className="h-3.5 w-3.5 text-zinc-400" />
                <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
                  Your Apps
                </p>
              </div>
              {hasMore ? (
                <Link
                  href="/worker/apps"
                  className="text-xs font-medium text-emerald-600 hover:text-emerald-800"
                >
                  View all {apps.length} →
                </Link>
              ) : null}
            </div>

            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {displayedApps.map((app) => {
                const c = getAvatarColor(app.logical_name);
                return (
                  <Link
                    key={app.logical_name}
                    href={`/worker/apps/${encodeURIComponent(app.logical_name)}`}
                    className="group flex items-center gap-3 rounded-xl border border-zinc-200 bg-white p-4 shadow-sm transition-all hover:border-emerald-200 hover:shadow-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500"
                  >
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
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-semibold text-zinc-900 group-hover:text-emerald-800">
                        {app.display_name}
                      </p>
                      <p className="truncate font-mono text-[10px] text-zinc-400">
                        {app.logical_name}
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

            {hasMore ? (
              <Link
                href="/worker/apps"
                className={cn(
                  buttonVariants({ variant: "outline", size: "sm" }),
                  "mt-3 w-full justify-center",
                )}
              >
                Browse all {apps.length} apps
              </Link>
            ) : null}
          </section>
        ) : (
          <EmptyState
            title="No apps assigned"
            description="Your account has not been assigned to any app. Contact your administrator to request access."
          />
        )}
      </div>
    </div>
  );
}
