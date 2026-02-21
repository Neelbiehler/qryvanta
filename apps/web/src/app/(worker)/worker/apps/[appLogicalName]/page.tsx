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
import { apiServerFetch, type AppEntityBindingResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type WorkerAppHomePageProps = {
  params: Promise<{
    appLogicalName: string;
  }>;
};

export default async function WorkerAppHomePage({
  params,
}: WorkerAppHomePageProps) {
  const { appLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();
  const navigationResponse = await apiServerFetch(
    `/api/workspace/apps/${appLogicalName}/navigation`,
    cookieHeader,
  );

  if (navigationResponse.status === 401) {
    redirect("/login");
  }

  if (navigationResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Worker Apps"
        title="App Access"
        message="Your account does not have access to this app."
      />
    );
  }

  if (!navigationResponse.ok) {
    throw new Error("Failed to load app navigation");
  }

  const navigation =
    (await navigationResponse.json()) as AppEntityBindingResponse[];

  return (
    <div className="space-y-4">
      <PageHeader
        eyebrow="Worker Apps"
        title={appLogicalName}
        description="Select an entity area to view records and execute tasks."
        actions={
          <Link
            href="/worker/apps"
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Back to apps
          </Link>
        }
      />

      <div className="grid gap-4 xl:grid-cols-[300px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>App Queue</CardTitle>
            <CardDescription>
              Entity work areas available to your role.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <StatusBadge tone="neutral">
              Entities {navigation.length}
            </StatusBadge>
            <p className="text-sm text-zinc-600">
              Open an entity workspace to create, update, or inspect runtime
              records.
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Entity</TableHead>
                  <TableHead>Label</TableHead>
                  <TableHead>Open</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {navigation.length > 0 ? (
                  navigation.map((item) => (
                    <TableRow
                      key={`${item.app_logical_name}.${item.entity_logical_name}`}
                    >
                      <TableCell className="font-mono text-xs">
                        {item.entity_logical_name}
                      </TableCell>
                      <TableCell>
                        {item.navigation_label ?? item.entity_logical_name}
                      </TableCell>
                      <TableCell>
                        <Link
                          className={cn(
                            buttonVariants({ size: "sm", variant: "outline" }),
                          )}
                          href={`/worker/apps/${appLogicalName}/${item.entity_logical_name}`}
                        >
                          Open
                        </Link>
                      </TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell className="text-zinc-500" colSpan={3}>
                      No entities are configured for this app yet.
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
