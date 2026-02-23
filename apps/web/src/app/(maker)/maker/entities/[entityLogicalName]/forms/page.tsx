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

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiServerFetch, type FormResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerEntityFormsPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerEntityFormsPage({
  params,
}: MakerEntityFormsPageProps) {
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
        title="Forms"
        message="Your account does not have metadata field read permissions."
      />
    );
  }
  if (!formsResponse.ok) {
    throw new Error("Failed to load forms.");
  }

  const forms = (await formsResponse.json()) as FormResponse[];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Maker Center
            </p>
            <CardTitle className="font-serif text-3xl">
              {entityLogicalName} Forms
            </CardTitle>
            <CardDescription>
              Manage standalone form definitions for this entity.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Forms {forms.length}</StatusBadge>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/forms/new`}
              className={cn(buttonVariants())}
            >
              New Form
            </Link>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to Entity
            </Link>
          </div>
        </CardHeader>
      </Card>

      {forms.length > 0 ? (
        <div className="grid gap-4 lg:grid-cols-2">
          {forms.map((form) => (
            <Card key={form.logical_name}>
              <CardHeader>
                <CardTitle>{form.display_name}</CardTitle>
                <CardDescription className="font-mono text-xs">
                  {form.logical_name}
                </CardDescription>
              </CardHeader>
              <CardContent className="flex items-center gap-2">
                <StatusBadge tone="neutral">{form.form_type}</StatusBadge>
                <Link
                  href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/forms/${encodeURIComponent(form.logical_name)}`}
                  className={cn(buttonVariants({ size: "sm", variant: "outline" }), "ml-auto")}
                >
                  Open Designer
                </Link>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>No forms defined yet</CardTitle>
            <CardDescription>
              Create a main form to start designing worker form layouts.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/forms/new`}
              className={cn(buttonVariants())}
            >
              Create First Form
            </Link>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

