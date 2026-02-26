"use client";

import { useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Checkbox,
  Input,
  Label,
  Notice,
  Select,
  StatusBadge,
} from "@qryvanta/ui";

import {
  apiFetch,
  type BusinessRuleResponse,
  type CreateBusinessRuleRequest,
  type FieldResponse,
  type FormResponse,
} from "@/lib/api";

type BusinessRuleDesignerPanelProps = {
  entityLogicalName: string;
  initialRule: BusinessRuleResponse | null;
  initialRules: BusinessRuleResponse[];
  initialForms: FormResponse[];
  initialFields: FieldResponse[];
};

type ScopeValue = "entity" | "form";
type OperatorValue = "eq" | "neq" | "gt" | "gte" | "lt" | "lte" | "contains";
type ActionTypeValue =
  | "show_field"
  | "hide_field"
  | "set_required"
  | "set_optional"
  | "set_default_value"
  | "set_field_value"
  | "lock_field"
  | "unlock_field"
  | "show_error";

type RuleConditionDraft = {
  field_logical_name: string;
  operator: OperatorValue;
  value_input: string;
};

type RuleActionDraft = {
  action_type: ActionTypeValue;
  target_field_logical_name: string;
  value_input: string;
  error_message: string;
};

type ValidationTemplateType = "min_value" | "max_value" | "contains_text";

const OPERATOR_OPTIONS: Array<{ value: OperatorValue; label: string }> = [
  { value: "eq", label: "Equals" },
  { value: "neq", label: "Not equals" },
  { value: "gt", label: "Greater than" },
  { value: "gte", label: "Greater or equal" },
  { value: "lt", label: "Less than" },
  { value: "lte", label: "Less or equal" },
  { value: "contains", label: "Contains" },
];

const ACTION_OPTIONS: Array<{ value: ActionTypeValue; label: string }> = [
  { value: "show_field", label: "Show field" },
  { value: "hide_field", label: "Hide field" },
  { value: "set_required", label: "Set required" },
  { value: "set_optional", label: "Set optional" },
  { value: "set_default_value", label: "Set default value" },
  { value: "set_field_value", label: "Set field value" },
  { value: "lock_field", label: "Lock field" },
  { value: "unlock_field", label: "Unlock field" },
  { value: "show_error", label: "Show error" },
];

const DEFAULT_CONDITION: RuleConditionDraft = {
  field_logical_name: "",
  operator: "eq",
  value_input: "",
};

const DEFAULT_ACTION: RuleActionDraft = {
  action_type: "show_field",
  target_field_logical_name: "",
  value_input: "",
  error_message: "",
};

function asObject(value: unknown): Record<string, unknown> | null {
  if (typeof value !== "object" || value === null) {
    return null;
  }

  return value as Record<string, unknown>;
}

function toInputValue(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }

  if (value === undefined) {
    return "";
  }

  return JSON.stringify(value);
}

function parseValueInput(valueInput: string): unknown {
  const trimmed = valueInput.trim();
  if (trimmed.length === 0) {
    return "";
  }

  try {
    return JSON.parse(trimmed);
  } catch {
    return valueInput;
  }
}

function parseInitialConditions(initialRule: BusinessRuleResponse | null): RuleConditionDraft[] {
  if (!initialRule || !Array.isArray(initialRule.conditions)) {
    return [{ ...DEFAULT_CONDITION }];
  }

  const parsed = initialRule.conditions
    .map((entry) => {
      const condition = asObject(entry);
      if (!condition) {
        return null;
      }

      const fieldLogicalName =
        typeof condition.field_logical_name === "string"
          ? condition.field_logical_name
          : "";
      const operator =
        typeof condition.operator === "string" &&
        OPERATOR_OPTIONS.some((option) => option.value === condition.operator)
          ? (condition.operator as OperatorValue)
          : "eq";

      return {
        field_logical_name: fieldLogicalName,
        operator,
        value_input: toInputValue(condition.value),
      } satisfies RuleConditionDraft;
    })
    .filter((value): value is RuleConditionDraft => value !== null);

  return parsed.length > 0 ? parsed : [{ ...DEFAULT_CONDITION }];
}

function parseInitialActions(initialRule: BusinessRuleResponse | null): RuleActionDraft[] {
  if (!initialRule || !Array.isArray(initialRule.actions)) {
    return [{ ...DEFAULT_ACTION }];
  }

  const parsed = initialRule.actions
    .map((entry) => {
      const action = asObject(entry);
      if (!action) {
        return null;
      }

      const actionType =
        typeof action.action_type === "string" &&
        ACTION_OPTIONS.some((option) => option.value === action.action_type)
          ? (action.action_type as ActionTypeValue)
          : "show_field";

      return {
        action_type: actionType,
        target_field_logical_name:
          typeof action.target_field_logical_name === "string"
            ? action.target_field_logical_name
            : "",
        value_input: toInputValue(action.value),
        error_message:
          typeof action.error_message === "string" ? action.error_message : "",
      } satisfies RuleActionDraft;
    })
    .filter((value): value is RuleActionDraft => value !== null);

  return parsed.length > 0 ? parsed : [{ ...DEFAULT_ACTION }];
}

function actionNeedsTargetField(actionType: ActionTypeValue): boolean {
  return actionType !== "show_error";
}

function actionNeedsValue(actionType: ActionTypeValue): boolean {
  return actionType === "set_default_value" || actionType === "set_field_value";
}

function fieldSupportsRange(fieldType: string): boolean {
  return fieldType === "number" || fieldType === "date" || fieldType === "datetime";
}

function fieldSupportsContains(fieldType: string): boolean {
  return fieldType === "text";
}

function toRuleLogicalName(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 64);
}

export function BusinessRuleDesignerPanel(props: BusinessRuleDesignerPanelProps) {
  return useBusinessRuleDesignerPanelContent(props);
}

function useBusinessRuleDesignerPanelContent({
  entityLogicalName,
  initialRule,
  initialRules,
  initialForms,
  initialFields,
}: BusinessRuleDesignerPanelProps) {
  const router = useRouter();
  const isEditMode = initialRule !== null;

  const [logicalName, setLogicalName] = useState(
    initialRule?.logical_name ?? "default_rule",
  );
  const [displayName, setDisplayName] = useState(
    initialRule?.display_name ?? "Default Rule",
  );
  const [scope, setScope] = useState<ScopeValue>(
    (initialRule?.scope as ScopeValue | undefined) ?? "entity",
  );
  const [formLogicalName, setFormLogicalName] = useState(
    initialRule?.form_logical_name ?? "",
  );
  const [conditions, setConditions] = useState<RuleConditionDraft[]>(() =>
    parseInitialConditions(initialRule),
  );
  const [actions, setActions] = useState<RuleActionDraft[]>(() =>
    parseInitialActions(initialRule),
  );
  const [isActive, setIsActive] = useState(initialRule?.is_active ?? true);
  const [isSaving, setIsSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [templateFieldLogicalName, setTemplateFieldLogicalName] = useState(
    initialFields[0]?.logical_name ?? "",
  );
  const [templateType, setTemplateType] = useState<ValidationTemplateType>("min_value");
  const [templateValue, setTemplateValue] = useState("");
  const [templateErrorMessage, setTemplateErrorMessage] = useState("");

  const initialSnapshot = useMemo(
    () =>
      JSON.stringify({
        logical_name: initialRule?.logical_name ?? "default_rule",
        display_name: initialRule?.display_name ?? "Default Rule",
        scope: (initialRule?.scope as ScopeValue | undefined) ?? "entity",
        form_logical_name: initialRule?.form_logical_name ?? "",
        conditions: parseInitialConditions(initialRule),
        actions: parseInitialActions(initialRule),
        is_active: initialRule?.is_active ?? true,
      }),
    [initialRule],
  );

  const currentSnapshot = useMemo(
    () =>
      JSON.stringify({
        logical_name: logicalName,
        display_name: displayName,
        scope,
        form_logical_name: scope === "form" ? formLogicalName : "",
        conditions,
        actions,
        is_active: isActive,
      }),
    [actions, conditions, displayName, formLogicalName, isActive, logicalName, scope],
  );

  const hasDraftChanges = currentSnapshot !== initialSnapshot;

  const templateField = useMemo(
    () =>
      initialFields.find((field) => field.logical_name === templateFieldLogicalName) ?? null,
    [initialFields, templateFieldLogicalName],
  );

  const templateOptions = useMemo(() => {
    if (!templateField) {
      return [] as Array<{ value: ValidationTemplateType; label: string }>;
    }

    const options: Array<{ value: ValidationTemplateType; label: string }> = [];
    if (fieldSupportsRange(templateField.field_type)) {
      options.push({ value: "min_value", label: "Minimum value" });
      options.push({ value: "max_value", label: "Maximum value" });
    }
    if (fieldSupportsContains(templateField.field_type)) {
      options.push({ value: "contains_text", label: "Contains text" });
    }

    return options;
  }, [templateField]);

  function applyValidationTemplate() {
    setStatusMessage(null);
    setErrorMessage(null);

    if (!templateField) {
      setErrorMessage("Select a field to apply a validation template.");
      return;
    }

    if (!templateOptions.some((option) => option.value === templateType)) {
      setErrorMessage("Selected template is not supported for this field type.");
      return;
    }

    if (templateValue.trim().length === 0) {
      setErrorMessage("Template value is required.");
      return;
    }

    const defaultMessage =
      templateType === "min_value"
        ? `${templateField.display_name} must be greater than or equal to ${templateValue}.`
        : templateType === "max_value"
          ? `${templateField.display_name} must be less than or equal to ${templateValue}.`
          : `${templateField.display_name} must contain \"${templateValue}\".`;

    setScope("entity");
    setIsActive(true);
    setConditions([
      {
        field_logical_name: templateField.logical_name,
        operator:
          templateType === "min_value"
            ? "lt"
            : templateType === "max_value"
              ? "gt"
              : "contains",
        value_input: templateValue,
      },
    ]);
    setActions([
      {
        action_type: "show_error",
        target_field_logical_name: "",
        value_input: "",
        error_message:
          templateErrorMessage.trim().length > 0 ? templateErrorMessage.trim() : defaultMessage,
      },
    ]);

    if (!isEditMode) {
      const templateSuffix =
        templateType === "min_value"
          ? "min_value"
          : templateType === "max_value"
            ? "max_value"
            : "contains_text";
      const nextLogicalName = toRuleLogicalName(
        `${templateField.logical_name}_${templateSuffix}_validation`,
      );
      const nextDisplayName =
        templateType === "min_value"
          ? `${templateField.display_name} minimum validation`
          : templateType === "max_value"
            ? `${templateField.display_name} maximum validation`
            : `${templateField.display_name} text validation`;
      if (nextLogicalName.length > 0) {
        setLogicalName(nextLogicalName);
      }
      setDisplayName(nextDisplayName);
    }

    setStatusMessage("Applied validation template to rule conditions and actions.");
  }

  async function handleSave(): Promise<void> {
    setStatusMessage(null);
    setErrorMessage(null);

    if (logicalName.trim().length === 0 || displayName.trim().length === 0) {
      setErrorMessage("Logical name and display name are required.");
      return;
    }

    if (conditions.length === 0) {
      setErrorMessage("At least one condition is required.");
      return;
    }

    if (actions.length === 0) {
      setErrorMessage("At least one action is required.");
      return;
    }

    if (scope === "form" && formLogicalName.trim().length === 0) {
      setErrorMessage("Form scope requires selecting a form.");
      return;
    }

    const conditionPayload: Array<{ field_logical_name: string; operator: string; value: unknown }> = [];
    for (const [index, condition] of conditions.entries()) {
      if (condition.field_logical_name.trim().length === 0) {
        setErrorMessage(`Condition ${String(index + 1)} must select a field.`);
        return;
      }

      conditionPayload.push({
        field_logical_name: condition.field_logical_name,
        operator: condition.operator,
        value: parseValueInput(condition.value_input),
      });
    }

    const actionPayload: Array<{
      action_type: string;
      target_field_logical_name: string | null;
      value: unknown;
      error_message: string | null;
    }> = [];

    for (const [index, action] of actions.entries()) {
      if (
        actionNeedsTargetField(action.action_type) &&
        action.target_field_logical_name.trim().length === 0
      ) {
        setErrorMessage(`Action ${String(index + 1)} must select a target field.`);
        return;
      }

      if (actionNeedsValue(action.action_type) && action.value_input.trim().length === 0) {
        setErrorMessage(`Action ${String(index + 1)} requires a value.`);
        return;
      }

      if (action.action_type === "show_error" && action.error_message.trim().length === 0) {
        setErrorMessage(`Action ${String(index + 1)} requires an error message.`);
        return;
      }

      actionPayload.push({
        action_type: action.action_type,
        target_field_logical_name: actionNeedsTargetField(action.action_type)
          ? action.target_field_logical_name
          : null,
        value: actionNeedsValue(action.action_type)
          ? parseValueInput(action.value_input)
          : null,
        error_message:
          action.action_type === "show_error" ? action.error_message.trim() : null,
      });
    }

    setIsSaving(true);
    try {
      const payload: CreateBusinessRuleRequest = {
        logical_name: logicalName.trim(),
        display_name: displayName.trim(),
        scope,
        form_logical_name: scope === "form" ? formLogicalName : null,
        conditions: conditionPayload,
        actions: actionPayload,
        is_active: isActive,
      };
      const path = isEditMode
        ? `/api/entities/${entityLogicalName}/business-rules/${initialRule.logical_name}`
        : `/api/entities/${entityLogicalName}/business-rules`;
      const response = await apiFetch(path, {
        method: isEditMode ? "PUT" : "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to save business rule.");
        return;
      }

      setStatusMessage("Business rule saved.");
      if (!isEditMode) {
        router.replace(
          `/maker/entities/${encodeURIComponent(entityLogicalName)}/business-rules/${encodeURIComponent(logicalName)}`,
        );
      } else {
        router.refresh();
      }
    } catch {
      setErrorMessage("Unable to save business rule.");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <CardTitle>{isEditMode ? "Business Rule Designer" : "New Business Rule"}</CardTitle>
            <CardDescription>
              Define condition-action rules for form or entity behavior.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Rules {initialRules.length}</StatusBadge>
            <StatusBadge tone={hasDraftChanges ? "warning" : "neutral"}>
              {hasDraftChanges ? "Draft changes" : "Draft saved"}
            </StatusBadge>
            <Button type="button" disabled={isSaving} onClick={handleSave}>
              {isSaving ? "Saving..." : "Save Rule"}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-4">
          <div className="space-y-2">
            <Label htmlFor="rule_logical_name">Logical Name</Label>
            <Input
              id="rule_logical_name"
              value={logicalName}
              disabled={isEditMode}
              onChange={(event) => setLogicalName(event.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="rule_display_name">Display Name</Label>
            <Input
              id="rule_display_name"
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="rule_scope">Scope</Label>
            <Select
              id="rule_scope"
              value={scope}
              onChange={(event) => setScope(event.target.value as ScopeValue)}
            >
              <option value="entity">Entity</option>
              <option value="form">Form</option>
            </Select>
          </div>
          <div className="flex items-end gap-2">
            <Checkbox
              id="rule_active"
              checked={isActive}
              onChange={(event) => setIsActive(event.target.checked)}
            />
            <Label htmlFor="rule_active">Active</Label>
          </div>
        </CardContent>
      </Card>

      {scope === "form" ? (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Form Target</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label htmlFor="rule_form_logical_name">Form</Label>
            <Select
              id="rule_form_logical_name"
              value={formLogicalName}
              onChange={(event) => setFormLogicalName(event.target.value)}
            >
              <option value="">Select form...</option>
              {initialForms.map((form) => (
                <option key={form.logical_name} value={form.logical_name}>
                  {form.display_name} ({form.logical_name})
                </option>
              ))}
            </Select>
          </CardContent>
        </Card>
      ) : null}

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Validation Templates</CardTitle>
          <CardDescription>
            Generate server-enforced field validation rules as condition + show-error actions.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-4">
          <div className="space-y-1">
            <Label htmlFor="template_field">Field</Label>
            <Select
              id="template_field"
              value={templateFieldLogicalName}
              onChange={(event) => {
                const nextFieldLogicalName = event.target.value;
                setTemplateFieldLogicalName(nextFieldLogicalName);

                const nextField =
                  initialFields.find((field) => field.logical_name === nextFieldLogicalName) ??
                  null;
                if (!nextField) {
                  return;
                }

                if (fieldSupportsRange(nextField.field_type)) {
                  setTemplateType("min_value");
                  return;
                }

                if (fieldSupportsContains(nextField.field_type)) {
                  setTemplateType("contains_text");
                }
              }}
            >
              <option value="">Select field...</option>
              {initialFields.map((field) => (
                <option key={field.logical_name} value={field.logical_name}>
                  {field.display_name} ({field.field_type})
                </option>
              ))}
            </Select>
          </div>

          <div className="space-y-1">
            <Label htmlFor="template_type">Template</Label>
            <Select
              id="template_type"
              value={templateType}
              onChange={(event) =>
                setTemplateType(event.target.value as ValidationTemplateType)
              }
              disabled={templateOptions.length === 0}
            >
              {templateOptions.length === 0 ? (
                <option value="min_value">No templates for field type</option>
              ) : (
                templateOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))
              )}
            </Select>
          </div>

          <div className="space-y-1">
            <Label htmlFor="template_value">
              {templateType === "contains_text" ? "Text pattern" : "Threshold"}
            </Label>
            <Input
              id="template_value"
              value={templateValue}
              onChange={(event) => setTemplateValue(event.target.value)}
              placeholder={
                templateType === "contains_text"
                  ? "e.g. @example.com"
                  : templateField?.field_type === "date" ||
                      templateField?.field_type === "datetime"
                    ? "e.g. 2026-12-31"
                    : "e.g. 100"
              }
            />
          </div>

          <div className="space-y-1">
            <Label htmlFor="template_message">Error Message (optional)</Label>
            <Input
              id="template_message"
              value={templateErrorMessage}
              onChange={(event) => setTemplateErrorMessage(event.target.value)}
              placeholder="Custom validation message"
            />
          </div>

          <div className="md:col-span-4">
            <Button
              type="button"
              variant="outline"
              disabled={templateOptions.length === 0}
              onClick={applyValidationTemplate}
            >
              Apply Template to Rule
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Conditions</CardTitle>
          <CardDescription>
            Add one or more conditions. All conditions must match for this rule to apply.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {conditions.map((condition, index) => (
            <div
              key={`condition-${String(index)}`}
              className="grid gap-2 rounded-md border border-zinc-200 p-3 md:grid-cols-[1.2fr_1fr_1fr_auto]"
            >
              <div className="space-y-1">
                <Label htmlFor={`condition_field_${String(index)}`}>Field</Label>
                <Select
                  id={`condition_field_${String(index)}`}
                  value={condition.field_logical_name}
                  onChange={(event) =>
                    setConditions((current) =>
                      current.map((item, itemIndex) =>
                        itemIndex === index
                          ? { ...item, field_logical_name: event.target.value }
                          : item,
                      ),
                    )
                  }
                >
                  <option value="">Select field...</option>
                  {initialFields.map((field) => (
                    <option key={field.logical_name} value={field.logical_name}>
                      {field.display_name} ({field.logical_name})
                    </option>
                  ))}
                </Select>
              </div>

              <div className="space-y-1">
                <Label htmlFor={`condition_operator_${String(index)}`}>Operator</Label>
                <Select
                  id={`condition_operator_${String(index)}`}
                  value={condition.operator}
                  onChange={(event) =>
                    setConditions((current) =>
                      current.map((item, itemIndex) =>
                        itemIndex === index
                          ? { ...item, operator: event.target.value as OperatorValue }
                          : item,
                      ),
                    )
                  }
                >
                  {OPERATOR_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </Select>
              </div>

              <div className="space-y-1">
                <Label htmlFor={`condition_value_${String(index)}`}>Value</Label>
                <Input
                  id={`condition_value_${String(index)}`}
                  value={condition.value_input}
                  onChange={(event) =>
                    setConditions((current) =>
                      current.map((item, itemIndex) =>
                        itemIndex === index
                          ? { ...item, value_input: event.target.value }
                          : item,
                      ),
                    )
                  }
                  placeholder='e.g. active, 10, "text", true'
                />
              </div>

              <div className="flex items-end">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() =>
                    setConditions((current) =>
                      current.length > 1
                        ? current.filter((_, itemIndex) => itemIndex !== index)
                        : current,
                    )
                  }
                  disabled={conditions.length <= 1}
                >
                  Remove
                </Button>
              </div>
            </div>
          ))}

          <Button
            type="button"
            variant="outline"
            onClick={() =>
              setConditions((current) => [...current, { ...DEFAULT_CONDITION }])
            }
          >
            Add Condition
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Actions</CardTitle>
          <CardDescription>
            Actions run when conditions match.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {actions.map((action, index) => (
            <div
              key={`action-${String(index)}`}
              className="space-y-2 rounded-md border border-zinc-200 p-3"
            >
              <div className="grid gap-2 md:grid-cols-[1fr_1fr_auto]">
                <div className="space-y-1">
                  <Label htmlFor={`action_type_${String(index)}`}>Action</Label>
                  <Select
                    id={`action_type_${String(index)}`}
                    value={action.action_type}
                    onChange={(event) =>
                      setActions((current) =>
                        current.map((item, itemIndex) =>
                          itemIndex === index
                            ? {
                                ...item,
                                action_type: event.target.value as ActionTypeValue,
                              }
                            : item,
                        ),
                      )
                    }
                  >
                    {ACTION_OPTIONS.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </Select>
                </div>

                {actionNeedsTargetField(action.action_type) ? (
                  <div className="space-y-1">
                    <Label htmlFor={`action_target_${String(index)}`}>Target Field</Label>
                    <Select
                      id={`action_target_${String(index)}`}
                      value={action.target_field_logical_name}
                      onChange={(event) =>
                        setActions((current) =>
                          current.map((item, itemIndex) =>
                            itemIndex === index
                              ? {
                                  ...item,
                                  target_field_logical_name: event.target.value,
                                }
                              : item,
                          ),
                        )
                      }
                    >
                      <option value="">Select field...</option>
                      {initialFields.map((field) => (
                        <option key={field.logical_name} value={field.logical_name}>
                          {field.display_name} ({field.logical_name})
                        </option>
                      ))}
                    </Select>
                  </div>
                ) : (
                  <div className="space-y-1">
                    <p className="text-sm font-medium text-zinc-700">Target Field</p>
                    <Input value="Not required" disabled />
                  </div>
                )}

                <div className="flex items-end">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() =>
                      setActions((current) =>
                        current.length > 1
                          ? current.filter((_, itemIndex) => itemIndex !== index)
                          : current,
                      )
                    }
                    disabled={actions.length <= 1}
                  >
                    Remove
                  </Button>
                </div>
              </div>

              {actionNeedsValue(action.action_type) ? (
                <div className="space-y-1">
                  <Label htmlFor={`action_value_${String(index)}`}>Value</Label>
                  <Input
                    id={`action_value_${String(index)}`}
                    value={action.value_input}
                    onChange={(event) =>
                      setActions((current) =>
                        current.map((item, itemIndex) =>
                          itemIndex === index
                            ? { ...item, value_input: event.target.value }
                            : item,
                        ),
                      )
                    }
                    placeholder='e.g. 100, "open", true, {"k":"v"}'
                  />
                </div>
              ) : null}

              {action.action_type === "show_error" ? (
                <div className="space-y-1">
                  <Label htmlFor={`action_error_${String(index)}`}>Error Message</Label>
                  <Input
                    id={`action_error_${String(index)}`}
                    value={action.error_message}
                    onChange={(event) =>
                      setActions((current) =>
                        current.map((item, itemIndex) =>
                          itemIndex === index
                            ? { ...item, error_message: event.target.value }
                            : item,
                        ),
                      )
                    }
                    placeholder="Explain what should be fixed"
                  />
                </div>
              ) : null}
            </div>
          ))}

          <Button
            type="button"
            variant="outline"
            onClick={() => setActions((current) => [...current, { ...DEFAULT_ACTION }])}
          >
            Add Action
          </Button>
        </CardContent>
      </Card>

      {initialFields.length === 0 ? (
        <Notice tone="warning">
          No entity fields are currently available. You can still draft a rule, but save may fail until fields are defined and published.
        </Notice>
      ) : null}

      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}
