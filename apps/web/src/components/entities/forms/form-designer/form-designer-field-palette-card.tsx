import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
} from "@qryvanta/ui";

import type { PublishedSchemaResponse } from "@/lib/api";

type FormDesignerFieldPaletteCardProps = {
  paletteQuery: string;
  filteredPaletteFields: PublishedSchemaResponse["fields"];
  placedFieldNames: Set<string>;
  onPaletteQueryChange: (query: string) => void;
  onSetDragLabel: (label: string | null) => void;
};

export function FormDesignerFieldPaletteCard({
  paletteQuery,
  filteredPaletteFields,
  placedFieldNames,
  onPaletteQueryChange,
  onSetDragLabel,
}: FormDesignerFieldPaletteCardProps) {
  return (
    <Card className="h-fit">
      <CardHeader>
        <CardTitle className="text-base">Field Palette</CardTitle>
        <CardDescription>Drag fields into section columns, or use quick add in drop zones.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <Input
          value={paletteQuery}
          onChange={(event) => onPaletteQueryChange(event.target.value)}
          placeholder="Search field palette"
        />
        <div className="max-h-[500px] space-y-2 overflow-y-auto">
          {filteredPaletteFields.map((field) => (
            <button
              key={field.logical_name}
              type="button"
              draggable
              onDragStart={(event) => {
                event.dataTransfer.setData("text/plain", field.logical_name);
                event.dataTransfer.setData("text/form-field-source", "palette");
                onSetDragLabel(field.display_name || field.logical_name);
              }}
              onDragEnd={() => onSetDragLabel(null)}
              className="w-full rounded-md border border-zinc-200 bg-white px-3 py-2 text-left text-sm hover:border-emerald-300"
            >
              <p className="font-medium text-zinc-900">{field.display_name}</p>
              <p className="font-mono text-xs text-zinc-500">
                {field.logical_name} · {field.field_type}
              </p>
              {placedFieldNames.has(field.logical_name) ? (
                <p className="mt-1 text-xs text-emerald-700">Placed</p>
              ) : null}
            </button>
          ))}
          {filteredPaletteFields.length === 0 ? (
            <p className="text-xs text-zinc-500">No fields match this filter.</p>
          ) : null}
        </div>
      </CardContent>
    </Card>
  );
}
