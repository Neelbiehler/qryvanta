import type { LucideIcon } from "lucide-react";
import { Binary, Blocks, HardDriveDownload } from "lucide-react";

export type PlatformPillar = {
  title: string;
  body: string;
  icon: LucideIcon;
};

export type SurfaceTrack = {
  title: string;
  route: string;
  tone: "success" | "warning" | "critical" | "neutral";
  body: string;
};

export const platformPillars: PlatformPillar[] = [
  {
    title: "Rust-first Core",
    body: "Layered domain and application boundaries keep business rules portable and testable.",
    icon: Binary,
  },
  {
    title: "Metadata Runtime",
    body: "Define entities once and turn published metadata into runtime APIs and workflows.",
    icon: Blocks,
  },
  {
    title: "Self-hostable Ops",
    body: "Run everything locally and move to your own infrastructure without platform lock-in.",
    icon: HardDriveDownload,
  },
];

export const surfaceTracks: SurfaceTrack[] = [
  {
    title: "Admin Center",
    route: "/admin",
    tone: "critical",
    body: "Role governance, audit controls, and tenant security settings.",
  },
  {
    title: "Maker Center",
    route: "/maker",
    tone: "warning",
    body: "Entity modeling, app studio workflows, and automation authoring.",
  },
  {
    title: "Worker Apps",
    route: "/worker",
    tone: "success",
    body: "Focused operational surfaces for day-to-day record work.",
  },
];
