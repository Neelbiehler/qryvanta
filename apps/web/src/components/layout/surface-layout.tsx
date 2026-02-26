"use client";

import { Header } from "@/components/layout/header";
import { SurfaceSidebar } from "@/components/layout/surface-sidebar";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { type UserIdentityResponse } from "@/lib/api";
import { useSidebarState } from "@/lib/hooks/use-sidebar-state";
import {
  type SurfaceId,
  SURFACES,
  hasSurfaceAccess,
  readAccessibleSurfaces,
} from "@/lib/surfaces";
import { cn } from "@/lib/utils";

type SurfaceLayoutProps = {
  children: React.ReactNode;
  surfaceId: SurfaceId;
  user: UserIdentityResponse;
  commandBar?: React.ReactNode;
};

/**
 * Shared layout for surface-scoped route groups.
 *
 * Renders the surface-specific sidebar, header, and content area.
 * Note: Auth check should be done at the page level before rendering this layout.
 */
export function SurfaceLayout({
  children,
  surfaceId,
  user,
  commandBar,
}: SurfaceLayoutProps) {
  const accessibleSurfaces = readAccessibleSurfaces(user);
  const definition = SURFACES[surfaceId];
  const { collapsed, toggleCollapsed } = useSidebarState();

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
    <div 
      className={cn(
        "grid min-h-screen grid-cols-1 bg-app transition-all duration-300 ease-in-out",
        collapsed ? "lg:grid-cols-[64px_1fr]" : "lg:grid-cols-[260px_1fr]"
      )}
      suppressHydrationWarning
    >
      <SurfaceSidebar
        surface={surfaceId}
        accessibleSurfaces={accessibleSurfaces}
        collapsed={collapsed}
        onToggle={toggleCollapsed}
      />
      <div className="flex min-h-screen min-w-0 flex-col">
        <Header user={user} surfaceId={surfaceId} />
        {commandBar ? <div className="shrink-0">{commandBar}</div> : null}
        <main className="flex-1 px-4 py-5 md:px-8 md:py-8">{children}</main>
      </div>
    </div>
  );
}
