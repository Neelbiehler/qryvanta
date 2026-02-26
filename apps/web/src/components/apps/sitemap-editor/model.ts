import type {
  AppSitemapAreaDto,
  AppSitemapResponse,
  AppSitemapSubAreaDto,
  EntityResponse,
} from "@/lib/api";
import type { DragPayload } from "@/components/apps/sitemap-editor/types";

export function parseDragPayload(rawPayload: string): DragPayload | null {
  try {
    return JSON.parse(rawPayload) as DragPayload;
  } catch {
    return null;
  }
}

export function reorderList<T>(items: T[], sourceIndex: number, targetIndex: number): T[] {
  if (sourceIndex === targetIndex) {
    return items;
  }
  const next = [...items];
  const [entry] = next.splice(sourceIndex, 1);
  next.splice(targetIndex, 0, entry);
  return next;
}

export function moveGroup(
  areas: AppSitemapAreaDto[],
  sourceAreaIndex: number,
  sourceGroupIndex: number,
  targetAreaIndex: number,
  targetGroupIndex: number,
): AppSitemapAreaDto[] {
  const sourceArea = areas[sourceAreaIndex];
  if (!sourceArea) {
    return areas;
  }
  const movingGroup = sourceArea.groups[sourceGroupIndex];
  if (!movingGroup) {
    return areas;
  }

  const withoutSource = areas.map((area, areaIndex) => {
    if (areaIndex !== sourceAreaIndex) {
      return area;
    }
    return {
      ...area,
      groups: area.groups.filter((_, groupIndex) => groupIndex !== sourceGroupIndex),
    };
  });

  const normalizedTargetIndex =
    sourceAreaIndex === targetAreaIndex && sourceGroupIndex < targetGroupIndex
      ? targetGroupIndex - 1
      : targetGroupIndex;

  return withoutSource.map((area, areaIndex) => {
    if (areaIndex !== targetAreaIndex) {
      return area;
    }
    const nextGroups = [...area.groups];
    nextGroups.splice(Math.max(0, normalizedTargetIndex), 0, movingGroup);
    return {
      ...area,
      groups: nextGroups,
    };
  });
}

export function moveSubArea(
  areas: AppSitemapAreaDto[],
  sourceAreaIndex: number,
  sourceGroupIndex: number,
  sourceSubAreaIndex: number,
  targetAreaIndex: number,
  targetGroupIndex: number,
  targetSubAreaIndex: number,
): AppSitemapAreaDto[] {
  const sourceGroup = areas[sourceAreaIndex]?.groups[sourceGroupIndex];
  if (!sourceGroup) {
    return areas;
  }
  const movingSubArea = sourceGroup.sub_areas[sourceSubAreaIndex];
  if (!movingSubArea) {
    return areas;
  }

  const withoutSource = areas.map((area, areaIndex) => {
    if (areaIndex !== sourceAreaIndex) {
      return area;
    }

    return {
      ...area,
      groups: area.groups.map((group, groupIndex) => {
        if (groupIndex !== sourceGroupIndex) {
          return group;
        }
        return {
          ...group,
          sub_areas: group.sub_areas.filter((_, subAreaIndex) => subAreaIndex !== sourceSubAreaIndex),
        };
      }),
    };
  });

  const normalizedTargetIndex =
    sourceAreaIndex === targetAreaIndex &&
    sourceGroupIndex === targetGroupIndex &&
    sourceSubAreaIndex < targetSubAreaIndex
      ? targetSubAreaIndex - 1
      : targetSubAreaIndex;

  return withoutSource.map((area, areaIndex) => {
    if (areaIndex !== targetAreaIndex) {
      return area;
    }

    return {
      ...area,
      groups: area.groups.map((group, groupIndex) => {
        if (groupIndex !== targetGroupIndex) {
          return group;
        }
        const nextSubAreas = [...group.sub_areas];
        nextSubAreas.splice(Math.max(0, normalizedTargetIndex), 0, movingSubArea);
        return {
          ...group,
          sub_areas: nextSubAreas,
        };
      }),
    };
  });
}

export function normalizeSitemap(sitemap: AppSitemapResponse): AppSitemapResponse {
  return {
    ...sitemap,
    areas: sitemap.areas.map((area, areaIndex) => ({
      ...area,
      position: areaIndex,
      groups: area.groups.map((group, groupIndex) => ({
        ...group,
        position: groupIndex,
        sub_areas: group.sub_areas.map((subArea, subAreaIndex) => ({
          ...subArea,
          position: subAreaIndex,
        })),
      })),
    })),
  };
}

export function createDefaultArea(index: number): AppSitemapAreaDto {
  return {
    logical_name: `area_${index + 1}`,
    display_name: `Area ${index + 1}`,
    position: index,
    icon: null,
    groups: [],
  };
}

export function createDefaultGroup(areaIndex: number, groupIndex: number) {
  return {
    logical_name: `group_${areaIndex + 1}_${groupIndex + 1}`,
    display_name: `Group ${groupIndex + 1}`,
    position: groupIndex,
    sub_areas: [],
  };
}

export function createDefaultSubArea(
  areaIndex: number,
  groupIndex: number,
  subAreaIndex: number,
  entityLogicalName: string | null,
): AppSitemapSubAreaDto {
  const target = entityLogicalName
    ? {
        type: "entity" as const,
        entity_logical_name: entityLogicalName,
        default_form: null,
        default_view: null,
      }
    : {
        type: "custom_page" as const,
        url: "",
      };

  return {
    logical_name: `sub_area_${areaIndex + 1}_${groupIndex + 1}_${subAreaIndex + 1}`,
    display_name: `Sub Area ${subAreaIndex + 1}`,
    position: subAreaIndex,
    icon: null,
    target,
  };
}

export function createCrmTemplateSitemap(
  appLogicalName: string,
  entities: EntityResponse[],
): AppSitemapResponse {
  const salesEntities = entities.filter((entity) =>
    ["account", "contact", "lead", "opportunity"].includes(entity.logical_name),
  );
  const activityEntities = entities.filter((entity) =>
    ["task", "appointment", "email", "phone_call"].includes(entity.logical_name),
  );

  return normalizeSitemap({
    app_logical_name: appLogicalName,
    areas: [
      {
        logical_name: "sales",
        display_name: "Sales",
        position: 0,
        icon: null,
        groups: [
          {
            logical_name: "customers",
            display_name: "Customers",
            position: 0,
            sub_areas: salesEntities.map((entity, index) => ({
              logical_name: `${entity.logical_name}_customers`,
              display_name: entity.display_name,
              position: index,
              icon: null,
              target: {
                type: "entity",
                entity_logical_name: entity.logical_name,
                default_form: null,
                default_view: null,
              },
            })),
          },
          {
            logical_name: "activity",
            display_name: "Activity",
            position: 1,
            sub_areas: activityEntities.map((entity, index) => ({
              logical_name: `${entity.logical_name}_activity`,
              display_name: entity.display_name,
              position: index,
              icon: null,
              target: {
                type: "entity",
                entity_logical_name: entity.logical_name,
                default_form: null,
                default_view: null,
              },
            })),
          },
        ],
      },
    ],
  });
}
