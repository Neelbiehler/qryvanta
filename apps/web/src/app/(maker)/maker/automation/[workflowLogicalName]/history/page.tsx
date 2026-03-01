import { cookies } from "next/headers";
import { notFound, redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@qryvanta/ui";

import { WorkflowHistoryPanel } from "@/components/automation/workflow-history-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type WorkflowResponse,
  type WorkflowRunResponse,
} from "@/lib/api";

type AutomationWorkflowHistoryPageProps = {
  params: Promise<{
    workflowLogicalName: string;
  }>;
};

export default async function AutomationWorkflowHistoryPage({
  params,
}: AutomationWorkflowHistoryPageProps) {
  const { workflowLogicalName } = await params;

  if (workflowLogicalName === "new") {
    redirect("/maker/automation");
  }

  const cookieHeader = (await cookies()).toString();

  const [workflowsResponse, runsResponse] = await Promise.all([
    apiServerFetch("/api/workflows", cookieHeader),
    apiServerFetch(
      `/api/workflows/runs?workflow_logical_name=${encodeURIComponent(workflowLogicalName)}&limit=100&offset=0`,
      cookieHeader,
    ),
  ]);

  if (workflowsResponse.status === 401 || runsResponse.status === 401) {
    redirect("/login");
  }

  if (workflowsResponse.status === 403 || runsResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Workflow History"
        message="Your account does not have the required permissions for workflow automation management."
      />
    );
  }

  if (!workflowsResponse.ok || !runsResponse.ok) {
    const [workflowsError, runsError] = await Promise.all([
      workflowsResponse.ok ? Promise.resolve("") : workflowsResponse.text(),
      runsResponse.ok ? Promise.resolve("") : runsResponse.text(),
    ]);

    return (
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>History unavailable</CardTitle>
            <CardDescription>
              The workflow API returned an unexpected response.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="font-mono text-xs text-zinc-600">
              /api/workflows status: {workflowsResponse.status}
            </p>
            <p className="font-mono text-xs text-zinc-600">
              /api/workflows/runs status: {runsResponse.status}
            </p>
            {workflowsError ? (
              <p className="font-mono text-xs text-zinc-500">{workflowsError}</p>
            ) : null}
            {runsError ? (
              <p className="font-mono text-xs text-zinc-500">{runsError}</p>
            ) : null}
          </CardContent>
        </Card>
      </div>
    );
  }

  const workflows = (await workflowsResponse.json()) as WorkflowResponse[];
  const runs = (await runsResponse.json()) as WorkflowRunResponse[];

  const workflow = workflows.find((w) => w.logical_name === workflowLogicalName);
  if (!workflow) {
    notFound();
  }

  return <WorkflowHistoryPanel workflow={workflow} runs={runs} />;
}
