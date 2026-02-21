import { SurfaceLayout } from "@/components/layout/surface-layout";

type WorkerLayoutProps = {
  children: React.ReactNode;
};

export default function WorkerLayout({ children }: WorkerLayoutProps) {
  return <SurfaceLayout surfaceId="worker">{children}</SurfaceLayout>;
}
