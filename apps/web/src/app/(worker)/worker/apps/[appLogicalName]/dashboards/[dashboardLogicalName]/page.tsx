import type { Metadata } from "next";
import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  StatusBadge,
  buttonVariants,
} from "@qryvanta/ui";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { WorkerCommandRibbon } from "@/components/apps/worker-command-ribbon";
import { WorkerSitemapSidebar } from "@/components/apps/worker-sitemap-sidebar";
import { WorkerSplitShell } from "@/components/apps/worker-split-shell";
import {
  apiServerFetch,
  type AppSitemapResponse,
  type WorkspaceDashboardResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Worker Dashboard",
  description: "Metadata-driven dashboard baseline for worker apps.",
};

type WorkerDashboardPageProps = {
  params: Promise<{
    appLogicalName: string;
    dashboardLogicalName: string;
  }>;
};

export default async function WorkerDashboardPage({
  params,
}: WorkerDashboardPageProps) {
  const { appLogicalName, dashboardLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const [dashboardResponse, navigationResponse] = await Promise.all([
    apiServerFetch(
      `/api/workspace/apps/${appLogicalName}/dashboards/${dashboardLogicalName}`,
      cookieHeader,
    ),
    apiServerFetch(`/api/workspace/apps/${appLogicalName}/navigation`, cookieHeader),
  ]);

  if (dashboardResponse.status === 401) {
    redirect("/login");
  }

  if (dashboardResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Worker Apps"
        title="Dashboard Access"
        message="Your account does not have access to this app dashboard."
      />
    );
  }

  if (dashboardResponse.status === 404) {
    return (
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle className="font-serif text-3xl">Dashboard Not Found</CardTitle>
            <CardDescription>
              {`Dashboard "${dashboardLogicalName}" is not present in this app sitemap.`}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href={`/worker/apps/${appLogicalName}`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to app hub
            </Link>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!dashboardResponse.ok) {
    throw new Error("Failed to load workspace dashboard");
  }

  const dashboard = (await dashboardResponse.json()) as WorkspaceDashboardResponse;
  const sitemap = navigationResponse.ok
    ? ((await navigationResponse.json()) as AppSitemapResponse)
    : null;

  return (
    <WorkerSplitShell
      storageKey={`worker_sidebar_width_${appLogicalName}`}
      sidebar={
        sitemap ? (
          <WorkerSitemapSidebar
            appLogicalName={appLogicalName}
            sitemap={sitemap}
            activeDashboardLogicalName={dashboardLogicalName}
          />
        ) : (
          <Card className="h-fit border-zinc-200 bg-zinc-50">
            <CardHeader>
              <CardTitle className="text-base">Sitemap</CardTitle>
              <CardDescription>Navigation unavailable for this app.</CardDescription>
            </CardHeader>
          </Card>
        )
      }
      content={<div className="min-h-0 overflow-y-auto bg-zinc-50">
        <WorkerCommandRibbon
          title={dashboard.display_name}
          subtitle={`/${appLogicalName}/dashboards/${dashboardLogicalName}`}
          badges={<StatusBadge tone="neutral">Widgets {dashboard.widgets.length}</StatusBadge>}
          actions={
            <>
              <Link
                href={`/worker/apps/${encodeURIComponent(appLogicalName)}/dashboards/${encodeURIComponent(dashboardLogicalName)}`}
                className={buttonVariants({ size: "sm", variant: "outline" })}
              >
                Refresh
              </Link>
              <Link
                href={`/worker/apps/${appLogicalName}`}
                className={buttonVariants({ size: "sm", variant: "outline" })}
              >
                Back to App
              </Link>
            </>
          }
        />

        <Card className="m-4 shadow-sm">
          <CardHeader>
            <CardTitle>Dashboard Widgets</CardTitle>
            <CardDescription>
              Chart metadata is available now; data-query rendering is the next phase.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
            {dashboard.widgets.length > 0 ? (
              dashboard.widgets.map((widget) => (
                <div key={widget.logical_name} className="rounded-lg border border-emerald-100 bg-emerald-50/30 p-3">
                  <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">Widget</p>
                  <p className="mt-0.5 text-sm font-semibold text-zinc-900">{widget.display_name}</p>
                  <p className="font-mono text-[10px] text-zinc-400">{widget.chart.logical_name}</p>
                  <div className="mt-2 space-y-0.5 text-xs text-zinc-600">
                    <p>Entity: <span className="font-medium text-zinc-800">{widget.chart.entity_logical_name}</span></p>
                    <p>Type: <span className="font-medium text-zinc-800">{widget.chart.chart_type}</span></p>
                    <p>Aggregation: <span className="font-medium text-zinc-800">{widget.chart.aggregation}</span></p>
                    <p>View: <span className="font-medium text-zinc-800">{widget.chart.view_logical_name ?? "(default)"}</span></p>
                  </div>
                </div>
              ))
            ) : (
              <p className="col-span-full text-sm text-zinc-500">
                No widgets configured yet. Add app entity bindings to generate KPI cards.
              </p>
            )}
          </CardContent>
        </Card>
      </div>}
    />
  );
}
