import { Input, Label } from "@qryvanta/ui";

import type {
  GroupSelection,
  SitemapGroup,
  UpdateSitemap,
} from "@/components/apps/sitemap-editor/properties/types";

type GroupPropertiesSectionProps = {
  selection: GroupSelection;
  selectedGroup: SitemapGroup;
  onUpdateSitemap: UpdateSitemap;
};

export function GroupPropertiesSection({
  selection,
  selectedGroup,
  onUpdateSitemap,
}: GroupPropertiesSectionProps) {
  return (
    <>
      <div className="space-y-2">
        <Label htmlFor="group_display_name">Group Display Name</Label>
        <Input
          id="group_display_name"
          value={selectedGroup.display_name}
          onChange={(event) =>
            onUpdateSitemap((current) => ({
              ...current,
              areas: current.areas.map((area, areaIndex) => {
                if (areaIndex !== selection.areaIndex) {
                  return area;
                }
                return {
                  ...area,
                  groups: area.groups.map((group, groupIndex) =>
                    groupIndex === selection.groupIndex
                      ? { ...group, display_name: event.target.value }
                      : group,
                  ),
                };
              }),
            }))
          }
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="group_logical_name">Group Logical Name</Label>
        <Input
          id="group_logical_name"
          value={selectedGroup.logical_name}
          onChange={(event) =>
            onUpdateSitemap((current) => ({
              ...current,
              areas: current.areas.map((area, areaIndex) => {
                if (areaIndex !== selection.areaIndex) {
                  return area;
                }
                return {
                  ...area,
                  groups: area.groups.map((group, groupIndex) =>
                    groupIndex === selection.groupIndex
                      ? { ...group, logical_name: event.target.value }
                      : group,
                  ),
                };
              }),
            }))
          }
        />
      </div>
    </>
  );
}
