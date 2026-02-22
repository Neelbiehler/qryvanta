"use client";

import { useEffect, useRef, useState } from "react";

import { Home } from "lucide-react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  StatusBadge,
} from "@qryvanta/ui";
import { showcaseFrames } from "@/components/platform-showcase-data";
import {
  cx,
  frameStyle,
  revealStyle,
  wordStyle,
} from "@/components/platform-showcase-utils";

export function PlatformKeyframeShowcase() {
  const [isInView, setIsInView] = useState(false);
  const showcaseRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const node = showcaseRef.current;
    if (!node) {
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setIsInView(true);
            observer.disconnect();
            break;
          }
        }
      },
      {
        threshold: 0.2,
        rootMargin: "0px 0px -12% 0px",
      },
    );

    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  return (
    <div ref={showcaseRef}>
      <Card
        className="platform-showcase-card border-emerald-100/90 bg-white/92"
        data-in-view={isInView ? "true" : "false"}
      >
        <CardHeader>
          <div
            className="platform-reveal flex items-center justify-between gap-3"
            style={revealStyle(30)}
          >
            <CardTitle className="landing-display text-3xl">
              Product Walkthrough
            </CardTitle>
            <StatusBadge tone="neutral">Live Platform Flow</StatusBadge>
          </div>
          <CardDescription className="platform-reveal" style={revealStyle(110)}>
            App-like scenes based on the real `apps/web` surface model and
            routes.
          </CardDescription>
        </CardHeader>

        <CardContent>
          <div className="grid gap-4 xl:grid-cols-[1.4fr_0.6fr] 2xl:grid-cols-[1.48fr_0.52fr]">
            <div className="platform-reveal" style={revealStyle(160)}>
              <div className="platform-showcase-stage">
                <div className="platform-stage-hud" aria-hidden>
                  <span className="platform-stage-live-pill">
                    <span className="platform-stage-live-dot" />
                    Live walkthrough stream
                  </span>
                  <span className="platform-stage-live-time">
                    00:24 runtime
                  </span>
                </div>

                {showcaseFrames.map((frame, index) => {
                  return (
                    <article
                      key={frame.id}
                      className="platform-scene"
                      style={frameStyle(index, frame.accent, frame.glow)}
                    >
                      <div className="platform-scene-window platform-scene-camera">
                        <header className="platform-scene-window-top">
                          <div className="platform-window-lights" aria-hidden>
                            <span />
                            <span />
                            <span />
                          </div>
                          <p className="platform-scene-route">{frame.route}</p>
                          <span className="platform-scene-capture">
                            {frame.capture}
                          </span>
                          <StatusBadge tone={frame.tone}>
                            {frame.badge}
                          </StatusBadge>
                        </header>

                        <div className="platform-scene-shell-grid">
                          <aside className="platform-scene-sidebar">
                            <div className="platform-scene-brand">
                              <span className="platform-scene-brand-mark">
                                Q
                              </span>
                              <span>Qryvanta</span>
                            </div>
                            <p className="platform-scene-surface">
                              {frame.surface}
                            </p>
                            <ul className="platform-scene-nav">
                              {frame.navItems.map((item) => (
                                <li
                                  key={item}
                                  className={cx(
                                    "platform-scene-nav-item",
                                    item === frame.activeNav &&
                                      "platform-scene-nav-item-active",
                                  )}
                                >
                                  {item === frame.activeNav ? (
                                    <span className="platform-scene-nav-dot" />
                                  ) : null}
                                  <span className="truncate">{item}</span>
                                </li>
                              ))}
                            </ul>
                          </aside>

                          <section className="platform-scene-content">
                            <div className="platform-scene-content-head">
                              <div>
                                <p className="platform-scene-kicker">
                                  {frame.lane}
                                </p>
                                <p className="platform-scene-title">
                                  {frame.title}
                                </p>
                              </div>
                              <div className="platform-scene-command">
                                <Home className="h-3 w-3" /> / jump to route
                              </div>
                            </div>

                            <p className="platform-scene-summary">
                              {frame.summary}
                            </p>

                            <div className="platform-scene-actions">
                              {frame.quickActions.map((action) => (
                                <span
                                  key={action}
                                  className="platform-scene-action-pill"
                                >
                                  {action}
                                </span>
                              ))}
                            </div>

                            <div className="platform-scene-metrics">
                              {frame.metrics.map((metric) => (
                                <article
                                  key={metric.label}
                                  className="platform-scene-metric"
                                >
                                  <p className="platform-scene-metric-label">
                                    {metric.label}
                                  </p>
                                  <p className="platform-scene-metric-value">
                                    {metric.value}
                                  </p>
                                </article>
                              ))}
                            </div>

                            <div className="platform-scene-table">
                              <div className="platform-scene-table-head">
                                <span>Item</span>
                                <span className="platform-scene-cell-muted">
                                  Context
                                </span>
                                <span>State</span>
                              </div>
                              {frame.rows.map((row) => (
                                <div
                                  key={row.primary}
                                  className="platform-scene-table-row"
                                >
                                  <span className="platform-scene-cell-primary">
                                    {row.primary}
                                  </span>
                                  <span className="platform-scene-cell-muted">
                                    {row.context}
                                  </span>
                                  <span>
                                    <StatusBadge tone={row.tone}>
                                      {row.status}
                                    </StatusBadge>
                                  </span>
                                </div>
                              ))}
                            </div>

                            <p className="platform-scene-narration platform-scene-narration-stream">
                              {frame.narration
                                .split(" ")
                                .map((word, wordIndex, words) => (
                                  <span
                                    key={`${frame.id}-word-${wordIndex}`}
                                    className="platform-scene-narration-word"
                                    style={wordStyle(wordIndex)}
                                  >
                                    {word}
                                    {wordIndex < words.length - 1 ? " " : ""}
                                  </span>
                                ))}
                            </p>

                            <div className="platform-scene-events">
                              {frame.events.map((eventLine) => (
                                <p
                                  key={eventLine}
                                  className="platform-scene-event"
                                >
                                  {eventLine}
                                </p>
                              ))}
                            </div>
                          </section>
                        </div>
                      </div>
                    </article>
                  );
                })}

                <div className="platform-stage-controls" aria-hidden>
                  <span className="platform-stage-controls-label">
                    Walkthrough Reel
                  </span>
                  <div className="platform-stage-controls-track">
                    <span className="platform-stage-controls-progress" />
                  </div>
                  <span className="platform-stage-controls-range">
                    00:00 / 00:24
                  </span>
                </div>
              </div>
            </div>

            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-1">
              {showcaseFrames.map((frame, index) => {
                const Icon = frame.icon;

                return (
                  <div
                    key={`${frame.id}-detail`}
                    className="platform-reveal"
                    style={revealStyle(260 + index * 90)}
                  >
                    <article
                      className="platform-step rounded-xl border border-emerald-100/90 bg-emerald-50/45 p-3"
                      style={frameStyle(index, frame.accent, frame.glow)}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="flex min-w-0 items-start gap-3">
                          <span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-emerald-200 bg-white text-emerald-700">
                            <Icon className="h-4 w-4" />
                          </span>
                          <div className="min-w-0">
                            <p className="text-[0.64rem] font-semibold uppercase tracking-[0.16em] text-slate-500">
                              {frame.frame}
                            </p>
                            <p className="text-sm font-semibold text-slate-900">
                              {frame.title}
                            </p>
                            <p className="mt-0.5 truncate text-[0.68rem] text-slate-500">
                              {frame.route}
                            </p>
                          </div>
                        </div>
                        <StatusBadge tone={frame.tone}>
                          {frame.badge}
                        </StatusBadge>
                      </div>

                      <p className="mt-2 text-xs text-slate-600">
                        {frame.summary}
                      </p>

                      <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-emerald-100">
                        <span className="platform-step-progress" />
                      </div>
                    </article>
                  </div>
                );
              })}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
