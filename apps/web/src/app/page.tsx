import type { Metadata } from "next";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiServerFetch, type UserIdentityResponse } from "@/lib/api";
import {
  readAccessibleSurfaces,
  resolveDefaultSurface,
  SURFACES,
} from "@/lib/surfaces";

export const metadata: Metadata = {
  title: "Qryvanta",
  description: "Open your available Qryvanta surface.",
};

export default async function HomePage() {
  const cookieHeader = (await cookies()).toString();
  const meResponse = await apiServerFetch("/auth/me", cookieHeader);

  if (meResponse.status === 401) {
    redirect("/login");
  }

  if (!meResponse.ok) {
    throw new Error("Failed to load current user");
  }

  const user = (await meResponse.json()) as UserIdentityResponse;
  const accessibleSurfaces = readAccessibleSurfaces(user);
  const defaultSurface = resolveDefaultSurface(accessibleSurfaces);

  if (!defaultSurface) {
    return (
      <main className="mx-auto flex min-h-screen w-full max-w-3xl items-center p-6">
        <AccessDeniedCard
          section="Workspace"
          title="No Surface Access"
          message="Your account is authenticated but is not assigned to Admin Center, Maker Center, or Worker Apps yet. Contact your tenant administrator."
        />
      </main>
    );
  }

  redirect(SURFACES[defaultSurface].basePath);
}
