import { useMemo, useState } from "react";
import { Bolt } from "lucide-react";

import { Button, Input, Label } from "@qryvanta/ui";

import type { DynamicTokenOption } from "@/components/automation/workflow-studio/model";

const OPERATOR_TOKENS = ["==", "!=", ">", "<", ">=", "<=", "&&", "||", "??"];

type ExpressionBuilderPopoverProps = {
  title: string;
  currentValue: string;
  tokens: DynamicTokenOption[];
  onInsertExpression: (value: string) => void;
};

function appendToken(value: string, token: string): string {
  return value.trim().length === 0 ? token : `${value} ${token}`;
}

export function ExpressionBuilderPopover({
  title,
  currentValue,
  tokens,
  onInsertExpression,
}: ExpressionBuilderPopoverProps) {
  const [open, setOpen] = useState(false);
  const [expression, setExpression] = useState("");
  const [mode, setMode] = useState<"simple" | "expression">("simple");

  const preview = useMemo(() => {
    if (expression.trim().length === 0) {
      return currentValue;
    }

    return appendToken(currentValue, expression.trim());
  }, [currentValue, expression]);

  const tokenGroups = useMemo(() => {
    const groups: Record<string, DynamicTokenOption[]> = {
      trigger: [],
      step: [],
      runtime: [],
    };

    for (const token of tokens) {
      groups[token.source].push(token);
    }

    return [
      { key: "trigger", label: "Trigger", tokens: groups.trigger },
      { key: "step", label: "Previous Steps", tokens: groups.step },
      { key: "runtime", label: "Runtime", tokens: groups.runtime },
    ].filter((group) => group.tokens.length > 0);
  }, [tokens]);

  return (
    <div className="space-y-2">
      <Button
        type="button"
        size="sm"
        variant="outline"
        onClick={() => setOpen((current) => !current)}
      >
        <Bolt className="mr-1 size-3.5" aria-hidden />
        {open ? "Hide Expression Builder" : "Open Expression Builder"}
      </Button>

      {open ? (
        <div className="space-y-2 rounded-md border border-zinc-200 bg-zinc-50 p-2">
          <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
            {title}
          </p>

          <div className="inline-flex rounded-md border border-zinc-300 bg-white p-0.5">
            <button
              type="button"
              className={`rounded px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${
                mode === "simple" ? "bg-emerald-100 text-emerald-800" : "text-zinc-600"
              }`}
              onClick={() => setMode("simple")}
            >
              Simple
            </button>
            <button
              type="button"
              className={`rounded px-2 py-1 text-[10px] font-semibold uppercase tracking-wide ${
                mode === "expression" ? "bg-emerald-100 text-emerald-800" : "text-zinc-600"
              }`}
              onClick={() => setMode("expression")}
            >
              Expression
            </button>
          </div>

          <div className="space-y-1">
            <Label htmlFor="expression_builder_input">
              {mode === "simple" ? "Builder input" : "Expression"}
            </Label>
            <Input
              id="expression_builder_input"
              value={expression}
              onChange={(event) => setExpression(event.target.value)}
              placeholder={
                mode === "simple"
                  ? "Build with tokens and operators"
                  : "concat({{trigger.payload.id}}, '-', {{run.id}})"
              }
            />
          </div>

          {mode === "simple"
            ? tokenGroups.map((group) => (
            <div key={group.key} className="space-y-1">
              <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-500">
                {group.label}
              </p>
              <div className="flex flex-wrap gap-1">
                {group.tokens.map((token) => (
                  <button
                    key={token.token}
                    type="button"
                    className="rounded border border-zinc-300 bg-white px-2 py-1 font-mono text-[10px] text-zinc-700 transition hover:border-emerald-300 hover:text-emerald-700"
                    onClick={() =>
                      setExpression((current) => appendToken(current, token.token))
                    }
                    title={token.label}
                  >
                    {token.token}
                  </button>
                ))}
              </div>
            </div>
            ))
            : null}

          {mode === "simple" ? (
          <div className="space-y-1">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-500">
              Operators
            </p>
            <div className="flex flex-wrap gap-1">
              {OPERATOR_TOKENS.map((token) => (
                <button
                  key={token}
                  type="button"
                  className="rounded border border-zinc-300 bg-white px-2 py-1 font-mono text-[10px] text-zinc-700 transition hover:border-emerald-300 hover:text-emerald-700"
                  onClick={() => setExpression((current) => appendToken(current, token))}
                >
                  {token}
                </button>
              ))}
            </div>
          </div>
          ) : null}

          <div className="space-y-1">
            <p className="text-[10px] font-semibold uppercase tracking-wide text-zinc-500">
              Preview
            </p>
            <p className="max-h-20 overflow-auto rounded border border-zinc-200 bg-white px-2 py-1 font-mono text-[10px] text-zinc-700">
              {preview}
            </p>
          </div>

          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              size="sm"
              onClick={() => {
                if (expression.trim().length === 0) {
                  return;
                }

                onInsertExpression(expression.trim());
                setExpression("");
              }}
            >
              Insert Expression
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={() => setExpression("")}
            >
              Clear
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
