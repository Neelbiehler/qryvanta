import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { SurfaceLayout } from "@/components/layout/surface-layout";
import { apiServerFetch } from "@/lib/api";

type MakerLayoutProps = {
  children: React.ReactNode;
};

export default async function MakerLayout({ children }: MakerLayoutProps) {
  const cookieHeader = (await cookies()).toString();
  const meResponse = await apiServerFetch("/auth/me", cookieHeader);

  if (meResponse.status === 401) {
    redirect("/login");
  }

  if (!meResponse.ok) {
    throw new Error("Failed to load user");
  }

  const user = await meResponse.json();

  return (
    <SurfaceLayout surfaceId="maker" user={user}>
      {children}
    </SurfaceLayout>
  );
}
