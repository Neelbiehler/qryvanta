import { SurfaceLayout } from "@/components/layout/surface-layout";
import { requireSurfaceUser } from "@/lib/surface-access";

type AdminLayoutProps = {
  children: React.ReactNode;
};

export default async function AdminLayout({ children }: AdminLayoutProps) {
  const user = await requireSurfaceUser("admin");

  return (
    <SurfaceLayout surfaceId="admin" user={user}>
      {children}
    </SurfaceLayout>
  );
}
