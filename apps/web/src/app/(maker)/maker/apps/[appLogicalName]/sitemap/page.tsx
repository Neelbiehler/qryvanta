import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
  StatusBadge,
  buttonVariants,
} from "@qryvanta/ui";

import { SitemapEditorPanel } from "@/components/apps/sitemap-editor-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppResponse,
  type AppSitemapResponse,
  type EntityResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerAppSitemapPageProps = {
  params: Promise<{
    appLogicalName: string;
  }>;
};

export default async function MakerAppSitemapPage({ params }: MakerAppSitemapPageProps) {
  const { appLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const [appsResponse, entitiesResponse, sitemapResponse] = await Promise.all([
    apiServerFetch("/api/apps", cookieHeader),
    apiServerFetch("/api/entities", cookieHeader),
    apiServerFetch(`/api/apps/${appLogicalName}/sitemap`, cookieHeader),
  ]);

  if (
    appsResponse.status === 401 ||
    entitiesResponse.status === 401 ||
    sitemapResponse.status === 401
  ) {
    redirect("/login");
  }

  if (
    appsResponse.status === 403 ||
    entitiesResponse.status === 403 ||
    sitemapResponse.status === 403
  ) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Sitemap Editor"
        message="Your account does not have app administration permissions."
      />
    );
  }

  if (!appsResponse.ok || !entitiesResponse.ok || !sitemapResponse.ok) {
    throw new Error("Failed to load sitemap editor data.");
  }

  const apps = (await appsResponse.json()) as AppResponse[];
  const entities = (await entitiesResponse.json()) as EntityResponse[];
  const sitemap = (await sitemapResponse.json()) as AppSitemapResponse;
  const appDisplayName =
    apps.find((app) => app.logical_name === appLogicalName)?.display_name ?? appLogicalName;

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Maker Center
            </p>
            <CardTitle className="font-serif text-3xl">{appDisplayName} Sitemap</CardTitle>
            <CardDescription>
              Configure hierarchy, ordering, and target metadata for app navigation.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">App {appLogicalName}</StatusBadge>
            <StatusBadge tone="neutral">Entities {entities.length}</StatusBadge>
            <Link href="/maker/apps" className={cn(buttonVariants({ variant: "outline" }))}>
              Back to App Studio
            </Link>
          </div>
        </CardHeader>
      </Card>

      <SitemapEditorPanel
        appLogicalName={appLogicalName}
        initialSitemap={sitemap}
        entities={entities}
      />
    </div>
  );
}

