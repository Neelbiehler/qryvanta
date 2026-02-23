import { WorkflowStudioShell } from "../../workflow-studio-shell";
import { redirect } from "next/navigation";

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

  return (
    <WorkflowStudioShell
      initialSelectedWorkflow={workflowLogicalName}
      initialWorkspaceMode="history"
    />
  );
}
