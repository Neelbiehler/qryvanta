"use client";

import { type FormEvent, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Select,
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

export function EntityWorkbenchPanel({
  entityLogicalName,
  initialFields,
  initialPublishedSchema,
  initialRecords,
}: EntityWorkbenchPanelProps) {
  const router = useRouter();

  const [logicalName, setLogicalName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [fieldType, setFieldType] = useState<(typeof FIELD_TYPE_OPTIONS)[number]>("text");
  const [isRequired, setIsRequired] = useState(false);
  const [isUnique, setIsUnique] = useState(false);
  const [defaultValueText, setDefaultValueText] = useState("");
  const [relationTargetEntity, setRelationTargetEntity] = useState("");

  const [recordPayload, setRecordPayload] = useState("{}");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const [isSavingField, setIsSavingField] = useState(false);
  const [isPublishing, setIsPublishing] = useState(false);
  const [isCreatingRecord, setIsCreatingRecord] = useState(false);
  const [deletingRecordId, setDeletingRecordId] = useState<string | null>(null);

  function clearMessages() {
    setErrorMessage(null);
    setStatusMessage(null);
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

      const response = await apiFetch(`/api/entities/${entityLogicalName}/fields`, {
        method: "POST",
        body: JSON.stringify(payload),
      });

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

  async function handleCreateRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    clearMessages();
    setIsCreatingRecord(true);

    try {
      const parsed = JSON.parse(recordPayload) as unknown;
      if (parsed === null || Array.isArray(parsed) || typeof parsed !== "object") {
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
      router.refresh();
    } catch {
      setErrorMessage("Runtime record payload must be valid JSON.");
    } finally {
      setIsCreatingRecord(false);
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
      router.refresh();
    } catch {
      setErrorMessage("Unable to delete runtime record.");
    } finally {
      setDeletingRecordId(null);
    }
  }

  return (
    <div className="space-y-8">
      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <p className="text-sm font-medium text-zinc-800">Draft Fields</p>
          <Button disabled={isPublishing} onClick={handlePublish} type="button" variant="outline">
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
              onChange={(event) => setFieldType(event.target.value as (typeof FIELD_TYPE_OPTIONS)[number])}
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

      <section className="space-y-3">
        <div>
          <p className="text-sm font-medium text-zinc-800">Runtime Records</p>
          <p className="text-xs text-zinc-500">
            {initialPublishedSchema
              ? `Using published schema version ${initialPublishedSchema.version}.`
              : "Publish this entity before creating runtime records."}
          </p>
        </div>

        <form className="space-y-3 rounded-md border border-emerald-100 bg-white p-4" onSubmit={handleCreateRecord}>
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

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Record ID</TableHead>
              <TableHead>Data</TableHead>
              <TableHead>Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {initialRecords.length > 0 ? (
              initialRecords.map((record) => (
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
                  No runtime records yet.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </section>

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
