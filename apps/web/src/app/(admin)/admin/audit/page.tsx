import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  PageHeader,
  StatusBadge,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@qryvanta/ui";

import { AuditControlsPanel } from "@/components/security/audit-controls-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type AuditLogEntryResponse,
  type AuditRetentionPolicyResponse,
} from "@/lib/api";

type AuditLogPageProps = {
  searchParams?: Promise<{
    limit?: string;
    offset?: string;
    action?: string;
    subject?: string;
  }>;
};

export default async function AdminAuditLogPage({
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
  const [response, retentionResponse] = await Promise.all([
    apiServerFetch(`/api/security/audit-log?${query.toString()}`, cookieHeader),
    apiServerFetch("/api/security/audit-retention-policy", cookieHeader),
  ]);

  if (response.status === 401) {
    redirect("/login");
  }

  if (response.status === 403) {
    return (
      <AccessDeniedCard
        section="Admin Center"
        title="Audit Log"
        message="Your account does not have audit log read permissions."
      />
    );
  }

  if (!response.ok) {
    throw new Error("Failed to load audit log");
  }

  if (retentionResponse.status === 401) {
    redirect("/login");
  }

  if (retentionResponse.status !== 200 && retentionResponse.status !== 403) {
    throw new Error("Failed to load audit retention policy");
  }

  const entries = (await response.json()) as AuditLogEntryResponse[];
  const retentionPolicy = retentionResponse.ok
    ? ((await retentionResponse.json()) as AuditRetentionPolicyResponse)
    : null;
  const previousOffset = Math.max(0, safeOffset - safeLimit);
  const nextOffset = safeOffset + safeLimit;

  const previousParams = new URLSearchParams(query);
  previousParams.set("offset", String(previousOffset));

  const nextParams = new URLSearchParams(query);
  nextParams.set("offset", String(nextOffset));

  return (
    <div className="space-y-4">
      <PageHeader
        eyebrow="Admin Center"
        title="Audit Log"
        description="Filter security and governance events across your tenant."
      />

      <div className="grid gap-4 xl:grid-cols-[300px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Audit Scope</CardTitle>
            <CardDescription>
              Current result-window and policy context.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <StatusBadge tone="neutral">Rows {entries.length}</StatusBadge>
            <StatusBadge tone="neutral">Limit {safeLimit}</StatusBadge>
            <StatusBadge tone="neutral">Offset {safeOffset}</StatusBadge>
            <StatusBadge tone={action ? "success" : "neutral"}>
              Action {action || "any"}
            </StatusBadge>
            <StatusBadge tone={subject ? "success" : "neutral"}>
              Subject {subject || "any"}
            </StatusBadge>
            <StatusBadge tone={retentionPolicy ? "warning" : "neutral"}>
              Retention {retentionPolicy?.retention_days ?? "n/a"}d
            </StatusBadge>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="space-y-4 pt-6">
            <AuditControlsPanel
              queryString={query.toString()}
              retentionDays={retentionPolicy?.retention_days ?? null}
            />

            <form className="grid gap-3 rounded-md border border-emerald-100 bg-white p-3 md:grid-cols-4">
              <Input
                defaultValue={action}
                name="action"
                placeholder="Filter action"
              />
              <Input
                defaultValue={subject}
                name="subject"
                placeholder="Filter subject"
              />
              <Input
                defaultValue={String(safeLimit)}
                name="limit"
                placeholder="Rows"
              />
              <Input
                defaultValue="0"
                name="offset"
                placeholder="Offset"
                type="hidden"
              />
              <Button className="md:col-span-4" type="submit" variant="outline">
                Apply Filters
              </Button>
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
                href={`/admin/audit?${previousParams.toString()}`}
              >
                Previous
              </Link>
              <p className="text-xs text-zinc-500">Offset {safeOffset}</p>
              <Link
                className="rounded-md border border-emerald-100 bg-white px-3 py-2 text-sm"
                href={`/admin/audit?${nextParams.toString()}`}
              >
                Next
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
