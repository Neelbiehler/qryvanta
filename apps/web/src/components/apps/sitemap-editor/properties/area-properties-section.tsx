import { Input, Label } from "@qryvanta/ui";

import type {
  AreaSelection,
  SitemapArea,
  UpdateSitemap,
} from "@/components/apps/sitemap-editor/properties/types";

type AreaPropertiesSectionProps = {
  selection: AreaSelection;
  selectedArea: SitemapArea;
  onUpdateSitemap: UpdateSitemap;
};

export function AreaPropertiesSection({
  selection,
  selectedArea,
  onUpdateSitemap,
}: AreaPropertiesSectionProps) {
  return (
    <>
      <div className="space-y-2">
        <Label htmlFor="area_display_name">Area Display Name</Label>
        <Input
          id="area_display_name"
          value={selectedArea.display_name}
          onChange={(event) =>
            onUpdateSitemap((current) => ({
              ...current,
              areas: current.areas.map((area, index) =>
                index === selection.areaIndex
                  ? { ...area, display_name: event.target.value }
                  : area,
              ),
            }))
          }
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="area_logical_name">Area Logical Name</Label>
        <Input
          id="area_logical_name"
          value={selectedArea.logical_name}
          onChange={(event) =>
            onUpdateSitemap((current) => ({
              ...current,
              areas: current.areas.map((area, index) =>
                index === selection.areaIndex
                  ? { ...area, logical_name: event.target.value }
                  : area,
              ),
            }))
          }
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="area_icon">Area Icon</Label>
        <Input
          id="area_icon"
          value={selectedArea.icon ?? ""}
          onChange={(event) =>
            onUpdateSitemap((current) => ({
              ...current,
              areas: current.areas.map((area, index) =>
                index === selection.areaIndex
                  ? {
                      ...area,
                      icon: event.target.value.trim().length > 0 ? event.target.value : null,
                    }
                  : area,
              ),
            }))
          }
        />
      </div>
    </>
  );
}
