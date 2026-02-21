import Link from "next/link";

import { Sidebar as SidebarContainer } from "@qryvanta/ui";

import {
  type SurfaceDefinition,
  type SurfaceId,
  SURFACES,
  SURFACE_ORDER,
} from "@/lib/surfaces";

type SurfaceSidebarProps = {
  surface: SurfaceId;
  accessibleSurfaces: string[];
};

export function SurfaceSidebar({
  surface,
  accessibleSurfaces,
}: SurfaceSidebarProps) {
  const definition: SurfaceDefinition = SURFACES[surface];

  return (
    <SidebarContainer className="flex h-full flex-col p-5">
      <div className="mb-8">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-emerald-700">
          Qryvanta
        </p>
        <h2 className="mt-2 font-serif text-xl font-semibold text-zinc-900">
          {definition.label}
        </h2>
        <p className="mt-1 text-xs text-zinc-500">{definition.description}</p>
      </div>

      <nav className="space-y-2">
        {definition.navigationItems.map((item) => (
          <Link
            key={item.href}
            href={item.href}
            className="block rounded-md bg-white px-3 py-2 text-sm font-medium text-zinc-900 shadow-sm transition hover:bg-emerald-100"
          >
            {item.label}
          </Link>
        ))}
      </nav>

      <div className="mt-auto pt-8">
        <p className="mb-2 text-xs font-semibold uppercase tracking-[0.14em] text-zinc-400">
          Surfaces
        </p>
        <nav className="space-y-1">
          {SURFACE_ORDER.filter((id) => accessibleSurfaces.includes(id)).map(
            (id) => {
              const target = SURFACES[id];
              const isCurrent = id === surface;
              return (
                <Link
                  key={id}
                  href={target.basePath}
                  className={
                    isCurrent
                      ? "block rounded-md bg-emerald-100 px-3 py-1.5 text-xs font-semibold text-emerald-800"
                      : "block rounded-md px-3 py-1.5 text-xs text-zinc-600 transition hover:bg-zinc-100"
                  }
                >
                  {target.label}
                </Link>
              );
            },
          )}
        </nav>
      </div>
    </SidebarContainer>
  );
}
