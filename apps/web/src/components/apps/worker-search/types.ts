import type { QrywellSearchResponse } from "@/lib/api";

export type ParsedField = {
  key: string;
  value: string;
};

export type ParsedResult = {
  fields: ParsedField[];
  byKey: Record<string, string>;
};

export type SearchHit = QrywellSearchResponse["hits"][number];

export type SearchHitView = {
  hit: SearchHit;
  parsed: ParsedResult;
  groupLabel: string;
};

export type ActiveFacet = {
  key: string;
  value: string;
};

export type FacetSuggestion = ActiveFacet & {
  count: number;
};
