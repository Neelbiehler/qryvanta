import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { Card, CardContent, CardHeader, CardTitle } from "@qryvanta/ui";
import { apiServerFetch, type EntityResponse } from "@/lib/api";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { EntityLibraryPanel } from "@/components/entities/entity-library-panel";

export default async function MakerEntitiesPage() {
  const cookieHeader = (await cookies()).toString();
  const response = await apiServerFetch("/api/entities", cookieHeader);

  if (response.status === 401) {
    redirect("/login");
  }

  if (response.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Entities"
        message="Your account does not have metadata read permissions."
      />
    );
  }

  if (!response.ok) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Entity metadata unavailable</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="font-mono text-xs text-zinc-600">
            /api/entities status: {response.status}
          </p>
        </CardContent>
      </Card>
    );
  }

  const entities = (await response.json()) as EntityResponse[];

  return <EntityLibraryPanel entities={entities} />;
}
