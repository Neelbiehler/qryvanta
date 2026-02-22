import { SurfaceLayout } from "@/components/layout/surface-layout";
import { requireSurfaceUser } from "@/lib/surface-access";

type WorkerLayoutProps = {
  children: React.ReactNode;
};

export default async function WorkerLayout({ children }: WorkerLayoutProps) {
  const user = await requireSurfaceUser("worker");

  return (
    <SurfaceLayout surfaceId="worker" user={user}>
      {children}
    </SurfaceLayout>
  );
}
