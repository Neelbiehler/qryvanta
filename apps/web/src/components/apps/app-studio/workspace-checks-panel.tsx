import Link from "next/link";

import { Notice } from "@qryvanta/ui";

import type {
  PublishCheckCategoryDto,
  PublishCheckIssueResponse,
  PublishCheckSeverityDto,
} from "@/lib/api";

type WorkspaceChecksPanelProps = {
  publishCheckErrors: string[];
  workspaceIssues: PublishCheckIssueResponse[];
};

export function WorkspaceChecksPanel({
  publishCheckErrors,
  workspaceIssues,
}: WorkspaceChecksPanelProps) {
  const groupedWorkspaceIssues = groupIssuesBySeverityAndCategory(workspaceIssues);

  return (
    <>
      {publishCheckErrors.length > 0 ? (
        <Notice tone="error">
          <p className="font-semibold">App publish blockers</p>
          <ul className="mt-1 list-disc pl-5">
            {publishCheckErrors.map((error) => (
              <li key={error}>{error}</li>
            ))}
          </ul>
        </Notice>
      ) : null}

      {workspaceIssues.length > 0 ? (
        <Notice tone="error">
          <p className="font-semibold">Workspace publish blockers</p>
          {groupedWorkspaceIssues.map((severityGroup) => (
            <div
              key={`workspace-issues-severity-${severityGroup.severity}`}
              className="mt-3 rounded border border-zinc-200 bg-white p-2"
            >
              <p className="text-xs font-semibold uppercase tracking-wide">
                {severityGroup.severity} ({severityGroup.total})
              </p>
              {severityGroup.categories.map(({ category, items }) => (
                <div
                  key={`workspace-issues-${severityGroup.severity}-${category}`}
                  className="mt-2"
                >
                  <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                    {category}
                  </p>
                  <ul className="mt-1 list-disc pl-5">
                    {items.map((issue, index) => (
                      <li
                        key={`${issue.scope}-${issue.scope_logical_name}-${String(index)}`}
                      >
                        <span className="font-medium">{issue.scope}:</span>{" "}
                        {issue.scope_logical_name} - {issue.message}{" "}
                        {issue.dependency_path ? (
                          <span className="text-xs text-zinc-600">
                            [{issue.dependency_path}] 
                          </span>
                        ) : null}
                        {issue.fix_path ? (
                          <Link href={issue.fix_path} className="underline">
                            Open fix
                          </Link>
                        ) : null}
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          ))}
        </Notice>
      ) : null}
    </>
  );
}

function groupIssuesBySeverityAndCategory(
  issues: PublishCheckIssueResponse[],
): Array<{
  severity: PublishCheckSeverityDto;
  total: number;
  categories: Array<{
    category: PublishCheckCategoryDto;
    items: PublishCheckIssueResponse[];
  }>;
}> {
  const severityGroups = new Map<
    PublishCheckSeverityDto,
    Map<PublishCheckCategoryDto, PublishCheckIssueResponse[]>
  >();

  for (const issue of issues) {
    const categoryMap =
      severityGroups.get(issue.severity) ??
      new Map<PublishCheckCategoryDto, PublishCheckIssueResponse[]>();
    const existing = categoryMap.get(issue.category) ?? [];
    existing.push(issue);
    categoryMap.set(issue.category, existing);
    severityGroups.set(issue.severity, categoryMap);
  }

  return Array.from(severityGroups.entries())
    .sort(
      ([leftSeverity], [rightSeverity]) =>
        severitySortWeight(leftSeverity) - severitySortWeight(rightSeverity),
    )
    .map(([severity, categories]) => ({
      severity,
      total: Array.from(categories.values()).reduce(
        (sum, bucket) => sum + bucket.length,
        0,
      ),
      categories: Array.from(categories.entries()).map(([category, items]) => ({
        category,
        items,
      })),
    }));
}

function severitySortWeight(severity: PublishCheckSeverityDto): number {
  switch (severity) {
    case "error":
      return 0;
    default:
      return 10;
  }
}
