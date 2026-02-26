import { type FormEvent, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";

import {
  apiFetch,
  type CreateFieldRequest,
  type CreateRuntimeRecordRequest,
  type EntityResponse,
  type FieldResponse,
  type PublishChecksResponse,
  type PublishedSchemaResponse,
  type QueryRuntimeRecordsRequest,
  type RuntimeRecordQueryFilterRequest,
  type RuntimeRecordQuerySortRequest,
  type RuntimeRecordResponse,
} from "@/lib/api";
import {
  normalizeQueryPresets,
  type QueryPreset,
} from "@/components/entities/entity-workbench/presets";

export const FIELD_TYPE_OPTIONS = [
  "text",
  "number",
  "boolean",
  "date",
  "datetime",
  "json",
  "relation",
] as const;

export type WorkbenchSection = "schema" | "runtime";
export type RuntimeSection = "create" | "query";
export type RelationAuthoringMode = "lookup" | "junction_pair";

type UseEntityWorkbenchPanelInput = {
  entityLogicalName: string;
  initialEntities: EntityResponse[];
  initialFields: FieldResponse[];
  initialPublishedSchema: PublishedSchemaResponse | null;
  initialRecords: RuntimeRecordResponse[];
};

export function useEntityWorkbenchPanel({
  entityLogicalName,
  initialEntities,
  initialFields,
  initialPublishedSchema,
  initialRecords,
}: UseEntityWorkbenchPanelInput) {
  const router = useRouter();

  const [logicalName, setLogicalName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [fieldType, setFieldType] =
    useState<(typeof FIELD_TYPE_OPTIONS)[number]>("text");
  const [isRequired, setIsRequired] = useState(false);
  const [isUnique, setIsUnique] = useState(false);
  const [defaultValueText, setDefaultValueText] = useState("");
  const [calculationExpressionText, setCalculationExpressionText] = useState("");
  const [relationTargetEntity, setRelationTargetEntity] = useState("");
  const [relationAuthoringMode, setRelationAuthoringMode] =
    useState<RelationAuthoringMode>("lookup");
  const [secondaryRelationTargetEntity, setSecondaryRelationTargetEntity] =
    useState("");
  const [secondaryLogicalName, setSecondaryLogicalName] = useState("");
  const [secondaryDisplayName, setSecondaryDisplayName] = useState("");

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
  const [publishCheckErrors, setPublishCheckErrors] = useState<string[]>([]);

  const [isSavingField, setIsSavingField] = useState(false);
  const [isPublishing, setIsPublishing] = useState(false);
  const [isCheckingPublish, setIsCheckingPublish] = useState(false);
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

    let nextPresets: QueryPreset[] = [];
    const rawPresets = window.localStorage.getItem(queryPresetsStorageKey);
    if (rawPresets) {
      try {
        const parsed = JSON.parse(rawPresets) as unknown;
        nextPresets = normalizeQueryPresets(parsed);
      } catch {
        nextPresets = [];
      }
    }

    setQueryPresets(nextPresets);
    setSelectedPresetName("");
  }, [queryPresetsStorageKey]);

  useEffect(() => {
    return () => {
      if (presetCopiedTimeoutRef.current !== null) {
        window.clearTimeout(presetCopiedTimeoutRef.current);
      }
    };
  }, []);

  useEffect(() => {
    if (fieldType !== "relation") {
      if (relationTargetEntity !== "") {
        setRelationTargetEntity("");
      }
      if (secondaryRelationTargetEntity !== "") {
        setSecondaryRelationTargetEntity("");
      }
      if (secondaryLogicalName !== "") {
        setSecondaryLogicalName("");
      }
      if (secondaryDisplayName !== "") {
        setSecondaryDisplayName("");
      }
      if (relationAuthoringMode !== "lookup") {
        setRelationAuthoringMode("lookup");
      }
      return;
    }

    const fallbackTarget = initialEntities[0]?.logical_name ?? "";
    const primaryTarget =
      relationTargetEntity.trim().length > 0
        ? relationTargetEntity.trim()
        : fallbackTarget;

    if (relationTargetEntity.trim().length === 0 && fallbackTarget.length > 0) {
      setRelationTargetEntity(fallbackTarget);
    }

    if (relationAuthoringMode === "junction_pair") {
      const fallbackSecondaryTarget =
        initialEntities.find(
          (entity) => entity.logical_name !== primaryTarget,
        )?.logical_name ?? primaryTarget;
      if (
        secondaryRelationTargetEntity.trim().length === 0 &&
        fallbackSecondaryTarget.length > 0
      ) {
        setSecondaryRelationTargetEntity(fallbackSecondaryTarget);
      }
    }
  }, [
    fieldType,
    initialEntities,
    relationTargetEntity,
    relationAuthoringMode,
    secondaryRelationTargetEntity,
  ]);

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

  function clearPublishChecks() {
    setPublishCheckErrors([]);
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

  function handleLoadSelectedPreset() {
    clearMessages();
    loadPreset(selectedPresetName);
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
    clearPublishChecks();
    setIsSavingField(true);

    try {
      if (fieldType === "relation" && relationTargetEntity.trim().length === 0) {
        setErrorMessage("Relation fields require a relation target entity.");
        return;
      }

      if (fieldType === "relation" && relationAuthoringMode === "junction_pair") {
        if (secondaryLogicalName.trim().length === 0) {
          setErrorMessage(
            "Junction pair template requires a secondary logical field name.",
          );
          return;
        }

        if (secondaryDisplayName.trim().length === 0) {
          setErrorMessage(
            "Junction pair template requires a secondary display field name.",
          );
          return;
        }

        if (secondaryRelationTargetEntity.trim().length === 0) {
          setErrorMessage(
            "Junction pair template requires a secondary relation target entity.",
          );
          return;
        }

        if (
          secondaryLogicalName.trim().toLowerCase() ===
          logicalName.trim().toLowerCase()
        ) {
          setErrorMessage(
            "Primary and secondary relation field logical names must be different.",
          );
          return;
        }
      }

      if (
        calculationExpressionText.trim().length > 0 &&
        fieldType !== "text" &&
        fieldType !== "number"
      ) {
        setErrorMessage(
          "Calculated fields currently support text and number field types only.",
        );
        return;
      }

      if (
        calculationExpressionText.trim().length > 0 &&
        defaultValueText.trim().length > 0
      ) {
        setErrorMessage(
          "Calculated fields cannot define a default value.",
        );
        return;
      }

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
        calculation_expression:
          calculationExpressionText.trim().length > 0
            ? calculationExpressionText.trim()
            : null,
        relation_target_entity:
          fieldType === "relation" && relationTargetEntity.trim().length > 0
            ? relationTargetEntity
            : null,
        option_set_logical_name: null,
      };

      const response = await apiFetch(`/api/entities/${entityLogicalName}/fields`, {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save field.");
        return;
      }

      if (fieldType === "relation" && relationAuthoringMode === "junction_pair") {
        const secondaryPayload: CreateFieldRequest = {
          logical_name: secondaryLogicalName.trim(),
          display_name: secondaryDisplayName.trim(),
          field_type: "relation",
          is_required: false,
          is_unique: false,
          default_value: null,
          calculation_expression: null,
          relation_target_entity: secondaryRelationTargetEntity.trim(),
          option_set_logical_name: null,
        };

        const secondaryResponse = await apiFetch(
          `/api/entities/${entityLogicalName}/fields`,
          {
            method: "POST",
            body: JSON.stringify(secondaryPayload),
          },
        );

        if (!secondaryResponse.ok) {
          const secondaryError = (await secondaryResponse.json()) as {
            message?: string;
          };
          setErrorMessage(
            secondaryError.message ??
              "Primary relation field was saved, but secondary relation field failed.",
          );
          setStatusMessage(
            `Saved relation field '${logicalName.trim()}' but secondary field '${secondaryLogicalName.trim()}' needs attention.`,
          );
          router.refresh();
          return;
        }
      }

      setLogicalName("");
      setDisplayName("");
      setFieldType("text");
      setIsRequired(false);
      setIsUnique(false);
      setDefaultValueText("");
      setCalculationExpressionText("");
      setRelationTargetEntity("");
      setRelationAuthoringMode("lookup");
      setSecondaryRelationTargetEntity("");
      setSecondaryLogicalName("");
      setSecondaryDisplayName("");
      setStatusMessage(
        fieldType === "relation" && relationAuthoringMode === "junction_pair"
          ? "Junction relation pair saved."
          : "Field saved.",
      );
      router.refresh();
    } catch {
      setErrorMessage("Unable to save field.");
    } finally {
      setIsSavingField(false);
    }
  }

  async function handlePublish() {
    clearMessages();
    clearPublishChecks();
    setIsPublishing(true);

    try {
      const response = await apiFetch(`/api/entities/${entityLogicalName}/publish`, {
        method: "POST",
      });

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

  async function handlePublishChecks() {
    clearMessages();
    setIsCheckingPublish(true);

    try {
      const response = await apiFetch(
        `/api/entities/${entityLogicalName}/publish-checks`,
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to run publish checks.");
        return;
      }

      const payload = (await response.json()) as PublishChecksResponse;
      setPublishCheckErrors(payload.errors);
      if (payload.is_publishable) {
        setStatusMessage("Publish checks passed.");
      } else {
        setErrorMessage("Publish checks found issues. Resolve them before publishing.");
      }
    } catch {
      setErrorMessage("Unable to run publish checks.");
    } finally {
      setIsCheckingPublish(false);
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

      const response = await apiFetch(`/api/runtime/${entityLogicalName}/records`, {
        method: "POST",
        body: JSON.stringify(payload),
      });

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
        filters: Object.keys(parsedFilters).length > 0 ? parsedFilters : null,
      };

      const response = await apiFetch(`/api/runtime/${entityLogicalName}/records/query`, {
        method: "POST",
        body: JSON.stringify(payload),
      });

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
      const response = await apiFetch(`/api/runtime/${entityLogicalName}/records/${recordId}`, {
        method: "DELETE",
      });

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

  function handleClearQuery() {
    clearMessages();
    setQueriedRecords(null);
  }

  const displayedRecords = queriedRecords ?? initialRecords;

  return {
    activeRuntimeSection,
    activeSection,
    clearMessages,
    defaultValueText,
    calculationExpressionText,
    deletingRecordId,
    displayName,
    displayedRecords,
    errorMessage,
    fieldType,
    handleClearQuery,
    handleCreateRecord,
    handleDeleteRecord,
    handleDeleteSelectedPreset,
    handleExportQueryPresets,
    handleImportQueryPresets,
    handleLoadSelectedPreset,
    handlePublish,
    handlePublishChecks,
    handleQueryRecords,
    handleSaveField,
    handleSaveQueryPreset,
    initialFields,
    initialEntities,
    initialPublishedSchema,
    isCreatingRecord,
    isPresetCopied,
    isPublishing,
    isCheckingPublish,
    isQueryingRecords,
    isRequired,
    isSavingField,
    isSavingPreset,
    isUnique,
    logicalName,
    presetTransferText,
    publishCheckErrors,
    queriedRecords,
    queryConditionsText,
    queryFiltersText,
    queryLimitText,
    queryLogicalMode,
    queryOffsetText,
    queryPresetName,
    queryPresets,
    querySortText,
    recordPayload,
    relationAuthoringMode,
    relationTargetEntity,
    secondaryDisplayName,
    secondaryLogicalName,
    secondaryRelationTargetEntity,
    selectedPresetName,
    setActiveRuntimeSection,
    setActiveSection,
    setDefaultValueText,
    setCalculationExpressionText,
    setDisplayName,
    setFieldType,
    setIsRequired,
    setIsUnique,
    setLogicalName,
    setPresetTransferText,
    setQueryConditionsText,
    setQueryFiltersText,
    setQueryLimitText,
    setQueryLogicalMode,
    setQueryOffsetText,
    setQueryPresetName,
    setQuerySortText,
    setRecordPayload,
    setRelationAuthoringMode,
    setRelationTargetEntity,
    setSecondaryDisplayName,
    setSecondaryLogicalName,
    setSecondaryRelationTargetEntity,
    setSelectedPresetName,
    statusMessage,
  };
}
