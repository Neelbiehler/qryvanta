type CanvasEdgeProps = {
  fromX: number;
  fromY: number;
  toX: number;
  toY: number;
  label?: string;
  stroke?: string;
  dashed?: boolean;
};

export function CanvasEdge({
  fromX,
  fromY,
  toX,
  toY,
  label,
  stroke = "#4ade80",
  dashed = false,
}: CanvasEdgeProps) {
  const travelX = toX - fromX;
  const bendX = travelX >= 0 ? fromX + Math.max(48, travelX * 0.45) : fromX + 72;
  const pathData = `M ${fromX} ${fromY} L ${bendX} ${fromY} L ${bendX} ${toY} L ${toX} ${toY}`;

  return (
    <g>
      <path
        d={pathData}
        fill="none"
        stroke={stroke}
        strokeWidth="2"
        strokeLinejoin="round"
        strokeDasharray={dashed ? "6 4" : undefined}
      />
      {label ? (
        <text
          x={(fromX + toX) / 2}
          y={(fromY + toY) / 2 - 8}
          textAnchor="middle"
          className="fill-zinc-500 text-[10px] font-semibold uppercase tracking-wide"
        >
          {label}
        </text>
      ) : null}
    </g>
  );
}
