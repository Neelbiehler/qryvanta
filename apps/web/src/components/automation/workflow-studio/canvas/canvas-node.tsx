import type { ReactNode } from "react";

type CanvasNodeProps = {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  children: ReactNode;
};

export function CanvasNode({ id, x, y, width, height, children }: CanvasNodeProps) {
  return (
    <div
      className="absolute"
      data-canvas-node="true"
      data-canvas-node-id={id}
      style={{ left: `${x}px`, top: `${y}px`, width: `${width}px`, height: `${height}px` }}
    >
      {children}
    </div>
  );
}
