import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardHeader,
  StatusBadge,
} from "@qryvanta/ui";

import { AppStudioPanel } from "@/components/apps/app-studio-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppResponse,
  type EntityResponse,
  type RoleResponse,
} from "@/lib/api";

export default async function MakerAppStudioPage() {
  const cookieHeader = (await cookies()).toString();

  const [adminAppsResponse, entitiesResponse, rolesResponse] =
    await Promise.all([
      apiServerFetch("/api/apps", cookieHeader),
      apiServerFetch("/api/entities", cookieHeader),
      apiServerFetch("/api/security/roles", cookieHeader),
    ]);

  if (adminAppsResponse.status === 401) {
    redirect("/login");
  }

  if (
    adminAppsResponse.status === 403 ||
    entitiesResponse.status === 403 ||
    rolesResponse.status === 403
  ) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="App Studio"
        message="Your account does not have the required permissions for the App Studio."
      />
    );
  }

  if (!adminAppsResponse.ok || !entitiesResponse.ok || !rolesResponse.ok) {
    throw new Error("Failed to load app studio data");
  }

  const apps = (await adminAppsResponse.json()) as AppResponse[];
  const entities = (await entitiesResponse.json()) as EntityResponse[];
  const roles = (await rolesResponse.json()) as RoleResponse[];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Maker Center
            </p>
            <h1 className="font-serif text-3xl text-zinc-900">Model-driven App Designer</h1>
            <p className="text-sm text-zinc-600">
              Configure sitemap navigation, role matrix permissions, and workspace presentation.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Apps {apps.length}</StatusBadge>
            <StatusBadge tone="neutral">Entities {entities.length}</StatusBadge>
            <StatusBadge tone="neutral">Roles {roles.length}</StatusBadge>
          </div>
        </CardHeader>
      </Card>

      <Card className="border-emerald-200 bg-white">
        <CardContent className="pt-6">
          <AppStudioPanel apps={apps} entities={entities} roles={roles} />
        </CardContent>
      </Card>
    </div>
  );
}
