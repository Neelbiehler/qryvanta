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
import { apiServerFetch, type WorkspaceDashboardResponse } from "@/lib/api";
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

  const dashboardResponse = await apiServerFetch(
    `/api/workspace/apps/${appLogicalName}/dashboards/${dashboardLogicalName}`,
    cookieHeader,
  );

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
              Dashboard "{dashboardLogicalName}" is not present in this app sitemap.
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

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Worker Dashboards
            </p>
            <CardTitle className="font-serif text-3xl">{dashboard.display_name}</CardTitle>
            <CardDescription>
              Baseline metadata-driven dashboard widgets derived from app bindings.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Widgets {dashboard.widgets.length}</StatusBadge>
            <Link
              href={`/worker/apps/${appLogicalName}`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to app
            </Link>
          </div>
        </CardHeader>
      </Card>

      <Card className="border-zinc-200 bg-white">
        <CardHeader>
          <CardTitle>Dashboard Widgets</CardTitle>
          <CardDescription>
            Chart metadata is available now; data-query rendering is the next phase.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          {dashboard.widgets.length > 0 ? (
            dashboard.widgets.map((widget) => (
              <div key={widget.logical_name} className="rounded-md border border-zinc-200 p-3">
                <p className="text-sm font-semibold text-zinc-900">{widget.display_name}</p>
                <p className="mt-1 font-mono text-[11px] text-zinc-500">
                  {widget.chart.logical_name}
                </p>
                <div className="mt-2 space-y-1 text-xs text-zinc-600">
                  <p>Entity: {widget.chart.entity_logical_name}</p>
                  <p>Chart Type: {widget.chart.chart_type}</p>
                  <p>Aggregation: {widget.chart.aggregation}</p>
                  <p>
                    View: {widget.chart.view_logical_name ?? "(default binding view)"}
                  </p>
                </div>
              </div>
            ))
          ) : (
            <p className="text-sm text-zinc-500">
              No widgets are available yet. Add app entity bindings to generate baseline KPI cards.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
