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

import { WorkspaceEntityPanel } from "@/components/apps/workspace-entity-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppEntityCapabilitiesResponse,
  type AppSitemapResponse,
  type BusinessRuleResponse,
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
import { cn } from "@/lib/utils";

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

  const [schemaResponse, capabilitiesResponse, recordsResponse, navigationResponse, formsResponse, viewsResponse, businessRulesResponse] =
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
      apiServerFetch(`/api/runtime/${entityLogicalName}/business-rules`, cookieHeader),
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
  const businessRules = businessRulesResponse.ok
    ? ((await businessRulesResponse.json()) as BusinessRuleResponse[])
    : [];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Worker Apps
            </p>
            <CardTitle className="font-serif text-3xl">
              {schema.entity_display_name} - {appLogicalName}
            </CardTitle>
            <CardDescription>
              Dynamics-style entity workspace with app-scoped capabilities and model-driven views.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="success">Published v{schema.version}</StatusBadge>
            <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
              Create {capabilities.can_create ? "Allowed" : "Blocked"}
            </StatusBadge>
            <StatusBadge tone={capabilities.can_update ? "success" : "warning"}>
              Update {capabilities.can_update ? "Allowed" : "Blocked"}
            </StatusBadge>
            <StatusBadge tone={capabilities.can_delete ? "warning" : "neutral"}>
              Delete {capabilities.can_delete ? "Allowed" : "Blocked"}
            </StatusBadge>
            <Link
              href={`/worker/apps/${appLogicalName}`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to app
            </Link>
          </div>
        </CardHeader>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[280px_1fr]">
        <Card className="h-fit border-zinc-200 bg-zinc-50">
          <CardHeader>
            <CardTitle className="text-base">Sitemap</CardTitle>
            <CardDescription>Switch entity work areas within this app.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {sortedNavigation.map((item) => {
              const isActive = item.entity_logical_name === entityLogicalName;
              return (
                <Link
                  key={item.entity_logical_name}
                  href={`/worker/apps/${appLogicalName}/${item.entity_logical_name}`}
                  className={`block rounded-md border px-3 py-2 text-sm transition ${
                    isActive
                      ? "border-emerald-400 bg-emerald-50"
                      : "border-zinc-200 bg-white hover:border-emerald-300"
                  }`}
                >
                  <p className="font-medium text-zinc-900">
                    {item.display_name}
                  </p>
                  <p className="font-mono text-[11px] text-zinc-500">
                    {item.entity_logical_name}
                  </p>
                </Link>
              );
            })}
          </CardContent>
        </Card>

        <Card className="border-zinc-200 bg-white">
          <CardContent className="pt-6">
            <WorkspaceEntityPanel
              appLogicalName={appLogicalName}
              entityLogicalName={entityLogicalName}
              binding={null}
              initialFormLogicalName={requestedForm ?? currentNavItem?.default_form ?? null}
              initialViewLogicalName={requestedView ?? currentNavItem?.default_view ?? null}
              schema={schema}
              capabilities={capabilities}
              businessRules={businessRules}
              records={records}
              forms={forms}
              views={views}
            />
          </CardContent>
        </Card>
      </div>

      <div className="flex justify-end">
        <Link
          href={`/worker/apps/${appLogicalName}`}
          className={cn(buttonVariants({ variant: "outline" }))}
        >
          Return to app hub
        </Link>
      </div>
    </div>
  );
}
