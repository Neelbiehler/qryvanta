import { WorkflowStudioShell } from "../../workflow-studio-shell";

type AutomationWorkflowEditPageProps = {
  params: Promise<{
    workflowLogicalName: string;
  }>;
};

export default async function AutomationWorkflowEditPage({
  params,
}: AutomationWorkflowEditPageProps) {
  const { workflowLogicalName } = await params;
  const initialSelectedWorkflow =
    workflowLogicalName === "new" ? undefined : workflowLogicalName;

  return (
    <WorkflowStudioShell
      initialSelectedWorkflow={initialSelectedWorkflow}
      initialWorkspaceMode="edit"
    />
  );
}
