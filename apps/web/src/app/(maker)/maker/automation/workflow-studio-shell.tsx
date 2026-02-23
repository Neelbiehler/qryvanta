import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@qryvanta/ui";

import {
  WorkflowStudioPanel,
  type WorkflowWorkspaceMode,
} from "@/components/automation/workflow-studio-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type WorkflowResponse,
  type WorkflowRunResponse,
} from "@/lib/api";

type WorkflowStudioShellProps = {
  initialSelectedWorkflow?: string;
  initialWorkspaceMode?: WorkflowWorkspaceMode;
};

export async function WorkflowStudioShell({
  initialSelectedWorkflow,
  initialWorkspaceMode,
}: WorkflowStudioShellProps) {
  const cookieHeader = (await cookies()).toString();

  const [workflowsResponse, runsResponse] = await Promise.all([
    apiServerFetch("/api/workflows", cookieHeader),
    apiServerFetch("/api/workflows/runs?limit=25&offset=0", cookieHeader),
  ]);

  if (workflowsResponse.status === 401 || runsResponse.status === 401) {
    redirect("/login");
  }

  if (workflowsResponse.status === 403 || runsResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Automation"
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
            <CardTitle>Automation data unavailable</CardTitle>
            <CardDescription>
              The workflow API returned an unexpected response. Confirm API migrations are applied and the API server is running the latest code.
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

  return (
    <div className="-mx-4 -my-5 h-[calc(100vh-64px)] md:-mx-8 md:-my-8">
      <WorkflowStudioPanel
        workflows={workflows}
        runs={runs}
        initialSelectedWorkflow={initialSelectedWorkflow}
        initialWorkspaceMode={initialWorkspaceMode}
      />
    </div>
  );
}
