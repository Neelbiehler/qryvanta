import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  StatusBadge,
  buttonVariants,
} from "@qryvanta/ui";

import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiServerFetch, type BusinessRuleResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerEntityBusinessRulesPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerEntityBusinessRulesPage({
  params,
}: MakerEntityBusinessRulesPageProps) {
  const { entityLogicalName } = await params;
  const cookieHeader = (await cookies()).toString();

  const rulesResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/business-rules`,
    cookieHeader,
  );
  if (rulesResponse.status === 401) {
    redirect("/login");
  }
  if (rulesResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Business Rules"
        message="Your account does not have metadata field read permissions."
      />
    );
  }
  if (!rulesResponse.ok) {
    throw new Error("Failed to load business rules.");
  }

  const rules = (await rulesResponse.json()) as BusinessRuleResponse[];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Maker Center
            </p>
            <CardTitle className="font-serif text-3xl">{entityLogicalName} Business Rules</CardTitle>
            <CardDescription>
              Configure condition-action business logic for this entity.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Rules {rules.length}</StatusBadge>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/business-rules/new`}
              className={cn(buttonVariants())}
            >
              New Rule
            </Link>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}`}
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to Entity
            </Link>
          </div>
        </CardHeader>
      </Card>

      {rules.length > 0 ? (
        <div className="grid gap-4 lg:grid-cols-2">
          {rules.map((rule) => (
            <Card key={rule.logical_name}>
              <CardHeader>
                <CardTitle>{rule.display_name}</CardTitle>
                <CardDescription className="font-mono text-xs">{rule.logical_name}</CardDescription>
              </CardHeader>
              <CardContent className="flex items-center gap-2">
                <StatusBadge tone="neutral">{rule.scope}</StatusBadge>
                <StatusBadge tone={rule.is_active ? "success" : "warning"}>
                  {rule.is_active ? "Active" : "Inactive"}
                </StatusBadge>
                <Link
                  href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/business-rules/${encodeURIComponent(rule.logical_name)}`}
                  className={cn(buttonVariants({ size: "sm", variant: "outline" }), "ml-auto")}
                >
                  Open Designer
                </Link>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>No business rules defined yet</CardTitle>
            <CardDescription>
              Create your first rule to drive required fields, visibility, and default values.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/business-rules/new`}
              className={cn(buttonVariants())}
            >
              Create First Rule
            </Link>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
