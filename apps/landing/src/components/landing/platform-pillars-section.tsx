import { Card, CardContent, CardHeader, CardTitle } from "@qryvanta/ui";

import type { PlatformPillar } from "@/components/landing/content";

type PlatformPillarsSectionProps = {
  pillars: PlatformPillar[];
};

export function PlatformPillarsSection({ pillars }: PlatformPillarsSectionProps) {
  return (
    <section
      id="architecture"
      className="animate-rise-delay mt-12 grid gap-4 md:grid-cols-3"
    >
      {pillars.map((pillar) => {
        const Icon = pillar.icon;

        return (
          <Card key={pillar.title} className="border-emerald-100/90 bg-white/88">
            <CardHeader>
              <div className="mb-3 inline-flex h-9 w-9 items-center justify-center rounded-lg border border-emerald-200 bg-emerald-50 text-emerald-700">
                <Icon className="h-4 w-4" />
              </div>
              <CardTitle className="landing-display text-2xl">
                {pillar.title}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-slate-600">{pillar.body}</p>
            </CardContent>
          </Card>
        );
      })}
    </section>
  );
}
