import type { FormEvent } from "react";

import {
  Button,
  Checkbox,
  Label,
  Select,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@qryvanta/ui";

import type {
  AppResponse,
  AppRoleEntityPermissionResponse,
  EntityResponse,
  RoleResponse,
} from "@/lib/api";
import type { PermissionDraft } from "@/components/apps/app-studio/sections/types";

type RolePermissionsSectionProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  isLoadingAppData: boolean;
  isSavingPermission: boolean;
  onChangeSelectedApp: (appLogicalName: string) => void;
  onSavePermission: (event: FormEvent<HTMLFormElement>) => void;
  onUpdatePermissionDraft: (next: PermissionDraft) => void;
  permissions: AppRoleEntityPermissionResponse[];
  roles: RoleResponse[];
  selectedApp: string;
  permissionDraft: PermissionDraft;
};

export function RolePermissionsSection({
  apps,
  entities,
  isLoadingAppData,
  isSavingPermission,
  onChangeSelectedApp,
  onSavePermission,
  onUpdatePermissionDraft,
  permissions,
  roles,
  selectedApp,
  permissionDraft,
}: RolePermissionsSectionProps) {
  return (
    <div className="space-y-3 rounded-md border border-zinc-200 bg-white p-4">
      <div>
        <p className="text-sm font-semibold text-zinc-900">Role Matrix</p>
        <p className="text-xs text-zinc-600">
          Configure per-role CRUD permissions for each app entity.
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="studio_permissions_app_selector">Active App</Label>
        <Select
          id="studio_permissions_app_selector"
          value={selectedApp}
          onChange={(event) => onChangeSelectedApp(event.target.value)}
        >
          {apps.map((app) => (
            <option key={app.logical_name} value={app.logical_name}>
              {app.display_name} ({app.logical_name})
            </option>
          ))}
        </Select>
      </div>

      <form className="grid gap-3 md:grid-cols-4" onSubmit={onSavePermission}>
        <div className="space-y-2">
          <Label htmlFor="permission_role_name">Role</Label>
          <Select
            id="permission_role_name"
            value={permissionDraft.roleName}
            onChange={(event) =>
              onUpdatePermissionDraft({
                ...permissionDraft,
                roleName: event.target.value,
              })
            }
          >
            {roles.map((role) => (
              <option key={role.role_id} value={role.name}>
                {role.name}
              </option>
            ))}
          </Select>
        </div>

        <div className="space-y-2">
          <Label htmlFor="permission_entity_name">Entity</Label>
          <Select
            id="permission_entity_name"
            value={permissionDraft.entityName}
            onChange={(event) =>
              onUpdatePermissionDraft({
                ...permissionDraft,
                entityName: event.target.value,
              })
            }
          >
            {entities.map((entity) => (
              <option key={entity.logical_name} value={entity.logical_name}>
                {entity.display_name} ({entity.logical_name})
              </option>
            ))}
          </Select>
        </div>

        <div className="space-y-1 pt-6 md:col-span-2">
          <PermissionToggle
            id="permission_can_read"
            label="Read"
            checked={permissionDraft.canRead}
            onChange={(checked) =>
              onUpdatePermissionDraft({ ...permissionDraft, canRead: checked })
            }
          />
          <PermissionToggle
            id="permission_can_create"
            label="Create"
            checked={permissionDraft.canCreate}
            onChange={(checked) =>
              onUpdatePermissionDraft({ ...permissionDraft, canCreate: checked })
            }
          />
          <PermissionToggle
            id="permission_can_update"
            label="Update"
            checked={permissionDraft.canUpdate}
            onChange={(checked) =>
              onUpdatePermissionDraft({ ...permissionDraft, canUpdate: checked })
            }
          />
          <PermissionToggle
            id="permission_can_delete"
            label="Delete"
            checked={permissionDraft.canDelete}
            onChange={(checked) =>
              onUpdatePermissionDraft({ ...permissionDraft, canDelete: checked })
            }
          />
        </div>

        <div className="md:col-span-4">
          <Button
            disabled={isSavingPermission || isLoadingAppData}
            type="submit"
            variant="outline"
          >
            {isSavingPermission ? "Saving..." : "Save Role Entity Permissions"}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Role</TableHead>
            <TableHead>Entity</TableHead>
            <TableHead>Permissions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {permissions.length > 0 ? (
            permissions.map((permission) => (
              <TableRow
                key={`${permission.app_logical_name}.${permission.role_name}.${permission.entity_logical_name}`}
              >
                <TableCell>{permission.role_name}</TableCell>
                <TableCell className="font-mono text-xs">
                  {permission.entity_logical_name}
                </TableCell>
                <TableCell className="font-mono text-xs">
                  {[
                    permission.can_read ? "read" : null,
                    permission.can_create ? "create" : null,
                    permission.can_update ? "update" : null,
                    permission.can_delete ? "delete" : null,
                  ]
                    .filter((value): value is string => value !== null)
                    .join(", ") || "none"}
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={3} className="text-zinc-500">
                No role entity permissions configured for this app.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

type PermissionToggleProps = {
  id: string;
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
};

function PermissionToggle({ id, label, checked, onChange }: PermissionToggleProps) {
  return (
    <div className="mr-3 inline-flex items-center gap-1 text-sm">
      <Checkbox id={id} checked={checked} onChange={(event) => onChange(event.target.checked)} />
      <Label htmlFor={id}>{label}</Label>
    </div>
  );
}
