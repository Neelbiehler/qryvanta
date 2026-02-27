import { SurfaceLayout } from "@/components/layout/surface-layout";
import { requireSurfaceUser } from "@/lib/surface-access";

type WorkerLayoutProps = {
  children: React.ReactNode;
};

export default async function WorkerLayout({ children }: WorkerLayoutProps) {
  const user = await requireSurfaceUser("worker");

  return (
    <SurfaceLayout
      surfaceId="worker"
      user={user}
      hideSidebar
      disableGlobalCommand
      disableSurfaceSwitcher
      mainClassName="h-[calc(100vh-3.25rem)] overflow-hidden p-0 md:p-0"
    >
      {children}
    </SurfaceLayout>
  );
}
