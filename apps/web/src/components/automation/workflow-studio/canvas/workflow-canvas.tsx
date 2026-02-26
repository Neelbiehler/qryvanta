import type { ReactNode, RefObject } from "react";

type WorkflowCanvasProps = {
  canvasRef: RefObject<HTMLDivElement | null>;
  width: number;
  height: number;
  leftOffset: number;
  rightOffset: number;
  snapToGrid: boolean;
  gridSize: number;
  onPointerMove: (event: React.PointerEvent<HTMLDivElement>) => void;
  onPointerDown: (event: React.PointerEvent<HTMLDivElement>) => void;
  children: ReactNode;
};

export function WorkflowCanvas({
  canvasRef,
  width,
  height,
  leftOffset,
  rightOffset,
  snapToGrid,
  gridSize,
  onPointerMove,
  onPointerDown,
  children,
}: WorkflowCanvasProps) {
  return (
    <div ref={canvasRef} className="absolute inset-0 overflow-auto pt-16" onPointerMove={onPointerMove}>
      <div
        className="relative"
        style={{
          width: `${width}px`,
          height: `${height}px`,
          marginLeft: `${leftOffset}px`,
          marginRight: `${rightOffset}px`,
          backgroundImage: snapToGrid
            ? "linear-gradient(to right, rgba(148,163,184,0.18) 1px, transparent 1px), linear-gradient(to bottom, rgba(148,163,184,0.18) 1px, transparent 1px)"
            : undefined,
          backgroundSize: snapToGrid ? `${gridSize}px ${gridSize}px` : undefined,
        }}
        onPointerDown={onPointerDown}
      >
        {children}
      </div>
    </div>
  );
}
