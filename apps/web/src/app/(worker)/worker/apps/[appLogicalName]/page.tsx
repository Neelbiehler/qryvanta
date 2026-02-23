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
import { apiServerFetch, type AppEntityBindingResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type WorkerAppHomePageProps = {
  params: Promise<{
    appLogicalName: string;
  }>;
};

export default async function WorkerAppHomePage({
  params,
}: WorkerAppHomePageProps) {
  const { appLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();
  const navigationResponse = await apiServerFetch(
    `/api/workspace/apps/${appLogicalName}/navigation`,
    cookieHeader,
  );

  if (navigationResponse.status === 401) {
    redirect("/login");
  }

  if (navigationResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Worker Apps"
        title="App Access"
        message="Your account does not have access to this app."
      />
    );
  }

  if (!navigationResponse.ok) {
    throw new Error("Failed to load app navigation");
  }

  const navigation =
    (await navigationResponse.json()) as AppEntityBindingResponse[];

  const sortedNavigation = [...navigation].sort(
    (left, right) => left.navigation_order - right.navigation_order,
  );

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Worker Apps
            </p>
            <CardTitle className="font-serif text-3xl">{appLogicalName}</CardTitle>
            <CardDescription>
              Model-driven sitemap for this business app. Pick an entity workspace to begin operations.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Areas {sortedNavigation.length}</StatusBadge>
            <Link
              href="/worker/apps"
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to apps
            </Link>
          </div>
        </CardHeader>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[300px_1fr]">
        <Card className="h-fit border-zinc-200 bg-zinc-50">
          <CardHeader>
            <CardTitle className="text-base">Sitemap</CardTitle>
            <CardDescription>Entity areas ordered for operator workflow.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {sortedNavigation.length > 0 ? (
              sortedNavigation.map((item) => (
                <Link
                  key={`${item.app_logical_name}.${item.entity_logical_name}`}
                  href={`/worker/apps/${appLogicalName}/${item.entity_logical_name}`}
                  className="block rounded-md border border-zinc-200 bg-white px-3 py-2 text-sm transition hover:border-emerald-300"
                >
                  <p className="font-medium text-zinc-900">
                    {item.navigation_label ?? item.entity_logical_name}
                  </p>
                  <p className="font-mono text-[11px] text-zinc-500">
                    {item.entity_logical_name}
                  </p>
                </Link>
              ))
            ) : (
              <p className="text-xs text-zinc-500">No entities configured yet.</p>
            )}
          </CardContent>
        </Card>

        <Card className="border-zinc-200 bg-white">
          <CardHeader>
            <CardTitle>Entity Work Areas</CardTitle>
            <CardDescription>
              Open a workspace to view records, create data, and run daily business processes.
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3 md:grid-cols-2">
            {sortedNavigation.length > 0 ? (
              sortedNavigation.map((item) => (
                <div
                  key={`${item.app_logical_name}.${item.entity_logical_name}.card`}
                  className="rounded-md border border-zinc-200 p-3"
                >
                  <p className="text-sm font-semibold text-zinc-900">
                    {item.navigation_label ?? item.entity_logical_name}
                  </p>
                  <p className="font-mono text-[11px] text-zinc-500">
                    {item.entity_logical_name}
                  </p>
                  <p className="mt-1 text-xs text-zinc-600">
                    Order {item.navigation_order} - default {item.default_view_mode.toUpperCase()} view
                  </p>
                  <Link
                    href={`/worker/apps/${appLogicalName}/${item.entity_logical_name}`}
                    className={cn(buttonVariants({ size: "sm", variant: "outline" }), "mt-3")}
                  >
                    Open Workspace
                  </Link>
                </div>
              ))
            ) : (
              <p className="text-sm text-zinc-500">No entities are configured for this app yet.</p>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="flex justify-end">
        <Link
          href="/worker/apps"
          className={cn(buttonVariants({ variant: "outline" }))}
        >
          Return to app catalog
        </Link>
      </div>
    </div>
  );
}
