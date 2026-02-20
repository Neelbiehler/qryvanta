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
import { apiServerFetch, type EntityResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

export default async function EntitiesPage() {
  const cookieHeader = (await cookies()).toString();
  const response = await apiServerFetch("/api/entities", cookieHeader);

  if (response.status === 401) {
    redirect("/login");
  }

  if (!response.ok) {
    throw new Error("Failed to load entities");
  }

  const entities = (await response.json()) as EntityResponse[];

  return (
    <Card>
      <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">Metadata</p>
          <CardTitle className="font-serif text-3xl">Entities</CardTitle>
        </div>
        <Link href="/entities/new" className={cn(buttonVariants())}>
          New Entity
        </Link>
      </CardHeader>

      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Logical Name</TableHead>
              <TableHead>Display Name</TableHead>
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
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell className="text-zinc-500" colSpan={2}>
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
