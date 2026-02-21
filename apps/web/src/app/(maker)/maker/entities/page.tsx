import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardHeader,
  PageHeader,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  buttonVariants,
} from "@qryvanta/ui";
import { apiServerFetch, type EntityResponse } from "@/lib/api";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { cn } from "@/lib/utils";

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
    throw new Error("Failed to load entities");
  }

  const entities = (await response.json()) as EntityResponse[];

  return (
    <Card>
      <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <PageHeader
          eyebrow="Maker Center"
          title="Entities"
          description="Model tenant entities and open the runtime workbench."
          actions={
            <Link href="/maker/entities/new" className={cn(buttonVariants())}>
              New Entity
            </Link>
          }
        />
      </CardHeader>

      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Logical Name</TableHead>
              <TableHead>Display Name</TableHead>
              <TableHead>Runtime</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {entities.length > 0 ? (
              entities.map((entity) => (
                <TableRow key={entity.logical_name}>
                  <TableCell className="font-mono text-xs">
                    {entity.logical_name}
                  </TableCell>
                  <TableCell>{entity.display_name}</TableCell>
                  <TableCell>
                    <Link
                      className={cn(
                        buttonVariants({ size: "sm", variant: "outline" }),
                      )}
                      href={`/maker/entities/${entity.logical_name}`}
                    >
                      Open
                    </Link>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell className="text-zinc-500" colSpan={3}>
                  No entities yet.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
