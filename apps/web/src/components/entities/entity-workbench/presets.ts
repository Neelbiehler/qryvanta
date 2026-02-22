export type QueryPreset = {
  name: string;
  limitText: string;
  offsetText: string;
  logicalMode: "and" | "or";
  conditionsText: string;
  sortText: string;
  filtersText: string;
};

export function normalizeQueryPresets(rawValue: unknown): QueryPreset[] {
  if (!Array.isArray(rawValue)) {
    return [];
  }

  return rawValue
    .filter(
      (preset): preset is QueryPreset =>
        typeof preset === "object" &&
        preset !== null &&
        "name" in preset &&
        "limitText" in preset &&
        "offsetText" in preset &&
        "filtersText" in preset,
    )
    .map((preset) => {
      const logicalMode: QueryPreset["logicalMode"] =
        "logicalMode" in preset && preset.logicalMode === "or" ? "or" : "and";

      return {
        name: String(preset.name),
        limitText: String(preset.limitText),
        offsetText: String(preset.offsetText),
        logicalMode,
        conditionsText:
          "conditionsText" in preset && typeof preset.conditionsText === "string"
            ? preset.conditionsText
            : "[]",
        sortText:
          "sortText" in preset && typeof preset.sortText === "string"
            ? preset.sortText
            : "[]",
        filtersText: String(preset.filtersText),
      };
    })
    .filter((preset) => preset.name.trim().length > 0)
    .sort((left, right) => left.name.localeCompare(right.name));
}
