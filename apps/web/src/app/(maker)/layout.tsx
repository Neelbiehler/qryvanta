import { SurfaceLayout } from "@/components/layout/surface-layout";
import { requireSurfaceUser } from "@/lib/surface-access";

type MakerLayoutProps = {
  children: React.ReactNode;
};

export default async function MakerLayout({ children }: MakerLayoutProps) {
  const user = await requireSurfaceUser("maker");

  return (
    <SurfaceLayout surfaceId="maker" user={user}>
      {children}
    </SurfaceLayout>
  );
}
