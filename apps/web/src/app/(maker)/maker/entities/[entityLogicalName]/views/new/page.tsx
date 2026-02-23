import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { Card, CardDescription, CardHeader, CardTitle, buttonVariants } from "@qryvanta/ui";

import { ViewDesignerPanel } from "@/components/entities/views/view-designer-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
  type ViewResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerNewEntityViewPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerNewEntityViewPage({
  params,
}: MakerNewEntityViewPageProps) {
  const { entityLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const viewsResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/views`,
    cookieHeader,
  );
  if (viewsResponse.status === 401) {
    redirect("/login");
  }
  if (viewsResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="View Designer"
        message="Your account does not have metadata field permissions."
      />
    );
  }
  if (!viewsResponse.ok) {
    throw new Error("Failed to load views.");
  }
  const views = (await viewsResponse.json()) as ViewResponse[];

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
        title="View Designer"
        message="Your account does not have metadata read permissions."
      />
    );
  }
  let publishedSchema: PublishedSchemaResponse | null = null;
  if (publishedResponse.status !== 404) {
    if (!publishedResponse.ok) {
      throw new Error("Failed to load published schema.");
    }
    publishedSchema =
      (await publishedResponse.json()) as PublishedSchemaResponse;
  }

  let previewRecords: RuntimeRecordResponse[] = [];
  if (publishedSchema) {
    const recordsResponse = await apiServerFetch(
      `/api/runtime/${entityLogicalName}/records?limit=5&offset=0`,
      cookieHeader,
    );
    if (recordsResponse.status === 401) {
      redirect("/login");
    }
    if (recordsResponse.ok) {
      previewRecords = (await recordsResponse.json()) as RuntimeRecordResponse[];
    }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
          <div className="space-y-1">
            <CardTitle className="font-serif text-2xl">New View</CardTitle>
            <CardDescription>
              Entity: <span className="font-mono">{entityLogicalName}</span>
            </CardDescription>
          </div>
          <Link
            href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/views`}
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Back to Views
          </Link>
        </CardHeader>
      </Card>

      <ViewDesignerPanel
        entityLogicalName={entityLogicalName}
        initialView={null}
        initialViews={views}
        publishedSchema={publishedSchema}
        initialPreviewRecords={previewRecords}
      />
    </div>
  );
}

