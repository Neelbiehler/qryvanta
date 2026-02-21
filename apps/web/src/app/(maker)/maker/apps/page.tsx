import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  PageHeader,
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
      <PageHeader
        eyebrow="Maker Center"
        title="App Studio"
        description="Model app navigation and role permissions with a task-driven builder flow."
      />

      <div className="grid gap-4 xl:grid-cols-[300px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Builder Checklist</CardTitle>
            <CardDescription>
              Recommended sequence for delivery-ready apps.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <StatusBadge tone="neutral">Apps {apps.length}</StatusBadge>
            <StatusBadge tone="neutral">Entities {entities.length}</StatusBadge>
            <StatusBadge tone="neutral">Roles {roles.length}</StatusBadge>
            <ol className="space-y-2 text-sm text-zinc-700">
              <li>1. Create app shell</li>
              <li>2. Bind entities to navigation</li>
              <li>3. Assign role entity permissions</li>
              <li>4. Validate in Worker Apps</li>
            </ol>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <AppStudioPanel apps={apps} entities={entities} roles={roles} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
