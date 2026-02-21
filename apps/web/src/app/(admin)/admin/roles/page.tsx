import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  PageHeader,
  StatusBadge,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@qryvanta/ui";

import { RoleManagementPanel } from "@/components/security/role-management-panel";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import {
  apiServerFetch,
  type RoleAssignmentResponse,
  type RoleResponse,
  type RuntimeFieldPermissionResponse,
  type TemporaryAccessGrantResponse,
  type TenantRegistrationModeResponse,
} from "@/lib/api";

export default async function AdminRolesPage() {
  const cookieHeader = (await cookies()).toString();
  const [
    response,
    assignmentsResponse,
    registrationModeResponse,
    runtimeFieldPermissionsResponse,
    temporaryAccessGrantsResponse,
  ] = await Promise.all([
    apiServerFetch("/api/security/roles", cookieHeader),
    apiServerFetch("/api/security/role-assignments", cookieHeader),
    apiServerFetch("/api/security/registration-mode", cookieHeader),
    apiServerFetch("/api/security/runtime-field-permissions", cookieHeader),
    apiServerFetch(
      "/api/security/temporary-access-grants?limit=50",
      cookieHeader,
    ),
  ]);

  if (response.status === 401) {
    redirect("/login");
  }

  if (
    response.status === 403 ||
    assignmentsResponse.status === 403 ||
    registrationModeResponse.status === 403 ||
    runtimeFieldPermissionsResponse.status === 403 ||
    temporaryAccessGrantsResponse.status === 403
  ) {
    return (
      <AccessDeniedCard
        section="Admin Center"
        title="Roles"
        message="Your account does not have role management permissions."
      />
    );
  }

  if (!response.ok) {
    throw new Error("Failed to load roles");
  }

  if (!assignmentsResponse.ok) {
    throw new Error("Failed to load role assignments");
  }

  if (!registrationModeResponse.ok) {
    throw new Error("Failed to load tenant registration mode");
  }

  if (!runtimeFieldPermissionsResponse.ok) {
    throw new Error("Failed to load runtime field permissions");
  }

  if (!temporaryAccessGrantsResponse.ok) {
    throw new Error("Failed to load temporary access grants");
  }

  const roles = (await response.json()) as RoleResponse[];
  const assignments =
    (await assignmentsResponse.json()) as RoleAssignmentResponse[];
  const registrationMode =
    (await registrationModeResponse.json()) as TenantRegistrationModeResponse;
  const runtimeFieldPermissions =
    (await runtimeFieldPermissionsResponse.json()) as RuntimeFieldPermissionResponse[];
  const temporaryAccessGrants =
    (await temporaryAccessGrantsResponse.json()) as TemporaryAccessGrantResponse[];

  return (
    <div className="space-y-4">
      <PageHeader
        eyebrow="Admin Center"
        title="Roles"
        description="Manage tenant access, assignments, and temporary elevation."
      />

      <div className="grid gap-4 xl:grid-cols-[300px_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Governance Snapshot</CardTitle>
            <CardDescription>Current authorization inventory.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <StatusBadge tone="neutral">Roles {roles.length}</StatusBadge>
            <StatusBadge tone="neutral">
              Assignments {assignments.length}
            </StatusBadge>
            <StatusBadge tone="warning">
              Temporary Grants {temporaryAccessGrants.length}
            </StatusBadge>
            <StatusBadge tone="neutral">
              Registration {registrationMode.registration_mode}
            </StatusBadge>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="space-y-8 pt-6">
            <RoleManagementPanel
              roles={roles}
              assignments={assignments}
              registrationMode={registrationMode.registration_mode}
              runtimeFieldPermissions={runtimeFieldPermissions}
              temporaryAccessGrants={temporaryAccessGrants}
            />

            <div className="space-y-3">
              <p className="text-sm font-medium text-zinc-900">Role Catalog</p>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Role Name</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Permissions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {roles.length > 0 ? (
                    roles.map((role) => (
                      <TableRow key={role.role_id}>
                        <TableCell className="font-medium">
                          {role.name}
                        </TableCell>
                        <TableCell>
                          {role.is_system ? "System" : "Custom"}
                        </TableCell>
                        <TableCell className="font-mono text-xs">
                          {role.permissions.join(", ") || "No permissions"}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell className="text-zinc-500" colSpan={3}>
                        No roles found.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>

            <div className="space-y-3">
              <p className="text-sm font-medium text-zinc-900">
                Assignment Ledger
              </p>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Subject</TableHead>
                    <TableHead>Role</TableHead>
                    <TableHead>Assigned At</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {assignments.length > 0 ? (
                    assignments.map((assignment) => (
                      <TableRow
                        key={`${assignment.subject}-${assignment.role_id}`}
                      >
                        <TableCell>{assignment.subject}</TableCell>
                        <TableCell>{assignment.role_name}</TableCell>
                        <TableCell className="font-mono text-xs">
                          {assignment.assigned_at}
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell className="text-zinc-500" colSpan={3}>
                        No assignments found.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
