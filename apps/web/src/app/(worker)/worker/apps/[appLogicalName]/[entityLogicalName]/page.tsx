import type { Metadata } from "next";
import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  StatusBadge,
  buttonVariants,
} from "@qryvanta/ui";

import { WorkspaceEntityPanel } from "@/components/apps/workspace-entity-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppEntityCapabilitiesResponse,
  type AppSitemapResponse,
  type FormResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
  type ViewResponse,
} from "@/lib/api";
import {
  flattenSitemapToNavigation,
  parseFormResponse,
  parseViewResponse,
} from "@/components/apps/workspace-entity/helpers";
import { WorkerCommandRibbon } from "@/components/apps/worker-command-ribbon";
import { WorkerSitemapSidebar } from "@/components/apps/worker-sitemap-sidebar";
import { WorkerSplitShell } from "@/components/apps/worker-split-shell";

export const metadata: Metadata = {
  title: "Worker Entity",
  description:
    "Operate runtime records with app-scoped capabilities in Worker Apps.",
};

type WorkerAppEntityPageProps = {
  params: Promise<{
    appLogicalName: string;
    entityLogicalName: string;
  }>;
  searchParams: Promise<{
    form?: string;
    view?: string;
  }>;
};

export default async function WorkerAppEntityPage({
  params,
  searchParams,
}: WorkerAppEntityPageProps) {
  const { appLogicalName, entityLogicalName } = await params;
  const { form: requestedForm, view: requestedView } = await searchParams;
  const cookieHeader = (await cookies()).toString();

  const [schemaResponse, capabilitiesResponse, recordsResponse, navigationResponse, formsResponse, viewsResponse] =
    await Promise.all([
      apiServerFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/schema`,
        cookieHeader,
      ),
      apiServerFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/capabilities`,
        cookieHeader,
      ),
      apiServerFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records?limit=50&offset=0`,
        cookieHeader,
      ),
      apiServerFetch(`/api/workspace/apps/${appLogicalName}/navigation`, cookieHeader),
      apiServerFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/forms`,
        cookieHeader,
      ),
      apiServerFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/views`,
        cookieHeader,
      ),
    ]);

  if (schemaResponse.status === 401) {
    redirect("/login");
  }

  if (schemaResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Worker Apps"
        title="Entity Access"
        message="Your account does not have read access to this app entity."
      />
    );
  }

  if (
    !schemaResponse.ok ||
    !capabilitiesResponse.ok ||
    !recordsResponse.ok ||
    !navigationResponse.ok
  ) {
    throw new Error("Failed to load app entity workspace");
  }

  const schema = (await schemaResponse.json()) as PublishedSchemaResponse;
  const capabilities =
    (await capabilitiesResponse.json()) as AppEntityCapabilitiesResponse;
  const records = (await recordsResponse.json()) as RuntimeRecordResponse[];

  const sitemap = (await navigationResponse.json()) as AppSitemapResponse;
  const sortedNavigation = flattenSitemapToNavigation(sitemap);

  // Find the current entity's navigation item for defaults
  const currentNavItem = sortedNavigation.find(
    (item) => item.entity_logical_name === entityLogicalName,
  ) ?? null;

  // Parse form and view definitions (gracefully handle failures)
  const rawForms = formsResponse.ok
    ? ((await formsResponse.json()) as FormResponse[])
    : [];
  const rawViews = viewsResponse.ok
    ? ((await viewsResponse.json()) as ViewResponse[])
    : [];

  const forms = rawForms.map(parseFormResponse);
  const views = rawViews.map(parseViewResponse);

  return (
    <WorkerSplitShell
      storageKey={`worker_sidebar_width_${appLogicalName}`}
      sidebar={
        <WorkerSitemapSidebar
          appLogicalName={appLogicalName}
          sitemap={sitemap}
          activeEntityLogicalName={entityLogicalName}
        />
      }
      content={<div className="min-h-0 overflow-y-auto bg-zinc-50">
        <WorkerCommandRibbon
          title={schema.entity_display_name}
          subtitle={`/${appLogicalName}/${entityLogicalName}`}
          badges={
            <>
              <StatusBadge tone="success">Schema v{schema.version}</StatusBadge>
              <StatusBadge tone="neutral">Records {records.length}</StatusBadge>
              <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
                Create {capabilities.can_create ? "Allowed" : "Blocked"}
              </StatusBadge>
            </>
          }
          actions={
            <>
              <Link
                href={`/worker/apps/${encodeURIComponent(appLogicalName)}`}
                className={buttonVariants({ size: "sm", variant: "outline" })}
              >
                Back to App
              </Link>
            </>
          }
        />

        <Card className="m-4 shadow-sm">
          <CardContent className="pt-4">
            <WorkspaceEntityPanel
              appLogicalName={appLogicalName}
              entityLogicalName={entityLogicalName}
              binding={null}
              initialFormLogicalName={requestedForm ?? currentNavItem?.default_form ?? null}
              initialViewLogicalName={requestedView ?? currentNavItem?.default_view ?? null}
              schema={schema}
              capabilities={capabilities}
              records={records}
              forms={forms}
              views={views}
            />
          </CardContent>
        </Card>
      </div>}
    />
  );
}
