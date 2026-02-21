import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
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
  type TenantRegistrationModeResponse,
} from "@/lib/api";

export default async function RolesPage() {
  const cookieHeader = (await cookies()).toString();
  const [response, assignmentsResponse, registrationModeResponse] =
    await Promise.all([
      apiServerFetch("/api/security/roles", cookieHeader),
      apiServerFetch("/api/security/role-assignments", cookieHeader),
      apiServerFetch("/api/security/registration-mode", cookieHeader),
    ]);

  if (response.status === 401) {
    redirect("/login");
  }

  if (assignmentsResponse.status === 401) {
    redirect("/login");
  }

  if (registrationModeResponse.status === 401) {
    redirect("/login");
  }

  if (
    response.status === 403 ||
    assignmentsResponse.status === 403 ||
    registrationModeResponse.status === 403
  ) {
    return (
      <AccessDeniedCard
        section="Security"
        title="Roles"
        message="Your account is authenticated but does not have role management permissions."
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

  const roles = (await response.json()) as RoleResponse[];
  const assignments = (await assignmentsResponse.json()) as RoleAssignmentResponse[];
  const registrationMode =
    (await registrationModeResponse.json()) as TenantRegistrationModeResponse;

  return (
    <Card>
      <CardHeader>
        <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">Security</p>
        <CardTitle className="font-serif text-3xl">Roles</CardTitle>
      </CardHeader>

      <CardContent className="space-y-8">
        <RoleManagementPanel
          roles={roles}
          assignments={assignments}
          registrationMode={registrationMode.registration_mode}
        />

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
                  <TableCell className="font-medium">{role.name}</TableCell>
                  <TableCell>{role.is_system ? "System" : "Custom"}</TableCell>
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
                <TableRow key={`${assignment.subject}-${assignment.role_id}`}>
                  <TableCell>{assignment.subject}</TableCell>
                  <TableCell>{assignment.role_name}</TableCell>
                  <TableCell className="font-mono text-xs">{assignment.assigned_at}</TableCell>
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
      </CardContent>
    </Card>
  );
}
