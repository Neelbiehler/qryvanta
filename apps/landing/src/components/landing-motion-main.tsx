"use client";

import { useEffect, useRef, type ReactNode } from "react";

type LandingMotionMainProps = {
  children: ReactNode;
  className?: string;
};

function cx(...parts: Array<string | null | undefined | false>) {
  return parts.filter(Boolean).join(" ");
}

export function LandingMotionMain({
  children,
  className,
}: LandingMotionMainProps) {
  const mainRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    const node = mainRef.current;
    if (!node) {
      return;
    }

    let rafId = 0;
    let ticking = false;

    const update = () => {
      const scrollTop = Math.min(Math.max(window.scrollY, 0), 2200);
      node.style.setProperty("--landing-scroll", `${scrollTop}px`);
      ticking = false;
    };

    const queueUpdate = () => {
      if (ticking) {
        return;
      }

      ticking = true;
      rafId = window.requestAnimationFrame(update);
    };

    update();
    window.addEventListener("scroll", queueUpdate, { passive: true });
    window.addEventListener("resize", queueUpdate);

    return () => {
      window.removeEventListener("scroll", queueUpdate);
      window.removeEventListener("resize", queueUpdate);
      if (rafId) {
        window.cancelAnimationFrame(rafId);
      }
    };
  }, []);

  return (
    <main ref={mainRef} className={cx("landing-parallax-root", className)}>
      {children}
    </main>
  );
}
