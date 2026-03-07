import { useEffect, useRef, useState } from "react";
import {
  Bell,
  ChevronDown,
  ChevronUp,
  CheckCircle2,
  Clock3,
  Copy,
  Database,
  ExternalLink,
  GitBranch,
  Globe,
  Mail,
  MessageSquare,
  Plus,
  Trash2,
  XCircle,
} from "lucide-react";

import { Button, Input, Label, Select, Textarea } from "@qryvanta/ui";

import { ExpressionBuilderPopover } from "@/components/automation/workflow-studio/panels/expression-builder-popover";
import { TriggerConfigPanel } from "@/components/automation/workflow-studio/panels/trigger-config-panel";
import {
  CONDITION_OPERATORS,
  createDraftArrayItemsFromValue,
  createDraftObjectFieldsFromValue,
  createDraftFieldId,
  parseDraftArrayItems,
  parseDraftObjectFields,
  parseDraftValue,
  parseJsonValue,
  summarizeStep,
  type DraftArrayItem,
  type DraftObjectField,
  type DraftValueKind,
  type CatalogInsertMode,
  type DraftConditionStep,
  type DraftWorkflowStep,
  type DynamicTokenOption,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";
import type {
  RetryWorkflowStepStrategyDto,
  WorkflowConditionOperatorDto,
  WorkflowRunStepTraceResponse,
} from "@/lib/api";

type RetryPreset = "immediate" | "backoff_800" | "backoff_2000" | "backoff_5000";

function appendExpression(value: string, expression: string): string {
  return value.trim().length === 0 ? expression : `${value} ${expression}`;
}

function insertTokenMappingIntoDraftObjectFields(
  fields: DraftObjectField[],
  fieldPath: string,
): DraftObjectField[] {
  const trimmedFieldPath = fieldPath.trim();
  if (trimmedFieldPath.length === 0) {
    return fields;
  }

  const key =
    trimmedFieldPath.split(".").filter((segment) => segment.length > 0).at(-1) ??
    trimmedFieldPath;
  const token = `{{trigger.payload.${trimmedFieldPath}}}`;
  const existingIndex = fields.findIndex((field) => field.key.trim() === key);
  if (existingIndex >= 0) {
    return fields.map((field, index) =>
      index === existingIndex ? { ...field, valueKind: "string", value: token } : field,
    );
  }

  return [
    ...fields,
    {
      id: createDraftFieldId(),
      key,
      valueKind: "string",
      value: token,
    },
  ];
}

function tokenChipsFromValue(value: string): string[] {
  const matches = value.match(/\{\{[^}]+\}\}/g);
  if (!matches) return [];
  return Array.from(new Set(matches));
}

function tokenTextFromDraftObjectFields(fields: DraftObjectField[]): string {
  return fields
    .map((field) => `${field.key} ${field.value}`)
    .join(" ");
}

type StringMapEntry = {
  key: string;
  value: string;
};

type SecretHeaderAuthMode =
  | "none"
  | "authorization"
  | "x-api-key"
  | "cookie"
  | "custom";

type SecretReferenceProvider =
  | "raw"
  | "op"
  | "aws-sm"
  | "aws-ssm"
  | "vault"
  | "gcp-sm";

type SecretHeaderValueFormat = "raw" | "bearer" | "basic";

type SecretHeaderEditorState = {
  authMode: SecretHeaderAuthMode;
  authHeaderName: string;
  authValueFormat: SecretHeaderValueFormat;
  authSecretRef: string;
  secretRefProvider: SecretReferenceProvider;
  opVault: string;
  opItem: string;
  opField: string;
  awsSecretId: string;
  awsSsmParameter: string;
  vaultPath: string;
  vaultField: string;
  gcpProject: string;
  gcpSecret: string;
  gcpVersion: string;
  additionalHeadersJson: string;
  error: string | null;
};

function parseStringMapJson(value: string): {
  entries: StringMapEntry[];
  error: string | null;
} {
  try {
    const parsed = JSON.parse(value) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {
        entries: [],
        error: "Value must be a JSON object.",
      };
    }

    const entries = Object.entries(parsed as Record<string, unknown>).map(
      ([key, entry]) => ({
        key,
        value: typeof entry === "string" ? entry : JSON.stringify(entry),
      }),
    );

    const hasNonStringValue = Object.values(parsed as Record<string, unknown>).some(
      (entry) => typeof entry !== "string",
    );

    return {
      entries,
      error: hasNonStringValue ? "All values must be strings." : null,
    };
  } catch {
    return {
      entries: [],
      error: "Value contains invalid JSON.",
    };
  }
}

function stringifyStringMapEntries(entries: StringMapEntry[]): string {
  const normalized = entries.reduce<Record<string, string>>((acc, entry) => {
    const key = entry.key.trim();
    if (key.length === 0) {
      return acc;
    }

    acc[key] = entry.value;
    return acc;
  }, {});

  return JSON.stringify(normalized, null, 2);
}

function normalizeStringMapEntries(entries: StringMapEntry[]): Record<string, string> {
  return entries.reduce<Record<string, string>>((acc, entry) => {
    const key = entry.key.trim();
    if (key.length === 0) {
      return acc;
    }

    acc[key] = entry.value;
    return acc;
  }, {});
}

function parseSecretReferenceState(reference: string): Pick<
  SecretHeaderEditorState,
  | "authValueFormat"
  | "authSecretRef"
  | "secretRefProvider"
  | "opVault"
  | "opItem"
  | "opField"
  | "awsSecretId"
  | "awsSsmParameter"
  | "vaultPath"
  | "vaultField"
  | "gcpProject"
  | "gcpSecret"
  | "gcpVersion"
> {
  const prefixedReference = reference.trim();
  const formattedReference = prefixedReference.startsWith("bearer+")
    ? {
        authValueFormat: "bearer" as SecretHeaderValueFormat,
        authSecretRef: prefixedReference.slice("bearer+".length),
      }
    : prefixedReference.startsWith("basic+")
      ? {
          authValueFormat: "basic" as SecretHeaderValueFormat,
          authSecretRef: prefixedReference.slice("basic+".length),
        }
      : {
          authValueFormat: "raw" as SecretHeaderValueFormat,
          authSecretRef: prefixedReference,
        };

  const defaults = {
    authValueFormat: formattedReference.authValueFormat,
    authSecretRef: formattedReference.authSecretRef,
    secretRefProvider: "raw" as SecretReferenceProvider,
    opVault: "",
    opItem: "",
    opField: "",
    awsSecretId: "",
    awsSsmParameter: "",
    vaultPath: "",
    vaultField: "",
    gcpProject: "",
    gcpSecret: "",
    gcpVersion: "latest",
  };

  if (formattedReference.authSecretRef.startsWith("op://")) {
    const segments = formattedReference.authSecretRef.slice("op://".length).split("/");
    if (segments.length >= 3) {
      return {
        ...defaults,
        secretRefProvider: "op",
        opVault: segments[0] ?? "",
        opItem: segments[1] ?? "",
        opField: segments.slice(2).join("/"),
      };
    }
  }

  if (formattedReference.authSecretRef.startsWith("aws-sm://")) {
    return {
      ...defaults,
      secretRefProvider: "aws-sm",
      awsSecretId: formattedReference.authSecretRef.slice("aws-sm://".length),
    };
  }

  if (formattedReference.authSecretRef.startsWith("aws-ssm://")) {
    return {
      ...defaults,
      secretRefProvider: "aws-ssm",
      awsSsmParameter: formattedReference.authSecretRef.slice("aws-ssm://".length),
    };
  }

  if (formattedReference.authSecretRef.startsWith("vault://")) {
    const withoutPrefix = formattedReference.authSecretRef.slice("vault://".length);
    const splitIndex = withoutPrefix.lastIndexOf("#");
    if (splitIndex >= 0) {
      return {
        ...defaults,
        secretRefProvider: "vault",
        vaultPath: withoutPrefix.slice(0, splitIndex),
        vaultField: withoutPrefix.slice(splitIndex + 1),
      };
    }
  }

  const gcpMatch = formattedReference.authSecretRef.match(
    /^gcp-sm:\/\/projects\/([^/]+)\/secrets\/([^/]+)\/versions\/([^/]+)$/,
  );
  if (gcpMatch) {
    return {
      ...defaults,
      secretRefProvider: "gcp-sm",
      gcpProject: gcpMatch[1] ?? "",
      gcpSecret: gcpMatch[2] ?? "",
      gcpVersion: gcpMatch[3] ?? "latest",
    };
  }

  return defaults;
}

function buildSecretReferenceFromState(
  state: Pick<
    SecretHeaderEditorState,
    | "authMode"
    | "authValueFormat"
    | "authSecretRef"
    | "secretRefProvider"
    | "opVault"
    | "opItem"
    | "opField"
    | "awsSecretId"
    | "awsSsmParameter"
    | "vaultPath"
    | "vaultField"
    | "gcpProject"
    | "gcpSecret"
    | "gcpVersion"
  >,
): string {
  let reference = "";
  switch (state.secretRefProvider) {
    case "op": {
      const vault = state.opVault.trim();
      const item = state.opItem.trim();
      const field = state.opField.trim();
      if (vault.length === 0 || item.length === 0 || field.length === 0) {
        return "";
      }
      reference = `op://${vault}/${item}/${field}`;
      break;
    }
    case "aws-sm": {
      if (state.awsSecretId.trim().length === 0) {
        return "";
      }
      reference = `aws-sm://${state.awsSecretId.trim()}`;
      break;
    }
    case "aws-ssm": {
      if (state.awsSsmParameter.trim().length === 0) {
        return "";
      }
      reference = `aws-ssm://${state.awsSsmParameter.trim()}`;
      break;
    }
    case "vault": {
      const path = state.vaultPath.trim();
      const field = state.vaultField.trim();
      if (path.length === 0 || field.length === 0) {
        return "";
      }
      reference = `vault://${path}#${field}`;
      break;
    }
    case "gcp-sm": {
      const project = state.gcpProject.trim();
      const secret = state.gcpSecret.trim();
      const version = state.gcpVersion.trim();
      if (project.length === 0 || secret.length === 0 || version.length === 0) {
        return "";
      }
      reference = `gcp-sm://projects/${project}/secrets/${secret}/versions/${version}`;
      break;
    }
    case "raw": {
      reference = state.authSecretRef.trim();
      break;
    }
  }

  if (reference.length === 0) {
    return "";
  }

  if (state.authMode === "authorization") {
    if (state.authValueFormat === "bearer") {
      return `bearer+${reference}`;
    }

    if (state.authValueFormat === "basic") {
      return `basic+${reference}`;
    }
  }

  return reference;
}

function parseSecretHeaderEditorState(value: string): SecretHeaderEditorState {
  const parsed = parseStringMapJson(value);
  if (parsed.error) {
    return {
      authMode: "none",
      authHeaderName: "",
      authValueFormat: "raw",
      authSecretRef: "",
      secretRefProvider: "raw",
      opVault: "",
      opItem: "",
      opField: "",
      awsSecretId: "",
      awsSsmParameter: "",
      vaultPath: "",
      vaultField: "",
      gcpProject: "",
      gcpSecret: "",
      gcpVersion: "latest",
      additionalHeadersJson: JSON.stringify({}, null, 2),
      error: parsed.error,
    };
  }

  const normalized = normalizeStringMapEntries(parsed.entries);
  let consumedKey: string | null = null;
  let authMode: SecretHeaderAuthMode = "none";
  let authHeaderName = "";
  let authSecretRef = "";

  if (typeof normalized.authorization === "string") {
    authMode = "authorization";
    authHeaderName = "authorization";
    authSecretRef = normalized.authorization;
    consumedKey = "authorization";
  } else if (typeof normalized["x-api-key"] === "string") {
    authMode = "x-api-key";
    authHeaderName = "x-api-key";
    authSecretRef = normalized["x-api-key"];
    consumedKey = "x-api-key";
  } else if (typeof normalized.cookie === "string") {
    authMode = "cookie";
    authHeaderName = "cookie";
    authSecretRef = normalized.cookie;
    consumedKey = "cookie";
  } else {
    const keys = Object.keys(normalized);
    if (keys.length === 1) {
      authMode = "custom";
      authHeaderName = keys[0] ?? "";
      authSecretRef = normalized[keys[0] ?? ""];
      consumedKey = keys[0] ?? null;
    }
  }

  const additionalHeaders = Object.fromEntries(
    Object.entries(normalized).filter(([key]) => key !== consumedKey),
  );

  const secretReferenceState = parseSecretReferenceState(authSecretRef);

  return {
    authMode,
    authHeaderName,
    ...secretReferenceState,
    additionalHeadersJson: JSON.stringify(additionalHeaders, null, 2),
    error: null,
  };
}

function stringifySecretHeaderEditorState(
  state: Omit<SecretHeaderEditorState, "error">,
): string {
  const parsedAdditional = parseStringMapJson(state.additionalHeadersJson);
  const additionalHeaders = parsedAdditional.error
    ? {}
    : normalizeStringMapEntries(parsedAdditional.entries);
  const authSecretRef = buildSecretReferenceFromState(state);

  let authHeaderKey: string | null = null;
  if (state.authMode === "authorization") {
    authHeaderKey = "authorization";
  } else if (state.authMode === "x-api-key") {
    authHeaderKey = "x-api-key";
  } else if (state.authMode === "cookie") {
    authHeaderKey = "cookie";
  } else if (state.authMode === "custom") {
    const trimmed = state.authHeaderName.trim();
    authHeaderKey = trimmed.length > 0 ? trimmed : null;
  }

  const nextHeaders: Record<string, string> = {};
  for (const [key, value] of Object.entries(additionalHeaders)) {
    if (authHeaderKey && key.toLowerCase() === authHeaderKey.toLowerCase()) {
      continue;
    }
    nextHeaders[key] = value;
  }

  if (authHeaderKey && authSecretRef.length > 0) {
    nextHeaders[authHeaderKey] = authSecretRef;
  }

  return JSON.stringify(nextHeaders, null, 2);
}

function authModeHelperText(authMode: SecretHeaderAuthMode): string {
  switch (authMode) {
    case "authorization":
      return "Use a secret reference for the Authorization header. Choose raw, Bearer, or Basic formatting so the secret can store only the credential value.";
    case "x-api-key":
      return "Resolve the API key from the secret manager at dispatch time instead of storing it inline.";
    case "cookie":
      return "Resolve the full Cookie header value from a secret reference instead of storing it in the workflow draft.";
    case "custom":
      return "Use a secret-backed custom header when the integration expects a non-standard credential header name.";
    case "none":
      return "No primary credential header preset is configured for this step.";
  }
}

function secretReferenceProviderHelperText(provider: SecretReferenceProvider): string {
  switch (provider) {
    case "op":
      return "Build a 1Password reference in the form `op://vault/item/field`.";
    case "aws-sm":
      return "Build an AWS Secrets Manager reference in the form `aws-sm://<secret-id>`.";
    case "aws-ssm":
      return "Build an AWS SSM Parameter Store reference in the form `aws-ssm://<parameter-name>`.";
    case "vault":
      return "Build a Vault KV reference in the form `vault://<path>#<field>`.";
    case "gcp-sm":
      return "Build a GCP Secret Manager reference in the form `gcp-sm://projects/<project>/secrets/<secret>/versions/<version>`.";
    case "raw":
      return "Enter any supported secret reference string directly when you already have the exact provider URI.";
  }
}

function StringMapEditor({
  label,
  idPrefix,
  value,
  onChange,
  placeholderKey,
  placeholderValue,
  helperText,
}: {
  label: string;
  idPrefix: string;
  value: string;
  onChange: (nextValue: string) => void;
  placeholderKey: string;
  placeholderValue: string;
  helperText?: string;
}) {
  const { entries, error } = parseStringMapJson(value);
  const rows = entries.length > 0 ? entries : [{ key: "", value: "" }];

  function updateRow(index: number, field: "key" | "value", next: string) {
    const nextRows = rows.map((row, rowIndex) =>
      rowIndex === index ? { ...row, [field]: next } : row,
    );
    onChange(stringifyStringMapEntries(nextRows));
  }

  function addRow() {
    onChange(stringifyStringMapEntries([...rows, { key: "", value: "" }]));
  }

  function removeRow(index: number) {
    const nextRows = rows.filter((_, rowIndex) => rowIndex !== index);
    onChange(stringifyStringMapEntries(nextRows));
  }

  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between">
        <Label htmlFor={`${idPrefix}_key_0`}>{label}</Label>
        <Button type="button" variant="outline" size="sm" onClick={addRow}>
          <Plus className="size-3.5" />
          Add header
        </Button>
      </div>
      <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/60 p-3">
        {rows.map((row, index) => (
          <div key={`${idPrefix}_${index}`} className="grid grid-cols-[1fr_1fr_auto] gap-2">
            <Input
              id={`${idPrefix}_key_${index}`}
              value={row.key}
              onChange={(event) => updateRow(index, "key", event.target.value)}
              placeholder={placeholderKey}
            />
            <Input
              id={`${idPrefix}_value_${index}`}
              value={row.value}
              onChange={(event) => updateRow(index, "value", event.target.value)}
              placeholder={placeholderValue}
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => removeRow(index)}
              disabled={rows.length === 1 && row.key.trim().length === 0 && row.value.trim().length === 0}
            >
              <Trash2 className="size-4" />
            </Button>
          </div>
        ))}
      </div>
      {helperText ? <p className="text-[11px] text-zinc-500">{helperText}</p> : null}
      {error ? <p className="text-[11px] text-red-600">{error}</p> : null}
      <TokenChips value={value} />
    </div>
  );
}

function SecretHeaderEditor({
  idPrefix,
  value,
  onChange,
}: {
  idPrefix: string;
  value: string;
  onChange: (nextValue: string) => void;
}) {
  const state = parseSecretHeaderEditorState(value);

  function updateState(
    patch: Partial<Omit<SecretHeaderEditorState, "error">>,
  ) {
    const nextState = {
      authMode: state.authMode,
      authHeaderName: state.authHeaderName,
      authValueFormat: state.authValueFormat,
      authSecretRef: state.authSecretRef,
      secretRefProvider: state.secretRefProvider,
      opVault: state.opVault,
      opItem: state.opItem,
      opField: state.opField,
      awsSecretId: state.awsSecretId,
      awsSsmParameter: state.awsSsmParameter,
      vaultPath: state.vaultPath,
      vaultField: state.vaultField,
      gcpProject: state.gcpProject,
      gcpSecret: state.gcpSecret,
      gcpVersion: state.gcpVersion,
      additionalHeadersJson: state.additionalHeadersJson,
      ...patch,
    };
    onChange(stringifySecretHeaderEditorState(nextState));
  }

  return (
    <div className="space-y-3 rounded-xl border border-zinc-200 bg-zinc-50/60 p-3">
      <div className="grid gap-3 md:grid-cols-2">
        <div className="space-y-1.5">
          <Label htmlFor={`${idPrefix}_auth_mode`}>Credential Preset</Label>
          <Select
            id={`${idPrefix}_auth_mode`}
            value={state.authMode}
            onChange={(event) =>
              updateState({
                authMode: event.target.value as SecretHeaderAuthMode,
              })
            }
          >
            <option value="none">No secret-backed auth header</option>
            <option value="authorization">Authorization header</option>
            <option value="x-api-key">X-API-Key header</option>
            <option value="cookie">Cookie header</option>
            <option value="custom">Custom secret header</option>
          </Select>
        </div>
        {state.authMode === "authorization" ? (
          <div className="space-y-1.5">
            <Label htmlFor={`${idPrefix}_auth_value_format`}>Authorization Format</Label>
            <Select
              id={`${idPrefix}_auth_value_format`}
              value={state.authValueFormat}
              onChange={(event) =>
                updateState({
                  authValueFormat: event.target.value as SecretHeaderValueFormat,
                })
              }
            >
              <option value="raw">Raw header value</option>
              <option value="bearer">Bearer token</option>
              <option value="basic">Basic credentials</option>
            </Select>
          </div>
        ) : null}
        {state.authMode === "custom" ? (
          <div className="space-y-1.5">
            <Label htmlFor={`${idPrefix}_auth_header_name`}>Custom Header Name</Label>
            <Input
              id={`${idPrefix}_auth_header_name`}
              value={state.authHeaderName}
              onChange={(event) =>
                updateState({ authHeaderName: event.target.value })
              }
              placeholder="x-service-token"
            />
          </div>
        ) : null}
      </div>
      {state.authMode !== "none" ? (
        <div className="space-y-3">
          <div className="space-y-1.5">
            <Label htmlFor={`${idPrefix}_secret_provider`}>Secret Provider</Label>
            <Select
              id={`${idPrefix}_secret_provider`}
              value={state.secretRefProvider}
              onChange={(event) =>
                updateState({
                  secretRefProvider: event.target.value as SecretReferenceProvider,
                })
              }
            >
              <option value="op">1Password (`op://`)</option>
              <option value="aws-sm">AWS Secrets Manager (`aws-sm://`)</option>
              <option value="aws-ssm">AWS SSM (`aws-ssm://`)</option>
              <option value="vault">Vault KV (`vault://`)</option>
              <option value="gcp-sm">GCP Secret Manager (`gcp-sm://`)</option>
              <option value="raw">Raw reference</option>
            </Select>
            <p className="text-[11px] text-zinc-500">
              {authModeHelperText(state.authMode)}
            </p>
          </div>
          {state.secretRefProvider === "op" ? (
            <div className="grid gap-2 md:grid-cols-3">
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_op_vault`}>Vault</Label>
                <Input
                  id={`${idPrefix}_op_vault`}
                  value={state.opVault}
                  onChange={(event) => updateState({ opVault: event.target.value })}
                  placeholder="team-prod"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_op_item`}>Item</Label>
                <Input
                  id={`${idPrefix}_op_item`}
                  value={state.opItem}
                  onChange={(event) => updateState({ opItem: event.target.value })}
                  placeholder="stripe-api"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_op_field`}>Field</Label>
                <Input
                  id={`${idPrefix}_op_field`}
                  value={state.opField}
                  onChange={(event) => updateState({ opField: event.target.value })}
                  placeholder="token"
                />
              </div>
            </div>
          ) : null}
          {state.secretRefProvider === "aws-sm" ? (
            <div className="space-y-1.5">
              <Label htmlFor={`${idPrefix}_aws_secret_id`}>Secret Id</Label>
              <Input
                id={`${idPrefix}_aws_secret_id`}
                value={state.awsSecretId}
                onChange={(event) => updateState({ awsSecretId: event.target.value })}
                placeholder="prod/qryvanta/session"
              />
            </div>
          ) : null}
          {state.secretRefProvider === "aws-ssm" ? (
            <div className="space-y-1.5">
              <Label htmlFor={`${idPrefix}_aws_ssm_parameter`}>Parameter Name</Label>
              <Input
                id={`${idPrefix}_aws_ssm_parameter`}
                value={state.awsSsmParameter}
                onChange={(event) =>
                  updateState({ awsSsmParameter: event.target.value })
                }
                placeholder="/prod/qryvanta/session"
              />
            </div>
          ) : null}
          {state.secretRefProvider === "vault" ? (
            <div className="grid gap-2 md:grid-cols-[1.6fr_1fr]">
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_vault_path`}>Vault Path</Label>
                <Input
                  id={`${idPrefix}_vault_path`}
                  value={state.vaultPath}
                  onChange={(event) => updateState({ vaultPath: event.target.value })}
                  placeholder="kv/qryvanta/prod"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_vault_field`}>Field</Label>
                <Input
                  id={`${idPrefix}_vault_field`}
                  value={state.vaultField}
                  onChange={(event) => updateState({ vaultField: event.target.value })}
                  placeholder="session_secret"
                />
              </div>
            </div>
          ) : null}
          {state.secretRefProvider === "gcp-sm" ? (
            <div className="grid gap-2 md:grid-cols-3">
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_gcp_project`}>Project</Label>
                <Input
                  id={`${idPrefix}_gcp_project`}
                  value={state.gcpProject}
                  onChange={(event) => updateState({ gcpProject: event.target.value })}
                  placeholder="prod-project"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_gcp_secret`}>Secret</Label>
                <Input
                  id={`${idPrefix}_gcp_secret`}
                  value={state.gcpSecret}
                  onChange={(event) => updateState({ gcpSecret: event.target.value })}
                  placeholder="session-secret"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor={`${idPrefix}_gcp_version`}>Version</Label>
                <Input
                  id={`${idPrefix}_gcp_version`}
                  value={state.gcpVersion}
                  onChange={(event) => updateState({ gcpVersion: event.target.value })}
                  placeholder="latest"
                />
              </div>
            </div>
          ) : null}
          {state.secretRefProvider === "raw" ? (
            <div className="space-y-1.5">
              <Label htmlFor={`${idPrefix}_auth_secret_ref`}>Secret Reference</Label>
              <Input
                id={`${idPrefix}_auth_secret_ref`}
                value={state.authSecretRef}
                onChange={(event) =>
                  updateState({ authSecretRef: event.target.value })
                }
                placeholder="op://vault/item/password"
              />
            </div>
          ) : null}
          <p className="text-[11px] text-zinc-500">
            {secretReferenceProviderHelperText(state.secretRefProvider)}
          </p>
          <JsonPreviewCard
            label="Secret reference preview"
            value={{
              header:
                state.authMode === "custom"
                  ? state.authHeaderName || "(custom header)"
                  : state.authMode,
              secret_reference: buildSecretReferenceFromState(state) || null,
            }}
          />
        </div>
      ) : null}
      <StringMapEditor
        label="Additional Secret Headers"
        idPrefix={`${idPrefix}_additional`}
        value={state.additionalHeadersJson}
        onChange={(nextValue) => updateState({ additionalHeadersJson: nextValue })}
        placeholderKey="x-service-region"
        placeholderValue="vault://kv/team/prod#region"
        helperText="Add any other secret-backed headers that should resolve at dispatch time."
      />
      {state.error ? <p className="text-[11px] text-red-600">{state.error}</p> : null}
    </div>
  );
}

function draftObjectFieldValuePlaceholder(valueKind: DraftValueKind): string {
  switch (valueKind) {
    case "string":
      return "value or {{token}}";
    case "number":
      return "42";
    case "boolean":
      return "true";
    case "null":
      return "null";
    case "json":
      return '{\n  "nested": true\n}';
  }
}

function defaultDraftValueForKind(valueKind: DraftValueKind, currentValue: string): string {
  if (valueKind === "boolean") {
    return "true";
  }

  if (valueKind === "null") {
    return "";
  }

  return currentValue;
}

function DraftObjectFieldEditor({
  label,
  idPrefix,
  fields,
  onChange,
  helperText,
  placeholderKey,
  focusFieldKey,
  onFocusApplied,
}: {
  label: string;
  idPrefix: string;
  fields: DraftObjectField[];
  onChange: (nextFields: DraftObjectField[]) => void;
  helperText?: string;
  placeholderKey: string;
  focusFieldKey?: string | null;
  onFocusApplied?: () => void;
}) {
  const fieldRefs = useRef<Record<string, HTMLInputElement | null>>({});

  useEffect(() => {
    if (!focusFieldKey) {
      return;
    }

    const focusedField = fields.find((field) => field.key.trim() === focusFieldKey);
    if (!focusedField) {
      onFocusApplied?.();
      return;
    }

    const input = fieldRefs.current[focusedField.id];
    if (!input) {
      onFocusApplied?.();
      return;
    }

    input.focus();
    input.select();
    onFocusApplied?.();
  }, [fields, focusFieldKey, onFocusApplied]);

  function addField() {
    onChange([
      ...fields,
      {
        id: createDraftFieldId(),
        key: "",
        valueKind: "string",
        value: "",
      },
    ]);
  }

  function updateField(
    fieldId: string,
    patch: Partial<Pick<DraftObjectField, "key" | "valueKind" | "value">>,
  ) {
    onChange(
      fields.map((field) =>
        field.id === fieldId
          ? {
              ...field,
              ...patch,
              value:
                patch.valueKind && patch.valueKind !== field.valueKind
                  ? defaultDraftValueForKind(patch.valueKind, field.value)
                  : patch.value ?? field.value,
            }
          : field,
      ),
    );
  }

  function removeField(fieldId: string) {
    onChange(fields.filter((field) => field.id !== fieldId));
  }

  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between">
        <Label htmlFor={`${idPrefix}_key_0`}>{label}</Label>
        <Button type="button" variant="outline" size="sm" onClick={addField}>
          <Plus className="size-3.5" />
          Add field
        </Button>
      </div>
      <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/60 p-3">
        {fields.length === 0 ? (
          <p className="text-[11px] text-zinc-500">No fields yet. Add a field to build the object payload.</p>
        ) : (
          fields.map((field, index) => (
            <div key={field.id} className="space-y-2 rounded-lg border border-zinc-200 bg-white p-3">
              <div className="grid grid-cols-[1.4fr_0.8fr_auto] gap-2">
                <Input
                  id={`${idPrefix}_key_${index}`}
                  ref={(node) => {
                    fieldRefs.current[field.id] = node;
                  }}
                  value={field.key}
                  onChange={(event) => updateField(field.id, { key: event.target.value })}
                  placeholder={placeholderKey}
                />
                <Select
                  id={`${idPrefix}_kind_${index}`}
                  value={field.valueKind}
                  onChange={(event) =>
                    updateField(field.id, {
                      valueKind: event.target.value as DraftValueKind,
                    })
                  }
                >
                  <option value="string">Text</option>
                  <option value="number">Number</option>
                  <option value="boolean">Boolean</option>
                  <option value="null">Null</option>
                  <option value="json">JSON</option>
                </Select>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeField(field.id)}
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
              {field.valueKind === "boolean" ? (
                <Select
                  id={`${idPrefix}_value_${index}`}
                  value={field.value === "false" ? "false" : "true"}
                  onChange={(event) => updateField(field.id, { value: event.target.value })}
                >
                  <option value="true">True</option>
                  <option value="false">False</option>
                </Select>
              ) : field.valueKind === "null" ? (
                <p className="rounded border border-dashed border-zinc-200 px-3 py-2 text-[11px] text-zinc-500">
                  This field will be stored as `null`.
                </p>
              ) : field.valueKind === "json" ? (
                <Textarea
                  id={`${idPrefix}_value_${index}`}
                  className="font-mono text-xs"
                  rows={4}
                  value={field.value}
                  onChange={(event) => updateField(field.id, { value: event.target.value })}
                  placeholder={draftObjectFieldValuePlaceholder(field.valueKind)}
                />
              ) : (
                <Input
                  id={`${idPrefix}_value_${index}`}
                  value={field.value}
                  onChange={(event) => updateField(field.id, { value: event.target.value })}
                  placeholder={draftObjectFieldValuePlaceholder(field.valueKind)}
                />
              )}
            </div>
          ))
        )}
      </div>
      {helperText ? <p className="text-[11px] text-zinc-500">{helperText}</p> : null}
      <TokenChips value={tokenTextFromDraftObjectFields(fields)} />
    </div>
  );
}

function DraftArrayItemEditor({
  label,
  idPrefix,
  items,
  onChange,
  helperText,
}: {
  label: string;
  idPrefix: string;
  items: DraftArrayItem[];
  onChange: (nextItems: DraftArrayItem[]) => void;
  helperText?: string;
}) {
  function addItem() {
    onChange([
      ...items,
      {
        id: createDraftFieldId(),
        valueKind: "string",
        value: "",
      },
    ]);
  }

  function updateItem(
    itemId: string,
    patch: Partial<Pick<DraftArrayItem, "valueKind" | "value">>,
  ) {
    onChange(
      items.map((item) =>
        item.id === itemId
          ? {
              ...item,
              ...patch,
              value:
                patch.valueKind && patch.valueKind !== item.valueKind
                  ? defaultDraftValueForKind(patch.valueKind, item.value)
                  : patch.value ?? item.value,
            }
          : item,
      ),
    );
  }

  function removeItem(itemId: string) {
    onChange(items.filter((item) => item.id !== itemId));
  }

  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between">
        <Label htmlFor={`${idPrefix}_kind_0`}>{label}</Label>
        <Button type="button" variant="outline" size="sm" onClick={addItem}>
          <Plus className="size-3.5" />
          Add item
        </Button>
      </div>
      <div className="space-y-2 rounded-xl border border-zinc-200 bg-zinc-50/60 p-3">
        {items.length === 0 ? (
          <p className="text-[11px] text-zinc-500">No items yet. Add an item to build the array body.</p>
        ) : (
          items.map((item, index) => (
            <div key={item.id} className="space-y-2 rounded-lg border border-zinc-200 bg-white p-3">
              <div className="grid grid-cols-[0.9fr_auto] gap-2">
                <Select
                  id={`${idPrefix}_kind_${index}`}
                  value={item.valueKind}
                  onChange={(event) =>
                    updateItem(item.id, {
                      valueKind: event.target.value as DraftValueKind,
                    })
                  }
                >
                  <option value="string">Text</option>
                  <option value="number">Number</option>
                  <option value="boolean">Boolean</option>
                  <option value="null">Null</option>
                  <option value="json">JSON</option>
                </Select>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => removeItem(item.id)}
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
              {item.valueKind === "boolean" ? (
                <Select
                  id={`${idPrefix}_value_${index}`}
                  value={item.value === "false" ? "false" : "true"}
                  onChange={(event) => updateItem(item.id, { value: event.target.value })}
                >
                  <option value="true">True</option>
                  <option value="false">False</option>
                </Select>
              ) : item.valueKind === "null" ? (
                <p className="rounded border border-dashed border-zinc-200 px-3 py-2 text-[11px] text-zinc-500">
                  This item will be stored as `null`.
                </p>
              ) : item.valueKind === "json" ? (
                <Textarea
                  id={`${idPrefix}_value_${index}`}
                  className="font-mono text-xs"
                  rows={4}
                  value={item.value}
                  onChange={(event) => updateItem(item.id, { value: event.target.value })}
                  placeholder={draftObjectFieldValuePlaceholder(item.valueKind)}
                />
              ) : (
                <Input
                  id={`${idPrefix}_value_${index}`}
                  value={item.value}
                  onChange={(event) => updateItem(item.id, { value: event.target.value })}
                  placeholder={draftObjectFieldValuePlaceholder(item.valueKind)}
                />
              )}
            </div>
          ))
        )}
      </div>
      {helperText ? <p className="text-[11px] text-zinc-500">{helperText}</p> : null}
      <TokenChips value={items.map((item) => item.value).join(" ")} />
    </div>
  );
}

type AutoMappedFieldPreview = {
  key: string;
  sourcePath: string;
};

function triggerPayloadMappedFieldsFromJson(dataJson: string): AutoMappedFieldPreview[] {
  const pattern = /"([^"]+)"\s*:\s*"\{\{\s*trigger\.payload\.([^}\s]+)\s*\}\}"/g;
  const matches = Array.from(dataJson.matchAll(pattern));
  if (matches.length === 0) {
    return [];
  }

  const previews: AutoMappedFieldPreview[] = [];
  const seen = new Set<string>();
  for (const match of matches) {
    const key = (match[1] ?? "").trim();
    const sourcePath = (match[2] ?? "").trim();
    if (key.length === 0 || sourcePath.length === 0) {
      continue;
    }

    const dedupeKey = `${key}::${sourcePath}`;
    if (seen.has(dedupeKey)) {
      continue;
    }
    seen.add(dedupeKey);
    previews.push({ key, sourcePath });
  }

  return previews;
}

function triggerPayloadMappedFieldsFromText(
  value: string,
  key: string,
): AutoMappedFieldPreview[] {
  const matches = Array.from(value.matchAll(/\{\{\s*trigger\.payload\.([^}\s]+)\s*\}\}/g));
  return matches.reduce<AutoMappedFieldPreview[]>((previews, match) => {
    const sourcePath = (match[1] ?? "").trim();
    if (sourcePath.length === 0) {
      return previews;
    }

    if (previews.some((preview) => preview.key === key && preview.sourcePath === sourcePath)) {
      return previews;
    }

    previews.push({ key, sourcePath });
    return previews;
  }, []);
}

function triggerPayloadMappedFieldsFromDraftArrayItems(
  items: DraftArrayItem[],
): AutoMappedFieldPreview[] {
  return items.flatMap((item, index) =>
    triggerPayloadMappedFieldsFromText(item.value, `[${index + 1}]`),
  );
}

function triggerPayloadMappedFieldsFromDraftObjectFields(
  fields: DraftObjectField[],
): AutoMappedFieldPreview[] {
  return fields.reduce<AutoMappedFieldPreview[]>((previews, field) => {
    const match = field.value.match(/\{\{\s*trigger\.payload\.([^}\s]+)\s*\}\}/);
    const sourcePath = match?.[1]?.trim() ?? "";
    const key = field.key.trim();
    if (key.length === 0 || sourcePath.length === 0) {
      return previews;
    }

    if (previews.some((preview) => preview.key === key && preview.sourcePath === sourcePath)) {
      return previews;
    }

    previews.push({ key, sourcePath });
    return previews;
  }, []);
}

function stringifyDraftObjectFields(fields: DraftObjectField[], fieldLabel: string): string {
  return JSON.stringify(parseDraftObjectFields(fields, fieldLabel), null, 2);
}

function stringifyDraftObjectFieldsOrFallback(
  fields: DraftObjectField[],
  fieldLabel: string,
  fallback: string,
): string {
  try {
    return stringifyDraftObjectFields(fields, fieldLabel);
  } catch {
    return fallback;
  }
}

function createDraftHttpBodyFieldsFromJson(bodyJson: string): DraftObjectField[] {
  try {
    const parsed = parseJsonValue(bodyJson, "HTTP request body");
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return createDraftObjectFieldsFromValue(parsed as Record<string, unknown>);
    }
  } catch {
    return [];
  }

  return [];
}

function createDraftHttpBodyArrayItemsFromJson(bodyJson: string): DraftArrayItem[] {
  try {
    const parsed = parseJsonValue(bodyJson, "HTTP request body");
    if (Array.isArray(parsed)) {
      return createDraftArrayItemsFromValue(parsed);
    }
  } catch {
    return [];
  }

  return [];
}

function createDraftScalarBodyFromJson(bodyJson: string): {
  valueKind: DraftValueKind;
  value: string;
} {
  try {
    const parsed = parseJsonValue(bodyJson, "HTTP request body");
    if (
      parsed === null ||
      typeof parsed === "string" ||
      typeof parsed === "number" ||
      typeof parsed === "boolean"
    ) {
      return {
        valueKind:
          parsed === null
            ? "null"
            : typeof parsed === "string"
              ? "string"
              : typeof parsed === "number"
                ? "number"
                : "boolean",
        value: parsed === null ? "" : String(parsed),
      };
    }
  } catch {
    return {
      valueKind: "string",
      value: "",
    };
  }

  return {
    valueKind: "string",
    value: "",
  };
}

function stringifyDraftArrayItemsOrFallback(
  items: DraftArrayItem[],
  fieldLabel: string,
  fallback: string,
): string {
  try {
    return JSON.stringify(parseDraftArrayItems(items, fieldLabel), null, 2);
  } catch {
    return fallback;
  }
}

function stringifyDraftScalarOrFallback(
  valueKind: DraftValueKind,
  value: string,
  fieldLabel: string,
  fallback: string,
): string {
  try {
    return JSON.stringify(parseDraftValue(valueKind, value, fieldLabel), null, 2);
  } catch {
    return fallback;
  }
}

function JsonPreviewCard({
  label,
  value,
  error,
}: {
  label: string;
  value: unknown;
  error?: string | null;
}) {
  return (
    <div className="space-y-1.5 rounded-xl border border-zinc-200 bg-white/70 p-3">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
        {label}
      </p>
      <Textarea
        className="font-mono text-[11px]"
        rows={6}
        value={JSON.stringify(value, null, 2)}
        readOnly
      />
      {error ? <p className="text-[11px] text-red-600">{error}</p> : null}
    </div>
  );
}

function retryConfigFromPreset(preset: RetryPreset): {
  strategy: RetryWorkflowStepStrategyDto;
  backoffMs?: number;
} {
  if (preset === "immediate") return { strategy: "immediate" };
  if (preset === "backoff_2000") return { strategy: "backoff", backoffMs: 2000 };
  if (preset === "backoff_5000") return { strategy: "backoff", backoffMs: 5000 };
  return { strategy: "backoff", backoffMs: 800 };
}

function stepIcon(type: DraftWorkflowStep["type"]) {
  switch (type) {
    case "log_message":
      return <MessageSquare className="size-4" />;
    case "create_runtime_record":
      return <Database className="size-4" />;
    case "update_runtime_record":
      return <Database className="size-4" />;
    case "delete_runtime_record":
      return <XCircle className="size-4" />;
    case "send_email":
      return <Mail className="size-4" />;
    case "http_request":
      return <Globe className="size-4" />;
    case "webhook":
      return <ExternalLink className="size-4" />;
    case "assign_owner":
      return <Bell className="size-4" />;
    case "approval_request":
      return <CheckCircle2 className="size-4" />;
    case "delay":
      return <Clock3 className="size-4" />;
    case "condition":
      return <GitBranch className="size-4" />;
  }
}

function stepIconBg(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "bg-blue-100 text-blue-700";
    case "create_runtime_record":
      return "bg-sky-100 text-sky-700";
    case "update_runtime_record":
      return "bg-cyan-100 text-cyan-700";
    case "delete_runtime_record":
      return "bg-red-100 text-red-700";
    case "send_email":
      return "bg-rose-100 text-rose-700";
    case "http_request":
      return "bg-violet-100 text-violet-700";
    case "webhook":
      return "bg-emerald-100 text-emerald-700";
    case "assign_owner":
      return "bg-lime-100 text-lime-700";
    case "approval_request":
      return "bg-fuchsia-100 text-fuchsia-700";
    case "delay":
      return "bg-stone-100 text-stone-700";
    case "condition":
      return "bg-amber-100 text-amber-700";
  }
}

function stepBorderColor(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "border-blue-200";
    case "create_runtime_record":
      return "border-sky-200";
    case "update_runtime_record":
      return "border-cyan-200";
    case "delete_runtime_record":
      return "border-red-200";
    case "send_email":
      return "border-rose-200";
    case "http_request":
      return "border-violet-200";
    case "webhook":
      return "border-emerald-200";
    case "assign_owner":
      return "border-lime-200";
    case "approval_request":
      return "border-fuchsia-200";
    case "delay":
      return "border-stone-200";
    case "condition":
      return "border-amber-200";
  }
}

function stepTypeLabel(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "Log Message";
    case "create_runtime_record":
      return "Create Record";
    case "update_runtime_record":
      return "Update Record";
    case "delete_runtime_record":
      return "Delete Record";
    case "send_email":
      return "Send Email";
    case "http_request":
      return "HTTP Request";
    case "webhook":
      return "Webhook";
    case "assign_owner":
      return "Assign Owner";
    case "approval_request":
      return "Approval Request";
    case "delay":
      return "Delay";
    case "condition":
      return "Condition";
  }
}

function stepTypeLabelColor(type: DraftWorkflowStep["type"]): string {
  switch (type) {
    case "log_message":
      return "text-blue-700";
    case "create_runtime_record":
      return "text-sky-700";
    case "update_runtime_record":
      return "text-cyan-700";
    case "delete_runtime_record":
      return "text-red-700";
    case "send_email":
      return "text-rose-700";
    case "http_request":
      return "text-violet-700";
    case "webhook":
      return "text-emerald-700";
    case "assign_owner":
      return "text-lime-700";
    case "approval_request":
      return "text-fuchsia-700";
    case "delay":
      return "text-stone-700";
    case "condition":
      return "text-amber-700";
  }
}

export function FlowConnector({
  onAdd,
  disabled = false,
}: {
  onAdd: () => void;
  disabled?: boolean;
}) {
  return (
    <div className="flex flex-col items-center">
      <div className="h-6 w-px bg-zinc-200" />
      <button
        type="button"
        className="flex size-6 items-center justify-center rounded-full border border-zinc-200 bg-white text-zinc-400 shadow-sm transition enabled:hover:border-emerald-300 enabled:hover:bg-emerald-50 enabled:hover:text-emerald-600 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-400 disabled:cursor-not-allowed disabled:opacity-40"
        onClick={onAdd}
        disabled={disabled}
        title="Add step here"
      >
        <Plus className="size-3.5" />
      </button>
      <div className="h-6 w-px bg-zinc-200" />
    </div>
  );
}

function StepTraceStatus({
  trace,
  showNotExecuted = false,
}: {
  trace: WorkflowRunStepTraceResponse | null;
  showNotExecuted?: boolean;
}) {
  if (!trace) {
    if (!showNotExecuted) {
      return null;
    }

    return (
      <div className="mt-1 flex items-center gap-1 text-[10px] font-medium text-zinc-400">
        <Clock3 className="size-3" />
        <span>not executed</span>
      </div>
    );
  }

  const ok = trace.status === "succeeded";
  return (
    <div
      className={`mt-1 flex items-center gap-1 text-[10px] font-medium ${ok ? "text-emerald-600" : "text-red-600"}`}
    >
      {ok ? <CheckCircle2 className="size-3" /> : <XCircle className="size-3" />}
      <span className="capitalize">{trace.status}</span>
      {trace.duration_ms !== null && (
        <span className="text-zinc-400">· {String(trace.duration_ms)}ms</span>
      )}
    </div>
  );
}

type TriggerCardProps = {
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  isExpanded: boolean;
  onToggle: () => void;
  onTriggerTypeChange: (type: TriggerType) => void;
  onTriggerEntityChange: (entity: string) => void;
  runtimeEntityOptions: Array<{ value: string; label: string }>;
};

export function TriggerCard({
  triggerType,
  triggerEntityLogicalName,
  isExpanded,
  onToggle,
  onTriggerTypeChange,
  onTriggerEntityChange,
  runtimeEntityOptions,
}: TriggerCardProps) {
  const subtitle =
    triggerType === "manual"
      ? "Manual trigger"
      : triggerType === "schedule_tick"
        ? `Schedule tick · ${triggerEntityLogicalName.trim() || "schedule key not set"}`
        : triggerType === "webhook_received"
          ? `Webhook received · ${triggerEntityLogicalName.trim() || "webhook key not set"}`
          : triggerType === "form_submitted"
            ? `Form submitted · ${triggerEntityLogicalName.trim() || "form key not set"}`
            : triggerType === "inbound_email_received"
              ? `Inbound email · ${triggerEntityLogicalName.trim() || "mailbox key not set"}`
              : triggerType === "approval_event_received"
                ? `Approval event · ${triggerEntityLogicalName.trim() || "approval key not set"}`
        : `${
            triggerType === "runtime_record_updated"
              ? "Record updated"
              : triggerType === "runtime_record_deleted"
                ? "Record deleted"
                : "Record created"
          } · ${triggerEntityLogicalName.trim() || "entity not set"}`;

  return (
    <div
      className={`w-full overflow-hidden rounded-xl border bg-white shadow-sm transition-shadow ${
        isExpanded ? "border-emerald-300 shadow-md" : "border-emerald-200 hover:shadow"
      }`}
    >
      <button
        type="button"
        className="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-emerald-50/40"
        onClick={onToggle}
      >
        <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-emerald-100 text-emerald-700">
          <Bell className="size-4" />
        </div>
        <div className="min-w-0 flex-1">
          <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-emerald-700">
            Trigger
          </p>
          <p className="truncate text-sm font-medium text-zinc-900">{subtitle}</p>
        </div>
        <div className="shrink-0 text-zinc-400">
          {isExpanded ? <ChevronUp className="size-4" /> : <ChevronDown className="size-4" />}
        </div>
      </button>

      {isExpanded && (
        <div className="space-y-4 border-t border-emerald-100 bg-zinc-50/50 p-4">
          <TriggerConfigPanel
            triggerType={triggerType}
            triggerEntityLogicalName={triggerEntityLogicalName}
            runtimeEntityOptions={runtimeEntityOptions}
            onTriggerTypeChange={onTriggerTypeChange}
            onTriggerEntityChange={onTriggerEntityChange}
          />
        </div>
      )}
    </div>
  );
}

function TokenChips({ value }: { value: string }) {
  const chips = tokenChipsFromValue(value);
  if (chips.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-1">
      {chips.map((chip) => (
        <span
          key={chip}
          className="rounded-full border border-emerald-200 bg-emerald-50 px-2 py-0.5 font-mono text-[10px] text-emerald-800"
        >
          {chip}
        </span>
      ))}
    </div>
  );
}

function FieldSuggestionPicker({
  title,
  helper,
  fields,
  onPick,
}: {
  title: string;
  helper: string;
  fields: string[];
  onPick: (fieldPath: string) => void;
}) {
  const [query, setQuery] = useState("");

  const filtered = query.trim().length
    ? fields.filter((fieldPath) =>
        fieldPath.toLowerCase().includes(query.trim().toLowerCase()),
      )
    : fields;

  if (fields.length === 0) {
    return null;
  }

  return (
    <details className="rounded-md border border-zinc-200 bg-white p-2">
      <summary className="cursor-pointer text-[11px] font-semibold text-zinc-700">{title}</summary>
      <div className="mt-2 space-y-2">
        <p className="text-[10px] text-zinc-500">{helper}</p>
        <Input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Filter fields..."
          className="h-8 text-xs"
        />
        <div className="max-h-28 overflow-y-auto rounded border border-zinc-200 p-1">
          {filtered.length === 0 ? (
            <p className="px-2 py-1 text-[10px] text-zinc-400">No fields match.</p>
          ) : (
            <div className="flex flex-wrap gap-1">
              {filtered.slice(0, 40).map((fieldPath) => (
                <button
                  key={fieldPath}
                  type="button"
                  className="rounded border border-zinc-300 bg-zinc-50 px-2 py-1 font-mono text-[10px] text-zinc-700 hover:border-emerald-300 hover:bg-emerald-50 hover:text-emerald-700"
                  onClick={() => onPick(fieldPath)}
                >
                  {fieldPath}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </details>
  );
}

type UpdateFn = (updater: (step: DraftWorkflowStep) => DraftWorkflowStep) => void;

function LogMessageForm({
  step,
  availableTokens,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "log_message" }>;
  availableTokens: DynamicTokenOption[];
  onUpdate: UpdateFn;
}) {
  return (
    <div className="space-y-3">
      <div className="space-y-1.5">
        <Label htmlFor={`msg_${step.id}`}>Message</Label>
        <Input
          id={`msg_${step.id}`}
          value={step.message}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "log_message" ? { ...s, message: e.target.value } : s,
            )
          }
          placeholder="Enter message or build an expression..."
        />
        {step.message.trim().length === 0 && (
          <p className="text-[11px] text-red-600">Message is required.</p>
        )}
        <TokenChips value={step.message} />
      </div>
      <ExpressionBuilderPopover
        title="Message Expression"
        currentValue={step.message}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "log_message"
              ? { ...s, message: appendExpression(s.message, expr) }
              : s,
          )
        }
      />
    </div>
  );
}

function CreateRecordForm({
  step,
  runtimeEntityOptions,
  fieldPathSuggestions,
  focusedFieldKey,
  onFocusApplied,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "create_runtime_record" }>;
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  fieldPathSuggestions: string[];
  focusedFieldKey: string | null;
  onFocusApplied: () => void;
  onUpdate: UpdateFn;
}) {
  let previewPayload: Record<string, unknown> = {};
  let previewError: string | null = null;
  try {
    previewPayload = parseDraftObjectFields(step.dataFields, "Create record step data");
  } catch (error) {
    previewError =
      error instanceof Error ? error.message : "Create record step data contains invalid fields.";
  }

  return (
    <div className="space-y-3">
      <div className="space-y-1.5">
        <Label htmlFor={`entity_${step.id}`}>Entity logical name</Label>
        <Input
          id={`entity_${step.id}`}
          value={step.entityLogicalName}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "create_runtime_record"
                ? { ...s, entityLogicalName: e.target.value }
                : s,
            )
          }
          placeholder="contact, task, note..."
          list={`entity_suggestions_${step.id}`}
        />
        <datalist id={`entity_suggestions_${step.id}`}>
          {runtimeEntityOptions.map((entity) => (
            <option key={entity.value} value={entity.value} />
          ))}
        </datalist>
        {step.entityLogicalName.trim().length === 0 && (
          <p className="text-[11px] text-red-600">Entity name is required.</p>
        )}
        {fieldPathSuggestions.length > 0 ? (
          <p className="text-[11px] text-zinc-500">
            Known fields: {fieldPathSuggestions.slice(0, 8).join(", ")}
            {fieldPathSuggestions.length > 8 ? " ..." : ""}
          </p>
        ) : null}
        <FieldSuggestionPicker
          title="Field Mapping Picker"
          helper="Click a field to append a trigger token mapping into the record payload."
          fields={fieldPathSuggestions}
          onPick={(fieldPath) =>
            onUpdate((currentStep) =>
              currentStep.type === "create_runtime_record"
                ? {
                    ...currentStep,
                    dataFields: insertTokenMappingIntoDraftObjectFields(
                      currentStep.dataFields,
                      fieldPath,
                    ),
                  }
                : currentStep,
            )
          }
        />
      </div>
      <DraftObjectFieldEditor
        label="Record Fields"
        idPrefix={`data_${step.id}`}
        fields={step.dataFields}
        onChange={(nextFields) =>
          onUpdate((s) =>
            s.type === "create_runtime_record" ? { ...s, dataFields: nextFields } : s,
          )
        }
        placeholderKey="title"
        helperText="Build the record payload field by field. Use Text for tokens, Number/Boolean/Null for typed values, or JSON for nested objects."
        focusFieldKey={focusedFieldKey}
        onFocusApplied={onFocusApplied}
      />
      <JsonPreviewCard label="Create payload preview" value={previewPayload} error={previewError} />
    </div>
  );
}

function UpdateRecordForm({
  step,
  runtimeEntityOptions,
  fieldPathSuggestions,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "update_runtime_record" }>;
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  fieldPathSuggestions: string[];
  onUpdate: UpdateFn;
}) {
  let previewPayload: Record<string, unknown> = {};
  let previewError: string | null = null;
  try {
    previewPayload = parseDraftObjectFields(step.dataFields, "Update record step data");
  } catch (error) {
    previewError =
      error instanceof Error ? error.message : "Update record step data contains invalid fields.";
  }

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`update_entity_${step.id}`}>Entity logical name</Label>
          <Input
            id={`update_entity_${step.id}`}
            value={step.entityLogicalName}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "update_runtime_record"
                  ? { ...s, entityLogicalName: e.target.value }
                  : s,
              )
            }
            list={`update_entity_suggestions_${step.id}`}
          />
          <datalist id={`update_entity_suggestions_${step.id}`}>
            {runtimeEntityOptions.map((entity) => (
              <option key={entity.value} value={entity.value} />
            ))}
          </datalist>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`update_record_id_${step.id}`}>Record id</Label>
          <Input
            id={`update_record_id_${step.id}`}
            value={step.recordId}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "update_runtime_record" ? { ...s, recordId: e.target.value } : s,
              )
            }
            placeholder="{{trigger.payload.record_id}}"
          />
          <TokenChips value={step.recordId} />
        </div>
      </div>
      {fieldPathSuggestions.length > 0 ? (
        <p className="text-[11px] text-zinc-500">
          Known fields: {fieldPathSuggestions.slice(0, 8).join(", ")}
          {fieldPathSuggestions.length > 8 ? " ..." : ""}
        </p>
      ) : null}
      <FieldSuggestionPicker
        title="Field Mapping Picker"
        helper="Click a field to add a mapped update value."
        fields={fieldPathSuggestions}
        onPick={(fieldPath) =>
          onUpdate((currentStep) =>
            currentStep.type === "update_runtime_record"
              ? {
                  ...currentStep,
                  dataFields: insertTokenMappingIntoDraftObjectFields(
                    currentStep.dataFields,
                    fieldPath,
                  ),
                }
              : currentStep,
          )
        }
      />
      <DraftObjectFieldEditor
        label="Updated Fields"
        idPrefix={`update_data_${step.id}`}
        fields={step.dataFields}
        onChange={(nextFields) =>
          onUpdate((s) =>
            s.type === "update_runtime_record" ? { ...s, dataFields: nextFields } : s,
          )
        }
        placeholderKey="status"
        helperText="Only the listed fields are written during the update step."
      />
      <JsonPreviewCard label="Update payload preview" value={previewPayload} error={previewError} />
    </div>
  );
}

function DeleteRecordForm({
  step,
  availableTokens,
  runtimeEntityOptions,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "delete_runtime_record" }>;
  availableTokens: DynamicTokenOption[];
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  onUpdate: UpdateFn;
}) {
  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`delete_entity_${step.id}`}>Entity logical name</Label>
          <Input
            id={`delete_entity_${step.id}`}
            value={step.entityLogicalName}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "delete_runtime_record"
                  ? { ...s, entityLogicalName: e.target.value }
                  : s,
              )
            }
            list={`delete_entity_suggestions_${step.id}`}
          />
          <datalist id={`delete_entity_suggestions_${step.id}`}>
            {runtimeEntityOptions.map((entity) => (
              <option key={entity.value} value={entity.value} />
            ))}
          </datalist>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`delete_record_id_${step.id}`}>Record id</Label>
          <Input
            id={`delete_record_id_${step.id}`}
            value={step.recordId}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "delete_runtime_record" ? { ...s, recordId: e.target.value } : s,
              )
            }
            placeholder="{{trigger.payload.record_id}}"
          />
          <TokenChips value={step.recordId} />
        </div>
      </div>
      <ExpressionBuilderPopover
        title="Delete Record Expression"
        currentValue={step.recordId}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "delete_runtime_record"
              ? { ...s, recordId: appendExpression(s.recordId, expr) }
              : s,
          )
        }
      />
    </div>
  );
}

function SendEmailForm({
  step,
  availableTokens,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "send_email" }>;
  availableTokens: DynamicTokenOption[];
  onUpdate: UpdateFn;
}) {
  return (
    <div className="space-y-3">
      <div className="space-y-1.5">
        <Label htmlFor={`email_to_${step.id}`}>To</Label>
        <Input
          id={`email_to_${step.id}`}
          value={step.to}
          onChange={(e) =>
            onUpdate((s) => (s.type === "send_email" ? { ...s, to: e.target.value } : s))
          }
          placeholder="ops@example.com"
        />
      </div>
      <div className="space-y-1.5">
        <Label htmlFor={`email_subject_${step.id}`}>Subject</Label>
        <Input
          id={`email_subject_${step.id}`}
          value={step.subject}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "send_email" ? { ...s, subject: e.target.value } : s,
            )
          }
          placeholder="Workflow alert"
        />
        <TokenChips value={step.subject} />
      </div>
      <div className="space-y-1.5">
        <Label htmlFor={`email_body_${step.id}`}>Body</Label>
        <Textarea
          id={`email_body_${step.id}`}
          rows={5}
          value={step.body}
          onChange={(e) =>
            onUpdate((s) => (s.type === "send_email" ? { ...s, body: e.target.value } : s))
          }
          placeholder="Explain what happened..."
        />
        <TokenChips value={step.body} />
      </div>
      <div className="space-y-1.5">
        <Label htmlFor={`email_html_${step.id}`}>HTML body (optional)</Label>
        <Textarea
          id={`email_html_${step.id}`}
          rows={4}
          value={step.htmlBody}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "send_email" ? { ...s, htmlBody: e.target.value } : s,
            )
          }
          placeholder="<p>Optional rich content</p>"
        />
        <TokenChips value={step.htmlBody} />
      </div>
      <ExpressionBuilderPopover
        title="Email Body Expression"
        currentValue={step.body}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "send_email" ? { ...s, body: appendExpression(s.body, expr) } : s,
          )
        }
      />
    </div>
  );
}

function HttpRequestForm({
  step,
  availableTokens,
  fieldPathSuggestions,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "http_request" }>;
  availableTokens: DynamicTokenOption[];
  fieldPathSuggestions: string[];
  onUpdate: UpdateFn;
}) {
  let bodyError: string | null = null;
  let requestPreview: Record<string, unknown> = {
    method: step.method,
    url: step.url,
    headers: step.headersJson,
    header_secret_refs: step.headerSecretRefsJson,
    body: null,
  };
  try {
    const bodyPreview =
      step.bodyMode === "none"
        ? null
        : step.bodyMode === "object"
          ? parseDraftObjectFields(step.bodyFields, "HTTP request body")
          : step.bodyMode === "array"
            ? parseDraftArrayItems(step.bodyArrayItems, "HTTP request body")
            : step.bodyMode === "scalar"
              ? parseDraftValue(
                  step.bodyScalarKind,
                  step.bodyScalarValue,
                  "HTTP request body",
                )
          : parseJsonValue(step.bodyJson, "HTTP request body");
    requestPreview = {
      method: step.method,
      url: step.url,
      headers: JSON.parse(step.headersJson) as unknown,
      header_secret_refs: JSON.parse(step.headerSecretRefsJson) as unknown,
      body: bodyPreview,
    };
  } catch (error) {
    bodyError =
      error instanceof Error ? error.message : "HTTP request body contains an invalid value.";
  }

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-3 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`http_method_${step.id}`}>Method</Label>
          <Input
            id={`http_method_${step.id}`}
            value={step.method}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "http_request" ? { ...s, method: e.target.value } : s,
              )
            }
            placeholder="POST"
          />
        </div>
        <div className="col-span-2 space-y-1.5">
          <Label htmlFor={`http_url_${step.id}`}>URL</Label>
          <Input
            id={`http_url_${step.id}`}
            value={step.url}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "http_request" ? { ...s, url: e.target.value } : s,
              )
            }
            placeholder="https://api.example.com/hooks/workflow"
          />
          <TokenChips value={step.url} />
        </div>
      </div>
      <StringMapEditor
        label="Headers"
        idPrefix={`http_headers_${step.id}`}
        value={step.headersJson}
        onChange={(nextValue) =>
          onUpdate((s) =>
            s.type === "http_request" ? { ...s, headersJson: nextValue } : s,
          )
        }
        placeholderKey="content-type"
        placeholderValue="application/json"
      />
      <SecretHeaderEditor
        idPrefix={`http_secret_headers_${step.id}`}
        value={step.headerSecretRefsJson}
        onChange={(nextValue) =>
          onUpdate((s) =>
            s.type === "http_request"
              ? { ...s, headerSecretRefsJson: nextValue }
              : s,
          )
        }
      />
      <div className="space-y-1.5">
        <Label htmlFor={`http_body_mode_${step.id}`}>Body mode</Label>
        <Select
          id={`http_body_mode_${step.id}`}
          value={step.bodyMode}
          onChange={(event) =>
            onUpdate((s) => {
              if (s.type !== "http_request") {
                return s;
              }

              const nextMode = event.target.value as
                | "none"
                | "object"
                | "array"
                | "scalar"
                | "json";
              if (nextMode === "object") {
                const nextFields =
                  s.bodyFields.length > 0
                    ? s.bodyFields
                    : createDraftHttpBodyFieldsFromJson(s.bodyJson);

                return {
                  ...s,
                  bodyMode: "object",
                  bodyFields: nextFields,
                  bodyJson:
                    nextFields.length > 0
                      ? stringifyDraftObjectFieldsOrFallback(
                          nextFields,
                          "HTTP request body",
                          s.bodyJson,
                        )
                      : JSON.stringify({}, null, 2),
                };
              }

              if (nextMode === "array") {
                const nextItems =
                  s.bodyArrayItems.length > 0
                    ? s.bodyArrayItems
                    : createDraftHttpBodyArrayItemsFromJson(s.bodyJson);

                return {
                  ...s,
                  bodyMode: "array",
                  bodyArrayItems: nextItems,
                  bodyJson:
                    nextItems.length > 0
                      ? stringifyDraftArrayItemsOrFallback(
                          nextItems,
                          "HTTP request body",
                          s.bodyJson,
                        )
                      : JSON.stringify([], null, 2),
                };
              }

              if (nextMode === "scalar") {
                const nextScalar =
                  s.bodyMode === "scalar"
                    ? {
                        valueKind: s.bodyScalarKind,
                        value: s.bodyScalarValue,
                      }
                    : createDraftScalarBodyFromJson(s.bodyJson);

                return {
                  ...s,
                  bodyMode: "scalar",
                  bodyScalarKind: nextScalar.valueKind,
                  bodyScalarValue: nextScalar.value,
                  bodyJson: stringifyDraftScalarOrFallback(
                    nextScalar.valueKind,
                    nextScalar.value,
                    "HTTP request body",
                    s.bodyJson,
                  ),
                };
              }

              if (nextMode === "json") {
                return {
                  ...s,
                  bodyMode: "json",
                  bodyJson:
                    s.bodyMode === "object"
                      ? stringifyDraftObjectFieldsOrFallback(
                          s.bodyFields,
                          "HTTP request body",
                          s.bodyJson,
                        )
                      : s.bodyMode === "array"
                        ? stringifyDraftArrayItemsOrFallback(
                            s.bodyArrayItems,
                            "HTTP request body",
                            s.bodyJson,
                          )
                        : s.bodyMode === "scalar"
                          ? stringifyDraftScalarOrFallback(
                              s.bodyScalarKind,
                              s.bodyScalarValue,
                              "HTTP request body",
                              s.bodyJson,
                            )
                      : s.bodyJson,
                };
              }

              return { ...s, bodyMode: "none" };
            })
          }
        >
          <option value="none">No request body</option>
          <option value="object">Typed object body</option>
          <option value="array">Typed array body</option>
          <option value="scalar">Typed scalar body</option>
          <option value="json">Raw JSON body</option>
        </Select>
      </div>
      {step.bodyMode === "object" ? (
        <>
          <FieldSuggestionPicker
            title="Body Mapping Picker"
            helper="Click a trigger field to append a token mapping into the HTTP body."
            fields={fieldPathSuggestions}
            onPick={(fieldPath) =>
              onUpdate((currentStep) => {
                if (currentStep.type !== "http_request") {
                  return currentStep;
                }

                const nextFields = insertTokenMappingIntoDraftObjectFields(
                  currentStep.bodyFields,
                  fieldPath,
                );
                return {
                  ...currentStep,
                  bodyFields: nextFields,
                  bodyJson: stringifyDraftObjectFieldsOrFallback(
                    nextFields,
                    "HTTP request body",
                    currentStep.bodyJson,
                  ),
                };
              })
            }
          />
          <DraftObjectFieldEditor
            label="Body Fields"
            idPrefix={`http_body_fields_${step.id}`}
            fields={step.bodyFields}
            onChange={(nextFields) =>
              onUpdate((s) =>
                s.type === "http_request"
                  ? {
                      ...s,
                      bodyFields: nextFields,
                      bodyJson: stringifyDraftObjectFieldsOrFallback(
                        nextFields,
                        "HTTP request body",
                        s.bodyJson,
                      ),
                    }
                  : s,
              )
            }
            placeholderKey="record_id"
            helperText="Build common JSON object bodies field by field. Switch body mode for arrays, scalar payloads, or full raw JSON."
          />
        </>
      ) : null}
      {step.bodyMode === "array" ? (
        <>
          <FieldSuggestionPicker
            title="Array Mapping Picker"
            helper="Click a trigger field to append a token item into the HTTP array body."
            fields={fieldPathSuggestions}
            onPick={(fieldPath) =>
              onUpdate((currentStep) => {
                if (currentStep.type !== "http_request") {
                  return currentStep;
                }

                const nextItems = [
                  ...currentStep.bodyArrayItems,
                  {
                    id: createDraftFieldId(),
                    valueKind: "string" as const,
                    value: `{{trigger.payload.${fieldPath}}}`,
                  },
                ];

                return {
                  ...currentStep,
                  bodyArrayItems: nextItems,
                  bodyJson: stringifyDraftArrayItemsOrFallback(
                    nextItems,
                    "HTTP request body",
                    currentStep.bodyJson,
                  ),
                };
              })
            }
          />
          <DraftArrayItemEditor
            label="Body Items"
            idPrefix={`http_body_items_${step.id}`}
            items={step.bodyArrayItems}
            onChange={(nextItems) =>
              onUpdate((s) =>
                s.type === "http_request"
                  ? {
                      ...s,
                      bodyArrayItems: nextItems,
                      bodyJson: stringifyDraftArrayItemsOrFallback(
                        nextItems,
                        "HTTP request body",
                        s.bodyJson,
                      ),
                    }
                  : s,
              )
            }
            helperText="Build array payloads item by item. Use JSON items for nested objects or nested arrays."
          />
        </>
      ) : null}
      {step.bodyMode === "scalar" ? (
        <>
          <div className="space-y-1.5">
            <Label htmlFor={`http_body_scalar_kind_${step.id}`}>Scalar type</Label>
            <Select
              id={`http_body_scalar_kind_${step.id}`}
              value={step.bodyScalarKind}
              onChange={(event) =>
                onUpdate((s) => {
                  if (s.type !== "http_request") {
                    return s;
                  }

                  const nextValueKind = event.target.value as DraftValueKind;
                  const nextValue = defaultDraftValueForKind(
                    nextValueKind,
                    s.bodyScalarValue,
                  );

                  return {
                    ...s,
                    bodyScalarKind: nextValueKind,
                    bodyScalarValue: nextValue,
                    bodyJson: stringifyDraftScalarOrFallback(
                      nextValueKind,
                      nextValue,
                      "HTTP request body",
                      s.bodyJson,
                    ),
                  };
                })
              }
            >
              <option value="string">Text</option>
              <option value="number">Number</option>
              <option value="boolean">Boolean</option>
              <option value="null">Null</option>
            </Select>
          </div>
          {step.bodyScalarKind === "boolean" ? (
            <div className="space-y-1.5">
              <Label htmlFor={`http_body_scalar_value_${step.id}`}>Body value</Label>
              <Select
                id={`http_body_scalar_value_${step.id}`}
                value={step.bodyScalarValue === "false" ? "false" : "true"}
                onChange={(event) =>
                  onUpdate((s) =>
                    s.type === "http_request"
                      ? {
                          ...s,
                          bodyScalarValue: event.target.value,
                          bodyJson: stringifyDraftScalarOrFallback(
                            s.bodyScalarKind,
                            event.target.value,
                            "HTTP request body",
                            s.bodyJson,
                          ),
                        }
                      : s,
                  )
                }
              >
                <option value="true">True</option>
                <option value="false">False</option>
              </Select>
            </div>
          ) : step.bodyScalarKind === "null" ? (
            <p className="rounded border border-dashed border-zinc-200 px-3 py-2 text-[11px] text-zinc-500">
              This request body will be stored as `null`.
            </p>
          ) : step.bodyScalarKind === "json" ? (
            <div className="space-y-1.5">
              <Label htmlFor={`http_body_scalar_value_${step.id}`}>Body value</Label>
              <Textarea
                id={`http_body_scalar_value_${step.id}`}
                className="font-mono text-xs"
                rows={4}
                value={step.bodyScalarValue}
                onChange={(event) =>
                  onUpdate((s) =>
                    s.type === "http_request"
                      ? {
                          ...s,
                          bodyScalarValue: event.target.value,
                          bodyJson: stringifyDraftScalarOrFallback(
                            s.bodyScalarKind,
                            event.target.value,
                            "HTTP request body",
                            s.bodyJson,
                          ),
                        }
                      : s,
                  )
                }
                placeholder='{"nested": true}'
              />
              <TokenChips value={step.bodyScalarValue} />
            </div>
          ) : (
            <div className="space-y-1.5">
              <Label htmlFor={`http_body_scalar_value_${step.id}`}>Body value</Label>
              <Input
                id={`http_body_scalar_value_${step.id}`}
                value={step.bodyScalarValue}
                onChange={(event) =>
                  onUpdate((s) =>
                    s.type === "http_request"
                      ? {
                          ...s,
                          bodyScalarValue: event.target.value,
                          bodyJson: stringifyDraftScalarOrFallback(
                            s.bodyScalarKind,
                            event.target.value,
                            "HTTP request body",
                            s.bodyJson,
                          ),
                        }
                      : s,
                  )
                }
                placeholder={
                  step.bodyScalarKind === "number" ? "42" : "value or {{token}}"
                }
              />
              <TokenChips value={step.bodyScalarValue} />
            </div>
          )}
          {step.bodyScalarKind === "string" ? (
            <ExpressionBuilderPopover
              title="HTTP Body Expression"
              currentValue={step.bodyScalarValue}
              tokens={availableTokens}
              onInsertExpression={(expr) =>
                onUpdate((s) =>
                  s.type === "http_request"
                    ? {
                        ...s,
                        bodyScalarValue: appendExpression(s.bodyScalarValue, expr),
                        bodyJson: stringifyDraftScalarOrFallback(
                          s.bodyScalarKind,
                          appendExpression(s.bodyScalarValue, expr),
                          "HTTP request body",
                          s.bodyJson,
                        ),
                      }
                    : s,
                )
              }
            />
          ) : null}
        </>
      ) : null}
      {step.bodyMode === "json" ? (
        <>
          <div className="space-y-1.5">
            <Label htmlFor={`http_body_${step.id}`}>Body (JSON)</Label>
            <Textarea
              id={`http_body_${step.id}`}
              className="font-mono text-xs"
              rows={5}
              value={step.bodyJson}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "http_request" ? { ...s, bodyJson: e.target.value } : s,
                )
              }
            />
            {bodyError && <p className="text-[11px] text-red-600">{bodyError}</p>}
            <TokenChips value={step.bodyJson} />
          </div>
          <ExpressionBuilderPopover
            title="HTTP Body Expression"
            currentValue={step.bodyJson}
            tokens={availableTokens}
            onInsertExpression={(expr) =>
              onUpdate((s) =>
                s.type === "http_request"
                  ? { ...s, bodyJson: appendExpression(s.bodyJson, expr) }
                  : s,
              )
            }
          />
        </>
      ) : null}
      <JsonPreviewCard label="HTTP request preview" value={requestPreview} error={bodyError} />
    </div>
  );
}

function WebhookForm({
  step,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "webhook" }>;
  onUpdate: UpdateFn;
}) {
  let payloadPreview: Record<string, unknown> = {};
  let payloadError: string | null = null;
  try {
    payloadPreview = parseDraftObjectFields(step.payloadFields, "Webhook payload");
  } catch (error) {
    payloadError =
      error instanceof Error ? error.message : "Webhook payload contains invalid fields.";
  }

  return (
    <div className="space-y-3">
      <div className="space-y-1.5">
        <Label htmlFor={`webhook_endpoint_${step.id}`}>Endpoint</Label>
        <Input
          id={`webhook_endpoint_${step.id}`}
          value={step.endpoint}
          onChange={(e) =>
            onUpdate((s) =>
              s.type === "webhook" ? { ...s, endpoint: e.target.value } : s,
            )
          }
          placeholder="https://example.org/workflow-callback"
        />
        <TokenChips value={step.endpoint} />
      </div>
      <div className="space-y-1.5">
        <Label htmlFor={`webhook_event_${step.id}`}>Event</Label>
        <Input
          id={`webhook_event_${step.id}`}
          value={step.event}
          onChange={(e) =>
            onUpdate((s) => (s.type === "webhook" ? { ...s, event: e.target.value } : s))
          }
          placeholder="workflow.completed"
        />
      </div>
      <StringMapEditor
        label="Headers"
        idPrefix={`webhook_headers_${step.id}`}
        value={step.headersJson}
        onChange={(nextValue) =>
          onUpdate((s) =>
            s.type === "webhook" ? { ...s, headersJson: nextValue } : s,
          )
        }
        placeholderKey="content-type"
        placeholderValue="application/json"
      />
      <SecretHeaderEditor
        idPrefix={`webhook_secret_headers_${step.id}`}
        value={step.headerSecretRefsJson}
        onChange={(nextValue) =>
          onUpdate((s) =>
            s.type === "webhook"
              ? { ...s, headerSecretRefsJson: nextValue }
              : s,
          )
        }
      />
      <DraftObjectFieldEditor
        label="Payload Fields"
        idPrefix={`webhook_payload_${step.id}`}
        fields={step.payloadFields}
        onChange={(nextFields) =>
          onUpdate((s) => (s.type === "webhook" ? { ...s, payloadFields: nextFields } : s))
        }
        placeholderKey="run_id"
        helperText="Compose the outbound webhook body as a typed object instead of a raw JSON blob."
      />
      <JsonPreviewCard label="Webhook payload preview" value={payloadPreview} error={payloadError} />
    </div>
  );
}

function AssignOwnerForm({
  step,
  availableTokens,
  runtimeEntityOptions,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "assign_owner" }>;
  availableTokens: DynamicTokenOption[];
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  onUpdate: UpdateFn;
}) {
  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`assign_entity_${step.id}`}>Entity logical name</Label>
          <Input
            id={`assign_entity_${step.id}`}
            value={step.entityLogicalName}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "assign_owner" ? { ...s, entityLogicalName: e.target.value } : s,
              )
            }
            list={`assign_entity_suggestions_${step.id}`}
          />
          <datalist id={`assign_entity_suggestions_${step.id}`}>
            {runtimeEntityOptions.map((entity) => (
              <option key={entity.value} value={entity.value} />
            ))}
          </datalist>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`assign_record_id_${step.id}`}>Record id</Label>
          <Input
            id={`assign_record_id_${step.id}`}
            value={step.recordId}
            onChange={(e) =>
              onUpdate((s) => (s.type === "assign_owner" ? { ...s, recordId: e.target.value } : s))
            }
            placeholder="{{trigger.payload.record_id}}"
          />
          <TokenChips value={step.recordId} />
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`assign_owner_${step.id}`}>Owner id</Label>
          <Input
            id={`assign_owner_${step.id}`}
            value={step.ownerId}
            onChange={(e) =>
              onUpdate((s) => (s.type === "assign_owner" ? { ...s, ownerId: e.target.value } : s))
            }
            placeholder="triage_queue"
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`assign_reason_${step.id}`}>Reason</Label>
          <Input
            id={`assign_reason_${step.id}`}
            value={step.reason}
            onChange={(e) =>
              onUpdate((s) => (s.type === "assign_owner" ? { ...s, reason: e.target.value } : s))
            }
            placeholder="workflow routing"
          />
        </div>
      </div>
      <ExpressionBuilderPopover
        title="Assign Owner Expression"
        currentValue={step.recordId}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "assign_owner"
              ? { ...s, recordId: appendExpression(s.recordId, expr) }
              : s,
          )
        }
      />
    </div>
  );
}

function ApprovalRequestForm({
  step,
  runtimeEntityOptions,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "approval_request" }>;
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  onUpdate: UpdateFn;
}) {
  let payloadPreview: Record<string, unknown> = {};
  let payloadError: string | null = null;
  try {
    payloadPreview = parseDraftObjectFields(step.payloadFields, "Approval request payload");
  } catch (error) {
    payloadError =
      error instanceof Error ? error.message : "Approval request payload contains invalid fields.";
  }

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`approval_entity_${step.id}`}>Entity logical name</Label>
          <Input
            id={`approval_entity_${step.id}`}
            value={step.entityLogicalName}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "approval_request"
                  ? { ...s, entityLogicalName: e.target.value }
                  : s,
              )
            }
            list={`approval_entity_suggestions_${step.id}`}
          />
          <datalist id={`approval_entity_suggestions_${step.id}`}>
            {runtimeEntityOptions.map((entity) => (
              <option key={entity.value} value={entity.value} />
            ))}
          </datalist>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`approval_record_id_${step.id}`}>Record id</Label>
          <Input
            id={`approval_record_id_${step.id}`}
            value={step.recordId}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "approval_request" ? { ...s, recordId: e.target.value } : s,
              )
            }
            placeholder="{{trigger.payload.record_id}}"
          />
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`approval_request_type_${step.id}`}>Request type</Label>
          <Input
            id={`approval_request_type_${step.id}`}
            value={step.requestType}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "approval_request" ? { ...s, requestType: e.target.value } : s,
              )
            }
            placeholder="record_change"
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`approval_approver_${step.id}`}>Approver id</Label>
          <Input
            id={`approval_approver_${step.id}`}
            value={step.approverId}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "approval_request" ? { ...s, approverId: e.target.value } : s,
              )
            }
            placeholder="manager-7"
          />
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`approval_requested_by_${step.id}`}>Requested by</Label>
          <Input
            id={`approval_requested_by_${step.id}`}
            value={step.requestedBy}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "approval_request" ? { ...s, requestedBy: e.target.value } : s,
              )
            }
            placeholder="{{trigger.payload.triggered_by}}"
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`approval_reason_${step.id}`}>Reason</Label>
          <Input
            id={`approval_reason_${step.id}`}
            value={step.reason}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "approval_request" ? { ...s, reason: e.target.value } : s,
              )
            }
            placeholder="Please review"
          />
        </div>
      </div>
      <DraftObjectFieldEditor
        label="Approval Payload Fields"
        idPrefix={`approval_payload_${step.id}`}
        fields={step.payloadFields}
        onChange={(nextFields) =>
          onUpdate((s) =>
            s.type === "approval_request" ? { ...s, payloadFields: nextFields } : s,
          )
        }
        placeholderKey="status"
        helperText="Attach typed context fields that approvers or callbacks can use later."
      />
      <JsonPreviewCard
        label="Approval payload preview"
        value={payloadPreview}
        error={payloadError}
      />
    </div>
  );
}

function DelayForm({
  step,
  availableTokens,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "delay" }>;
  availableTokens: DynamicTokenOption[];
  onUpdate: UpdateFn;
}) {
  const parsed = Number.parseInt(step.durationMs, 10);
  const durationError =
    !Number.isFinite(parsed) || parsed <= 0
      ? "Duration must be a positive integer number of milliseconds."
      : null;

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`delay_duration_${step.id}`}>Duration (ms)</Label>
          <Input
            id={`delay_duration_${step.id}`}
            value={step.durationMs}
            onChange={(e) =>
              onUpdate((s) => (s.type === "delay" ? { ...s, durationMs: e.target.value } : s))
            }
            placeholder="5000"
          />
          {durationError && <p className="text-[11px] text-red-600">{durationError}</p>}
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`delay_reason_${step.id}`}>Reason</Label>
          <Input
            id={`delay_reason_${step.id}`}
            value={step.reason}
            onChange={(e) =>
              onUpdate((s) => (s.type === "delay" ? { ...s, reason: e.target.value } : s))
            }
            placeholder="wait for downstream consistency"
          />
        </div>
      </div>
      <ExpressionBuilderPopover
        title="Delay Expression"
        currentValue={step.reason}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "delay" ? { ...s, reason: appendExpression(s.reason, expr) } : s,
          )
        }
      />
    </div>
  );
}

function ConditionForm({
  step,
  availableTokens,
  fieldPathSuggestions,
  onUpdate,
}: {
  step: Extract<DraftWorkflowStep, { type: "condition" }>;
  availableTokens: DynamicTokenOption[];
  fieldPathSuggestions: string[];
  onUpdate: UpdateFn;
}) {
  const [showLabels, setShowLabels] = useState(false);

  let valueError: string | null = null;
  if (step.operator !== "exists") {
    if (step.valueKind === "number" && step.valueText.trim().length > 0) {
      valueError = Number.isFinite(Number(step.valueText))
        ? null
        : "Number conditions require a valid numeric value.";
    } else if (step.valueKind === "json") {
      try {
        JSON.parse(step.valueText);
      } catch {
        valueError = "JSON conditions require valid JSON.";
      }
    }
  }

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-3 items-end gap-2">
        <div className="space-y-1.5">
          <Label htmlFor={`field_${step.id}`}>Field path</Label>
          <Input
            id={`field_${step.id}`}
            value={step.fieldPath}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "condition" ? { ...s, fieldPath: e.target.value } : s,
              )
            }
            placeholder="payload.status"
            list={`field_path_suggestions_${step.id}`}
          />
          <datalist id={`field_path_suggestions_${step.id}`}>
            {fieldPathSuggestions.map((fieldPath) => (
              <option key={fieldPath} value={fieldPath} />
            ))}
          </datalist>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`op_${step.id}`}>Operator</Label>
          <Select
            id={`op_${step.id}`}
            value={step.operator}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "condition"
                  ? { ...s, operator: e.target.value as WorkflowConditionOperatorDto }
                  : s,
              )
            }
          >
            {CONDITION_OPERATORS.map((op) => (
              <option key={op} value={op}>
                {op}
              </option>
            ))}
          </Select>
        </div>
        <div className="space-y-1.5">
          <Label htmlFor={`value_kind_${step.id}`}>Value type</Label>
          <Select
            id={`value_kind_${step.id}`}
            value={step.valueKind}
            disabled={step.operator === "exists"}
            onChange={(e) =>
              onUpdate((s) =>
                s.type === "condition"
                  ? {
                      ...s,
                      valueKind: e.target.value as DraftValueKind,
                      valueText:
                        e.target.value === "boolean"
                          ? "true"
                          : e.target.value === "null"
                            ? ""
                            : s.valueText,
                    }
                  : s,
              )
            }
          >
            <option value="string">Text</option>
            <option value="number">Number</option>
            <option value="boolean">Boolean</option>
            <option value="null">Null</option>
            <option value="json">JSON</option>
          </Select>
        </div>
      </div>
      {step.operator !== "exists" ? (
        <div className="space-y-1.5">
          <Label htmlFor={`val_${step.id}`}>Condition value</Label>
          {step.valueKind === "boolean" ? (
            <Select
              id={`val_${step.id}`}
              value={step.valueText === "false" ? "false" : "true"}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "condition" ? { ...s, valueText: e.target.value } : s,
                )
              }
            >
              <option value="true">True</option>
              <option value="false">False</option>
            </Select>
          ) : step.valueKind === "null" ? (
            <p className="rounded border border-dashed border-zinc-200 px-3 py-2 text-[11px] text-zinc-500">
              This condition compares against `null`.
            </p>
          ) : step.valueKind === "json" ? (
            <Textarea
              id={`val_${step.id}`}
              className="font-mono text-xs"
              rows={4}
              value={step.valueText}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "condition" ? { ...s, valueText: e.target.value } : s,
                )
              }
              placeholder='{"status":"active"}'
            />
          ) : (
            <Input
              id={`val_${step.id}`}
              value={step.valueText}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "condition" ? { ...s, valueText: e.target.value } : s,
                )
              }
              placeholder={step.valueKind === "number" ? "42" : "active"}
            />
          )}
        </div>
      ) : null}
      {step.fieldPath.trim().length === 0 && (
        <p className="text-[11px] text-red-600">Field path is required.</p>
      )}
      {valueError && <p className="text-[11px] text-red-600">{valueError}</p>}
      <FieldSuggestionPicker
        title="Field Path Picker"
        helper="Pick a trigger field path to set this condition quickly."
        fields={fieldPathSuggestions}
        onPick={(fieldPath) =>
          onUpdate((currentStep) =>
            currentStep.type === "condition"
              ? { ...currentStep, fieldPath }
              : currentStep,
          )
        }
      />
      <ExpressionBuilderPopover
        title="Condition Value Expression"
        currentValue={step.valueText}
        tokens={availableTokens}
        onInsertExpression={(expr) =>
          onUpdate((s) =>
            s.type === "condition"
              ? { ...s, valueText: appendExpression(s.valueText, expr) }
              : s,
          )
        }
      />
      <button
        type="button"
        className="text-[11px] text-zinc-500 underline-offset-2 hover:text-zinc-700 hover:underline"
        onClick={() => setShowLabels((v) => !v)}
      >
        {showLabels ? "Hide branch labels" : "Customize branch labels"}
      </button>
      {showLabels && (
        <div className="grid grid-cols-2 gap-2">
          <div className="space-y-1.5">
            <Label htmlFor={`then_${step.id}`}>Yes label</Label>
            <Input
              id={`then_${step.id}`}
              value={step.thenLabel}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "condition" ? { ...s, thenLabel: e.target.value } : s,
                )
              }
              placeholder="Yes"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`else_${step.id}`}>No label</Label>
            <Input
              id={`else_${step.id}`}
              value={step.elseLabel}
              onChange={(e) =>
                onUpdate((s) =>
                  s.type === "condition" ? { ...s, elseLabel: e.target.value } : s,
                )
              }
              placeholder="No"
            />
          </div>
        </div>
      )}
    </div>
  );
}

function StepTraceDebug({
  trace,
  isRetryingStep,
  onRetryStep,
}: {
  trace: WorkflowRunStepTraceResponse;
  isRetryingStep: boolean;
  onRetryStep: (
    stepPath: string,
    strategy: RetryWorkflowStepStrategyDto,
    backoffMs?: number,
  ) => void;
}) {
  const [preset, setPreset] = useState<RetryPreset>("immediate");

  return (
    <div className="space-y-3 rounded-lg border border-zinc-200 bg-zinc-50 p-3">
      <div className="flex items-center justify-between">
        <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-zinc-500">
          Run debug
        </p>
        <span className="font-mono text-[10px] text-zinc-400">{trace.step_path}</span>
      </div>

      <div className="grid grid-cols-2 gap-2">
        <div className="space-y-1">
          <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-400">Input</p>
          <Textarea
            className="font-mono text-[10px]"
            rows={4}
            value={JSON.stringify(trace.input_payload, null, 2)}
            readOnly
          />
        </div>
        <div className="space-y-1">
          <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-400">Output</p>
          <Textarea
            className="font-mono text-[10px]"
            rows={4}
            value={JSON.stringify(trace.output_payload, null, 2)}
            readOnly
          />
        </div>
      </div>

      {trace.error_message && (
        <div className="space-y-2">
          <p className="rounded border border-red-200 bg-red-50 px-2 py-1.5 text-[11px] text-red-700">
            {trace.error_message}
          </p>
          <div className="flex items-center gap-2">
            <Select
              id={`retry_${trace.step_path}`}
              value={preset}
              onChange={(e) => setPreset(e.target.value as RetryPreset)}
            >
              <option value="immediate">Immediate retry</option>
              <option value="backoff_800">Backoff 0.8s</option>
              <option value="backoff_2000">Backoff 2s</option>
              <option value="backoff_5000">Backoff 5s</option>
            </Select>
            <Button
              type="button"
              size="sm"
              disabled={isRetryingStep}
              onClick={() => {
                const { strategy, backoffMs } = retryConfigFromPreset(preset);
                onRetryStep(trace.step_path, strategy, backoffMs);
              }}
            >
              {isRetryingStep ? "Retrying..." : "Retry step"}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

type StepCardProps = {
  step: DraftWorkflowStep;
  isExpanded: boolean;
  readOnly: boolean;
  trace: WorkflowRunStepTraceResponse | null;
  availableTokens: DynamicTokenOption[];
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  triggerFieldPathSuggestions: string[];
  getEntityFieldPathSuggestions: (entityLogicalName: string) => string[];
  onToggle: () => void;
  onUpdate: UpdateFn;
  onRemove: () => void;
  onDuplicate: () => void;
  isRetryingStep: boolean;
  onRetryStep: (
    stepPath: string,
    strategy: RetryWorkflowStepStrategyDto,
    backoffMs?: number,
  ) => void;
};

function StepCard({
  step,
  isExpanded,
  readOnly,
  trace,
  availableTokens,
  runtimeEntityOptions,
  triggerFieldPathSuggestions,
  getEntityFieldPathSuggestions,
  onToggle,
  onUpdate,
  onRemove,
  onDuplicate,
  isRetryingStep,
  onRetryStep,
}: StepCardProps) {
  const [focusedMappedFieldKey, setFocusedMappedFieldKey] = useState<string | null>(null);

  const autoMappedFields =
    step.type === "create_runtime_record"
      ? triggerPayloadMappedFieldsFromDraftObjectFields(step.dataFields)
      : step.type === "update_runtime_record"
        ? triggerPayloadMappedFieldsFromDraftObjectFields(step.dataFields)
        : step.type === "approval_request"
          ? triggerPayloadMappedFieldsFromDraftObjectFields(step.payloadFields)
          : step.type === "http_request"
            ? step.bodyMode === "object"
              ? triggerPayloadMappedFieldsFromDraftObjectFields(step.bodyFields)
              : step.bodyMode === "array"
                ? triggerPayloadMappedFieldsFromDraftArrayItems(step.bodyArrayItems)
                : step.bodyMode === "scalar"
                  ? triggerPayloadMappedFieldsFromText(step.bodyScalarValue, "body")
                  : triggerPayloadMappedFieldsFromJson(step.bodyJson)
        : step.type === "webhook"
          ? triggerPayloadMappedFieldsFromDraftObjectFields(step.payloadFields)
          : [];

  return (
    <div
      className={`w-full overflow-hidden rounded-xl border bg-white shadow-sm transition-shadow ${stepBorderColor(step.type)} ${
        isExpanded ? "shadow-md" : "hover:shadow"
      }`}
    >
      <div className="flex items-center gap-3 px-4 py-3">
        <button
          type="button"
          className="flex min-w-0 flex-1 items-start gap-3 text-left"
          onClick={onToggle}
        >
          <div
            className={`mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-lg ${stepIconBg(step.type)}`}
          >
            {stepIcon(step.type)}
          </div>
          <div className="min-w-0 flex-1">
            <p
              className={`text-[10px] font-semibold uppercase tracking-[0.12em] ${stepTypeLabelColor(step.type)}`}
            >
              {stepTypeLabel(step.type)}
            </p>
            <p className="truncate text-sm text-zinc-800">{summarizeStep(step)}</p>
            {autoMappedFields.length > 0 ? (
              <div className="mt-1 flex flex-wrap items-center gap-1">
                <span className="text-[10px] text-emerald-700">Mapped:</span>
                {autoMappedFields.slice(0, 5).map((mapped) => (
                  <button
                    key={`${mapped.key}:${mapped.sourcePath}`}
                    type="button"
                    className="rounded border border-emerald-300 bg-emerald-50 px-1.5 py-0.5 font-mono text-[10px] text-emerald-800 hover:bg-emerald-100"
                    title={`Map ${mapped.key} from trigger.payload.${mapped.sourcePath}`}
                    onClick={(event) => {
                      event.stopPropagation();
                      if (!isExpanded) {
                        onToggle();
                      }
                      setFocusedMappedFieldKey(mapped.key);
                    }}
                  >
                    {mapped.key}
                  </button>
                ))}
                {autoMappedFields.length > 5 ? (
                  <span className="text-[10px] text-zinc-500">...</span>
                ) : null}
              </div>
            ) : null}
            <StepTraceStatus trace={trace} showNotExecuted={readOnly} />
          </div>
        </button>

        <div className="flex shrink-0 items-center gap-0.5">
          {!readOnly ? (
            <>
              <button
                type="button"
                className="flex size-7 items-center justify-center rounded-md text-zinc-400 transition-colors hover:bg-zinc-100 hover:text-zinc-600"
                onClick={onDuplicate}
                title="Duplicate step"
              >
                <Copy className="size-3.5" />
              </button>
              <button
                type="button"
                className="flex size-7 items-center justify-center rounded-md text-zinc-400 transition-colors hover:bg-red-50 hover:text-red-600"
                onClick={onRemove}
                title="Remove step"
              >
                <Trash2 className="size-3.5" />
              </button>
            </>
          ) : null}
          <button
            type="button"
            className={`flex size-7 items-center justify-center rounded-md transition-colors ${
              isExpanded
                ? "bg-zinc-100 text-zinc-600"
                : "text-zinc-400 hover:bg-zinc-100 hover:text-zinc-600"
            }`}
            onClick={onToggle}
            title={isExpanded ? "Collapse" : "Configure"}
          >
            {isExpanded ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
          </button>
        </div>
      </div>

      {isExpanded && (
        <div className="space-y-4 border-t border-zinc-100 bg-zinc-50/40 p-4">
          {step.type === "log_message" && (
            <LogMessageForm step={step} availableTokens={availableTokens} onUpdate={onUpdate} />
          )}
          {step.type === "create_runtime_record" && (
            <CreateRecordForm
              step={step}
              runtimeEntityOptions={runtimeEntityOptions}
              fieldPathSuggestions={getEntityFieldPathSuggestions(step.entityLogicalName)}
              focusedFieldKey={focusedMappedFieldKey}
              onFocusApplied={() => setFocusedMappedFieldKey(null)}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "update_runtime_record" && (
            <UpdateRecordForm
              step={step}
              runtimeEntityOptions={runtimeEntityOptions}
              fieldPathSuggestions={getEntityFieldPathSuggestions(step.entityLogicalName)}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "delete_runtime_record" && (
            <DeleteRecordForm
              step={step}
              availableTokens={availableTokens}
              runtimeEntityOptions={runtimeEntityOptions}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "send_email" && (
            <SendEmailForm
              step={step}
              availableTokens={availableTokens}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "http_request" && (
            <HttpRequestForm
              step={step}
              availableTokens={availableTokens}
              fieldPathSuggestions={triggerFieldPathSuggestions}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "webhook" && (
            <WebhookForm
              step={step}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "assign_owner" && (
            <AssignOwnerForm
              step={step}
              availableTokens={availableTokens}
              runtimeEntityOptions={runtimeEntityOptions}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "approval_request" && (
            <ApprovalRequestForm
              step={step}
              runtimeEntityOptions={runtimeEntityOptions}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "delay" && (
            <DelayForm
              step={step}
              availableTokens={availableTokens}
              onUpdate={onUpdate}
            />
          )}
          {step.type === "condition" && (
            <ConditionForm
              step={step}
              availableTokens={availableTokens}
              fieldPathSuggestions={triggerFieldPathSuggestions}
              onUpdate={onUpdate}
            />
          )}
          {trace && (
            <StepTraceDebug
              trace={trace}
              isRetryingStep={isRetryingStep}
              onRetryStep={onRetryStep}
            />
          )}
        </div>
      )}
    </div>
  );
}

export type SharedStepProps = {
  readOnly: boolean;
  expandedNodeId: string | null;
  onExpandNode: (id: string | null) => void;
  onUpdateStep: (
    stepId: string,
    updater: (s: DraftWorkflowStep) => DraftWorkflowStep,
  ) => void;
  onRemoveStep: (stepId: string) => void;
  onDuplicateStep: (stepId: string) => void;
  onOpenNodePicker: (mode: CatalogInsertMode, stepId?: string) => void;
  getAvailableTokens: (stepId: string) => DynamicTokenOption[];
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  triggerFieldPathSuggestions: string[];
  getEntityFieldPathSuggestions: (entityLogicalName: string) => string[];
  stepTraceByPath: Record<string, WorkflowRunStepTraceResponse>;
  stepPathByStepId: Record<string, string>;
  isRetryingStep: boolean;
  onRetryStep: (
    stepPath: string,
    strategy: RetryWorkflowStepStrategyDto,
    backoffMs?: number,
  ) => void;
};

function BranchColumn({
  label,
  isYes,
  steps,
  conditionId,
  ...shared
}: SharedStepProps & {
  label: string;
  isYes: boolean;
  steps: DraftWorkflowStep[];
  conditionId: string;
}) {
  const addMode: CatalogInsertMode = isYes ? "then_selected" : "else_selected";

  return (
    <div className="flex min-w-0 flex-1 flex-col overflow-hidden rounded-xl border border-zinc-200 bg-zinc-50">
      <div
        className={`flex items-center gap-2 border-b border-zinc-200 px-3 py-2 ${
          isYes ? "bg-emerald-50" : "bg-red-50"
        }`}
      >
        <div className={`size-2 rounded-full ${isYes ? "bg-emerald-500" : "bg-red-400"}`} />
        <p
          className={`text-[11px] font-semibold uppercase tracking-[0.12em] ${
            isYes ? "text-emerald-700" : "text-red-700"
          }`}
        >
          {label || (isYes ? "Yes" : "No")}
        </p>
      </div>

      <div className="flex flex-col items-center gap-0 p-3">
        {steps.length === 0 ? (
          <p className="py-2 text-[11px] text-zinc-400">No steps yet</p>
        ) : (
          steps.map((step) => (
            <StepBlock key={step.id} step={step} {...shared} />
          ))
        )}
        {!shared.readOnly ? (
          <button
            type="button"
            className="mt-2 flex w-full items-center justify-center gap-1.5 rounded-lg border border-dashed border-zinc-300 px-3 py-2 text-[11px] text-zinc-400 transition hover:border-emerald-300 hover:bg-emerald-50 hover:text-emerald-600"
            onClick={() => shared.onOpenNodePicker(addMode, conditionId)}
          >
            <Plus className="size-3.5" />
            Add step
          </button>
        ) : null}
      </div>
    </div>
  );
}

function ConditionBlock({
  step,
  ...shared
}: SharedStepProps & { step: DraftConditionStep }) {
  const isExpanded = shared.expandedNodeId === step.id;
  const stepPath = shared.stepPathByStepId[step.id];
  const trace = stepPath ? (shared.stepTraceByPath[stepPath] ?? null) : null;
  const tokens = shared.getAvailableTokens(step.id);

  return (
    <div className="w-full">
      <StepCard
        step={step}
        isExpanded={isExpanded}
        readOnly={shared.readOnly}
        trace={trace}
        availableTokens={tokens}
        runtimeEntityOptions={shared.runtimeEntityOptions}
        triggerFieldPathSuggestions={shared.triggerFieldPathSuggestions}
        getEntityFieldPathSuggestions={shared.getEntityFieldPathSuggestions}
        onToggle={() => shared.onExpandNode(isExpanded ? null : step.id)}
        onUpdate={(updater) => shared.onUpdateStep(step.id, updater)}
        onRemove={() => shared.onRemoveStep(step.id)}
        onDuplicate={() => shared.onDuplicateStep(step.id)}
        isRetryingStep={shared.isRetryingStep}
        onRetryStep={shared.onRetryStep}
      />

      <div className="flex justify-center">
        <div className="h-4 w-px bg-zinc-200" />
      </div>

      <div className="flex gap-3">
        <BranchColumn
          label={step.thenLabel || "Yes"}
          isYes={true}
          steps={step.thenSteps}
          conditionId={step.id}
          {...shared}
        />
        <BranchColumn
          label={step.elseLabel || "No"}
          isYes={false}
          steps={step.elseSteps}
          conditionId={step.id}
          {...shared}
        />
      </div>

      <div className="flex justify-center">
        <div className="h-4 w-px bg-zinc-200" />
      </div>
    </div>
  );
}

export function StepBlock({ step, ...shared }: SharedStepProps & { step: DraftWorkflowStep }) {
  if (step.type === "condition") {
    return (
      <>
        <ConditionBlock step={step} {...shared} />
        <FlowConnector
          disabled={shared.readOnly}
          onAdd={() => shared.onOpenNodePicker("after_selected", step.id)}
        />
      </>
    );
  }

  const isExpanded = shared.expandedNodeId === step.id;
  const stepPath = shared.stepPathByStepId[step.id];
  const trace = stepPath ? (shared.stepTraceByPath[stepPath] ?? null) : null;
  const tokens = shared.getAvailableTokens(step.id);

  return (
    <>
      <StepCard
        step={step}
        isExpanded={isExpanded}
        readOnly={shared.readOnly}
        trace={trace}
        availableTokens={tokens}
        runtimeEntityOptions={shared.runtimeEntityOptions}
        triggerFieldPathSuggestions={shared.triggerFieldPathSuggestions}
        getEntityFieldPathSuggestions={shared.getEntityFieldPathSuggestions}
        onToggle={() => shared.onExpandNode(isExpanded ? null : step.id)}
        onUpdate={(updater) => shared.onUpdateStep(step.id, updater)}
        onRemove={() => shared.onRemoveStep(step.id)}
        onDuplicate={() => shared.onDuplicateStep(step.id)}
        isRetryingStep={shared.isRetryingStep}
        onRetryStep={shared.onRetryStep}
      />
      <FlowConnector
        disabled={shared.readOnly}
        onAdd={() => shared.onOpenNodePicker("after_selected", step.id)}
      />
    </>
  );
}
