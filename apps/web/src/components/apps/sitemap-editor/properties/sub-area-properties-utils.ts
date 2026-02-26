import type {
  SitemapSubArea,
  SubAreaSelection,
} from "@/components/apps/sitemap-editor/properties/types";
import type { AppSitemapResponse } from "@/lib/api";

export function updateSubArea(
  current: AppSitemapResponse,
  selection: SubAreaSelection,
  mutator: (subArea: SitemapSubArea) => SitemapSubArea,
): AppSitemapResponse {
  return {
    ...current,
    areas: current.areas.map((area, areaIndex) => {
      if (areaIndex !== selection.areaIndex) {
        return area;
      }

      return {
        ...area,
        groups: area.groups.map((group, groupIndex) => {
          if (groupIndex !== selection.groupIndex) {
            return group;
          }

          return {
            ...group,
            sub_areas: group.sub_areas.map((subArea, subAreaIndex) =>
              subAreaIndex === selection.subAreaIndex ? mutator(subArea) : subArea,
            ),
          };
        }),
      };
    }),
  };
}
