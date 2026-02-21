"use client";

import { type FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Checkbox,
  Input,
  Label,
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
  type AppEntityCapabilitiesResponse,
  type CreateRuntimeRecordRequest,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";

type WorkspaceEntityPanelProps = {
  appLogicalName: string;
  entityLogicalName: string;
  schema: PublishedSchemaResponse;
  capabilities: AppEntityCapabilitiesResponse;
  records: RuntimeRecordResponse[];
};

type WorkerViewMode = "grid" | "json";

function buildInitialValues(
  schema: PublishedSchemaResponse,
): Record<string, unknown> {
  const values: Record<string, unknown> = {};
  for (const field of schema.fields) {
    if (field.default_value !== null) {
      values[field.logical_name] = field.default_value;
      continue;
    }

    if (field.field_type === "boolean") {
      values[field.logical_name] = false;
      continue;
    }

    values[field.logical_name] = "";
  }

  return values;
}

export function WorkspaceEntityPanel({
  appLogicalName,
  entityLogicalName,
  schema,
  capabilities,
  records,
}: WorkspaceEntityPanelProps) {
  const router = useRouter();

  const [formValues, setFormValues] = useState<Record<string, unknown>>(() =>
    buildInitialValues(schema),
  );
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [deletingRecordId, setDeletingRecordId] = useState<string | null>(null);
  const [recordSearch, setRecordSearch] = useState("");
  const [viewMode, setViewMode] = useState<WorkerViewMode>("grid");
  const [showCreatePanel, setShowCreatePanel] = useState(
    capabilities.can_create,
  );

  const gridFields = useMemo(
    () =>
      schema.fields.filter((field) => field.field_type !== "json").slice(0, 5),
    [schema.fields],
  );

  const filteredRecords = useMemo(() => {
    const normalizedQuery = recordSearch.trim().toLowerCase();
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
  }, [recordSearch, records]);

  function setFieldValue(fieldLogicalName: string, value: unknown) {
    setFormValues((current) => ({
      ...current,
      [fieldLogicalName]: value,
    }));
  }

  function buildPayloadFromForm() {
    const payload: Record<string, unknown> = {};

    for (const field of schema.fields) {
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

    setErrorMessage(null);
    setStatusMessage(null);
    setIsSaving(true);

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
      setIsSaving(false);
    }
  }

  async function handleDeleteRecord(recordId: string) {
    if (!capabilities.can_delete) {
      setErrorMessage(
        "You do not have delete permission for this entity in this app.",
      );
      return;
    }

    setDeletingRecordId(recordId);
    setErrorMessage(null);
    setStatusMessage(null);

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
      setDeletingRecordId(null);
    }
  }

  function formatValue(value: unknown): string {
    if (value === null || value === undefined || value === "") {
      return "-";
    }

    if (
      typeof value === "string" ||
      typeof value === "number" ||
      typeof value === "boolean"
    ) {
      return String(value);
    }

    return JSON.stringify(value);
  }

  return (
    <div className="space-y-6">
      <section className="space-y-3 rounded-md border border-emerald-100 bg-white p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="success">Schema v{schema.version}</StatusBadge>
            <StatusBadge tone={capabilities.can_create ? "success" : "warning"}>
              Create {capabilities.can_create ? "Enabled" : "Disabled"}
            </StatusBadge>
            <StatusBadge tone={capabilities.can_delete ? "warning" : "neutral"}>
              Delete {capabilities.can_delete ? "Enabled" : "Disabled"}
            </StatusBadge>
            <StatusBadge tone="neutral">
              Records {filteredRecords.length}
            </StatusBadge>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <Button
              type="button"
              variant={showCreatePanel ? "default" : "outline"}
              onClick={() => setShowCreatePanel((current) => !current)}
            >
              {showCreatePanel ? "Hide New Form" : "New Record"}
            </Button>
            <Button
              type="button"
              variant={viewMode === "grid" ? "default" : "outline"}
              onClick={() => setViewMode("grid")}
            >
              Grid View
            </Button>
            <Button
              type="button"
              variant={viewMode === "json" ? "default" : "outline"}
              onClick={() => setViewMode("json")}
            >
              JSON View
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setErrorMessage(null);
                setStatusMessage(null);
                router.refresh();
              }}
            >
              Refresh
            </Button>
          </div>
        </div>

        <div className="grid gap-3 md:grid-cols-[1fr_auto]">
          <Input
            value={recordSearch}
            onChange={(event) => setRecordSearch(event.target.value)}
            placeholder="Search by record id or any field value"
          />
          <p className="text-xs text-zinc-500 md:self-center">
            {filteredRecords.length} visible row(s)
          </p>
        </div>
      </section>

      {showCreatePanel ? (
        <form
          className="space-y-4 rounded-md border border-emerald-100 bg-white p-4"
          onSubmit={handleCreateRecord}
        >
          <div>
            <p className="text-sm font-medium text-zinc-800">
              New {schema.entity_display_name} Record
            </p>
            <p className="text-xs text-zinc-500">
              Fill fields and create a runtime row in {appLogicalName}.
            </p>
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            {schema.fields.map((field) => {
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
                          setFieldValue(
                            field.logical_name,
                            event.target.checked,
                          )
                        }
                      />
                      {field.display_name}
                    </label>
                  </div>
                );
              }

              if (field.field_type === "json") {
                return (
                  <div
                    className="space-y-2 md:col-span-2"
                    key={field.logical_name}
                  >
                    <Label htmlFor={fieldId}>{field.display_name}</Label>
                    <Textarea
                      id={fieldId}
                      className="font-mono text-xs"
                      value={String(value ?? "")}
                      onChange={(event) =>
                        setFieldValue(field.logical_name, event.target.value)
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
                      setFieldValue(field.logical_name, event.target.value)
                    }
                    required={field.is_required}
                  />
                </div>
              );
            })}
          </div>

          <Button disabled={!capabilities.can_create || isSaving} type="submit">
            {isSaving ? "Saving..." : "Create Record"}
          </Button>
        </form>
      ) : null}

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Record ID</TableHead>
            {viewMode === "grid"
              ? gridFields.map((field) => (
                  <TableHead key={field.logical_name}>
                    {field.display_name}
                  </TableHead>
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
                <TableCell className="font-mono text-xs">
                  {record.record_id}
                </TableCell>
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
                      onClick={() => handleDeleteRecord(record.record_id)}
                    >
                      {deletingRecordId === record.record_id
                        ? "Deleting..."
                        : "Delete"}
                    </Button>
                  ) : (
                    <span className="text-xs text-zinc-500">
                      No delete access
                    </span>
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
                {records.length > 0
                  ? "No records match this search."
                  : "No records yet."}
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>

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
