import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  StatusBadge,
} from "@qryvanta/ui";

import type { SurfaceTrack } from "@/components/landing/content";

type SurfaceTracksSectionProps = {
  tracks: SurfaceTrack[];
};

export function SurfaceTracksSection({ tracks }: SurfaceTracksSectionProps) {
  return (
    <section className="animate-rise-delay mt-12">
      <div className="mb-4 flex items-center justify-between gap-3">
        <h2 className="landing-display text-3xl text-slate-900">Surface Tracks</h2>
        <StatusBadge tone="neutral">Role-aware Navigation</StatusBadge>
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        {tracks.map((surface) => (
          <Card key={surface.title} className="border-emerald-100/90 bg-white/88">
            <CardHeader>
              <div className="flex items-center justify-between gap-2">
                <CardTitle className="landing-display text-2xl">
                  {surface.title}
                </CardTitle>
                <StatusBadge tone={surface.tone}>{surface.route}</StatusBadge>
              </div>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-slate-600">{surface.body}</p>
            </CardContent>
          </Card>
        ))}
      </div>
    </section>
  );
}
