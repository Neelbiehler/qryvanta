import type { Metadata } from "next";

import { PlatformPillarsSection } from "@/components/landing/platform-pillars-section";
import { ContributeSection } from "@/components/landing/contribute-section";
import { FooterStrip } from "@/components/landing/footer-strip";
import { HeaderNav } from "@/components/landing/header-nav";
import { HeroProjectGrid } from "@/components/landing/hero-project-grid";
import { OssCloudSection } from "@/components/landing/oss-cloud-section";
import { SurfaceTracksSection } from "@/components/landing/surface-tracks-section";
import { platformPillars, surfaceTracks } from "@/components/landing/content";
import { LandingMotionMain } from "@/components/landing-motion-main";
import { PlatformKeyframeShowcase } from "@/components/platform-keyframe-showcase";

export const metadata: Metadata = {
  title: "Qryvanta.org",
  description:
    "The open-source project hub for Qryvanta. Read architecture notes and track delivery progress. Find open issues on GitHub to contribute.",
  alternates: {
    canonical: "https://qryvanta.org",
  },
  openGraph: {
    url: "https://qryvanta.org",
  },
};

export default function LandingPage() {
  const docsUrl =
    process.env.NEXT_PUBLIC_DOCS_URL ?? "https://docs.qryvanta.org";
  const cloudUrl = process.env.NEXT_PUBLIC_CLOUD_URL ?? "https://qryvanta.com";
  const githubUrl = "https://github.com/neelbiehler/qryvanta";

  return (
    <LandingMotionMain className="relative isolate min-h-screen overflow-hidden bg-landing">
      <div className="landing-content-shell mx-auto w-full max-w-[96rem] px-6 pb-16 pt-10 md:pt-14">
        <HeaderNav githubUrl={githubUrl} />

        <HeroProjectGrid docsUrl={docsUrl} githubUrl={githubUrl} />

        <section className="animate-rise-delay mt-8">
          <PlatformKeyframeShowcase />
        </section>

        <PlatformPillarsSection pillars={platformPillars} />

        <SurfaceTracksSection tracks={surfaceTracks} />

        <OssCloudSection docsUrl={docsUrl} cloudUrl={cloudUrl} />

        <ContributeSection docsUrl={docsUrl} githubUrl={githubUrl} />

        <FooterStrip docsUrl={docsUrl} githubUrl={githubUrl} />
      </div>
    </LandingMotionMain>
  );
}
