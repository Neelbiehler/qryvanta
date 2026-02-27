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

import { RecordCreatePanel } from "@/components/apps/record-create-panel";
import { WorkerCommandRibbon } from "@/components/apps/worker-command-ribbon";
import { WorkerSitemapSidebar } from "@/components/apps/worker-sitemap-sidebar";
import { WorkerSplitShell } from "@/components/apps/worker-split-shell";
import { parseFormResponse } from "@/components/apps/workspace-entity/helpers";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppEntityCapabilitiesResponse,
  type AppSitemapResponse,
  type BusinessRuleResponse,
  type FormResponse,
  type PublishedSchemaResponse,
} from "@/lib/api";

export const metadata: Metadata = {
  title: "New Record",
  description: "Create a new runtime record using metadata-driven forms.",
};

type WorkerEntityNewRecordPageProps = {
  params: Promise<{ appLogicalName: string; entityLogicalName: string }>;
  searchParams: Promise<{ form?: string; view?: string }>;
};

export default async function WorkerEntityNewRecordPage({
  params,
  searchParams,
}: WorkerEntityNewRecordPageProps) {
  const { appLogicalName, entityLogicalName } = await params;
  const { form: requestedForm, view: requestedView } = await searchParams;
  const cookieHeader = (await cookies()).toString();

  const [schemaResponse, capabilitiesResponse, formsResponse, navigationResponse, businessRulesResponse] =
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
        title="Create Record"
        message="Your account does not have access to this app entity."
      />
    );
  }

  if (!schemaResponse.ok || !capabilitiesResponse.ok || !formsResponse.ok) {
    throw new Error("Failed to load create record workspace");
  }

  const schema = (await schemaResponse.json()) as PublishedSchemaResponse;
  const capabilities =
    (await capabilitiesResponse.json()) as AppEntityCapabilitiesResponse;
  const rawForms = (await formsResponse.json()) as FormResponse[];
  const forms = rawForms.map(parseFormResponse);
  const sitemap = navigationResponse.ok
    ? ((await navigationResponse.json()) as AppSitemapResponse)
    : null;
  const businessRules = businessRulesResponse.ok
    ? ((await businessRulesResponse.json()) as BusinessRuleResponse[])
    : [];
  const listHref = `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(entityLogicalName)}${requestedView ? `?view=${encodeURIComponent(requestedView)}` : ""}`;

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
          title={`${schema.entity_display_name} · New`}
          subtitle={`/${appLogicalName}/${entityLogicalName}/new`}
          badges={
            <>
              <StatusBadge tone="success">Schema v{schema.version}</StatusBadge>
              <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
                Create {capabilities.can_create ? "Allowed" : "Blocked"}
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
            <RecordCreatePanel
              appLogicalName={appLogicalName}
              entityLogicalName={entityLogicalName}
              capabilities={capabilities}
              schema={schema}
              forms={forms}
              businessRules={businessRules}
              initialFormLogicalName={requestedForm ?? null}
              returnViewLogicalName={requestedView ?? null}
            />
          </CardContent>
        </Card>
      </div>}
    />
  );
}
