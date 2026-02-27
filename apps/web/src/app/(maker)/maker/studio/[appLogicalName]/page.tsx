import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { UnifiedStudioPanel } from "@/components/studio/unified-studio-panel";
import {
  apiServerFetch,
  type AppEntityBindingResponse,
  type AppResponse,
  type EntityResponse,
  type RoleResponse,
} from "@/lib/api";

type StudioPageProps = {
  params: Promise<{ appLogicalName: string }>;
};

export default async function StudioPage({ params }: StudioPageProps) {
  const { appLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const [appsResponse, entitiesResponse, rolesResponse, bindingsResponse] =
    await Promise.all([
      apiServerFetch("/api/apps", cookieHeader),
      apiServerFetch("/api/entities", cookieHeader),
      apiServerFetch("/api/security/roles", cookieHeader),
      apiServerFetch(
        `/api/apps/${encodeURIComponent(appLogicalName)}/entities`,
        cookieHeader,
      ),
    ]);

  if (appsResponse.status === 401) {
    redirect("/login");
  }

  if (
    appsResponse.status === 403 ||
    entitiesResponse.status === 403 ||
    rolesResponse.status === 403
  ) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Studio"
        message="Your account does not have the required permissions for the Studio."
      />
    );
  }

  if (!appsResponse.ok || !entitiesResponse.ok || !rolesResponse.ok) {
    throw new Error("Failed to load studio data");
  }

  const apps = (await appsResponse.json()) as AppResponse[];
  const entities = (await entitiesResponse.json()) as EntityResponse[];
  const roles = (await rolesResponse.json()) as RoleResponse[];
  const bindings = bindingsResponse.ok
    ? ((await bindingsResponse.json()) as AppEntityBindingResponse[])
    : [];

  return (
    <div className="-mx-4 -my-5 flex h-[calc(100vh-3.75rem)] min-h-[760px] flex-col md:-mx-8 md:-my-8">
      <div className="border-b border-zinc-200 bg-white/90 px-4 py-3 backdrop-blur-sm md:px-6">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
          Maker Center
        </p>
        <h1 className="font-serif text-2xl text-zinc-900">Studio</h1>
        <p className="text-sm text-zinc-600">
          Design forms, configure views, and compose your app — all in one place.
        </p>
      </div>

      <div className="min-h-0 flex-1">
        <UnifiedStudioPanel
          initialAppLogicalName={appLogicalName}
          apps={apps}
          entities={entities}
          roles={roles}
          bindings={bindings}
        />
      </div>
    </div>
  );
}
