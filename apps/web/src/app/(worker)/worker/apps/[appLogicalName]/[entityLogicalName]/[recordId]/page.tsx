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

import { RecordDetailPanel } from "@/components/apps/record-detail-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppEntityCapabilitiesResponse,
  type AppSitemapResponse,
  type BusinessRuleResponse,
  type FormResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";
import {
  flattenSitemapToNavigation,
  parseFormResponse,
} from "@/components/apps/workspace-entity/helpers";
import { WorkerCommandRibbon } from "@/components/apps/worker-command-ribbon";
import { WorkerSitemapSidebar } from "@/components/apps/worker-sitemap-sidebar";
import { WorkerSplitShell } from "@/components/apps/worker-split-shell";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Record Detail",
  description: "View and edit a runtime record using metadata-driven forms.",
};

type RecordDetailPageProps = {
  params: Promise<{
    appLogicalName: string;
    entityLogicalName: string;
    recordId: string;
  }>;
  searchParams: Promise<{
    form?: string;
  }>;
};

export default async function RecordDetailPage({
  params,
  searchParams,
}: RecordDetailPageProps) {
  const { appLogicalName, entityLogicalName, recordId } = await params;
  const { form: requestedForm } = await searchParams;
  const cookieHeader = (await cookies()).toString();

  const [schemaResponse, capabilitiesResponse, recordResponse, formsResponse, navigationResponse, businessRulesResponse] =
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
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records/${recordId}`,
        cookieHeader,
      ),
      apiServerFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/forms`,
        cookieHeader,
      ),
      apiServerFetch(`/api/workspace/apps/${appLogicalName}/navigation`, cookieHeader),
      apiServerFetch(`/api/runtime/${entityLogicalName}/business-rules`, cookieHeader),
    ]);

  if (schemaResponse.status === 401) {
    redirect("/login");
  }

  if (schemaResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Worker Apps"
        title="Record Access"
        message="Your account does not have read access to this record."
      />
    );
  }

  if (!schemaResponse.ok || !capabilitiesResponse.ok) {
    throw new Error("Failed to load record detail");
  }

  if (recordResponse.status === 404) {
    return (
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle className="font-serif text-3xl">Record Not Found</CardTitle>
            <CardDescription>
              The record with ID &quot;{recordId}&quot; does not exist or has been deleted.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href={`/worker/apps/${appLogicalName}/${entityLogicalName}`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to entity list
            </Link>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!recordResponse.ok) {
    throw new Error("Failed to load record");
  }

  const schema = (await schemaResponse.json()) as PublishedSchemaResponse;
  const capabilities =
    (await capabilitiesResponse.json()) as AppEntityCapabilitiesResponse;
  const record = (await recordResponse.json()) as RuntimeRecordResponse;

  const rawForms = formsResponse.ok
    ? ((await formsResponse.json()) as FormResponse[])
    : [];
  const forms = rawForms.map(parseFormResponse);
  const businessRules = businessRulesResponse.ok
    ? ((await businessRulesResponse.json()) as BusinessRuleResponse[])
    : [];

  const sitemap = navigationResponse.ok
    ? ((await navigationResponse.json()) as AppSitemapResponse)
    : null;
  const navItem = sitemap
    ? flattenSitemapToNavigation(sitemap).find(
        (item) => item.entity_logical_name === entityLogicalName,
      ) ?? null
    : null;
  const listHref = `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(entityLogicalName)}`;

  return (
    <WorkerSplitShell
      storageKey={`worker_sidebar_width_${appLogicalName}`}
      sidebar={
        sitemap ? (
          <WorkerSitemapSidebar
            appLogicalName={appLogicalName}
            sitemap={sitemap}
            activeEntityLogicalName={entityLogicalName}
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
          title={`${schema.entity_display_name} · Record`}
          subtitle={record.record_id}
          badges={
            <>
              <StatusBadge tone="success">Schema v{schema.version}</StatusBadge>
              <StatusBadge tone={capabilities.can_update ? "success" : "warning"}>
                Update {capabilities.can_update ? "Allowed" : "Blocked"}
              </StatusBadge>
            </>
          }
          actions={
            <>
              <Link
                href={listHref}
                className={buttonVariants({ size: "sm", variant: "outline" })}
              >
                Back to List
              </Link>
            </>
          }
        />

        <Card className="m-4 shadow-sm">
          <CardContent className="pt-4">
            <RecordDetailPanel
              appLogicalName={appLogicalName}
              entityLogicalName={entityLogicalName}
              capabilities={capabilities}
              forms={forms}
              businessRules={businessRules}
              initialFormLogicalName={requestedForm ?? navItem?.default_form ?? null}
              record={record}
              schema={schema}
            />
          </CardContent>
        </Card>
      </div>}
    />
  );
}
