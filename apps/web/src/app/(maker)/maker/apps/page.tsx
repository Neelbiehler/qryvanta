import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { Card, CardContent, CardHeader, CardTitle } from "@qryvanta/ui";

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
    <Card>
      <CardHeader>
        <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">
          Maker Center
        </p>
        <CardTitle className="font-serif text-3xl">App Studio</CardTitle>
      </CardHeader>
      <CardContent>
        <AppStudioPanel apps={apps} entities={entities} roles={roles} />
      </CardContent>
    </Card>
  );
}
