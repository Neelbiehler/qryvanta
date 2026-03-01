import { cookies, headers } from "next/headers";
import { redirect } from "next/navigation";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Notice,
  PageHeader,
  StatusBadge,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@qryvanta/ui";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type QrywellSearchAnalyticsResponse,
  type QrywellSyncHealthResponse,
} from "@/lib/api";

async function triggerSyncAllAction(formData: FormData) {
  "use server";

  const cookieHeader = (await cookies()).toString();
  const incomingHeaders = await headers();
  const forwardedOrigin =
    incomingHeaders.get("origin") ?? process.env.NEXT_PUBLIC_FRONTEND_URL ?? "http://localhost:3000";
  const forwardedReferer =
    incomingHeaders.get("referer") ?? `${forwardedOrigin.replace(/\/$/, "")}/admin/search-sync`;
  const limit = Number.parseInt(String(formData.get("limit") ?? "200"), 10);
  const offset = Number.parseInt(String(formData.get("offset") ?? "0"), 10);
  const safeLimit = Number.isNaN(limit) ? 200 : Math.max(1, Math.min(limit, 1000));
  const safeOffset = Number.isNaN(offset) ? 0 : Math.max(0, offset);

  const response = await apiServerFetch("/api/search/qrywell/sync-all", cookieHeader, {
    method: "POST",
    headers: {
      origin: forwardedOrigin,
      referer: forwardedReferer,
    },
    body: JSON.stringify({
      limit: safeLimit,
      offset: safeOffset,
    }),
  });

  if (!response.ok) {
    redirect(`/admin/search-sync?sync=error&status=${response.status}`);
  }

  redirect("/admin/search-sync?sync=ok");
}

type AdminSearchSyncPageProps = {
  searchParams?: Promise<{ sync?: string; status?: string }>;
};

export default async function AdminSearchSyncPage({ searchParams }: AdminSearchSyncPageProps) {
  const status = (await searchParams) ?? {};
  const cookieHeader = (await cookies()).toString();
  const response = await apiServerFetch("/api/search/qrywell/queue-health", cookieHeader);
  const analyticsResponse = await apiServerFetch("/api/search/qrywell/analytics?window_days=14", cookieHeader);

  if (response.status === 401) {
    redirect("/login");
  }

  if (response.status === 403) {
    return (
      <AccessDeniedCard
        section="Admin Center"
        title="Search Sync"
        message="Your account does not have access to search sync monitoring."
      />
    );
  }

  if (!response.ok) {
    throw new Error("Failed to load Qrywell sync health");
  }
  if (!analyticsResponse.ok) {
    throw new Error("Failed to load Qrywell analytics");
  }

  const health = (await response.json()) as QrywellSyncHealthResponse;
  const analytics = (await analyticsResponse.json()) as QrywellSearchAnalyticsResponse;

  return (
    <div className="space-y-4">
      {status.sync === "ok" ? (
        <Notice tone="success">Full backfill started successfully.</Notice>
      ) : null}
      {status.sync === "error" ? (
        <Notice tone="error">
          Failed to start full backfill (status {status.status ?? "unknown"}).
        </Notice>
      ) : null}
      <PageHeader
        eyebrow="Admin Center"
        title="Qrywell Sync Health"
        description="Monitor queued, processing, and failed sync jobs between Qryvanta and Qrywell."
      />

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Pending</CardTitle>
            <CardDescription>Waiting for worker processing.</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone="neutral">{health.pending_jobs}</StatusBadge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Processing</CardTitle>
            <CardDescription>Currently claimed by sync worker.</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone="info">{health.processing_jobs}</StatusBadge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Failed</CardTitle>
            <CardDescription>Reached retry threshold or blocked.</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone={health.failed_jobs > 0 ? "critical" : "success"}>
              {health.failed_jobs}
            </StatusBadge>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Total Succeeded</CardTitle>
            <CardDescription>Completed sync jobs since tracking started.</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone="success">{health.total_succeeded}</StatusBadge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Total Failed Attempts</CardTitle>
            <CardDescription>All failed processing attempts (including retries).</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone={health.total_failed > 0 ? "warning" : "neutral"}>
              {health.total_failed}
            </StatusBadge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Last Activity</CardTitle>
            <CardDescription>Most recent worker processing timestamps.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-1 text-sm text-zinc-600">
            <p>Attempt: {health.last_attempt_at ?? "none"}</p>
            <p>Success: {health.last_success_at ?? "none"}</p>
            <p>Failure: {health.last_failure_at ?? "none"}</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Backfill All Data</CardTitle>
          <CardDescription>
            Trigger a full-tenant sync across all entities to refresh Qrywell index content.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form action={triggerSyncAllAction} className="grid gap-3 md:grid-cols-3">
            <Input name="limit" defaultValue="200" placeholder="Limit per entity" />
            <Input name="offset" defaultValue="0" placeholder="Offset" />
            <Button type="submit">Sync All Now</Button>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Recent Failed Jobs</CardTitle>
          <CardDescription>Most recent sync failures for this tenant.</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Updated (UTC)</TableHead>
                <TableHead>Entity</TableHead>
                <TableHead>Record</TableHead>
                <TableHead>Operation</TableHead>
                <TableHead>Attempts</TableHead>
                <TableHead>Error</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {health.failed_recent.length > 0 ? (
                health.failed_recent.map((job) => (
                  <TableRow key={job.job_id}>
                    <TableCell className="font-mono text-xs">{job.updated_at}</TableCell>
                    <TableCell>{job.entity_logical_name}</TableCell>
                    <TableCell className="font-mono text-xs">{job.record_id}</TableCell>
                    <TableCell>{job.operation}</TableCell>
                    <TableCell>
                      {job.attempt_count}/{job.max_attempts}
                    </TableCell>
                    <TableCell>{job.last_error ?? "-"}</TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell className="text-zinc-500" colSpan={6}>
                    No failed sync jobs.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Total Queries (14d)</CardTitle>
            <CardDescription>Search requests observed in selected analytics window.</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone="info">{analytics.total_queries}</StatusBadge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Total Clicks (14d)</CardTitle>
            <CardDescription>Search result interactions recorded from worker UI.</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone="success">{analytics.total_clicks}</StatusBadge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Observed Query CTR</CardTitle>
            <CardDescription>Queries with at least one click over total queries.</CardDescription>
          </CardHeader>
          <CardContent>
            <StatusBadge tone="neutral">
              {analytics.total_queries > 0
                ? `${((Number(analytics.total_clicks) / Number(analytics.total_queries)) * 100).toFixed(1)}%`
                : "0%"}
            </StatusBadge>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Top Queries</CardTitle>
            <CardDescription>Most frequent normalized query strings.</CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Query</TableHead>
                  <TableHead>Runs</TableHead>
                  <TableHead>Clicks</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {analytics.top_queries.length > 0 ? (
                  analytics.top_queries.map((row) => (
                    <TableRow key={row.query}>
                      <TableCell className="font-mono text-xs">{row.query}</TableCell>
                      <TableCell>{row.runs}</TableCell>
                      <TableCell>{row.clicks}</TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell className="text-zinc-500" colSpan={3}>
                      No query analytics yet.
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Rank Click Share</CardTitle>
            <CardDescription>Distribution of clicks by result rank position.</CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Rank</TableHead>
                  <TableHead>Clicks</TableHead>
                  <TableHead>Share</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {analytics.rank_metrics.length > 0 ? (
                  analytics.rank_metrics.map((row) => (
                    <TableRow key={row.rank}>
                      <TableCell>#{row.rank}</TableCell>
                      <TableCell>{row.clicks}</TableCell>
                      <TableCell>{(row.click_share * 100).toFixed(1)}%</TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell className="text-zinc-500" colSpan={3}>
                      No click distribution yet.
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Zero-Click Queries</CardTitle>
            <CardDescription>Queries with results but no recorded clicks.</CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Query</TableHead>
                  <TableHead>Runs</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {analytics.zero_click_queries.length > 0 ? (
                  analytics.zero_click_queries.map((row) => (
                    <TableRow key={row.query}>
                      <TableCell className="font-mono text-xs">{row.query}</TableCell>
                      <TableCell>{row.runs}</TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell className="text-zinc-500" colSpan={2}>
                      No zero-click queries in this window.
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Low-Relevance Clicks</CardTitle>
            <CardDescription>
              Clicked results with low average score, useful for ranking review.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Query</TableHead>
                  <TableHead>Title</TableHead>
                  <TableHead>Avg score</TableHead>
                  <TableHead>Clicks</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {analytics.low_relevance_clicks.length > 0 ? (
                  analytics.low_relevance_clicks.map((row) => (
                    <TableRow key={`${row.query}-${row.title}`}>
                      <TableCell className="font-mono text-xs">{row.query}</TableCell>
                      <TableCell>{row.title}</TableCell>
                      <TableCell>{(row.avg_score * 100).toFixed(1)}%</TableCell>
                      <TableCell>{row.clicks}</TableCell>
                    </TableRow>
                  ))
                ) : (
                  <TableRow>
                    <TableCell className="text-zinc-500" colSpan={4}>
                      No low-relevance click signals yet.
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
