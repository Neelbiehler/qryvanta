export const PERMISSION_OPTIONS = [
  "metadata.entity.read",
  "metadata.entity.create",
  "metadata.field.read",
  "metadata.field.write",
  "runtime.record.read",
  "runtime.record.read.own",
  "runtime.record.write",
  "runtime.record.write.own",
  "security.audit.read",
  "security.role.manage",
  "security.invite.send",
] as const;

export type EditableFieldPermission = {
  fieldLogicalName: string;
  canRead: boolean;
  canWrite: boolean;
};

export type SecurityAdminSection =
  | "roles"
  | "registration"
  | "fieldPermissions"
  | "temporaryAccess";
