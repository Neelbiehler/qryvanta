import type { Metadata } from "next";
import { redirect } from "next/navigation";

export const metadata: Metadata = {
  title: "Qryvanta Workspace",
  description: "Open the Qryvanta workspace dashboard.",
};

export default function HomePage() {
  redirect("/entities");
}
