"use client";

import { type FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Notice,
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
  type AppEntityBindingResponse,
  type AppEntityCapabilitiesResponse,
  type CreateRuntimeRecordRequest,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";
import {
  buildInitialValues,
  formatValue,
  resolveConfiguredFields,
} from "@/components/apps/workspace-entity/helpers";

type WorkspaceEntityPanelProps = {
  appLogicalName: string;
  entityLogicalName: string;
  binding: AppEntityBindingResponse | null;
  schema: PublishedSchemaResponse;
  capabilities: AppEntityCapabilitiesResponse;
  records: RuntimeRecordResponse[];
};

type WorkerViewMode = "grid" | "json";

type PanelMessages = {
  errorMessage: string | null;
  statusMessage: string | null;
};

type MutationState = {
  isSaving: boolean;
  deletingRecordId: string | null;
};

type ViewState = {
  recordSearch: string;
  viewMode: WorkerViewMode;
  showCreatePanel: boolean;
};

export function WorkspaceEntityPanel({
  appLogicalName,
  entityLogicalName,
  binding,
  schema,
  capabilities,
  records,
}: WorkspaceEntityPanelProps) {
  const router = useRouter();

  const [formValues, setFormValues] = useState<Record<string, unknown>>(() =>
    buildInitialValues(schema),
  );
  const [messages, setMessages] = useState<PanelMessages>({
    errorMessage: null,
    statusMessage: null,
  });
  const [mutationState, setMutationState] = useState<MutationState>({
    isSaving: false,
    deletingRecordId: null,
  });
  const [viewState, setViewState] = useState<ViewState>({
    recordSearch: "",
    viewMode: binding?.default_view_mode ?? "grid",
    showCreatePanel: capabilities.can_create,
  });

  function setErrorMessage(next: string | null) {
    setMessages((current) => ({ ...current, errorMessage: next }));
  }

  function setStatusMessage(next: string | null) {
    setMessages((current) => ({ ...current, statusMessage: next }));
  }

  function clearMessages() {
    setMessages({ errorMessage: null, statusMessage: null });
  }

  const formFields = useMemo(
    () => resolveConfiguredFields(schema, binding?.form_field_logical_names ?? []),
    [binding?.form_field_logical_names, schema],
  );

  const gridFields = useMemo(
    () => {
      const configuredListFields = resolveConfiguredFields(
        schema,
        binding?.list_field_logical_names ?? [],
      ).filter((field) => field.field_type !== "json");

      if (configuredListFields.length > 0) {
        return configuredListFields;
      }

      return schema.fields.filter((field) => field.field_type !== "json").slice(0, 5);
    },
    [binding?.list_field_logical_names, schema],
  );

  const filteredRecords = useMemo(() => {
    const normalizedQuery = viewState.recordSearch.trim().toLowerCase();
    if (!normalizedQuery) {
      return records;
    }

    return records.filter((record) => {
      if (record.record_id.toLowerCase().includes(normalizedQuery)) {
        return true;
      }

      return JSON.stringify(record.data)
        .toLowerCase()
        .includes(normalizedQuery);
    });
  }, [viewState.recordSearch, records]);

  function setFieldValue(fieldLogicalName: string, value: unknown) {
    setFormValues((current) => ({
      ...current,
      [fieldLogicalName]: value,
    }));
  }

  function buildPayloadFromForm() {
    const payload: Record<string, unknown> = {};

    for (const field of formFields) {
      const value = formValues[field.logical_name];
      if (field.field_type === "boolean") {
        payload[field.logical_name] = Boolean(value);
        continue;
      }

      if (typeof value === "string") {
        const trimmed = value.trim();
        if (!trimmed) {
          continue;
        }

        if (field.field_type === "number") {
          payload[field.logical_name] = Number.parseFloat(trimmed);
          continue;
        }

        if (field.field_type === "json") {
          payload[field.logical_name] = JSON.parse(trimmed);
          continue;
        }

        payload[field.logical_name] = trimmed;
        continue;
      }

      if (value !== null && value !== undefined) {
        payload[field.logical_name] = value;
      }
    }

    return payload;
  }

  async function handleCreateRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!capabilities.can_create) {
      setErrorMessage(
        "You do not have create permission for this entity in this app.",
      );
      return;
    }

    clearMessages();
    setMutationState((current) => ({ ...current, isSaving: true }));

    try {
      const payload: CreateRuntimeRecordRequest = {
        data: buildPayloadFromForm(),
      };
      const response = await apiFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to create record.");
        return;
      }

      setStatusMessage("Record created.");
      setFormValues(buildInitialValues(schema));
      router.refresh();
    } catch {
      setErrorMessage("Unable to create record.");
    } finally {
      setMutationState((current) => ({ ...current, isSaving: false }));
    }
  }

  async function handleDeleteRecord(recordId: string) {
    if (!capabilities.can_delete) {
      setErrorMessage(
        "You do not have delete permission for this entity in this app.",
      );
      return;
    }

    setMutationState((current) => ({ ...current, deletingRecordId: recordId }));
    clearMessages();

    try {
      const response = await apiFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records/${recordId}`,
        {
          method: "DELETE",
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to delete record.");
        return;
      }

      setStatusMessage("Record deleted.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to delete record.");
    } finally {
      setMutationState((current) => ({ ...current, deletingRecordId: null }));
    }
  }

  return (
    <div className="space-y-6">
      <section className="space-y-3 rounded-md border border-emerald-100 bg-white p-4">
        <WorkspaceToolbar
          capabilities={capabilities}
          filteredRecordCount={filteredRecords.length}
          schemaVersion={schema.version}
          onRefresh={() => {
            clearMessages();
            router.refresh();
          }}
          onSearchChange={(recordSearch) =>
            setViewState((current) => ({ ...current, recordSearch }))
          }
          onToggleCreatePanel={() =>
            setViewState((current) => ({
              ...current,
              showCreatePanel: !current.showCreatePanel,
            }))
          }
          onViewModeChange={(viewMode) =>
            setViewState((current) => ({ ...current, viewMode }))
          }
          recordSearch={viewState.recordSearch}
          showCreatePanel={viewState.showCreatePanel}
          viewMode={viewState.viewMode}
        />
      </section>

      {viewState.showCreatePanel ? (
        <CreateRecordForm
          appLogicalName={appLogicalName}
          canCreate={capabilities.can_create}
          entityDisplayName={schema.entity_display_name}
          formFields={formFields}
          formValues={formValues}
          isSaving={mutationState.isSaving}
          onFieldValueChange={setFieldValue}
          onSubmit={handleCreateRecord}
        />
      ) : null}

      <RuntimeRecordsTable
        capabilities={capabilities}
        deletingRecordId={mutationState.deletingRecordId}
        filteredRecords={filteredRecords}
        gridFields={gridFields}
        onDeleteRecord={handleDeleteRecord}
        records={records}
        viewMode={viewState.viewMode}
      />

      {messages.errorMessage ? (
        <Notice tone="error">{messages.errorMessage}</Notice>
      ) : null}
      {messages.statusMessage ? (
        <Notice tone="success">{messages.statusMessage}</Notice>
      ) : null}
    </div>
  );
}

type WorkspaceToolbarProps = {
  capabilities: AppEntityCapabilitiesResponse;
  filteredRecordCount: number;
  schemaVersion: number;
  onRefresh: () => void;
  onSearchChange: (value: string) => void;
  onToggleCreatePanel: () => void;
  onViewModeChange: (viewMode: WorkerViewMode) => void;
  recordSearch: string;
  showCreatePanel: boolean;
  viewMode: WorkerViewMode;
};

function WorkspaceToolbar({
  capabilities,
  filteredRecordCount,
  schemaVersion,
  onRefresh,
  onSearchChange,
  onToggleCreatePanel,
  onViewModeChange,
  recordSearch,
  showCreatePanel,
  viewMode,
}: WorkspaceToolbarProps) {
  return (
    <>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-2">
          <StatusBadge tone="success">Schema v{schemaVersion}</StatusBadge>
          <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
            Create {capabilities.can_create ? "Enabled" : "Disabled"}
          </StatusBadge>
          <StatusBadge tone={capabilities.can_delete ? "warning" : "neutral"}>
            Delete {capabilities.can_delete ? "Enabled" : "Disabled"}
          </StatusBadge>
          <StatusBadge tone="neutral">Records {filteredRecordCount}</StatusBadge>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant={showCreatePanel ? "default" : "outline"}
            onClick={onToggleCreatePanel}
          >
            {showCreatePanel ? "Hide New Form" : "New Record"}
          </Button>
          <Button
            type="button"
            variant={viewMode === "grid" ? "default" : "outline"}
            onClick={() => onViewModeChange("grid")}
          >
            Grid View
          </Button>
          <Button
            type="button"
            variant={viewMode === "json" ? "default" : "outline"}
            onClick={() => onViewModeChange("json")}
          >
            JSON View
          </Button>
          <Button type="button" variant="outline" onClick={onRefresh}>
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid gap-3 md:grid-cols-[1fr_auto]">
        <Input
          value={recordSearch}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder="Search by record id or any field value"
        />
        <p className="text-xs text-zinc-500 md:self-center">
          {filteredRecordCount} visible row(s)
        </p>
      </div>
    </>
  );
}

type CreateRecordFormProps = {
  appLogicalName: string;
  canCreate: boolean;
  entityDisplayName: string;
  formFields: PublishedSchemaResponse["fields"];
  formValues: Record<string, unknown>;
  isSaving: boolean;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
};

function CreateRecordForm({
  appLogicalName,
  canCreate,
  entityDisplayName,
  formFields,
  formValues,
  isSaving,
  onFieldValueChange,
  onSubmit,
}: CreateRecordFormProps) {
  return (
    <form
      className="space-y-4 rounded-md border border-emerald-100 bg-white p-4"
      onSubmit={onSubmit}
    >
      <div>
        <p className="text-sm font-medium text-zinc-800">New {entityDisplayName} Record</p>
        <p className="text-xs text-zinc-500">
          Fill fields and create a runtime row in {appLogicalName}.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {formFields.map((field) => {
          const fieldId = `field_${field.logical_name}`;
          const value = formValues[field.logical_name];

          if (field.field_type === "boolean") {
            return (
              <div className="space-y-2" key={field.logical_name}>
                <Label htmlFor={fieldId}>{field.display_name}</Label>
                <label className="inline-flex items-center gap-2 text-sm text-zinc-700">
                  <Checkbox
                    id={fieldId}
                    checked={Boolean(value)}
                    onChange={(event) =>
                      onFieldValueChange(field.logical_name, event.target.checked)
                    }
                  />
                  {field.display_name}
                </label>
              </div>
            );
          }

          if (field.field_type === "json") {
            return (
              <div className="space-y-2 md:col-span-2" key={field.logical_name}>
                <Label htmlFor={fieldId}>{field.display_name}</Label>
                <Textarea
                  id={fieldId}
                  className="font-mono text-xs"
                  value={String(value ?? "")}
                  onChange={(event) =>
                    onFieldValueChange(field.logical_name, event.target.value)
                  }
                  placeholder='{"value":"example"}'
                />
              </div>
            );
          }

          return (
            <div className="space-y-2" key={field.logical_name}>
              <Label htmlFor={fieldId}>{field.display_name}</Label>
              <Input
                id={fieldId}
                type={
                  field.field_type === "number"
                    ? "number"
                    : field.field_type === "date"
                      ? "date"
                      : field.field_type === "datetime"
                        ? "datetime-local"
                        : "text"
                }
                value={String(value ?? "")}
                onChange={(event) =>
                  onFieldValueChange(field.logical_name, event.target.value)
                }
                required={field.is_required}
              />
            </div>
          );
        })}
      </div>

      <Button disabled={!canCreate || isSaving} type="submit">
        {isSaving ? "Saving..." : "Create Record"}
      </Button>
    </form>
  );
}

type RuntimeRecordsTableProps = {
  capabilities: AppEntityCapabilitiesResponse;
  deletingRecordId: string | null;
  filteredRecords: RuntimeRecordResponse[];
  gridFields: PublishedSchemaResponse["fields"];
  onDeleteRecord: (recordId: string) => void;
  records: RuntimeRecordResponse[];
  viewMode: WorkerViewMode;
};

function RuntimeRecordsTable({
  capabilities,
  deletingRecordId,
  filteredRecords,
  gridFields,
  onDeleteRecord,
  records,
  viewMode,
}: RuntimeRecordsTableProps) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Record ID</TableHead>
          {viewMode === "grid"
            ? gridFields.map((field) => (
                <TableHead key={field.logical_name}>{field.display_name}</TableHead>
              ))
            : null}
          <TableHead>{viewMode === "grid" ? "Snapshot" : "Data"}</TableHead>
          <TableHead>Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {filteredRecords.length > 0 ? (
          filteredRecords.map((record) => (
            <TableRow key={record.record_id}>
              <TableCell className="font-mono text-xs">{record.record_id}</TableCell>
              {viewMode === "grid"
                ? gridFields.map((field) => (
                    <TableCell
                      className="max-w-[220px] truncate"
                      key={`${record.record_id}-${field.logical_name}`}
                      title={formatValue(record.data[field.logical_name])}
                    >
                      {formatValue(record.data[field.logical_name])}
                    </TableCell>
                  ))
                : null}
              <TableCell className="font-mono text-xs">
                {viewMode === "grid"
                  ? `${Object.keys(record.data).length} populated field(s)`
                  : JSON.stringify(record.data)}
              </TableCell>
              <TableCell>
                {capabilities.can_delete ? (
                  <Button
                    disabled={deletingRecordId === record.record_id}
                    variant="outline"
                    size="sm"
                    type="button"
                    onClick={() => onDeleteRecord(record.record_id)}
                  >
                    {deletingRecordId === record.record_id ? "Deleting..." : "Delete"}
                  </Button>
                ) : (
                  <span className="text-xs text-zinc-500">No delete access</span>
                )}
              </TableCell>
            </TableRow>
          ))
        ) : (
          <TableRow>
            <TableCell
              className="text-zinc-500"
              colSpan={viewMode === "grid" ? gridFields.length + 3 : 3}
            >
              {records.length > 0 ? "No records match this search." : "No records yet."}
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}
