"use client";

import { Orb, emeraldPreset } from "react-ai-orb";

type NavOrbProps = {
  className?: string;
};

export function NavOrb({ className }: NavOrbProps) {
  return (
    <div
      className={[
        "relative h-5 w-5 overflow-hidden rounded-full border border-emerald-200/90 bg-white/70",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
      aria-hidden
    >
      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 scale-[0.3]">
        <Orb
          {...emeraldPreset}
          size={1}
          noShadow
          animationSpeedBase={0.35}
          animationSpeedHue={0.1}
          mainOrbHueAnimation
        />
      </div>
    </div>
  );
}
