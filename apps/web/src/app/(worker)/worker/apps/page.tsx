import type { Metadata } from "next";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

export const metadata: Metadata = {
  title: "My Apps — Worker Portal",
};

import { Card, CardContent, CardHeader, CardTitle } from "@qryvanta/ui";

import { WorkerAppLibraryPanel } from "@/components/apps/worker-app-library-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiServerFetch, type AppResponse } from "@/lib/api";

export default async function WorkerAppsPage() {
  const cookieHeader = (await cookies()).toString();
  const workspaceAppsResponse = await apiServerFetch(
    "/api/workspace/apps",
    cookieHeader,
  );

  if (workspaceAppsResponse.status === 401) {
    redirect("/login");
  }

  if (workspaceAppsResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Worker Apps"
        title="My Apps"
        message="Your account is not assigned to any app yet."
      />
    );
  }

  if (!workspaceAppsResponse.ok) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Worker app catalog unavailable</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="font-mono text-xs text-zinc-600">
            /api/workspace/apps status: {workspaceAppsResponse.status}
          </p>
        </CardContent>
      </Card>
    );
  }

  const workspaceApps = (await workspaceAppsResponse.json()) as AppResponse[];

  return <WorkerAppLibraryPanel apps={workspaceApps} />;
}
