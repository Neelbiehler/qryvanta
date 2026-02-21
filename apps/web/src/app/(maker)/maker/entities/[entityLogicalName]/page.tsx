import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardHeader,
  PageHeader,
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
    <Card>
      <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <PageHeader
          eyebrow="Maker Center"
          title={entityLogicalName}
          description="Design fields, publish schemas, and query runtime records."
          actions={
            <Link
              href="/maker/entities"
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to entities
            </Link>
          }
        />
      </CardHeader>

      <CardContent>
        <EntityWorkbenchPanel
          entityLogicalName={entityLogicalName}
          initialFields={fields}
          initialPublishedSchema={publishedSchema}
          initialRecords={records}
        />
      </CardContent>
    </Card>
  );
}
