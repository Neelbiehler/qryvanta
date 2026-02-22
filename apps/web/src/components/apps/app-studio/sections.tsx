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
} from "@qryvanta/ui";

import {
  type AppEntityBindingResponse,
  type AppResponse,
  type AppRoleEntityPermissionResponse,
  type EntityResponse,
  type RoleResponse,
} from "@/lib/api";

export type AppEntityViewMode = "grid" | "json";
export type AppStudioSection = "apps" | "navigation" | "permissions";

export type NewAppDraft = {
  logicalName: string;
  displayName: string;
  description: string;
};

export type BindingDraft = {
  entityToBind: string;
  navigationLabel: string;
  navigationOrder: number;
  formFieldLogicalNames: string;
  listFieldLogicalNames: string;
  defaultViewMode: AppEntityViewMode;
};

export type PermissionDraft = {
  roleName: string;
  entityName: string;
  canRead: boolean;
  canCreate: boolean;
  canUpdate: boolean;
  canDelete: boolean;
};

type AppStudioOverviewProps = {
  activeSection: AppStudioSection;
  appsCount: number;
  canOpenNavigation: boolean;
  canOpenPermissions: boolean;
  entitiesCount: number;
  hasStudioData: boolean;
  onSectionChange: (section: AppStudioSection) => void;
  rolesCount: number;
  selectedAppDisplayName: string;
};

export function AppStudioOverview({
  activeSection,
  appsCount,
  canOpenNavigation,
  canOpenPermissions,
  entitiesCount,
  hasStudioData,
  onSectionChange,
  rolesCount,
  selectedAppDisplayName,
}: AppStudioOverviewProps) {
  return (
    <>
      {!hasStudioData ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          Create at least one app, one entity, and one role before configuring
          app access.
        </p>
      ) : null}

      <div className="flex flex-wrap items-center gap-2 rounded-md border border-emerald-100 bg-white/90 p-3">
        <StatusBadge tone="neutral">Apps {appsCount}</StatusBadge>
        <StatusBadge tone="neutral">Entities {entitiesCount}</StatusBadge>
        <StatusBadge tone="neutral">Roles {rolesCount}</StatusBadge>
        <StatusBadge tone="success">Active {selectedAppDisplayName}</StatusBadge>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant={activeSection === "apps" ? "default" : "outline"}
          onClick={() => onSectionChange("apps")}
        >
          App Catalog
        </Button>
        <Button
          type="button"
          variant={activeSection === "navigation" ? "default" : "outline"}
          onClick={() => onSectionChange("navigation")}
          disabled={!canOpenNavigation}
        >
          Navigation Binding
        </Button>
        <Button
          type="button"
          variant={activeSection === "permissions" ? "default" : "outline"}
          onClick={() => onSectionChange("permissions")}
          disabled={!canOpenPermissions}
        >
          Role Permissions
        </Button>
      </div>
    </>
  );
}

type AppCatalogSectionProps = {
  apps: AppResponse[];
  isCreatingApp: boolean;
  newAppDraft: NewAppDraft;
  onCreateApp: (event: FormEvent<HTMLFormElement>) => void;
  onUpdateDraft: (next: NewAppDraft) => void;
};

export function AppCatalogSection({
  apps,
  isCreatingApp,
  newAppDraft,
  onCreateApp,
  onUpdateDraft,
}: AppCatalogSectionProps) {
  return (
    <div className="space-y-3 rounded-md border border-emerald-100 bg-white p-4">
      <form className="grid gap-3 md:grid-cols-3" onSubmit={onCreateApp}>
        <div className="space-y-2">
          <Label htmlFor="new_app_logical_name">App Logical Name</Label>
          <Input
            id="new_app_logical_name"
            value={newAppDraft.logicalName}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, logicalName: event.target.value })
            }
            placeholder="sales"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_display_name">App Display Name</Label>
          <Input
            id="new_app_display_name"
            value={newAppDraft.displayName}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, displayName: event.target.value })
            }
            placeholder="Sales App"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_description">Description</Label>
          <Input
            id="new_app_description"
            value={newAppDraft.description}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, description: event.target.value })
            }
            placeholder="Lead and account workflows"
          />
        </div>
        <div className="md:col-span-3">
          <Button disabled={isCreatingApp} type="submit">
            {isCreatingApp ? "Creating..." : "Create App"}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>App</TableHead>
            <TableHead>Description</TableHead>
            <TableHead>Logical Name</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {apps.length > 0 ? (
            apps.map((app) => (
              <TableRow key={app.logical_name}>
                <TableCell>{app.display_name}</TableCell>
                <TableCell>{app.description ?? "-"}</TableCell>
                <TableCell className="font-mono text-xs">
                  {app.logical_name}
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={3} className="text-zinc-500">
                No apps yet.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

type NavigationBindingSectionProps = {
  apps: AppResponse[];
  bindings: AppEntityBindingResponse[];
  entities: EntityResponse[];
  isBindingEntity: boolean;
  isLoadingAppData: boolean;
  onBindEntity: (event: FormEvent<HTMLFormElement>) => void;
  onChangeSelectedApp: (appLogicalName: string) => void;
  onUpdateBindingDraft: (next: BindingDraft) => void;
  selectedApp: string;
  selectedAppDisplayName: string;
  bindingDraft: BindingDraft;
};

export function NavigationBindingSection({
  apps,
  bindings,
  entities,
  isBindingEntity,
  isLoadingAppData,
  onBindEntity,
  onChangeSelectedApp,
  onUpdateBindingDraft,
  selectedApp,
  selectedAppDisplayName,
  bindingDraft,
}: NavigationBindingSectionProps) {
  return (
    <div className="space-y-3 rounded-md border border-emerald-100 bg-white p-4">
      <div className="space-y-2">
        <Label htmlFor="studio_app_selector">Active App</Label>
        <Select
          id="studio_app_selector"
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

      <form className="grid gap-3 md:grid-cols-2" onSubmit={onBindEntity}>
        <div className="space-y-2">
          <Label htmlFor="bind_entity_name">Entity</Label>
          <Select
            id="bind_entity_name"
            value={bindingDraft.entityToBind}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                entityToBind: event.target.value,
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

        <div className="space-y-2">
          <Label htmlFor="bind_navigation_label">Navigation Label</Label>
          <Input
            id="bind_navigation_label"
            value={bindingDraft.navigationLabel}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                navigationLabel: event.target.value,
              })
            }
            placeholder="Accounts"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_navigation_order">Navigation Order</Label>
          <Input
            id="bind_navigation_order"
            value={String(bindingDraft.navigationOrder)}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                navigationOrder: Number.parseInt(event.target.value || "0", 10),
              })
            }
            type="number"
            min={0}
          />
        </div>

        <div className="space-y-2 md:col-span-2">
          <Label htmlFor="bind_form_fields">Form Fields (comma separated)</Label>
          <Input
            id="bind_form_fields"
            value={bindingDraft.formFieldLogicalNames}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                formFieldLogicalNames: event.target.value,
              })
            }
            placeholder="name, email, owner"
          />
        </div>

        <div className="space-y-2 md:col-span-2">
          <Label htmlFor="bind_list_fields">List Fields (comma separated)</Label>
          <Input
            id="bind_list_fields"
            value={bindingDraft.listFieldLogicalNames}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                listFieldLogicalNames: event.target.value,
              })
            }
            placeholder="name, status, updated_at"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_default_view_mode">Default View Mode</Label>
          <Select
            id="bind_default_view_mode"
            value={bindingDraft.defaultViewMode}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                defaultViewMode: event.target.value as AppEntityViewMode,
              })
            }
          >
            <option value="grid">Grid</option>
            <option value="json">JSON</option>
          </Select>
        </div>

        <div className="md:col-span-3">
          <Button
            disabled={isBindingEntity || isLoadingAppData}
            type="submit"
            variant="outline"
          >
            {isBindingEntity ? "Saving..." : `Bind Entity to ${selectedAppDisplayName}`}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Bound Entity</TableHead>
            <TableHead>Label</TableHead>
            <TableHead>Order</TableHead>
            <TableHead>Default View</TableHead>
            <TableHead>Presentation</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {bindings.length > 0 ? (
            bindings.map((binding) => (
              <TableRow
                key={`${binding.app_logical_name}.${binding.entity_logical_name}`}
              >
                <TableCell className="font-mono text-xs">
                  {binding.entity_logical_name}
                </TableCell>
                <TableCell>
                  {binding.navigation_label ?? binding.entity_logical_name}
                </TableCell>
                <TableCell>{binding.navigation_order}</TableCell>
                <TableCell className="uppercase">{binding.default_view_mode}</TableCell>
                <TableCell className="text-xs text-zinc-600">
                  form {binding.form_field_logical_names.length || "auto"} / list{" "}
                  {binding.list_field_logical_names.length || "auto"}
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={5} className="text-zinc-500">
                No entity bindings for this app.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

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
    <div className="space-y-3 rounded-md border border-emerald-100 bg-white p-4">
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
          <div className="mr-3 inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_read"
              checked={permissionDraft.canRead}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canRead: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_read">Read</Label>
          </div>
          <div className="mr-3 inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_create"
              checked={permissionDraft.canCreate}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canCreate: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_create">Create</Label>
          </div>
          <div className="mr-3 inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_update"
              checked={permissionDraft.canUpdate}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canUpdate: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_update">Update</Label>
          </div>
          <div className="inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_delete"
              checked={permissionDraft.canDelete}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canDelete: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_delete">Delete</Label>
          </div>
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
