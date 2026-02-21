"use client";

import { type FormEvent, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Select,
  StatusBadge,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Textarea,
} from "@qryvanta/ui";

import {
  apiFetch,
  type CreateFieldRequest,
  type CreateRuntimeRecordRequest,
  type FieldResponse,
  type PublishedSchemaResponse,
  type QueryRuntimeRecordsRequest,
  type RuntimeRecordQueryFilterRequest,
  type RuntimeRecordQuerySortRequest,
  type RuntimeRecordResponse,
} from "@/lib/api";

const FIELD_TYPE_OPTIONS = [
  "text",
  "number",
  "boolean",
  "date",
  "datetime",
  "json",
  "relation",
] as const;

type EntityWorkbenchPanelProps = {
  entityLogicalName: string;
  initialFields: FieldResponse[];
  initialPublishedSchema: PublishedSchemaResponse | null;
  initialRecords: RuntimeRecordResponse[];
};

type QueryPreset = {
  name: string;
  limitText: string;
  offsetText: string;
  logicalMode: "and" | "or";
  conditionsText: string;
  sortText: string;
  filtersText: string;
};

type WorkbenchSection = "schema" | "runtime";
type RuntimeSection = "create" | "query";

function normalizeQueryPresets(rawValue: unknown): QueryPreset[] {
  if (!Array.isArray(rawValue)) {
    return [];
  }

  return rawValue
    .filter(
      (preset): preset is QueryPreset =>
        typeof preset === "object" &&
        preset !== null &&
        "name" in preset &&
        "limitText" in preset &&
        "offsetText" in preset &&
        "filtersText" in preset,
    )
    .map((preset) => {
      const logicalMode: QueryPreset["logicalMode"] =
        "logicalMode" in preset && preset.logicalMode === "or" ? "or" : "and";

      return {
        name: String(preset.name),
        limitText: String(preset.limitText),
        offsetText: String(preset.offsetText),
        logicalMode,
        conditionsText:
          "conditionsText" in preset && typeof preset.conditionsText === "string"
            ? preset.conditionsText
            : "[]",
        sortText:
          "sortText" in preset && typeof preset.sortText === "string"
            ? preset.sortText
            : "[]",
        filtersText: String(preset.filtersText),
      };
    })
    .filter((preset) => preset.name.trim().length > 0)
    .sort((left, right) => left.name.localeCompare(right.name));
}

export function EntityWorkbenchPanel({
  entityLogicalName,
  initialFields,
  initialPublishedSchema,
  initialRecords,
}: EntityWorkbenchPanelProps) {
  const router = useRouter();

  const [logicalName, setLogicalName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [fieldType, setFieldType] =
    useState<(typeof FIELD_TYPE_OPTIONS)[number]>("text");
  const [isRequired, setIsRequired] = useState(false);
  const [isUnique, setIsUnique] = useState(false);
  const [defaultValueText, setDefaultValueText] = useState("");
  const [relationTargetEntity, setRelationTargetEntity] = useState("");

  const [recordPayload, setRecordPayload] = useState("{}");
  const [queryLogicalMode, setQueryLogicalMode] = useState<"and" | "or">("and");
  const [queryConditionsText, setQueryConditionsText] = useState("[]");
  const [querySortText, setQuerySortText] = useState("[]");
  const [queryFiltersText, setQueryFiltersText] = useState("{}");
  const [queryLimitText, setQueryLimitText] = useState("50");
  const [queryOffsetText, setQueryOffsetText] = useState("0");
  const [queriedRecords, setQueriedRecords] = useState<
    RuntimeRecordResponse[] | null
  >(null);
  const [queryPresetName, setQueryPresetName] = useState("");
  const [selectedPresetName, setSelectedPresetName] = useState("");
  const [queryPresets, setQueryPresets] = useState<QueryPreset[]>([]);
  const [presetTransferText, setPresetTransferText] = useState("");
  const [isPresetCopied, setIsPresetCopied] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const [isSavingField, setIsSavingField] = useState(false);
  const [isPublishing, setIsPublishing] = useState(false);
  const [isCreatingRecord, setIsCreatingRecord] = useState(false);
  const [isSavingPreset, setIsSavingPreset] = useState(false);
  const [isQueryingRecords, setIsQueryingRecords] = useState(false);
  const [deletingRecordId, setDeletingRecordId] = useState<string | null>(null);
  const [activeSection, setActiveSection] =
    useState<WorkbenchSection>("schema");
  const [activeRuntimeSection, setActiveRuntimeSection] =
    useState<RuntimeSection>("create");
  const presetCopiedTimeoutRef = useRef<number | null>(null);

  const queryPresetsStorageKey = `entity-workbench-query-presets:${entityLogicalName}`;

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const rawPresets = window.localStorage.getItem(queryPresetsStorageKey);
    if (!rawPresets) {
      setQueryPresets([]);
      setSelectedPresetName("");
      return;
    }

    try {
      const parsed = JSON.parse(rawPresets) as unknown;
      setQueryPresets(normalizeQueryPresets(parsed));
      setSelectedPresetName("");
    } catch {
      setQueryPresets([]);
      setSelectedPresetName("");
    }
  }, [queryPresetsStorageKey]);

  useEffect(() => {
    return () => {
      if (presetCopiedTimeoutRef.current !== null) {
        window.clearTimeout(presetCopiedTimeoutRef.current);
      }
    };
  }, []);

  function savePresetsToStorage(nextPresets: QueryPreset[]) {
    setQueryPresets(nextPresets);
    if (typeof window === "undefined") {
      return;
    }

    window.localStorage.setItem(
      queryPresetsStorageKey,
      JSON.stringify(nextPresets),
    );
  }

  function clearMessages() {
    setErrorMessage(null);
    setStatusMessage(null);
  }

  function showPresetCopiedIndicator() {
    setIsPresetCopied(true);
    if (presetCopiedTimeoutRef.current !== null) {
      window.clearTimeout(presetCopiedTimeoutRef.current);
    }
    presetCopiedTimeoutRef.current = window.setTimeout(() => {
      setIsPresetCopied(false);
      presetCopiedTimeoutRef.current = null;
    }, 1600);
  }

  function readPresetFiltersAsObject(): Record<string, unknown> | null {
    let parsedFilters: unknown;

    try {
      parsedFilters = JSON.parse(queryFiltersText);
    } catch {
      setErrorMessage("Runtime query filters must be valid JSON.");
      return null;
    }

    if (
      parsedFilters === null ||
      Array.isArray(parsedFilters) ||
      typeof parsedFilters !== "object"
    ) {
      setErrorMessage("Runtime query filters must be a JSON object.");
      return null;
    }

    return parsedFilters as Record<string, unknown>;
  }

  function readPresetConditions(): RuntimeRecordQueryFilterRequest[] | null {
    let parsedConditions: unknown;

    try {
      parsedConditions = JSON.parse(queryConditionsText);
    } catch {
      setErrorMessage("Runtime query conditions must be valid JSON.");
      return null;
    }

    if (!Array.isArray(parsedConditions)) {
      setErrorMessage("Runtime query conditions must be a JSON array.");
      return null;
    }

    const conditions: RuntimeRecordQueryFilterRequest[] = [];
    for (const condition of parsedConditions) {
      if (
        typeof condition !== "object" ||
        condition === null ||
        !("field_logical_name" in condition) ||
        !("operator" in condition) ||
        !("field_value" in condition) ||
        typeof condition.field_logical_name !== "string" ||
        condition.field_logical_name.trim().length === 0 ||
        typeof condition.operator !== "string" ||
        condition.operator.trim().length === 0
      ) {
        setErrorMessage(
          "Each query condition must include field_logical_name, operator, and field_value.",
        );
        return null;
      }

      conditions.push({
        scope_alias:
          "scope_alias" in condition && typeof condition.scope_alias === "string"
            ? condition.scope_alias
            : null,
        field_logical_name: condition.field_logical_name,
        operator: condition.operator,
        field_value: condition.field_value,
      });
    }

    return conditions;
  }

  function readPresetSort(): RuntimeRecordQuerySortRequest[] | null {
    let parsedSort: unknown;

    try {
      parsedSort = JSON.parse(querySortText);
    } catch {
      setErrorMessage("Runtime query sort must be valid JSON.");
      return null;
    }

    if (!Array.isArray(parsedSort)) {
      setErrorMessage("Runtime query sort must be a JSON array.");
      return null;
    }

    const sort: RuntimeRecordQuerySortRequest[] = [];
    for (const entry of parsedSort) {
      if (
        typeof entry !== "object" ||
        entry === null ||
        !("field_logical_name" in entry) ||
        typeof entry.field_logical_name !== "string" ||
        entry.field_logical_name.trim().length === 0
      ) {
        setErrorMessage(
          "Each sort entry must include a non-empty field_logical_name.",
        );
        return null;
      }

      let direction: "asc" | "desc" | null = null;
      if ("direction" in entry) {
        if (
          entry.direction !== null &&
          entry.direction !== "asc" &&
          entry.direction !== "desc"
        ) {
          setErrorMessage("Sort direction must be 'asc', 'desc', or null.");
          return null;
        }
        direction = entry.direction as "asc" | "desc" | null;
      }

      sort.push({
        scope_alias:
          "scope_alias" in entry && typeof entry.scope_alias === "string"
            ? entry.scope_alias
            : null,
        field_logical_name: entry.field_logical_name,
        direction,
      });
    }

    return sort;
  }

  function loadPreset(name: string) {
    const preset = queryPresets.find((candidate) => candidate.name === name);
    if (!preset) {
      setErrorMessage("Selected preset no longer exists.");
      return;
    }

    setQueryLimitText(preset.limitText);
    setQueryOffsetText(preset.offsetText);
    setQueryLogicalMode(preset.logicalMode);
    setQueryConditionsText(preset.conditionsText);
    setQuerySortText(preset.sortText);
    setQueryFiltersText(preset.filtersText);
    setSelectedPresetName(name);
    setStatusMessage(`Loaded query preset '${name}'.`);
  }

  function handleSaveQueryPreset() {
    clearMessages();
    setIsSavingPreset(true);

    try {
      const trimmedName = queryPresetName.trim();
      if (trimmedName.length === 0) {
        setErrorMessage("Preset name is required.");
        return;
      }

      if (readPresetFiltersAsObject() === null) {
        return;
      }

      if (readPresetConditions() === null) {
        return;
      }

      if (readPresetSort() === null) {
        return;
      }

      const nextPreset: QueryPreset = {
        name: trimmedName,
        limitText: queryLimitText,
        offsetText: queryOffsetText,
        logicalMode: queryLogicalMode,
        conditionsText: queryConditionsText,
        sortText: querySortText,
        filtersText: queryFiltersText,
      };

      const nextPresets = [
        ...queryPresets.filter((preset) => preset.name !== trimmedName),
        nextPreset,
      ].sort((left, right) => left.name.localeCompare(right.name));

      savePresetsToStorage(nextPresets);
      setSelectedPresetName(trimmedName);
      setStatusMessage(`Saved query preset '${trimmedName}'.`);
    } finally {
      setIsSavingPreset(false);
    }
  }

  function handleDeleteSelectedPreset() {
    clearMessages();
    if (selectedPresetName.length === 0) {
      setErrorMessage("Choose a preset to delete.");
      return;
    }

    const nextPresets = queryPresets.filter(
      (preset) => preset.name !== selectedPresetName,
    );
    savePresetsToStorage(nextPresets);
    setStatusMessage(`Deleted query preset '${selectedPresetName}'.`);
    setSelectedPresetName("");
  }

  async function handleExportQueryPresets() {
    clearMessages();
    setIsPresetCopied(false);
    const serialized = JSON.stringify(queryPresets, null, 2);
    setPresetTransferText(serialized);

    if (typeof navigator === "undefined" || !navigator.clipboard) {
      setStatusMessage("Exported presets to JSON field.");
      return;
    }

    try {
      await navigator.clipboard.writeText(serialized);
      showPresetCopiedIndicator();
      setStatusMessage("Exported presets and copied JSON to clipboard.");
    } catch {
      setStatusMessage("Exported presets to JSON field.");
    }
  }

  function handleImportQueryPresets() {
    clearMessages();
    if (presetTransferText.trim().length === 0) {
      setErrorMessage("Paste presets JSON before importing.");
      return;
    }

    let parsed: unknown;
    try {
      parsed = JSON.parse(presetTransferText);
    } catch {
      setErrorMessage("Preset import JSON is invalid.");
      return;
    }

    const importedPresets = normalizeQueryPresets(parsed);
    if (importedPresets.length === 0) {
      setErrorMessage("Preset import did not include valid presets.");
      return;
    }

    const nextPresets = [
      ...queryPresets.filter(
        (existingPreset) =>
          !importedPresets.some(
            (importedPreset) => importedPreset.name === existingPreset.name,
          ),
      ),
      ...importedPresets,
    ].sort((left, right) => left.name.localeCompare(right.name));

    savePresetsToStorage(nextPresets);
    setStatusMessage(`Imported ${importedPresets.length} preset(s).`);
  }

  async function handleSaveField(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    clearMessages();
    setIsSavingField(true);

    try {
      let parsedDefaultValue: unknown | null = null;
      if (defaultValueText.trim().length > 0) {
        parsedDefaultValue = JSON.parse(defaultValueText);
      }

      const payload: CreateFieldRequest = {
        logical_name: logicalName,
        display_name: displayName,
        field_type: fieldType,
        is_required: isRequired,
        is_unique: isUnique,
        default_value: parsedDefaultValue,
        relation_target_entity:
          relationTargetEntity.trim().length > 0 ? relationTargetEntity : null,
      };

      const response = await apiFetch(
        `/api/entities/${entityLogicalName}/fields`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save field.");
        return;
      }

      setLogicalName("");
      setDisplayName("");
      setFieldType("text");
      setIsRequired(false);
      setIsUnique(false);
      setDefaultValueText("");
      setRelationTargetEntity("");
      setStatusMessage("Field saved.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to save field.");
    } finally {
      setIsSavingField(false);
    }
  }

  async function handlePublish() {
    clearMessages();
    setIsPublishing(true);

    try {
      const response = await apiFetch(
        `/api/entities/${entityLogicalName}/publish`,
        {
          method: "POST",
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to publish entity.");
        return;
      }

      setStatusMessage("Entity published.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to publish entity.");
    } finally {
      setIsPublishing(false);
    }
  }

  async function handleCreateRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    clearMessages();
    setIsCreatingRecord(true);

    try {
      const parsed = JSON.parse(recordPayload) as unknown;
      if (
        parsed === null ||
        Array.isArray(parsed) ||
        typeof parsed !== "object"
      ) {
        setErrorMessage("Runtime record payload must be a JSON object.");
        return;
      }

      const payload: CreateRuntimeRecordRequest = {
        data: parsed as Record<string, unknown>,
      };

      const response = await apiFetch(
        `/api/runtime/${entityLogicalName}/records`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to create runtime record.");
        return;
      }

      setStatusMessage("Runtime record created.");
      setRecordPayload("{}");
      setQueriedRecords(null);
      router.refresh();
    } catch {
      setErrorMessage("Runtime record payload must be valid JSON.");
    } finally {
      setIsCreatingRecord(false);
    }
  }

  async function handleQueryRecords(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    clearMessages();
    setIsQueryingRecords(true);

    try {
      const parsedFilters = readPresetFiltersAsObject();
      if (parsedFilters === null) {
        return;
      }

      const parsedConditions = readPresetConditions();
      if (parsedConditions === null) {
        return;
      }

      const parsedSort = readPresetSort();
      if (parsedSort === null) {
        return;
      }

      const parsedLimit = Number.parseInt(queryLimitText, 10);
      const parsedOffset = Number.parseInt(queryOffsetText, 10);

      if (!Number.isFinite(parsedLimit) || parsedLimit <= 0) {
        setErrorMessage("Query limit must be a positive integer.");
        return;
      }

      if (!Number.isFinite(parsedOffset) || parsedOffset < 0) {
        setErrorMessage("Query offset must be zero or a positive integer.");
        return;
      }

      const payload: QueryRuntimeRecordsRequest = {
        limit: parsedLimit,
        offset: parsedOffset,
        logical_mode: queryLogicalMode,
        where: null,
        conditions: parsedConditions,
        link_entities: null,
        sort: parsedSort,
        filters:
          Object.keys(parsedFilters).length > 0
            ? parsedFilters
            : null,
      };

      const response = await apiFetch(
        `/api/runtime/${entityLogicalName}/records/query`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to query runtime records.");
        return;
      }

      const records = (await response.json()) as RuntimeRecordResponse[];
      setQueriedRecords(records);
      setStatusMessage(`Query returned ${records.length} record(s).`);
    } catch {
      setErrorMessage("Unable to query runtime records.");
    } finally {
      setIsQueryingRecords(false);
    }
  }

  async function handleDeleteRecord(recordId: string) {
    clearMessages();
    setDeletingRecordId(recordId);

    try {
      const response = await apiFetch(
        `/api/runtime/${entityLogicalName}/records/${recordId}`,
        {
          method: "DELETE",
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to delete runtime record.");
        return;
      }

      setStatusMessage("Runtime record deleted.");
      setQueriedRecords(null);
      router.refresh();
    } catch {
      setErrorMessage("Unable to delete runtime record.");
    } finally {
      setDeletingRecordId(null);
    }
  }

  const displayedRecords = queriedRecords ?? initialRecords;

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center gap-2 rounded-md border border-emerald-100 bg-white/90 p-3">
        <StatusBadge tone="neutral">Fields {initialFields.length}</StatusBadge>
        <StatusBadge tone="neutral">
          Records {displayedRecords.length}
        </StatusBadge>
        <StatusBadge tone={initialPublishedSchema ? "success" : "warning"}>
          {initialPublishedSchema
            ? `Published v${initialPublishedSchema.version}`
            : "Not Published"}
        </StatusBadge>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant={activeSection === "schema" ? "default" : "outline"}
          onClick={() => setActiveSection("schema")}
        >
          Schema Design
        </Button>
        <Button
          type="button"
          variant={activeSection === "runtime" ? "default" : "outline"}
          onClick={() => setActiveSection("runtime")}
        >
          Runtime Operations
        </Button>
      </div>

      {activeSection === "schema" ? (
        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <p className="text-sm font-medium text-zinc-800">Draft Fields</p>
            <Button
              disabled={isPublishing}
              onClick={handlePublish}
              type="button"
              variant="outline"
            >
              {isPublishing
                ? "Publishing..."
                : initialPublishedSchema
                  ? `Publish v${initialPublishedSchema.version + 1}`
                  : "Publish v1"}
            </Button>
          </div>

          <form
            className="grid gap-3 rounded-md border border-emerald-100 bg-white p-4 md:grid-cols-2"
            onSubmit={handleSaveField}
          >
            <div className="space-y-2">
              <Label htmlFor="field_logical_name">Logical Name</Label>
              <Input
                id="field_logical_name"
                onChange={(event) => setLogicalName(event.target.value)}
                placeholder="name"
                required
                value={logicalName}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="field_display_name">Display Name</Label>
              <Input
                id="field_display_name"
                onChange={(event) => setDisplayName(event.target.value)}
                placeholder="Name"
                required
                value={displayName}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="field_type">Field Type</Label>
              <Select
                id="field_type"
                onChange={(event) =>
                  setFieldType(
                    event.target.value as (typeof FIELD_TYPE_OPTIONS)[number],
                  )
                }
                value={fieldType}
              >
                {FIELD_TYPE_OPTIONS.map((option) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </Select>
            </div>

            <div className="space-y-2">
              <Label htmlFor="relation_target_entity">
                Relation Target Entity
              </Label>
              <Input
                id="relation_target_entity"
                onChange={(event) =>
                  setRelationTargetEntity(event.target.value)
                }
                placeholder="contact"
                value={relationTargetEntity}
              />
            </div>

            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="default_value">Default Value (JSON)</Label>
              <Textarea
                id="default_value"
                onChange={(event) => setDefaultValueText(event.target.value)}
                placeholder='"Acme" or true or {"enabled":true}'
                value={defaultValueText}
              />
            </div>

            <div className="flex items-center gap-2 text-sm text-zinc-700">
              <Checkbox
                id="field_is_required"
                checked={isRequired}
                onChange={(event) => setIsRequired(event.target.checked)}
              />
              <Label htmlFor="field_is_required">Required</Label>
            </div>

            <div className="flex items-center gap-2 text-sm text-zinc-700">
              <Checkbox
                id="field_is_unique"
                checked={isUnique}
                onChange={(event) => setIsUnique(event.target.checked)}
              />
              <Label htmlFor="field_is_unique">Unique</Label>
            </div>

            <div className="md:col-span-2">
              <Button disabled={isSavingField} type="submit">
                {isSavingField ? "Saving..." : "Save Field"}
              </Button>
            </div>
          </form>

          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Logical Name</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Required</TableHead>
                <TableHead>Unique</TableHead>
                <TableHead>Default</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {initialFields.length > 0 ? (
                initialFields.map((field) => (
                  <TableRow
                    key={`${field.entity_logical_name}.${field.logical_name}`}
                  >
                    <TableCell className="font-mono text-xs">
                      {field.logical_name}
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {field.field_type}
                    </TableCell>
                    <TableCell>{field.is_required ? "Yes" : "No"}</TableCell>
                    <TableCell>{field.is_unique ? "Yes" : "No"}</TableCell>
                    <TableCell className="font-mono text-xs">
                      {field.default_value === null
                        ? "-"
                        : JSON.stringify(field.default_value)}
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell className="text-zinc-500" colSpan={5}>
                    No fields defined yet.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </section>
      ) : null}

      {activeSection === "runtime" ? (
        <section className="space-y-3">
          <div>
            <p className="text-sm font-medium text-zinc-800">Runtime Records</p>
            <p className="text-xs text-zinc-500">
              {initialPublishedSchema
                ? `Using published schema version ${initialPublishedSchema.version}.`
                : "Publish this entity before creating runtime records."}
            </p>
          </div>

          <div className="flex flex-wrap gap-2 rounded-md border border-zinc-200 bg-zinc-50 p-3">
            <Button
              type="button"
              variant={
                activeRuntimeSection === "create" ? "default" : "outline"
              }
              onClick={() => setActiveRuntimeSection("create")}
            >
              Create Record
            </Button>
            <Button
              type="button"
              variant={activeRuntimeSection === "query" ? "default" : "outline"}
              onClick={() => setActiveRuntimeSection("query")}
            >
              Query & Presets
            </Button>
          </div>

          {activeRuntimeSection === "create" ? (
            <form
              className="space-y-3 rounded-md border border-emerald-100 bg-white p-4"
              onSubmit={handleCreateRecord}
            >
              <Label htmlFor="record_payload">
                Record Payload (JSON object)
              </Label>
              <Textarea
                id="record_payload"
                className="font-mono text-xs"
                onChange={(event) => setRecordPayload(event.target.value)}
                placeholder='{"name":"Alice"}'
                value={recordPayload}
              />
              <Button
                disabled={isCreatingRecord || !initialPublishedSchema}
                type="submit"
              >
                {isCreatingRecord ? "Creating..." : "Create Runtime Record"}
              </Button>
            </form>
          ) : null}

          {activeRuntimeSection === "query" ? (
            <form
              className="space-y-3 rounded-md border border-zinc-200 bg-zinc-50 p-4"
              onSubmit={handleQueryRecords}
            >
              <div className="grid gap-3 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="query_limit">Query Limit</Label>
                  <Input
                    id="query_limit"
                    min={1}
                    onChange={(event) => setQueryLimitText(event.target.value)}
                    type="number"
                    value={queryLimitText}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="query_offset">Query Offset</Label>
                  <Input
                    id="query_offset"
                    min={0}
                    onChange={(event) => setQueryOffsetText(event.target.value)}
                    type="number"
                    value={queryOffsetText}
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="query_logical_mode">Condition Mode</Label>
                <Select
                  id="query_logical_mode"
                  onChange={(event) =>
                    setQueryLogicalMode(
                      event.target.value === "or" ? "or" : "and",
                    )
                  }
                  value={queryLogicalMode}
                >
                  <option value="and">and</option>
                  <option value="or">or</option>
                </Select>
              </div>

              <div className="space-y-2">
                <Label htmlFor="query_conditions">Conditions (JSON array)</Label>
                <Textarea
                  id="query_conditions"
                  className="font-mono text-xs"
                  onChange={(event) => setQueryConditionsText(event.target.value)}
                  placeholder='[{"field_logical_name":"name","operator":"contains","field_value":"Ali"}]'
                  value={queryConditionsText}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="query_sort">Sort (JSON array)</Label>
                <Textarea
                  id="query_sort"
                  className="font-mono text-xs"
                  onChange={(event) => setQuerySortText(event.target.value)}
                  placeholder='[{"field_logical_name":"name","direction":"asc"}]'
                  value={querySortText}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="query_filters">
                  Legacy Exact-Match Filters (JSON object)
                </Label>
                <Textarea
                  id="query_filters"
                  className="font-mono text-xs"
                  onChange={(event) => setQueryFiltersText(event.target.value)}
                  placeholder='{"name":"Alice","active":true}'
                  value={queryFiltersText}
                />
              </div>

              <div className="grid gap-3 md:grid-cols-[1fr_auto]">
                <div className="space-y-2">
                  <Label htmlFor="query_preset_name">Preset Name</Label>
                  <Input
                    id="query_preset_name"
                    onChange={(event) => setQueryPresetName(event.target.value)}
                    placeholder="active-contacts"
                    value={queryPresetName}
                  />
                </div>
                <div className="flex items-end">
                  <Button
                    disabled={isSavingPreset}
                    onClick={handleSaveQueryPreset}
                    type="button"
                    variant="outline"
                  >
                    {isSavingPreset ? "Saving..." : "Save Preset"}
                  </Button>
                </div>
              </div>

              <div className="grid gap-3 md:grid-cols-[1fr_auto_auto]">
                <div className="space-y-2">
                  <Label htmlFor="saved_query_presets">Saved Presets</Label>
                  <Select
                    id="saved_query_presets"
                    onChange={(event) =>
                      setSelectedPresetName(event.target.value)
                    }
                    value={selectedPresetName}
                  >
                    <option value="">Select preset...</option>
                    {queryPresets.map((preset) => (
                      <option key={preset.name} value={preset.name}>
                        {preset.name}
                      </option>
                    ))}
                  </Select>
                </div>
                <div className="flex items-end">
                  <Button
                    disabled={selectedPresetName.length === 0}
                    onClick={() => {
                      clearMessages();
                      loadPreset(selectedPresetName);
                    }}
                    type="button"
                    variant="outline"
                  >
                    Load
                  </Button>
                </div>
                <div className="flex items-end">
                  <Button
                    disabled={selectedPresetName.length === 0}
                    onClick={handleDeleteSelectedPreset}
                    type="button"
                    variant="ghost"
                  >
                    Delete Preset
                  </Button>
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="query_preset_transfer">
                  Preset Import/Export (JSON)
                </Label>
                <Textarea
                  id="query_preset_transfer"
                  className="font-mono text-xs"
                  onChange={(event) =>
                    setPresetTransferText(event.target.value)
                  }
                  placeholder='[{"name":"active-contacts","limitText":"50","offsetText":"0","logicalMode":"and","conditionsText":"[{\"field_logical_name\":\"active\",\"operator\":\"eq\",\"field_value\":true}]","sortText":"[]","filtersText":"{}"}]'
                  value={presetTransferText}
                />
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <Button
                  onClick={handleExportQueryPresets}
                  type="button"
                  variant="outline"
                >
                  Export Presets
                </Button>
                <Button
                  onClick={handleImportQueryPresets}
                  type="button"
                  variant="outline"
                >
                  Import Presets
                </Button>
                {isPresetCopied ? (
                  <span className="text-xs text-emerald-700">Copied!</span>
                ) : null}
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <Button
                  disabled={isQueryingRecords || !initialPublishedSchema}
                  type="submit"
                  variant="outline"
                >
                  {isQueryingRecords ? "Querying..." : "Query Records"}
                </Button>
                <Button
                  disabled={queriedRecords === null}
                  onClick={() => {
                    clearMessages();
                    setQueriedRecords(null);
                  }}
                  type="button"
                  variant="ghost"
                >
                  Clear Query
                </Button>
              </div>
            </form>
          ) : null}

          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Record ID</TableHead>
                <TableHead>Data</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {displayedRecords.length > 0 ? (
                displayedRecords.map((record) => (
                  <TableRow key={record.record_id}>
                    <TableCell className="font-mono text-xs">
                      {record.record_id}
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {JSON.stringify(record.data)}
                    </TableCell>
                    <TableCell>
                      <Button
                        disabled={deletingRecordId === record.record_id}
                        onClick={() => handleDeleteRecord(record.record_id)}
                        size="sm"
                        type="button"
                        variant="outline"
                      >
                        {deletingRecordId === record.record_id
                          ? "Deleting..."
                          : "Delete"}
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell className="text-zinc-500" colSpan={3}>
                    {queriedRecords === null
                      ? "No runtime records yet."
                      : "No runtime records matched the query."}
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </section>
      ) : null}

      {errorMessage ? (
        <p className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {errorMessage}
        </p>
      ) : null}
      {statusMessage ? (
        <p className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {statusMessage}
        </p>
      ) : null}
    </div>
  );
}
