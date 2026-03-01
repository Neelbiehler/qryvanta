import { useMemo } from "react";

import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from "@qryvanta/ui";

import type { SearchHitView } from "./types";
import { formatValue, humanizeFieldName } from "./utils";

type ComparePanelProps = {
  items: SearchHitView[];
  onClear: () => void;
};

export function ComparePanel({ items, onClear }: ComparePanelProps) {
  const comparedFields = useMemo(() => {
    const allKeys = new Set<string>();
    for (const item of items) {
      for (const field of item.parsed.fields) {
        allKeys.add(field.key);
      }
    }
    return [...allKeys].slice(0, 16);
  }, [items]);

  return (
    <Card className="border-sky-200 bg-sky-50/40">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between gap-2">
          <CardTitle className="text-base">Compare Results</CardTitle>
          <Button type="button" size="sm" variant="outline" onClick={onClear}>
            Clear compare
          </Button>
        </div>
        <CardDescription>Compare key fields side-by-side for up to two selected results.</CardDescription>
      </CardHeader>
      <CardContent className="overflow-x-auto">
        <table className="min-w-full text-sm">
          <thead>
            <tr className="border-b border-sky-200 text-left text-xs text-zinc-600">
              <th className="py-2 pr-4 font-medium">Field</th>
              {items.map((item) => (
                <th key={item.hit.id} className="py-2 pr-4 font-medium text-zinc-800">
                  {item.hit.title}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {comparedFields.map((field) => (
              <tr key={field} className="border-b border-sky-100 align-top">
                <td className="py-2 pr-4 text-xs uppercase tracking-wide text-zinc-500">{humanizeFieldName(field)}</td>
                {items.map((item) => (
                  <td key={`${item.hit.id}-${field}`} className="py-2 pr-4 text-zinc-700">
                    {formatValue(field, item.parsed.byKey[field.toLowerCase()] ?? "-")}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </CardContent>
    </Card>
  );
}
