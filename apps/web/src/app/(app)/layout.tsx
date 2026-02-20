import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { Header } from "@/components/layout/header";
import { Sidebar } from "@/components/layout/sidebar";
import { apiServerFetch, type UserIdentityResponse } from "@/lib/api";

type AppLayoutProps = {
  children: React.ReactNode;
};

export default async function AppLayout({ children }: AppLayoutProps) {
  const cookieHeader = (await cookies()).toString();
  const meResponse = await apiServerFetch("/auth/me", cookieHeader);

  if (meResponse.status === 401) {
    redirect("/login");
  }

  if (!meResponse.ok) {
    throw new Error("Failed to load current user");
  }

  const user = (await meResponse.json()) as UserIdentityResponse;

  return (
    <div className="grid min-h-screen grid-cols-1 md:grid-cols-[260px_1fr]">
      <Sidebar />
      <div className="flex min-h-screen flex-col">
        <Header user={user} />
        <main className="flex-1 p-6 md:p-10">{children}</main>
      </div>
    </div>
  );
}
