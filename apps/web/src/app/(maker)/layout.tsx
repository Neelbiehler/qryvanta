import { SurfaceLayout } from "@/components/layout/surface-layout";

type MakerLayoutProps = {
  children: React.ReactNode;
};

export default function MakerLayout({ children }: MakerLayoutProps) {
  return <SurfaceLayout surfaceId="maker">{children}</SurfaceLayout>;
}
