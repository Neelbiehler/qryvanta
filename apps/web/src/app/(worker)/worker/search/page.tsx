import type { Metadata } from "next";

import { WorkerSearchPanel } from "@/components/apps/worker-search-panel";

export const metadata: Metadata = {
  title: "Worker Search",
};

export default function WorkerSearchPage() {
  return <WorkerSearchPanel />;
}
