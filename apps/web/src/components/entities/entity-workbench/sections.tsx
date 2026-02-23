import { type FormEvent } from "react";

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

import type {
  FieldResponse,
  PublishedSchemaResponse,
  RuntimeRecordResponse,
} from "@/lib/api";
import {
  FIELD_TYPE_OPTIONS,
  type RuntimeSection,
  type WorkbenchSection,
} from "@/components/entities/entity-workbench/use-entity-workbench-panel";
import type { QueryPreset } from "@/components/entities/entity-workbench/presets";

type WorkbenchOverviewProps = {
  activeSection: WorkbenchSection;
  fieldCount: number;
  hasPublishedSchema: boolean;
  onSectionChange: (section: WorkbenchSection) => void;
  publishedVersion: number | null;
  recordCount: number;
};

export function WorkbenchOverview({
  activeSection,
  fieldCount,
  hasPublishedSchema,
  onSectionChange,
  publishedVersion,
  recordCount,
}: WorkbenchOverviewProps) {
  return (
    <>
      <div className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-zinc-200 bg-zinc-50 p-3">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
          Entity Designer
        </p>
        <div className="flex flex-wrap items-center gap-2">
          <StatusBadge tone="neutral">Fields {fieldCount}</StatusBadge>
          <StatusBadge tone="neutral">Records {recordCount}</StatusBadge>
          <StatusBadge tone={hasPublishedSchema ? "success" : "warning"}>
            {hasPublishedSchema && publishedVersion !== null
              ? `Published v${publishedVersion}`
              : "Not Published"}
          </StatusBadge>
        </div>
      </div>

      <div className="flex flex-wrap gap-2 rounded-md border border-zinc-200 bg-white p-3">
        <Button
          type="button"
          variant={activeSection === "schema" ? "default" : "outline"}
          onClick={() => onSectionChange("schema")}
        >
          Data Model
        </Button>
        <Button
          type="button"
          variant={activeSection === "runtime" ? "default" : "outline"}
          onClick={() => onSectionChange("runtime")}
        >
          Runtime Operations
        </Button>
      </div>
    </>
  );
}

type SchemaDesignSectionProps = {
  defaultValueText: string;
  displayName: string;
  fieldType: (typeof FIELD_TYPE_OPTIONS)[number];
  handlePublish: () => Promise<void>;
  handleSaveField: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  initialFields: FieldResponse[];
  initialPublishedSchema: PublishedSchemaResponse | null;
  isPublishing: boolean;
  isRequired: boolean;
  isSavingField: boolean;
  isUnique: boolean;
  logicalName: string;
  relationTargetEntity: string;
  setDefaultValueText: (value: string) => void;
  setDisplayName: (value: string) => void;
  setFieldType: (value: (typeof FIELD_TYPE_OPTIONS)[number]) => void;
  setIsRequired: (value: boolean) => void;
  setIsUnique: (value: boolean) => void;
  setLogicalName: (value: string) => void;
  setRelationTargetEntity: (value: string) => void;
};

export function SchemaDesignSection({
  defaultValueText,
  displayName,
  fieldType,
  handlePublish,
  handleSaveField,
  initialFields,
  initialPublishedSchema,
  isPublishing,
  isRequired,
  isSavingField,
  isUnique,
  logicalName,
  relationTargetEntity,
  setDefaultValueText,
  setDisplayName,
  setFieldType,
  setIsRequired,
  setIsUnique,
  setLogicalName,
  setRelationTargetEntity,
}: SchemaDesignSectionProps) {
  return (
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
        className="grid gap-3 rounded-md border border-zinc-200 bg-white p-4 md:grid-cols-2"
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
              setFieldType(event.target.value as (typeof FIELD_TYPE_OPTIONS)[number])
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
          <Label htmlFor="relation_target_entity">Relation Target Entity</Label>
          <Input
            id="relation_target_entity"
            onChange={(event) => setRelationTargetEntity(event.target.value)}
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
              <TableRow key={`${field.entity_logical_name}.${field.logical_name}`}>
                <TableCell className="font-mono text-xs">{field.logical_name}</TableCell>
                <TableCell className="font-mono text-xs">{field.field_type}</TableCell>
                <TableCell>{field.is_required ? "Yes" : "No"}</TableCell>
                <TableCell>{field.is_unique ? "Yes" : "No"}</TableCell>
                <TableCell className="font-mono text-xs">
                  {field.default_value === null ? "-" : JSON.stringify(field.default_value)}
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
  );
}

type RuntimeOperationsSectionProps = {
  activeRuntimeSection: RuntimeSection;
  deletingRecordId: string | null;
  displayedRecords: RuntimeRecordResponse[];
  handleClearQuery: () => void;
  handleCreateRecord: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  handleDeleteRecord: (recordId: string) => Promise<void>;
  handleDeleteSelectedPreset: () => void;
  handleExportQueryPresets: () => Promise<void>;
  handleImportQueryPresets: () => void;
  handleLoadSelectedPreset: () => void;
  handleQueryRecords: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  handleSaveQueryPreset: () => void;
  initialPublishedSchema: PublishedSchemaResponse | null;
  isCreatingRecord: boolean;
  isPresetCopied: boolean;
  isQueryingRecords: boolean;
  isSavingPreset: boolean;
  presetTransferText: string;
  queriedRecords: RuntimeRecordResponse[] | null;
  queryConditionsText: string;
  queryFiltersText: string;
  queryLimitText: string;
  queryLogicalMode: "and" | "or";
  queryOffsetText: string;
  queryPresetName: string;
  queryPresets: QueryPreset[];
  querySortText: string;
  recordPayload: string;
  selectedPresetName: string;
  setActiveRuntimeSection: (section: RuntimeSection) => void;
  setPresetTransferText: (value: string) => void;
  setQueryConditionsText: (value: string) => void;
  setQueryFiltersText: (value: string) => void;
  setQueryLimitText: (value: string) => void;
  setQueryLogicalMode: (value: "and" | "or") => void;
  setQueryOffsetText: (value: string) => void;
  setQueryPresetName: (value: string) => void;
  setQuerySortText: (value: string) => void;
  setRecordPayload: (value: string) => void;
  setSelectedPresetName: (value: string) => void;
};

export function RuntimeOperationsSection({
  activeRuntimeSection,
  deletingRecordId,
  displayedRecords,
  handleClearQuery,
  handleCreateRecord,
  handleDeleteRecord,
  handleDeleteSelectedPreset,
  handleExportQueryPresets,
  handleImportQueryPresets,
  handleLoadSelectedPreset,
  handleQueryRecords,
  handleSaveQueryPreset,
  initialPublishedSchema,
  isCreatingRecord,
  isPresetCopied,
  isQueryingRecords,
  isSavingPreset,
  presetTransferText,
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
  selectedPresetName,
  setActiveRuntimeSection,
  setPresetTransferText,
  setQueryConditionsText,
  setQueryFiltersText,
  setQueryLimitText,
  setQueryLogicalMode,
  setQueryOffsetText,
  setQueryPresetName,
  setQuerySortText,
  setRecordPayload,
  setSelectedPresetName,
}: RuntimeOperationsSectionProps) {
  return (
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
          variant={activeRuntimeSection === "create" ? "default" : "outline"}
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
          className="space-y-3 rounded-md border border-zinc-200 bg-white p-4"
          onSubmit={handleCreateRecord}
        >
          <Label htmlFor="record_payload">Record Payload (JSON object)</Label>
          <Textarea
            id="record_payload"
            className="font-mono text-xs"
            onChange={(event) => setRecordPayload(event.target.value)}
            placeholder='{"name":"Alice"}'
            value={recordPayload}
          />
          <Button disabled={isCreatingRecord || !initialPublishedSchema} type="submit">
            {isCreatingRecord ? "Creating..." : "Create Runtime Record"}
          </Button>
        </form>
      ) : null}

      {activeRuntimeSection === "query" ? (
        <RuntimeQueryForm
          handleDeleteSelectedPreset={handleDeleteSelectedPreset}
          handleExportQueryPresets={handleExportQueryPresets}
          handleImportQueryPresets={handleImportQueryPresets}
          handleLoadSelectedPreset={handleLoadSelectedPreset}
          handleQueryRecords={handleQueryRecords}
          handleSaveQueryPreset={handleSaveQueryPreset}
          initialPublishedSchema={initialPublishedSchema}
          isPresetCopied={isPresetCopied}
          isQueryingRecords={isQueryingRecords}
          isSavingPreset={isSavingPreset}
          presetTransferText={presetTransferText}
          queryConditionsText={queryConditionsText}
          queryFiltersText={queryFiltersText}
          queryLimitText={queryLimitText}
          queryLogicalMode={queryLogicalMode}
          queryOffsetText={queryOffsetText}
          queryPresetName={queryPresetName}
          queryPresets={queryPresets}
          querySortText={querySortText}
          selectedPresetName={selectedPresetName}
          setPresetTransferText={setPresetTransferText}
          setQueryConditionsText={setQueryConditionsText}
          setQueryFiltersText={setQueryFiltersText}
          setQueryLimitText={setQueryLimitText}
          setQueryLogicalMode={setQueryLogicalMode}
          setQueryOffsetText={setQueryOffsetText}
          setQueryPresetName={setQueryPresetName}
          setQuerySortText={setQuerySortText}
          setSelectedPresetName={setSelectedPresetName}
        />
      ) : null}

      <RuntimeRecordsTable
        deletingRecordId={deletingRecordId}
        displayedRecords={displayedRecords}
        handleDeleteRecord={handleDeleteRecord}
        handleClearQuery={handleClearQuery}
        queriedRecords={queriedRecords}
      />
    </section>
  );
}

type RuntimeQueryFormProps = {
  handleDeleteSelectedPreset: () => void;
  handleExportQueryPresets: () => Promise<void>;
  handleImportQueryPresets: () => void;
  handleLoadSelectedPreset: () => void;
  handleQueryRecords: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  handleSaveQueryPreset: () => void;
  initialPublishedSchema: PublishedSchemaResponse | null;
  isPresetCopied: boolean;
  isQueryingRecords: boolean;
  isSavingPreset: boolean;
  presetTransferText: string;
  queryConditionsText: string;
  queryFiltersText: string;
  queryLimitText: string;
  queryLogicalMode: "and" | "or";
  queryOffsetText: string;
  queryPresetName: string;
  queryPresets: QueryPreset[];
  querySortText: string;
  selectedPresetName: string;
  setPresetTransferText: (value: string) => void;
  setQueryConditionsText: (value: string) => void;
  setQueryFiltersText: (value: string) => void;
  setQueryLimitText: (value: string) => void;
  setQueryLogicalMode: (value: "and" | "or") => void;
  setQueryOffsetText: (value: string) => void;
  setQueryPresetName: (value: string) => void;
  setQuerySortText: (value: string) => void;
  setSelectedPresetName: (value: string) => void;
};

function RuntimeQueryForm({
  handleDeleteSelectedPreset,
  handleExportQueryPresets,
  handleImportQueryPresets,
  handleLoadSelectedPreset,
  handleQueryRecords,
  handleSaveQueryPreset,
  initialPublishedSchema,
  isPresetCopied,
  isQueryingRecords,
  isSavingPreset,
  presetTransferText,
  queryConditionsText,
  queryFiltersText,
  queryLimitText,
  queryLogicalMode,
  queryOffsetText,
  queryPresetName,
  queryPresets,
  querySortText,
  selectedPresetName,
  setPresetTransferText,
  setQueryConditionsText,
  setQueryFiltersText,
  setQueryLimitText,
  setQueryLogicalMode,
  setQueryOffsetText,
  setQueryPresetName,
  setQuerySortText,
  setSelectedPresetName,
}: RuntimeQueryFormProps) {
  return (
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
            setQueryLogicalMode(event.target.value === "or" ? "or" : "and")
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
        <Label htmlFor="query_filters">Legacy Exact-Match Filters (JSON object)</Label>
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
            onChange={(event) => setSelectedPresetName(event.target.value)}
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
            onClick={handleLoadSelectedPreset}
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
        <Label htmlFor="query_preset_transfer">Preset Import/Export (JSON)</Label>
        <Textarea
          id="query_preset_transfer"
          className="font-mono text-xs"
          onChange={(event) => setPresetTransferText(event.target.value)}
          placeholder='[{"name":"active-contacts","limitText":"50","offsetText":"0","logicalMode":"and","conditionsText":"[{\"field_logical_name\":\"active\",\"operator\":\"eq\",\"field_value\":true}]","sortText":"[]","filtersText":"{}"}]'
          value={presetTransferText}
        />
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Button onClick={handleExportQueryPresets} type="button" variant="outline">
          Export Presets
        </Button>
        <Button onClick={handleImportQueryPresets} type="button" variant="outline">
          Import Presets
        </Button>
        {isPresetCopied ? <span className="text-xs text-emerald-700">Copied!</span> : null}
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Button
          disabled={isQueryingRecords || !initialPublishedSchema}
          type="submit"
          variant="outline"
        >
          {isQueryingRecords ? "Querying..." : "Query Records"}
        </Button>
      </div>
    </form>
  );
}

type RuntimeRecordsTableProps = {
  deletingRecordId: string | null;
  displayedRecords: RuntimeRecordResponse[];
  handleClearQuery: () => void;
  handleDeleteRecord: (recordId: string) => Promise<void>;
  queriedRecords: RuntimeRecordResponse[] | null;
};

function RuntimeRecordsTable({
  deletingRecordId,
  displayedRecords,
  handleClearQuery,
  handleDeleteRecord,
  queriedRecords,
}: RuntimeRecordsTableProps) {
  return (
    <>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          disabled={queriedRecords === null}
          onClick={handleClearQuery}
          type="button"
          variant="ghost"
        >
          Clear Query
        </Button>
      </div>

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
                <TableCell className="font-mono text-xs">{record.record_id}</TableCell>
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
                    {deletingRecordId === record.record_id ? "Deleting..." : "Delete"}
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
    </>
  );
}
