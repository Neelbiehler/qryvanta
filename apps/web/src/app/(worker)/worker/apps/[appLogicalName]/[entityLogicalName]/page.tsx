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
  PageHeader,
  StatusBadge,
  buttonVariants,
} from "@qryvanta/ui";

import { WorkspaceEntityPanel } from "@/components/apps/workspace-entity-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  type AppEntityBindingResponse,
  apiServerFetch,
  type AppEntityCapabilitiesResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";
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
};

export default async function WorkerAppEntityPage({
  params,
}: WorkerAppEntityPageProps) {
  const { appLogicalName, entityLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const [schemaResponse, capabilitiesResponse, recordsResponse, navigationResponse] =
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
  const navigation = (await navigationResponse.json()) as AppEntityBindingResponse[];
  const binding =
    navigation.find((item) => item.entity_logical_name === entityLogicalName) ??
    null;

  return (
    <div className="space-y-4">
      <PageHeader
        eyebrow="Worker Apps"
        title={`${schema.entity_display_name} | ${appLogicalName}`}
        description="Operate records using app-scoped capabilities and fast grid workflows."
        actions={
          <Link
            href={`/worker/apps/${appLogicalName}`}
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Back to app
          </Link>
        }
      />

      <div className="grid gap-4 xl:grid-cols-[300px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Entity Workspace</CardTitle>
            <CardDescription>
              Record operations for {schema.entity_display_name}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <StatusBadge tone="success">
              Published v{schema.version}
            </StatusBadge>
            <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
              Create {capabilities.can_create ? "Allowed" : "Blocked"}
            </StatusBadge>
            <StatusBadge tone={capabilities.can_update ? "success" : "warning"}>
              Update {capabilities.can_update ? "Allowed" : "Blocked"}
            </StatusBadge>
            <StatusBadge tone={capabilities.can_delete ? "warning" : "neutral"}>
              Delete {capabilities.can_delete ? "Allowed" : "Blocked"}
            </StatusBadge>
            <p className="text-xs text-zinc-500">
              Loaded {records.length} row(s) for initial grid view.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <WorkspaceEntityPanel
              appLogicalName={appLogicalName}
              entityLogicalName={entityLogicalName}
              binding={binding}
              schema={schema}
              capabilities={capabilities}
              records={records}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
