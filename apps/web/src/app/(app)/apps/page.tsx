import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  buttonVariants,
} from "@qryvanta/ui";

import { AppStudioPanel } from "@/components/apps/app-studio-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AppResponse,
  type EntityResponse,
  type RoleResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

export default async function AppsPage() {
  const cookieHeader = (await cookies()).toString();

  const [
    workspaceAppsResponse,
    adminAppsResponse,
    entitiesResponse,
    rolesResponse,
  ] = await Promise.all([
    apiServerFetch("/api/workspace/apps", cookieHeader),
    apiServerFetch("/api/apps", cookieHeader),
    apiServerFetch("/api/entities", cookieHeader),
    apiServerFetch("/api/security/roles", cookieHeader),
  ]);

  if (workspaceAppsResponse.status === 401) {
    redirect("/login");
  }

  if (workspaceAppsResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Workspace"
        title="Apps"
        message="Your account is authenticated but is not assigned to any app yet."
      />
    );
  }

  if (!workspaceAppsResponse.ok) {
    throw new Error("Failed to load workspace apps");
  }

  const workspaceApps = (await workspaceAppsResponse.json()) as AppResponse[];

  let adminApps: AppResponse[] = [];
  let entities: EntityResponse[] = [];
  let roles: RoleResponse[] = [];
  const hasAppStudioAccess =
    adminAppsResponse.ok && entitiesResponse.ok && rolesResponse.ok;

  if (hasAppStudioAccess) {
    adminApps = (await adminAppsResponse.json()) as AppResponse[];
    entities = (await entitiesResponse.json()) as EntityResponse[];
    roles = (await rolesResponse.json()) as RoleResponse[];
  }

  return (
    <div className="space-y-8">
      <Card>
        <CardHeader>
          <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">
            Workspace
          </p>
          <CardTitle className="font-serif text-3xl">My Apps</CardTitle>
        </CardHeader>

        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>App</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Open</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {workspaceApps.length > 0 ? (
                workspaceApps.map((app) => (
                  <TableRow key={app.logical_name}>
                    <TableCell>
                      <p className="font-medium text-zinc-900">
                        {app.display_name}
                      </p>
                      <p className="font-mono text-xs text-zinc-500">
                        {app.logical_name}
                      </p>
                    </TableCell>
                    <TableCell>{app.description ?? "-"}</TableCell>
                    <TableCell>
                      <Link
                        className={cn(
                          buttonVariants({ size: "sm", variant: "outline" }),
                        )}
                        href={`/apps/${app.logical_name}`}
                      >
                        Open
                      </Link>
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell colSpan={3} className="text-zinc-500">
                    No apps assigned yet.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {hasAppStudioAccess ? (
        <Card>
          <CardHeader>
            <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">
              Administration
            </p>
            <CardTitle className="font-serif text-3xl">App Studio</CardTitle>
          </CardHeader>
          <CardContent>
            <AppStudioPanel
              apps={adminApps}
              entities={entities}
              roles={roles}
            />
          </CardContent>
        </Card>
      ) : null}
    </div>
  );
}
