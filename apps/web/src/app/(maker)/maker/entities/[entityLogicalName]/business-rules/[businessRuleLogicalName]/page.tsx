import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { Card, CardDescription, CardHeader, CardTitle, buttonVariants } from "@qryvanta/ui";

import { BusinessRuleDesignerPanel } from "@/components/entities/business-rules/business-rule-designer-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type BusinessRuleResponse,
  type FieldResponse,
  type FormResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type MakerEntityBusinessRuleDesignerPageProps = {
  params: Promise<{
    entityLogicalName: string;
    businessRuleLogicalName: string;
  }>;
};

export default async function MakerEntityBusinessRuleDesignerPage({
  params,
}: MakerEntityBusinessRuleDesignerPageProps) {
  const { entityLogicalName, businessRuleLogicalName } = await params;
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
        title="Business Rule Designer"
        message="Your account does not have metadata field permissions."
      />
    );
  }
  if (!rulesResponse.ok) {
    throw new Error("Failed to load business rules.");
  }
  const rules = (await rulesResponse.json()) as BusinessRuleResponse[];

  const ruleResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/business-rules/${businessRuleLogicalName}`,
    cookieHeader,
  );
  if (ruleResponse.status === 401) {
    redirect("/login");
  }
  if (ruleResponse.status === 403) {
    return (
      <AccessDeniedCard
        section="Maker Center"
        title="Business Rule Designer"
        message="Your account does not have metadata field permissions."
      />
    );
  }
  if (ruleResponse.status === 404) {
    redirect(`/maker/entities/${encodeURIComponent(entityLogicalName)}/business-rules`);
  }
  if (!ruleResponse.ok) {
    throw new Error("Failed to load business rule.");
  }
  const rule = (await ruleResponse.json()) as BusinessRuleResponse;

  const formsResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/forms`,
    cookieHeader,
  );
  if (formsResponse.status === 401) {
    redirect("/login");
  }
  if (!formsResponse.ok && formsResponse.status !== 403) {
    throw new Error("Failed to load forms.");
  }
  const forms = formsResponse.ok ? ((await formsResponse.json()) as FormResponse[]) : [];

  const fieldsResponse = await apiServerFetch(
    `/api/entities/${entityLogicalName}/fields`,
    cookieHeader,
  );
  if (fieldsResponse.status === 401) {
    redirect("/login");
  }
  if (!fieldsResponse.ok && fieldsResponse.status !== 403) {
    throw new Error("Failed to load fields.");
  }
  const fields = fieldsResponse.ok
    ? ((await fieldsResponse.json()) as FieldResponse[])
    : [];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
          <div className="space-y-1">
            <CardTitle className="font-serif text-2xl">{rule.display_name}</CardTitle>
            <CardDescription>
              <span className="font-mono">{entityLogicalName}</span> ·{" "}
              <span className="font-mono">{rule.logical_name}</span>
            </CardDescription>
          </div>
          <Link
            href={`/maker/entities/${encodeURIComponent(entityLogicalName)}/business-rules`}
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Back to Rules
          </Link>
        </CardHeader>
      </Card>

      <BusinessRuleDesignerPanel
        entityLogicalName={entityLogicalName}
        initialRule={rule}
        initialRules={rules}
        initialForms={forms}
        initialFields={fields}
      />
    </div>
  );
}
