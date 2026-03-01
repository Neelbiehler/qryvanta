"use client";

import { useEffect, useMemo, useRef, useState } from "react";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Label,
  Notice,
  StatusBadge,
} from "@qryvanta/ui";
import { Loader2, Search, Sparkles, X } from "lucide-react";

import { apiFetch, type QrywellSearchResponse } from "@/lib/api";

import { ComparePanel } from "./worker-search/compare-panel";
import { SearchResultCard } from "./worker-search/result-card";
import type { ActiveFacet, SearchHitView } from "./worker-search/types";
import {
  buildFacetSuggestions,
  HISTORY_STORAGE_KEY,
  humanizeFieldName,
  inferGroupLabel,
  parseResultText,
  QUERY_EXAMPLES,
  saveQueryToHistory,
} from "./worker-search/utils";

export function WorkerSearchPanel() {
  const queryInputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [results, setResults] = useState<QrywellSearchResponse | null>(null);
  const [includeDebug, setIncludeDebug] = useState(false);
  const [lastCompletedAt, setLastCompletedAt] = useState<string | null>(null);
  const [recentQueries, setRecentQueries] = useState<string[]>([]);
  const [activeFacet, setActiveFacet] = useState<ActiveFacet | null>(null);
  const [compareIds, setCompareIds] = useState<string[]>([]);

  useEffect(() => {
    try {
      const raw = localStorage.getItem(HISTORY_STORAGE_KEY);
      if (!raw) {
        return;
      }
      const parsed = JSON.parse(raw) as string[];
      setRecentQueries(parsed.filter((value) => typeof value === "string").slice(0, 8));
    } catch {
      setRecentQueries([]);
    }
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const isTypingTarget =
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.getAttribute("contenteditable") === "true";

      if (event.key === "/" && !isTypingTarget) {
        event.preventDefault();
        queryInputRef.current?.focus();
      }

      if (event.key === "Escape") {
        setQuery("");
        setError(null);
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const scoreLabel = useMemo(() => {
    if (!results?.hits.length) {
      return null;
    }

    const best = Math.max(...results.hits.map((hit) => hit.score));
    return `Best relevance ${(best * 100).toFixed(0)}%`;
  }, [results]);

  const debugSummary = useMemo(() => {
    if (!results?.debug_query_normalized && !results?.debug_selected_entity) {
      return null;
    }

    return {
      normalizedQuery: results.debug_query_normalized ?? "-",
      selectedEntity: results.debug_selected_entity ?? "none",
      plannedFilters: results.debug_planned_filter_count ?? 0,
      negatedFilters: results.debug_negated_filter_count ?? 0,
    };
  }, [results]);

  const hitViews = useMemo<SearchHitView[]>(
    () =>
      (results?.hits ?? []).map((hit) => {
        const parsed = parseResultText(hit.text);
        return {
          hit,
          parsed,
          groupLabel: inferGroupLabel(parsed),
        };
      }),
    [results],
  );

  const facetSuggestions = useMemo(() => buildFacetSuggestions(hitViews), [hitViews]);

  const visibleHits = useMemo(() => {
    if (!activeFacet) {
      return hitViews;
    }

    return hitViews.filter(({ parsed }) => {
      const candidate = parsed.byKey[activeFacet.key.toLowerCase()];
      return candidate?.toLowerCase() === activeFacet.value.toLowerCase();
    });
  }, [activeFacet, hitViews]);

  const groupedHits = useMemo(() => {
    const groups = new Map<string, SearchHitView[]>();
    for (const item of visibleHits) {
      const existing = groups.get(item.groupLabel);
      if (existing) {
        existing.push(item);
      } else {
        groups.set(item.groupLabel, [item]);
      }
    }
    return [...groups.entries()].sort((left, right) => right[1].length - left[1].length);
  }, [visibleHits]);

  const compareItems = useMemo(() => {
    if (compareIds.length === 0) {
      return [];
    }
    const lookup = new Map(hitViews.map((item) => [item.hit.id, item]));
    return compareIds.map((id) => lookup.get(id)).filter((item): item is SearchHitView => Boolean(item));
  }, [compareIds, hitViews]);

  async function handleSearch(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!query.trim()) {
      setError("Enter a search query first.");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await apiFetch("/api/search/qrywell", {
        method: "POST",
        body: JSON.stringify({
          query,
          limit: 10,
          include_debug: includeDebug,
        }),
      });

      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as { message?: string } | null;
        throw new Error(payload?.message ?? `Search failed with status ${response.status}`);
      }

      setResults((await response.json()) as QrywellSearchResponse);
      setLastCompletedAt(new Date().toLocaleTimeString());
      setActiveFacet(null);
      setCompareIds([]);
      saveQueryToHistory(query, setRecentQueries);
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : "Search failed");
      setResults(null);
    } finally {
      setLoading(false);
    }
  }

  async function trackClick(item: SearchHitView, rank: number) {
    try {
      await apiFetch("/api/search/qrywell/events/click", {
        method: "POST",
        body: JSON.stringify({
          search_event_id: results?.search_event_id,
          query: results?.query ?? query,
          result_id: item.hit.id,
          rank,
          score: item.hit.score,
          title: item.hit.title,
          connector_type: item.hit.connector_type,
          group_label: item.groupLabel,
        }),
      });
    } catch {
      // analytics failure should not block navigation
    }
  }

  function toggleCompare(id: string) {
    setCompareIds((current) => {
      if (current.includes(id)) {
        return current.filter((value) => value !== id);
      }
      if (current.length >= 2) {
        return [current[1], id];
      }
      return [...current, id];
    });
  }

  return (
    <div className="space-y-5 p-4">
      <Card className="border-sky-100 shadow-sm">
        <CardHeader>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-sky-600">Smart Search</p>
              <CardTitle className="mt-1 text-2xl">Workspace Knowledge Search</CardTitle>
              <CardDescription className="mt-1">
                Powered by Qrywell retrieval with metadata-aware query planning.
              </CardDescription>
            </div>
            <div className="flex flex-wrap gap-2">
              <StatusBadge tone="success" dot>
                Tenant scoped
              </StatusBadge>
              <StatusBadge tone="info" dot>
                ACL aware
              </StatusBadge>
              {lastCompletedAt ? <StatusBadge tone="neutral">Last query {lastCompletedAt}</StatusBadge> : null}
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <form className="grid gap-3" onSubmit={handleSearch}>
            <div className="grid gap-1.5">
              <Label htmlFor="worker-smart-search-query">Search query</Label>
              <Input
                id="worker-smart-search-query"
                ref={queryInputRef}
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Find opportunities at risk in the current quarter"
              />
            </div>

            <div className="flex flex-wrap gap-2">
              {QUERY_EXAMPLES.map((example) => (
                <button
                  key={example}
                  type="button"
                  onClick={() => setQuery(example)}
                  className="rounded-full border border-zinc-200 bg-zinc-50 px-3 py-1 text-xs text-zinc-700 hover:bg-zinc-100"
                >
                  {example}
                </button>
              ))}
            </div>

            {recentQueries.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                <span className="text-xs text-zinc-500">Recent:</span>
                {recentQueries.map((item) => (
                  <button
                    key={item}
                    type="button"
                    onClick={() => setQuery(item)}
                    className="rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs text-zinc-700 hover:bg-zinc-100"
                  >
                    {item}
                  </button>
                ))}
              </div>
            ) : null}

            <div className="flex flex-wrap items-center gap-2">
              <Button type="submit" disabled={loading}>
                {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Search className="h-4 w-4" />}
                {loading ? "Searching..." : "Search"}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setQuery("");
                  setResults(null);
                  setError(null);
                  setActiveFacet(null);
                  setCompareIds([]);
                }}
                disabled={loading}
              >
                <X className="h-4 w-4" />
                Clear
              </Button>
              <label className="ml-2 inline-flex items-center gap-2 text-xs text-zinc-600">
                <input
                  type="checkbox"
                  checked={includeDebug}
                  onChange={(event) => setIncludeDebug(event.target.checked)}
                />
                Include planner debug
              </label>
              {scoreLabel ? <span className="text-xs text-zinc-500">{scoreLabel}</span> : null}
            </div>
          </form>
        </CardContent>
      </Card>

      {error ? <Notice tone="error">{error}</Notice> : null}

      {results ? (
        <section className="space-y-3 rounded-xl border border-zinc-200 bg-white p-4 shadow-sm">
          <div className="mb-3 flex items-center justify-between gap-2">
            <h2 className="text-sm font-semibold text-zinc-800">
              {visibleHits.length} result{visibleHits.length !== 1 ? "s" : ""}
            </h2>
            <p className="text-xs text-zinc-500">Query: {results.query}</p>
          </div>

          {debugSummary ? (
            <details className="rounded-md border border-zinc-200 bg-zinc-50 p-3 text-xs text-zinc-700">
              <summary className="cursor-pointer font-medium text-zinc-900">Planner diagnostics</summary>
              <div className="mt-2 grid gap-1">
                <p>Normalized query: {debugSummary.normalizedQuery}</p>
                <p>Selected entity: {debugSummary.selectedEntity}</p>
                <p>Planned filters: {debugSummary.plannedFilters}</p>
                <p>Negated filters: {debugSummary.negatedFilters}</p>
              </div>
            </details>
          ) : null}

          {facetSuggestions.length > 0 ? (
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-xs text-zinc-500">Quick filters:</span>
              {facetSuggestions.map((facet) => {
                const active =
                  activeFacet?.key === facet.key && activeFacet?.value.toLowerCase() === facet.value.toLowerCase();
                return (
                  <button
                    key={`${facet.key}-${facet.value}`}
                    type="button"
                    onClick={() => {
                      setActiveFacet(active ? null : { key: facet.key, value: facet.value });
                    }}
                    className={`rounded-full border px-3 py-1 text-xs ${
                      active
                        ? "border-sky-300 bg-sky-50 text-sky-800"
                        : "border-zinc-200 bg-white text-zinc-700 hover:bg-zinc-100"
                    }`}
                  >
                    {humanizeFieldName(facet.key)}: {facet.value} ({facet.count})
                  </button>
                );
              })}
              {activeFacet ? (
                <button
                  type="button"
                  onClick={() => setActiveFacet(null)}
                  className="text-xs text-zinc-500 underline"
                >
                  clear filter
                </button>
              ) : null}
            </div>
          ) : null}

          {compareItems.length > 0 ? <ComparePanel items={compareItems} onClear={() => setCompareIds([])} /> : null}

          <div className="space-y-4">
            {groupedHits.map(([groupLabel, items]) => (
              <div key={groupLabel} className="space-y-2">
                <div className="flex items-center justify-between">
                  <h3 className="text-xs font-semibold uppercase tracking-wide text-zinc-500">{groupLabel}</h3>
                  <StatusBadge tone="neutral">{items.length}</StatusBadge>
                </div>
                <div className="grid gap-3">
                  {items.map((item, groupIndex) => (
                    <SearchResultCard
                      key={item.hit.id}
                      hit={item.hit}
                      parsed={item.parsed}
                      rank={groupIndex + 1}
                      compared={compareIds.includes(item.hit.id)}
                      onToggleCompare={() => toggleCompare(item.hit.id)}
                      onTrackOpen={() => trackClick(item, groupIndex + 1)}
                    />
                  ))}
                </div>
              </div>
            ))}

            {visibleHits.length === 0 ? (
              <div className="rounded-lg border border-zinc-200 bg-zinc-50 p-4 text-sm text-zinc-600">
                No results match the selected quick filter.
              </div>
            ) : null}
          </div>
        </section>
      ) : (
        <Card>
          <CardContent className="py-8 text-center text-sm text-zinc-500">
            <div className="mx-auto mb-2 inline-flex h-10 w-10 items-center justify-center rounded-full bg-sky-50 text-sky-600">
              <Sparkles className="h-5 w-5" />
            </div>
            Run a query to explore tenant knowledge with semantic + schema-aware retrieval.
          </CardContent>
        </Card>
      )}
    </div>
  );
}
