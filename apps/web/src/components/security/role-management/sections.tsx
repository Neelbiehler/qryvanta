import { type FormEvent } from "react";

import { Button, Checkbox, Input, Label, Select } from "@qryvanta/ui";

import {
  PERMISSION_OPTIONS,
  type EditableFieldPermission,
  type SecurityAdminSection,
} from "@/components/security/role-management/constants";
import type {
  RoleAssignmentResponse,
  RuntimeFieldPermissionResponse,
  TemporaryAccessGrantResponse,
} from "@/lib/api";

type RoleManagementTabsProps = {
  activeSection: SecurityAdminSection;
  onSectionChange: (section: SecurityAdminSection) => void;
};

export function RoleManagementTabs({
  activeSection,
  onSectionChange,
}: RoleManagementTabsProps) {
  return (
    <div className="md:col-span-2 flex flex-wrap gap-2 rounded-md border border-emerald-100 bg-white/90 p-3">
      <Button
        type="button"
        variant={activeSection === "roles" ? "default" : "outline"}
        onClick={() => onSectionChange("roles")}
      >
        Roles & Assignments
      </Button>
      <Button
        type="button"
        variant={activeSection === "registration" ? "default" : "outline"}
        onClick={() => onSectionChange("registration")}
      >
        Registration Mode
      </Button>
      <Button
        type="button"
        variant={activeSection === "fieldPermissions" ? "default" : "outline"}
        onClick={() => onSectionChange("fieldPermissions")}
      >
        Field Permissions
      </Button>
      <Button
        type="button"
        variant={activeSection === "temporaryAccess" ? "default" : "outline"}
        onClick={() => onSectionChange("temporaryAccess")}
      >
        Temporary Access
      </Button>
    </div>
  );
}

type RolesAndAssignmentsSectionProps = {
  assignRoleName: string;
  assignments: RoleAssignmentResponse[];
  assignSubject: string;
  handleAssignSubmit: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  handleRoleSubmit: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  handleUnassign: (subject: string, roleName: string) => Promise<void>;
  isAssigning: boolean;
  isSubmittingRole: boolean;
  roleName: string;
  roleNames: string[];
  selectedPermissions: string[];
  setAssignRoleName: (value: string) => void;
  setAssignSubject: (value: string) => void;
  setRoleName: (value: string) => void;
  togglePermission: (permission: string) => void;
};

export function RolesAndAssignmentsSection({
  assignRoleName,
  assignments,
  assignSubject,
  handleAssignSubmit,
  handleRoleSubmit,
  handleUnassign,
  isAssigning,
  isSubmittingRole,
  roleName,
  roleNames,
  selectedPermissions,
  setAssignRoleName,
  setAssignSubject,
  setRoleName,
  togglePermission,
}: RolesAndAssignmentsSectionProps) {
  return (
    <>
      <form className="space-y-4" onSubmit={handleRoleSubmit}>
        <div className="space-y-2">
          <Label htmlFor="role_name">Role Name</Label>
          <Input
            id="role_name"
            value={roleName}
            onChange={(event) => setRoleName(event.target.value)}
            placeholder="operations_editor"
            required
          />
        </div>

        <fieldset className="space-y-2">
          <legend className="text-sm font-medium text-zinc-800">Permissions</legend>
          <div className="space-y-2 rounded-md border border-emerald-100 bg-white p-3">
            {PERMISSION_OPTIONS.map((permission) => (
              <label
                key={permission}
                className="flex items-center gap-2 text-sm text-zinc-700"
              >
                <Checkbox
                  checked={selectedPermissions.includes(permission)}
                  onChange={() => togglePermission(permission)}
                />
                <span className="font-mono text-xs">{permission}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <Button disabled={isSubmittingRole} type="submit">
          {isSubmittingRole ? "Creating..." : "Create Role"}
        </Button>
      </form>

      <form className="space-y-4" onSubmit={handleAssignSubmit}>
        <div className="space-y-2">
          <Label htmlFor="assign_subject">Subject</Label>
          <Input
            id="assign_subject"
            value={assignSubject}
            onChange={(event) => setAssignSubject(event.target.value)}
            placeholder="alice"
            required
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="assign_role_name">Role Name</Label>
          <Input
            id="assign_role_name"
            value={assignRoleName}
            onChange={(event) => setAssignRoleName(event.target.value)}
            list="role_names"
            placeholder="tenant_owner"
            required
          />
          <datalist id="role_names">
            {roleNames.map((name) => (
              <option key={name} value={name} />
            ))}
          </datalist>
        </div>

        <Button disabled={isAssigning} type="submit" variant="outline">
          {isAssigning ? "Assigning..." : "Assign Role"}
        </Button>
      </form>

      <div className="space-y-3 md:col-span-2">
        <p className="text-sm font-medium text-zinc-800">Quick Unassign</p>
        <div className="grid gap-2">
          {assignments.slice(0, 8).map((assignment) => (
            <div
              key={`${assignment.subject}-${assignment.role_id}`}
              className="flex items-center justify-between rounded-md border border-emerald-100 bg-white px-3 py-2"
            >
              <div>
                <p className="text-sm text-zinc-900">{assignment.subject}</p>
                <p className="font-mono text-xs text-zinc-500">{assignment.role_name}</p>
              </div>
              <Button
                type="button"
                variant="outline"
                onClick={() => handleUnassign(assignment.subject, assignment.role_name)}
              >
                Remove
              </Button>
            </div>
          ))}

          {assignments.length === 0 ? (
            <p className="text-sm text-zinc-500">No assignments available.</p>
          ) : null}
        </div>
      </div>
    </>
  );
}

type RegistrationModeSectionProps = {
  handleRegistrationModeSubmit: (
    event: FormEvent<HTMLFormElement>,
  ) => Promise<void>;
  isUpdatingRegistrationMode: boolean;
  registrationMode: string;
};

export function RegistrationModeSection({
  handleRegistrationModeSubmit,
  isUpdatingRegistrationMode,
  registrationMode,
}: RegistrationModeSectionProps) {
  return (
    <form className="space-y-4 md:col-span-2" onSubmit={handleRegistrationModeSubmit}>
      <div className="space-y-2">
        <Label htmlFor="tenant_registration_mode">Tenant Registration Mode</Label>
        <p className="text-sm text-zinc-600">
          Control whether users can self-register or only join by invite.
        </p>
        <Select
          id="tenant_registration_mode"
          defaultValue={registrationMode}
          name="tenant_registration_mode"
        >
          <option value="invite_only">Invite only</option>
          <option value="open">Open registration</option>
        </Select>
      </div>

      <Button
        disabled={isUpdatingRegistrationMode}
        type="submit"
        variant="outline"
      >
        {isUpdatingRegistrationMode ? "Saving..." : "Save Registration Mode"}
      </Button>
    </form>
  );
}

type FieldPermissionsSectionProps = {
  addFieldPermissionDraft: () => void;
  fieldPermissionCanRead: boolean;
  fieldPermissionCanWrite: boolean;
  fieldPermissionEntity: string;
  fieldPermissionFieldName: string;
  fieldPermissionSubject: string;
  fieldPermissionsDraft: EditableFieldPermission[];
  handleSaveFieldPermissions: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  isSavingFieldPermissions: boolean;
  removeFieldPermissionDraft: (fieldLogicalName: string) => void;
  runtimeFieldPermissions: RuntimeFieldPermissionResponse[];
  setFieldPermissionCanRead: (value: boolean) => void;
  setFieldPermissionCanWrite: (value: boolean) => void;
  setFieldPermissionEntity: (value: string) => void;
  setFieldPermissionFieldName: (value: string) => void;
  setFieldPermissionSubject: (value: string) => void;
};

export function FieldPermissionsSection({
  addFieldPermissionDraft,
  fieldPermissionCanRead,
  fieldPermissionCanWrite,
  fieldPermissionEntity,
  fieldPermissionFieldName,
  fieldPermissionSubject,
  fieldPermissionsDraft,
  handleSaveFieldPermissions,
  isSavingFieldPermissions,
  removeFieldPermissionDraft,
  runtimeFieldPermissions,
  setFieldPermissionCanRead,
  setFieldPermissionCanWrite,
  setFieldPermissionEntity,
  setFieldPermissionFieldName,
  setFieldPermissionSubject,
}: FieldPermissionsSectionProps) {
  return (
    <>
      <form
        className="space-y-4 rounded-md border border-emerald-100 bg-white p-4 md:col-span-2"
        onSubmit={handleSaveFieldPermissions}
      >
        <p className="text-sm font-medium text-zinc-900">Runtime Field Permissions</p>
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="field_permission_subject">Subject</Label>
            <Input
              id="field_permission_subject"
              value={fieldPermissionSubject}
              onChange={(event) => setFieldPermissionSubject(event.target.value)}
              placeholder="alice"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="field_permission_entity">Entity</Label>
            <Input
              id="field_permission_entity"
              value={fieldPermissionEntity}
              onChange={(event) => setFieldPermissionEntity(event.target.value)}
              placeholder="contact"
              required
            />
          </div>
        </div>

        <div className="grid gap-3 rounded-md border border-emerald-100 bg-emerald-50/40 p-3 md:grid-cols-4">
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="field_permission_field_name">Field</Label>
            <Input
              id="field_permission_field_name"
              value={fieldPermissionFieldName}
              onChange={(event) => setFieldPermissionFieldName(event.target.value)}
              placeholder="email"
            />
          </div>
          <div className="flex items-center gap-2 text-sm text-zinc-700 md:mt-7">
            <Checkbox
              id="field_permission_can_read"
              checked={fieldPermissionCanRead}
              onChange={() => setFieldPermissionCanRead(!fieldPermissionCanRead)}
            />
            <Label htmlFor="field_permission_can_read">Read</Label>
          </div>
          <div className="flex items-center gap-2 text-sm text-zinc-700 md:mt-7">
            <Checkbox
              id="field_permission_can_write"
              checked={fieldPermissionCanWrite}
              onChange={() => setFieldPermissionCanWrite(!fieldPermissionCanWrite)}
            />
            <Label htmlFor="field_permission_can_write">Write</Label>
          </div>
          <Button
            className="md:col-span-4"
            onClick={addFieldPermissionDraft}
            type="button"
            variant="outline"
          >
            Add Field Rule
          </Button>
        </div>

        <div className="space-y-2">
          {fieldPermissionsDraft.map((entry) => (
            <div
              key={entry.fieldLogicalName}
              className="flex items-center justify-between rounded-md border border-emerald-100 px-3 py-2"
            >
              <p className="font-mono text-xs text-zinc-700">
                {entry.fieldLogicalName} (read={String(entry.canRead)}, write=
                {String(entry.canWrite)})
              </p>
              <Button
                onClick={() => removeFieldPermissionDraft(entry.fieldLogicalName)}
                type="button"
                variant="outline"
              >
                Remove
              </Button>
            </div>
          ))}
          {fieldPermissionsDraft.length === 0 ? (
            <p className="text-sm text-zinc-500">No field rules staged.</p>
          ) : null}
        </div>

        <Button disabled={isSavingFieldPermissions} type="submit">
          {isSavingFieldPermissions ? "Saving..." : "Save Field Permissions"}
        </Button>
      </form>

      <div className="space-y-3 md:col-span-2">
        <p className="text-sm font-medium text-zinc-800">
          Runtime Field Permission Entries
        </p>
        <div className="grid gap-2">
          {runtimeFieldPermissions.slice(0, 20).map((entry) => (
            <div
              key={`${entry.subject}-${entry.entity_logical_name}-${entry.field_logical_name}`}
              className="rounded-md border border-emerald-100 bg-white px-3 py-2"
            >
              <p className="text-sm text-zinc-900">
                {entry.subject} / {entry.entity_logical_name}
              </p>
              <p className="font-mono text-xs text-zinc-600">
                {entry.field_logical_name} (read={String(entry.can_read)}, write=
                {String(entry.can_write)})
              </p>
            </div>
          ))}
          {runtimeFieldPermissions.length === 0 ? (
            <p className="text-sm text-zinc-500">No runtime field permissions found.</p>
          ) : null}
        </div>
      </div>
    </>
  );
}

type TemporaryAccessSectionProps = {
  cancelRevokeTemporaryGrant: () => void;
  grantToRevokeId: string | null;
  handleCreateTemporaryGrant: (event: FormEvent<HTMLFormElement>) => Promise<void>;
  handleRevokeTemporaryGrant: () => Promise<void>;
  isCreatingTemporaryGrant: boolean;
  isRevokingTemporaryGrant: boolean;
  revokeReason: string;
  setRevokeReason: (value: string) => void;
  setTemporaryDurationMinutes: (value: string) => void;
  setTemporaryReason: (value: string) => void;
  setTemporarySubject: (value: string) => void;
  startRevokeTemporaryGrant: (grantId: string) => void;
  temporaryAccessGrants: TemporaryAccessGrantResponse[];
  temporaryDurationMinutes: string;
  temporaryPermissions: string[];
  temporaryReason: string;
  temporarySubject: string;
  toggleTemporaryPermission: (permission: string) => void;
};

export function TemporaryAccessSection({
  cancelRevokeTemporaryGrant,
  grantToRevokeId,
  handleCreateTemporaryGrant,
  handleRevokeTemporaryGrant,
  isCreatingTemporaryGrant,
  isRevokingTemporaryGrant,
  revokeReason,
  setRevokeReason,
  setTemporaryDurationMinutes,
  setTemporaryReason,
  setTemporarySubject,
  startRevokeTemporaryGrant,
  temporaryAccessGrants,
  temporaryDurationMinutes,
  temporaryPermissions,
  temporaryReason,
  temporarySubject,
  toggleTemporaryPermission,
}: TemporaryAccessSectionProps) {
  return (
    <>
      <form
        className="space-y-4 rounded-md border border-emerald-100 bg-white p-4 md:col-span-2"
        onSubmit={handleCreateTemporaryGrant}
      >
        <p className="text-sm font-medium text-zinc-900">Temporary Access Grants</p>

        <div className="grid gap-3 md:grid-cols-3">
          <div className="space-y-2">
            <Label htmlFor="temporary_subject">Subject</Label>
            <Input
              id="temporary_subject"
              value={temporarySubject}
              onChange={(event) => setTemporarySubject(event.target.value)}
              placeholder="oncall-user"
              required
            />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="temporary_reason">Reason</Label>
            <Input
              id="temporary_reason"
              value={temporaryReason}
              onChange={(event) => setTemporaryReason(event.target.value)}
              placeholder="Incident triage"
              required
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="temporary_duration">Duration (minutes)</Label>
          <Input
            id="temporary_duration"
            value={temporaryDurationMinutes}
            onChange={(event) => setTemporaryDurationMinutes(event.target.value)}
            placeholder="60"
            type="number"
          />
        </div>

        <fieldset className="space-y-2">
          <legend className="text-sm font-medium text-zinc-800">Permissions</legend>
          <div className="grid gap-2 rounded-md border border-emerald-100 bg-emerald-50/40 p-3 md:grid-cols-2">
            {PERMISSION_OPTIONS.map((permission) => (
              <label
                key={`temporary-${permission}`}
                className="flex items-center gap-2 text-sm text-zinc-700"
              >
                <Checkbox
                  checked={temporaryPermissions.includes(permission)}
                  onChange={() => toggleTemporaryPermission(permission)}
                />
                <span className="font-mono text-xs">{permission}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <Button disabled={isCreatingTemporaryGrant} type="submit" variant="outline">
          {isCreatingTemporaryGrant ? "Creating..." : "Create Temporary Grant"}
        </Button>
      </form>

      <div className="space-y-3 md:col-span-2">
        <p className="text-sm font-medium text-zinc-800">Temporary Grants</p>
        <div className="grid gap-2">
          {temporaryAccessGrants.slice(0, 12).map((grant) => (
            <div
              key={grant.grant_id}
              className="flex items-center justify-between rounded-md border border-emerald-100 bg-white px-3 py-2"
            >
              <div className="space-y-1">
                <p className="text-sm text-zinc-900">{grant.subject}</p>
                <p className="font-mono text-xs text-zinc-500">
                  {grant.permissions.join(", ")} | expires {grant.expires_at}
                </p>
                <p className="text-xs text-zinc-600">{grant.reason}</p>
              </div>
              {grant.revoked_at ? (
                <p className="text-xs text-zinc-500">Revoked</p>
              ) : (
                <Button
                  onClick={() => startRevokeTemporaryGrant(grant.grant_id)}
                  type="button"
                  variant="outline"
                >
                  Revoke
                </Button>
              )}
            </div>
          ))}
          {temporaryAccessGrants.length === 0 ? (
            <p className="text-sm text-zinc-500">No temporary grants found.</p>
          ) : null}
        </div>

        {grantToRevokeId ? (
          <div className="space-y-3 rounded-md border border-amber-200 bg-amber-50 p-3">
            <p className="text-sm font-medium text-amber-900">
              Revoke temporary grant {grantToRevokeId}
            </p>
            <div className="space-y-2">
              <Label htmlFor="revoke_grant_reason">Revoke reason (optional)</Label>
              <Input
                id="revoke_grant_reason"
                value={revokeReason}
                onChange={(event) => setRevokeReason(event.target.value)}
                placeholder="Reason for revoking this grant"
              />
            </div>
            <div className="flex items-center gap-2">
              <Button
                onClick={handleRevokeTemporaryGrant}
                type="button"
                variant="outline"
                disabled={isRevokingTemporaryGrant}
              >
                {isRevokingTemporaryGrant ? "Revoking..." : "Confirm Revoke"}
              </Button>
              <Button onClick={cancelRevokeTemporaryGrant} type="button" variant="ghost">
                Cancel
              </Button>
            </div>
          </div>
        ) : null}
      </div>
    </>
  );
}
