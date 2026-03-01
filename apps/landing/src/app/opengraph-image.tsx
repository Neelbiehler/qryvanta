import { ImageResponse } from "next/og";

export const alt = "Qryvanta.org — Open-Source Business Platform";
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OgImage() {
  return new ImageResponse(
    (
      <div
        style={{
          height: "100%",
          width: "100%",
          display: "flex",
          flexDirection: "column",
          justifyContent: "flex-end",
          padding: "72px 80px",
          backgroundColor: "#f5fbf7",
          backgroundImage:
            "radial-gradient(circle at 18% 22%, rgba(16, 185, 129, 0.22) 0%, transparent 46%), radial-gradient(circle at 82% 12%, rgba(59, 130, 246, 0.16) 0%, transparent 40%)",
          fontFamily: "system-ui, -apple-system, sans-serif",
          position: "relative",
        }}
      >
        {/* Grid overlay — subtle */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            backgroundImage:
              "linear-gradient(rgba(15, 26, 23, 0.06) 1px, transparent 1px), linear-gradient(90deg, rgba(15, 26, 23, 0.06) 1px, transparent 1px)",
            backgroundSize: "48px 48px",
          }}
        />

        {/* Brand row */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "14px",
            marginBottom: "36px",
          }}
        >
          <div
            style={{
              width: "52px",
              height: "52px",
              borderRadius: "14px",
              backgroundColor: "#065f46",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: "26px",
              fontWeight: 700,
              color: "white",
              letterSpacing: "-0.02em",
            }}
          >
            Q
          </div>
          <span
            style={{
              fontSize: "17px",
              fontWeight: 700,
              letterSpacing: "0.2em",
              color: "#1e293b",
              textTransform: "uppercase",
            }}
          >
            QRYVANTA.ORG
          </span>
          <div
            style={{
              marginLeft: "8px",
              padding: "4px 12px",
              borderRadius: "999px",
              border: "1px solid rgba(5, 150, 105, 0.4)",
              backgroundColor: "rgba(236, 253, 245, 0.8)",
              fontSize: "12px",
              fontWeight: 600,
              color: "#065f46",
              letterSpacing: "0.08em",
            }}
          >
            OSS Preview
          </div>
        </div>

        {/* Headline */}
        <div
          style={{
            fontSize: "58px",
            fontWeight: 700,
            color: "#0f172a",
            lineHeight: 1.08,
            letterSpacing: "-0.02em",
            marginBottom: "22px",
            maxWidth: "860px",
          }}
        >
          Open-Source Business Platform
        </div>

        {/* Subheading */}
        <div
          style={{
            fontSize: "22px",
            color: "#475569",
            lineHeight: 1.45,
            maxWidth: "780px",
            marginBottom: "44px",
          }}
        >
          Build metadata-driven business systems in the open. Self-hostable
          and built to ship.
        </div>

        {/* Feature chips */}
        <div style={{ display: "flex", gap: "10px" }}>
          {["Rust Core", "Self-Hostable", "Metadata Runtime", "Open Source"].map(
            (label) => (
              <div
                key={label}
                style={{
                  padding: "7px 18px",
                  borderRadius: "999px",
                  border: "1px solid rgba(5, 150, 105, 0.35)",
                  backgroundColor: "rgba(236, 253, 245, 0.85)",
                  fontSize: "14px",
                  fontWeight: 600,
                  color: "#065f46",
                  letterSpacing: "0.02em",
                }}
              >
                {label}
              </div>
            ),
          )}
        </div>
      </div>
    ),
    { ...size },
  );
}
