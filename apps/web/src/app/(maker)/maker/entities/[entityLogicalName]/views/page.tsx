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
import { apiServerFetch, type ViewResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerEntityViewsPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerEntityViewsPage({
  params,
}: MakerEntityViewsPageProps) {
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
        title="Views"
        message="Your account does not have metadata field read permissions."
      />
    );
  }
  if (!viewsResponse.ok) {
    throw new Error("Failed to load views.");
  }
  const views = (await viewsResponse.json()) as ViewResponse[];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Maker Center
            </p>
            <CardTitle className="font-serif text-3xl">
              {entityLogicalName} Views
            </CardTitle>
            <CardDescription>
              Manage standalone view definitions for this entity.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Views {views.length}</StatusBadge>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/views/new`}
              className={cn(buttonVariants())}
            >
              New View
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

      {views.length > 0 ? (
        <div className="grid gap-4 lg:grid-cols-2">
          {views.map((view) => (
            <Card key={view.logical_name}>
              <CardHeader>
                <CardTitle>{view.display_name}</CardTitle>
                <CardDescription className="font-mono text-xs">
                  {view.logical_name}
                </CardDescription>
              </CardHeader>
              <CardContent className="flex items-center gap-2">
                <StatusBadge tone="neutral">{view.view_type}</StatusBadge>
                {view.is_default ? <StatusBadge tone="success">Default</StatusBadge> : null}
                <Link
                  href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/views/${encodeURIComponent(view.logical_name)}`}
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
            <CardTitle>No views defined yet</CardTitle>
            <CardDescription>
              Create a main grid view to configure worker record list layout.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/views/new`}
              className={cn(buttonVariants())}
            >
              Create First View
            </Link>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

