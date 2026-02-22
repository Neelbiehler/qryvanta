"use client";

import { Notice } from "@qryvanta/ui";

import {
  FieldPermissionsSection,
  RegistrationModeSection,
  RoleManagementTabs,
  RolesAndAssignmentsSection,
  TemporaryAccessSection,
} from "@/components/security/role-management/sections";
import { useRoleManagementPanel } from "@/components/security/role-management/use-role-management-panel";
import type {
  RoleAssignmentResponse,
  RoleResponse,
  RuntimeFieldPermissionResponse,
  TemporaryAccessGrantResponse,
} from "@/lib/api";

type RoleManagementPanelProps = {
  roles: RoleResponse[];
  assignments: RoleAssignmentResponse[];
  registrationMode: string;
  runtimeFieldPermissions: RuntimeFieldPermissionResponse[];
  temporaryAccessGrants: TemporaryAccessGrantResponse[];
};

export function RoleManagementPanel({
  roles,
  assignments,
  registrationMode,
  runtimeFieldPermissions,
  temporaryAccessGrants,
}: RoleManagementPanelProps) {
  const panel = useRoleManagementPanel({
    roles,
    assignments,
    registrationMode,
    runtimeFieldPermissions,
    temporaryAccessGrants,
  });

  return (
    <div className="grid gap-8 md:grid-cols-2">
      <RoleManagementTabs
        activeSection={panel.activeSection}
        onSectionChange={panel.setActiveSection}
      />

      {panel.activeSection === "roles" ? (
        <RolesAndAssignmentsSection
          assignRoleName={panel.assignRoleName}
          assignments={panel.assignments}
          assignSubject={panel.assignSubject}
          handleAssignSubmit={panel.handleAssignSubmit}
          handleRoleSubmit={panel.handleRoleSubmit}
          handleUnassign={panel.handleUnassign}
          isAssigning={panel.isAssigning}
          isSubmittingRole={panel.isSubmittingRole}
          roleName={panel.roleName}
          roleNames={panel.roleNames}
          selectedPermissions={panel.selectedPermissions}
          setAssignRoleName={panel.setAssignRoleName}
          setAssignSubject={panel.setAssignSubject}
          setRoleName={panel.setRoleName}
          togglePermission={panel.togglePermission}
        />
      ) : null}

      {panel.activeSection === "registration" ? (
        <RegistrationModeSection
          handleRegistrationModeSubmit={panel.handleRegistrationModeSubmit}
          isUpdatingRegistrationMode={panel.isUpdatingRegistrationMode}
          registrationMode={panel.registrationMode}
        />
      ) : null}

      {panel.activeSection === "fieldPermissions" ? (
        <FieldPermissionsSection
          addFieldPermissionDraft={panel.addFieldPermissionDraft}
          fieldPermissionCanRead={panel.fieldPermissionCanRead}
          fieldPermissionCanWrite={panel.fieldPermissionCanWrite}
          fieldPermissionEntity={panel.fieldPermissionEntity}
          fieldPermissionFieldName={panel.fieldPermissionFieldName}
          fieldPermissionSubject={panel.fieldPermissionSubject}
          fieldPermissionsDraft={panel.fieldPermissionsDraft}
          handleSaveFieldPermissions={panel.handleSaveFieldPermissions}
          isSavingFieldPermissions={panel.isSavingFieldPermissions}
          removeFieldPermissionDraft={panel.removeFieldPermissionDraft}
          runtimeFieldPermissions={panel.runtimeFieldPermissions}
          setFieldPermissionCanRead={panel.setFieldPermissionCanRead}
          setFieldPermissionCanWrite={panel.setFieldPermissionCanWrite}
          setFieldPermissionEntity={panel.setFieldPermissionEntity}
          setFieldPermissionFieldName={panel.setFieldPermissionFieldName}
          setFieldPermissionSubject={panel.setFieldPermissionSubject}
        />
      ) : null}

      {panel.activeSection === "temporaryAccess" ? (
        <TemporaryAccessSection
          cancelRevokeTemporaryGrant={panel.cancelRevokeTemporaryGrant}
          grantToRevokeId={panel.grantToRevokeId}
          handleCreateTemporaryGrant={panel.handleCreateTemporaryGrant}
          handleRevokeTemporaryGrant={panel.handleRevokeTemporaryGrant}
          isCreatingTemporaryGrant={panel.isCreatingTemporaryGrant}
          isRevokingTemporaryGrant={panel.isRevokingTemporaryGrant}
          revokeReason={panel.revokeReason}
          setRevokeReason={panel.setRevokeReason}
          setTemporaryDurationMinutes={panel.setTemporaryDurationMinutes}
          setTemporaryReason={panel.setTemporaryReason}
          setTemporarySubject={panel.setTemporarySubject}
          startRevokeTemporaryGrant={panel.startRevokeTemporaryGrant}
          temporaryAccessGrants={panel.temporaryAccessGrants}
          temporaryDurationMinutes={panel.temporaryDurationMinutes}
          temporaryPermissions={panel.temporaryPermissions}
          temporaryReason={panel.temporaryReason}
          temporarySubject={panel.temporarySubject}
          toggleTemporaryPermission={panel.toggleTemporaryPermission}
        />
      ) : null}

      {panel.errorMessage ? (
        <Notice tone="error" className="md:col-span-2">
          {panel.errorMessage}
        </Notice>
      ) : null}
    </div>
  );
}
