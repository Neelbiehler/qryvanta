import type { BusinessRuleResponse } from "@/lib/api";

type BusinessRuleCondition = {
  field_logical_name: string;
  operator: "eq" | "neq" | "gt" | "gte" | "lt" | "lte" | "contains";
  value: unknown;
};

type BusinessRuleAction = {
  action_type:
    | "show_field"
    | "hide_field"
    | "set_required"
    | "set_optional"
    | "set_default_value"
    | "set_field_value"
    | "lock_field"
    | "unlock_field"
    | "show_error";
  target_field_logical_name: string | null;
  value: unknown;
  error_message: string | null;
};

type ParsedBusinessRule = {
  scope: "entity" | "form";
  form_logical_name: string | null;
  conditions: BusinessRuleCondition[];
  actions: BusinessRuleAction[];
};

export type EvaluatedRuleState = {
  hiddenFieldNames: Set<string>;
  requiredOverrides: Map<string, boolean>;
  readOnlyOverrides: Map<string, boolean>;
  valuePatches: Map<string, unknown>;
  errorMessages: string[];
};

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function normalizeRules(rules: BusinessRuleResponse[]): ParsedBusinessRule[] {
  return rules
    .filter((rule) => rule.is_active)
    .map((rule) => {
      const conditions = Array.isArray(rule.conditions)
        ? rule.conditions
            .filter(isObject)
            .map((condition) => ({
              field_logical_name: String(condition.field_logical_name ?? ""),
              operator: String(condition.operator ?? "eq") as BusinessRuleCondition["operator"],
              value: condition.value,
            }))
            .filter((condition) => condition.field_logical_name.length > 0)
        : [];

      const actions = Array.isArray(rule.actions)
        ? rule.actions
            .filter(isObject)
            .map((action) => ({
              action_type: String(action.action_type ?? "show_error") as BusinessRuleAction["action_type"],
              target_field_logical_name:
                typeof action.target_field_logical_name === "string"
                  ? action.target_field_logical_name
                  : null,
              value: action.value,
              error_message:
                typeof action.error_message === "string" ? action.error_message : null,
            }))
        : [];

      return {
        scope: (rule.scope === "form" ? "form" : "entity") as "form" | "entity",
        form_logical_name: rule.form_logical_name,
        conditions,
        actions,
      };
    })
    .filter((rule) => rule.conditions.length > 0 && rule.actions.length > 0);
}

function matchesCondition(value: unknown, condition: BusinessRuleCondition): boolean {
  switch (condition.operator) {
    case "eq":
      return value === condition.value;
    case "neq":
      return value !== condition.value;
    case "gt":
      return Number(value ?? 0) > Number(condition.value ?? 0);
    case "gte":
      return Number(value ?? 0) >= Number(condition.value ?? 0);
    case "lt":
      return Number(value ?? 0) < Number(condition.value ?? 0);
    case "lte":
      return Number(value ?? 0) <= Number(condition.value ?? 0);
    case "contains":
      return String(value ?? "")
        .toLowerCase()
        .includes(String(condition.value ?? "").toLowerCase());
    default:
      return false;
  }
}

function isEmptyValue(value: unknown): boolean {
  return (
    value === null ||
    value === undefined ||
    (typeof value === "string" && value.trim().length === 0)
  );
}

export function evaluateRuleState(
  rules: BusinessRuleResponse[],
  activeFormLogicalName: string | null,
  formValues: Record<string, unknown>,
): EvaluatedRuleState {
  const parsedRules = normalizeRules(rules).filter((rule) => {
    if (rule.scope === "entity") {
      return true;
    }

    return (
      activeFormLogicalName !== null &&
      rule.form_logical_name === activeFormLogicalName
    );
  });

  const hiddenFieldNames = new Set<string>();
  const requiredOverrides = new Map<string, boolean>();
  const readOnlyOverrides = new Map<string, boolean>();
  const valuePatches = new Map<string, unknown>();
  const errorMessages: string[] = [];

  for (const rule of parsedRules) {
    const matched = rule.conditions.every((condition) =>
      matchesCondition(formValues[condition.field_logical_name], condition),
    );
    if (!matched) {
      continue;
    }

    for (const action of rule.actions) {
      const targetField = action.target_field_logical_name;
      if (!targetField) {
        continue;
      }

      switch (action.action_type) {
        case "show_field":
          hiddenFieldNames.delete(targetField);
          break;
        case "hide_field":
          hiddenFieldNames.add(targetField);
          break;
        case "set_required":
          requiredOverrides.set(targetField, true);
          break;
        case "set_optional":
          requiredOverrides.set(targetField, false);
          break;
        case "set_default_value": {
          const current = formValues[targetField];
          if (isEmptyValue(current)) {
            valuePatches.set(targetField, action.value);
          }
          break;
        }
        case "set_field_value":
          valuePatches.set(targetField, action.value);
          break;
        case "lock_field":
          readOnlyOverrides.set(targetField, true);
          break;
        case "unlock_field":
          readOnlyOverrides.set(targetField, false);
          break;
        case "show_error":
          if (action.error_message && action.error_message.trim().length > 0) {
            errorMessages.push(action.error_message);
          }
          break;
        default:
          break;
      }
    }
  }

  return {
    hiddenFieldNames,
    requiredOverrides,
    readOnlyOverrides,
    valuePatches,
    errorMessages,
  };
}
