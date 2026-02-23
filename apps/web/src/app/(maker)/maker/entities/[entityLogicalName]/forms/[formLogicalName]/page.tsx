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

type MakerEntityFormDesignerPageProps = {
  params: Promise<{
    entityLogicalName: string;
    formLogicalName: string;
  }>;
};

export default async function MakerEntityFormDesignerPage({
  params,
}: MakerEntityFormDesignerPageProps) {
  const { entityLogicalName, formLogicalName } = await params;
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

  const formResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/forms/${formLogicalName}`,
    cookieHeader,
  );
  if (formResponse.status === 401) {
    redirect("/login");
  }
  if (formResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Form Designer"
        message="Your account does not have metadata field permissions."
      />
    );
  }
  if (formResponse.status === 404) {
    redirect(`/maker/entities/${encodeURIComponent(entityLogicalName)}/forms`);
  }
  if (!formResponse.ok) {
    throw new Error("Failed to load form.");
  }
  const form = (await formResponse.json()) as FormResponse;

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
            <CardTitle className="font-serif text-2xl">{form.display_name}</CardTitle>
            <CardDescription>
              <span className="font-mono">{entityLogicalName}</span> ·{" "}
              <span className="font-mono">{form.logical_name}</span>
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
        initialForm={form}
        initialForms={forms}
        publishedSchema={publishedSchema}
      />
    </div>
  );
}

