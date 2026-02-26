import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@qryvanta/ui";

import { AreaPropertiesSection } from "@/components/apps/sitemap-editor/properties/area-properties-section";
import { GroupPropertiesSection } from "@/components/apps/sitemap-editor/properties/group-properties-section";
import { SubAreaPropertiesSection } from "@/components/apps/sitemap-editor/properties/sub-area-properties-section";
import type {
  SitemapArea,
  SitemapGroup,
  SitemapSubArea,
  SubAreaEntityContext,
  UpdateSitemap,
} from "@/components/apps/sitemap-editor/properties/types";
import type { SelectionState } from "@/components/apps/sitemap-editor/types";

type SitemapPropertiesCardProps = {
  selection: SelectionState | null;
  selectedArea: SitemapArea | null;
  selectedGroup: SitemapGroup | null;
  selectedSubArea: SitemapSubArea | null;
  onUpdateSitemap: UpdateSitemap;
} & SubAreaEntityContext;

export function SitemapPropertiesCard({
  selection,
  selectedArea,
  selectedGroup,
  selectedSubArea,
  entities,
  selectedEntityForms,
  selectedEntityViews,
  isLoadingTargetMetadata,
  onUpdateSitemap,
}: SitemapPropertiesCardProps) {
  return (
    <Card className="h-fit">
      <CardHeader>
        <CardTitle className="text-base">Properties</CardTitle>
        <CardDescription>
          Configure selected node metadata and target behavior.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {selection?.kind === "area" && selectedArea ? (
          <AreaPropertiesSection
            selection={selection}
            selectedArea={selectedArea}
            onUpdateSitemap={onUpdateSitemap}
          />
        ) : null}

        {selection?.kind === "group" && selectedGroup ? (
          <GroupPropertiesSection
            selection={selection}
            selectedGroup={selectedGroup}
            onUpdateSitemap={onUpdateSitemap}
          />
        ) : null}

        {selection?.kind === "sub_area" && selectedSubArea ? (
          <SubAreaPropertiesSection
            selection={selection}
            selectedSubArea={selectedSubArea}
            entities={entities}
            selectedEntityForms={selectedEntityForms}
            selectedEntityViews={selectedEntityViews}
            isLoadingTargetMetadata={isLoadingTargetMetadata}
            onUpdateSitemap={onUpdateSitemap}
          />
        ) : null}
      </CardContent>
    </Card>
  );
}
