import Link from "next/link";

import { Sidebar as SidebarContainer } from "@qryvanta/ui";

export function Sidebar() {
  return (
    <SidebarContainer className="h-full p-5">
      <div className="mb-8">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-emerald-700">
          Qryvanta
        </p>
        <h2 className="mt-2 font-serif text-xl font-semibold text-zinc-900">
          Foundation
        </h2>
      </div>

      <nav className="space-y-2">
        <Link
          href="/entities"
          className="block rounded-md bg-white px-3 py-2 text-sm font-medium text-zinc-900 shadow-sm transition hover:bg-emerald-100"
        >
          Entities
        </Link>
        <Link
          href="/apps"
          className="block rounded-md bg-white px-3 py-2 text-sm font-medium text-zinc-900 shadow-sm transition hover:bg-emerald-100"
        >
          Apps
        </Link>
        <Link
          href="/security/roles"
          className="block rounded-md bg-white px-3 py-2 text-sm font-medium text-zinc-900 shadow-sm transition hover:bg-emerald-100"
        >
          Roles
        </Link>
        <Link
          href="/security/audit"
          className="block rounded-md bg-white px-3 py-2 text-sm font-medium text-zinc-900 shadow-sm transition hover:bg-emerald-100"
        >
          Audit Log
        </Link>
        <Link
          href="/security/account"
          className="block rounded-md bg-white px-3 py-2 text-sm font-medium text-zinc-900 shadow-sm transition hover:bg-emerald-100"
        >
          Security Settings
        </Link>
      </nav>
    </SidebarContainer>
  );
}
