import { redirect } from "next/navigation";

import { WorkflowStudioShell } from "../../../workflow-studio-shell";

type AutomationWorkflowStepHistoryPageProps = {
  params: Promise<{
    workflowLogicalName: string;
  }>;
  searchParams: Promise<{
    run_id?: string;
  }>;
};

export default async function AutomationWorkflowStepHistoryPage({
  params,
  searchParams,
}: AutomationWorkflowStepHistoryPageProps) {
  const { workflowLogicalName } = await params;
  const { run_id: runId } = await searchParams;

  if (workflowLogicalName === "new") {
    redirect("/maker/automation");
  }

  return (
    <WorkflowStudioShell
      initialSelectedWorkflow={workflowLogicalName}
      initialWorkspaceMode="history"
      runsWorkflowLogicalNameFilter={workflowLogicalName}
      runsLimit={100}
      initialHistoryRunId={runId}
    />
  );
}
