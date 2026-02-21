import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { Header } from "@/components/layout/header";
import { SurfaceSidebar } from "@/components/layout/surface-sidebar";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiServerFetch, type UserIdentityResponse } from "@/lib/api";
import {
  type SurfaceId,
  SURFACES,
  hasSurfaceAccess,
  readAccessibleSurfaces,
} from "@/lib/surfaces";

type SurfaceLayoutProps = {
  children: React.ReactNode;
  surfaceId: SurfaceId;
};

/**
 * Shared layout for surface-scoped route groups.
 *
 * Resolves the authenticated user, checks surface access, and renders the
 * surface-specific sidebar, header, and content area. Redirects to login
 * if unauthenticated and shows an access-denied card when the user lacks
 * permissions for the requested surface.
 */
export async function SurfaceLayout({
  children,
  surfaceId,
}: SurfaceLayoutProps) {
  const cookieHeader = (await cookies()).toString();
  const meResponse = await apiServerFetch("/auth/me", cookieHeader);

  if (meResponse.status === 401) {
    redirect("/login");
  }

  if (meResponse.status === 403) {
    return (
      <div className="mx-auto flex min-h-screen w-full max-w-3xl items-center p-6">
        <AccessDeniedCard
          section="Workspace"
          title="Access Restricted"
          message="Your account is authenticated but does not have access to this workspace."
        />
      </div>
    );
  }

  if (!meResponse.ok) {
    throw new Error("Failed to load current user");
  }

  const user = (await meResponse.json()) as UserIdentityResponse;
  const accessibleSurfaces = readAccessibleSurfaces(user);
  const definition = SURFACES[surfaceId];

  if (!hasSurfaceAccess(accessibleSurfaces, surfaceId)) {
    return (
      <div className="mx-auto flex min-h-screen w-full max-w-3xl items-center p-6">
        <AccessDeniedCard
          section={definition.label}
          title="Surface Access Denied"
          message={`Your account does not have the required permissions to access the ${definition.label}. Contact your tenant administrator to request access.`}
        />
      </div>
    );
  }

  return (
    <div className="grid min-h-screen grid-cols-1 bg-app lg:grid-cols-[300px_1fr]">
      <SurfaceSidebar
        surface={surfaceId}
        accessibleSurfaces={accessibleSurfaces}
      />
      <div className="flex min-h-screen min-w-0 flex-col">
        <Header user={user} surfaceId={surfaceId} />
        <main className="flex-1 px-4 py-5 md:px-8 md:py-8">{children}</main>
      </div>
    </div>
  );
}
