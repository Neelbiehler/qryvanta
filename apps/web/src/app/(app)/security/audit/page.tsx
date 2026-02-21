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
} from "@qryvanta/ui";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiServerFetch, type AuditLogEntryResponse } from "@/lib/api";

type AuditLogPageProps = {
  searchParams?: Promise<{
    limit?: string;
    offset?: string;
    action?: string;
    subject?: string;
  }>;
};

export default async function AuditLogPage({
  searchParams,
}: AuditLogPageProps) {
  const resolvedParams = (await searchParams) ?? {};
  const limit = Number.parseInt(resolvedParams.limit ?? "50", 10);
  const offset = Number.parseInt(resolvedParams.offset ?? "0", 10);
  const action = resolvedParams.action?.trim() ?? "";
  const subject = resolvedParams.subject?.trim() ?? "";

  const safeLimit = Number.isNaN(limit)
    ? 50
    : Math.max(1, Math.min(limit, 100));
  const safeOffset = Number.isNaN(offset) ? 0 : Math.max(0, offset);

  const query = new URLSearchParams({
    limit: String(safeLimit),
    offset: String(safeOffset),
  });
  if (action) query.set("action", action);
  if (subject) query.set("subject", subject);

  const cookieHeader = (await cookies()).toString();
  const response = await apiServerFetch(
    `/api/security/audit-log?${query.toString()}`,
    cookieHeader,
  );

  if (response.status === 401) {
    redirect("/login");
  }

  if (response.status === 403) {
    return (
      <AccessDeniedCard
        section="Security"
        title="Audit Log"
        message="Your account is authenticated but does not have audit log read permissions."
      />
    );
  }

  if (!response.ok) {
    throw new Error("Failed to load audit log");
  }

  const entries = (await response.json()) as AuditLogEntryResponse[];
  const previousOffset = Math.max(0, safeOffset - safeLimit);
  const nextOffset = safeOffset + safeLimit;

  const previousParams = new URLSearchParams(query);
  previousParams.set("offset", String(previousOffset));

  const nextParams = new URLSearchParams(query);
  nextParams.set("offset", String(nextOffset));

  return (
    <Card>
      <CardHeader>
        <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">
          Security
        </p>
        <CardTitle className="font-serif text-3xl">Audit Log</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <form className="grid gap-3 rounded-md border border-emerald-100 bg-white p-3 md:grid-cols-4">
          <input
            className="rounded-md border border-emerald-100 px-3 py-2 text-sm"
            defaultValue={action}
            name="action"
            placeholder="Filter action"
          />
          <input
            className="rounded-md border border-emerald-100 px-3 py-2 text-sm"
            defaultValue={subject}
            name="subject"
            placeholder="Filter subject"
          />
          <input
            className="rounded-md border border-emerald-100 px-3 py-2 text-sm"
            defaultValue={String(safeLimit)}
            name="limit"
            placeholder="Rows"
          />
          <input
            className="rounded-md border border-emerald-100 px-3 py-2 text-sm"
            defaultValue="0"
            name="offset"
            placeholder="Offset"
            type="hidden"
          />
          <button
            className="md:col-span-4 rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm font-medium text-emerald-800"
            type="submit"
          >
            Apply Filters
          </button>
        </form>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Time (UTC)</TableHead>
              <TableHead>Subject</TableHead>
              <TableHead>Action</TableHead>
              <TableHead>Resource</TableHead>
              <TableHead>Detail</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {entries.length > 0 ? (
              entries.map((entry) => (
                <TableRow key={entry.event_id}>
                  <TableCell className="font-mono text-xs">
                    {entry.created_at}
                  </TableCell>
                  <TableCell>{entry.subject}</TableCell>
                  <TableCell className="font-mono text-xs">
                    {entry.action}
                  </TableCell>
                  <TableCell className="font-mono text-xs">
                    {entry.resource_type}:{entry.resource_id}
                  </TableCell>
                  <TableCell>{entry.detail ?? "-"}</TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell className="text-zinc-500" colSpan={5}>
                  No audit entries found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>

        <div className="flex items-center justify-between">
          <Link
            className="rounded-md border border-emerald-100 bg-white px-3 py-2 text-sm"
            href={`/security/audit?${previousParams.toString()}`}
          >
            Previous
          </Link>
          <p className="text-xs text-zinc-500">Offset {safeOffset}</p>
          <Link
            className="rounded-md border border-emerald-100 bg-white px-3 py-2 text-sm"
            href={`/security/audit?${nextParams.toString()}`}
          >
            Next
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
