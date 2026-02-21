import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  buttonVariants,
} from "@qryvanta/ui";

import { WorkspaceEntityPanel } from "@/components/apps/workspace-entity-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppEntityCapabilitiesResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type AppEntityPageProps = {
  params: Promise<{
    appLogicalName: string;
    entityLogicalName: string;
  }>;
};

export default async function AppEntityPage({ params }: AppEntityPageProps) {
  const { appLogicalName, entityLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const [schemaResponse, capabilitiesResponse, recordsResponse] =
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
    ]);

  if (schemaResponse.status === 401) {
    redirect("/login");
  }

  if (schemaResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Workspace"
        title="Entity Access"
        message="Your account is authenticated but does not have read access to this app entity."
      />
    );
  }

  if (!schemaResponse.ok || !capabilitiesResponse.ok || !recordsResponse.ok) {
    throw new Error("Failed to load app entity workspace");
  }

  const schema = (await schemaResponse.json()) as PublishedSchemaResponse;
  const capabilities =
    (await capabilitiesResponse.json()) as AppEntityCapabilitiesResponse;
  const records = (await recordsResponse.json()) as RuntimeRecordResponse[];

  return (
    <Card>
      <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">
            Worker App
          </p>
          <CardTitle className="font-serif text-3xl">
            {schema.entity_display_name} · {appLogicalName}
          </CardTitle>
        </div>
        <Link
          href={`/apps/${appLogicalName}`}
          className={cn(buttonVariants({ variant: "outline" }))}
        >
          Back to app
        </Link>
      </CardHeader>
      <CardContent>
        <WorkspaceEntityPanel
          appLogicalName={appLogicalName}
          entityLogicalName={entityLogicalName}
          schema={schema}
          capabilities={capabilities}
          records={records}
        />
      </CardContent>
    </Card>
  );
}
