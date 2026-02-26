import { useState } from "react";

import type { PermissionDraft } from "@/components/apps/app-studio/sections";
import type { EntityResponse, RoleResponse } from "@/lib/api";

type UsePermissionMatrixInput = {
  roles: RoleResponse[];
  entities: EntityResponse[];
};

export function usePermissionMatrix({
  roles,
  entities,
}: UsePermissionMatrixInput) {
  const [permissionDraft, setPermissionDraft] = useState<PermissionDraft>({
    roleName: roles.at(0)?.name ?? "",
    entityName: entities.at(0)?.logical_name ?? "",
    canRead: true,
    canCreate: false,
    canUpdate: false,
    canDelete: false,
  });

  return {
    permissionDraft,
    setPermissionDraft,
  };
}
