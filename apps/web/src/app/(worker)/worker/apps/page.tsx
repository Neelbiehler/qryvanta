import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  PageHeader,
  StatusBadge,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  buttonVariants,
} from "@qryvanta/ui";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiServerFetch, type AppResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

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
    throw new Error("Failed to load workspace apps");
  }

  const workspaceApps = (await workspaceAppsResponse.json()) as AppResponse[];

  return (
    <div className="space-y-4">
      <PageHeader
        eyebrow="Worker Apps"
        title="My Apps"
        description="Open assigned business applications and start daily work."
      />

      <div className="grid gap-4 xl:grid-cols-[300px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Work Queue</CardTitle>
            <CardDescription>
              Applications available for your current role set.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <StatusBadge tone="neutral">
              Assigned Apps {workspaceApps.length}
            </StatusBadge>
            <p className="text-sm text-zinc-600">
              Open an app to access entity workspaces and runtime data
              operations.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
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
                          href={`/worker/apps/${app.logical_name}`}
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
      </div>
    </div>
  );
}
