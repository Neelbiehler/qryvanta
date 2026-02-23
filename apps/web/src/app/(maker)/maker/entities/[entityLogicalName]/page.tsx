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

import { EntityWorkbenchPanel } from "@/components/entities/entity-workbench-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type FieldResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerEntityWorkbenchPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerEntityWorkbenchPage({
  params,
}: MakerEntityWorkbenchPageProps) {
  const { entityLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const fieldsResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/fields`,
    cookieHeader,
  );

  if (fieldsResponse.status === 401) {
    redirect("/login");
  }

  if (fieldsResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Entity Workbench"
        message="Your account does not have metadata field permissions."
      />
    );
  }

  if (!fieldsResponse.ok) {
    throw new Error("Failed to load entity fields");
  }

  const fields = (await fieldsResponse.json()) as FieldResponse[];

  const publishedResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/published`,
    cookieHeader,
  );

  if (publishedResponse.status === 401) {
    redirect("/login");
  }

  if (publishedResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Entity Workbench"
        message="Your account does not have metadata read permissions."
      />
    );
  }

  let publishedSchema: PublishedSchemaResponse | null = null;
  if (publishedResponse.status !== 404) {
    if (!publishedResponse.ok) {
      throw new Error("Failed to load published schema");
    }
    publishedSchema =
      (await publishedResponse.json()) as PublishedSchemaResponse;
  }

  let records: RuntimeRecordResponse[] = [];
  if (publishedSchema) {
    const recordsResponse = await apiServerFetch(
      `/api/runtime/${entityLogicalName}/records?limit=50&offset=0`,
      cookieHeader,
    );

    if (recordsResponse.status === 401) {
      redirect("/login");
    }

    if (recordsResponse.status === 403) {
      return (
        <AccessDeniedCard
          section="Maker Center"
          title="Records"
          message="Your account does not have runtime record read permissions."
        />
      );
    }

    if (!recordsResponse.ok) {
      throw new Error("Failed to load runtime records");
    }

    records = (await recordsResponse.json()) as RuntimeRecordResponse[];
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Maker Center
            </p>
            <CardTitle className="font-serif text-3xl">{entityLogicalName}</CardTitle>
            <CardDescription>
              Model-driven entity designer for schema, publishing, and runtime validation.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Fields {fields.length}</StatusBadge>
            <StatusBadge tone="neutral">Records {records.length}</StatusBadge>
            <StatusBadge tone={publishedSchema ? "success" : "warning"}>
              {publishedSchema ? `Published v${publishedSchema.version}` : "Draft only"}
            </StatusBadge>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/forms`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Forms
            </Link>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/views`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Views
            </Link>
            <Link
              href="/maker/entities"
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to library
            </Link>
          </div>
        </CardHeader>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[280px_1fr]">
        <Card className="h-fit border-emerald-200 bg-white">
          <CardHeader>
            <CardTitle className="text-base">Designer Map</CardTitle>
            <CardDescription>
              Follow schema-first model-driven delivery sequence.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-zinc-700">
            <p>1. Define fields and constraints</p>
            <p>2. Publish schema versions</p>
            <p>3. Create and query runtime data</p>
            <p>4. Validate app bindings in Worker Apps</p>
          </CardContent>
        </Card>

        <Card className="border-emerald-200 bg-white">
          <CardContent className="pt-6">
            <EntityWorkbenchPanel
              entityLogicalName={entityLogicalName}
              initialFields={fields}
              initialPublishedSchema={publishedSchema}
              initialRecords={records}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
