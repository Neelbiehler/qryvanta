export type SelectionState =
  | { kind: "area"; areaIndex: number }
  | { kind: "group"; areaIndex: number; groupIndex: number }
  | {
      kind: "sub_area";
      areaIndex: number;
      groupIndex: number;
      subAreaIndex: number;
    };

export type DragPayload =
  | { kind: "area"; areaIndex: number }
  | { kind: "group"; areaIndex: number; groupIndex: number }
  | {
      kind: "sub_area";
      areaIndex: number;
      groupIndex: number;
      subAreaIndex: number;
    };
