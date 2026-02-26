import type { SelectionState } from "@/components/apps/sitemap-editor/types";
import type {
  AppSitemapResponse,
  EntityResponse,
  FormResponse,
  ViewResponse,
} from "@/lib/api";

export type SitemapArea = AppSitemapResponse["areas"][number];
export type SitemapGroup = SitemapArea["groups"][number];
export type SitemapSubArea = SitemapGroup["sub_areas"][number];

export type UpdateSitemap = (
  mutator: (current: AppSitemapResponse) => AppSitemapResponse,
) => void;

export type AreaSelection = Extract<SelectionState, { kind: "area" }>;
export type GroupSelection = Extract<SelectionState, { kind: "group" }>;
export type SubAreaSelection = Extract<SelectionState, { kind: "sub_area" }>;

export type SubAreaEntityContext = {
  entities: EntityResponse[];
  selectedEntityForms: FormResponse[];
  selectedEntityViews: ViewResponse[];
  isLoadingTargetMetadata: boolean;
};
