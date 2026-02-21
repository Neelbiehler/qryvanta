import { SurfaceLayout } from "@/components/layout/surface-layout";

type AdminLayoutProps = {
  children: React.ReactNode;
};

export default function AdminLayout({ children }: AdminLayoutProps) {
  return <SurfaceLayout surfaceId="admin">{children}</SurfaceLayout>;
}
