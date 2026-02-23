import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { Card, CardDescription, CardHeader, CardTitle, buttonVariants } from "@qryvanta/ui";

import { FormDesignerPanel } from "@/components/entities/forms/form-designer-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type FormResponse,
  type PublishedSchemaResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerNewEntityFormPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerNewEntityFormPage({
  params,
}: MakerNewEntityFormPageProps) {
  const { entityLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const formsResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/forms`,
    cookieHeader,
  );
  if (formsResponse.status === 401) {
    redirect("/login");
  }
  if (formsResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Form Designer"
        message="Your account does not have metadata field permissions."
      />
    );
  }
  if (!formsResponse.ok) {
    throw new Error("Failed to load forms.");
  }
  const forms = (await formsResponse.json()) as FormResponse[];

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
        title="Form Designer"
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

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
          <div className="space-y-1">
            <CardTitle className="font-serif text-2xl">New Form</CardTitle>
            <CardDescription>
              Entity: <span className="font-mono">{entityLogicalName}</span>
            </CardDescription>
          </div>
          <Link
            href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/forms`}
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Back to Forms
          </Link>
        </CardHeader>
      </Card>

      <FormDesignerPanel
        entityLogicalName={entityLogicalName}
        initialForm={null}
        initialForms={forms}
        publishedSchema={publishedSchema}
      />
    </div>
  );
}

