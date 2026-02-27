import type { Metadata } from "next";
import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { StatusBadge, buttonVariants } from "@qryvanta/ui";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { WorkerAppHomePanel } from "@/components/apps/worker-app-home-panel";
import { WorkerCommandRibbon } from "@/components/apps/worker-command-ribbon";
import { WorkerSitemapSidebar } from "@/components/apps/worker-sitemap-sidebar";
import { WorkerSplitShell } from "@/components/apps/worker-split-shell";
import {
  flattenSitemapToDashboardNavigation,
  flattenSitemapToNavigation,
} from "@/components/apps/workspace-entity/helpers";
import { apiServerFetch, type AppSitemapResponse } from "@/lib/api";

type WorkerAppHomePageProps = {
  params: Promise<{
    appLogicalName: string;
  }>;
};

export async function generateMetadata({ params }: WorkerAppHomePageProps): Promise<Metadata> {
  const { appLogicalName } = await params;
  return {
    title: `${appLogicalName} — Worker Portal`,
  };
}

export default async function WorkerAppHomePage({ params }: WorkerAppHomePageProps) {
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

  const sitemap = (await navigationResponse.json()) as AppSitemapResponse;
  const entityItems = flattenSitemapToNavigation(sitemap);
  const dashboardItems = flattenSitemapToDashboardNavigation(sitemap);

  return (
    <WorkerSplitShell
      storageKey={`worker_sidebar_width_${appLogicalName}`}
      sidebar={<WorkerSitemapSidebar appLogicalName={appLogicalName} sitemap={sitemap} />}
      content={
        <div className="h-full overflow-y-auto bg-zinc-50">
          <WorkerCommandRibbon
            eyebrow="Worker Portal"
            title={appLogicalName}
            subtitle={`${entityItems.length} workspace${entityItems.length !== 1 ? "s" : ""} · ${dashboardItems.length} dashboard${dashboardItems.length !== 1 ? "s" : ""}`}
            badges={
              <StatusBadge tone="neutral">
                {sitemap.areas.length} area{sitemap.areas.length !== 1 ? "s" : ""}
              </StatusBadge>
            }
            actions={
              <Link
                href="/worker/apps"
                className={buttonVariants({ size: "sm", variant: "outline" })}
              >
                ← My Apps
              </Link>
            }
          />
          <WorkerAppHomePanel
            appLogicalName={appLogicalName}
            sitemap={sitemap}
            entityItems={entityItems}
            dashboardItems={dashboardItems}
          />
        </div>
      }
    />
  );
}
