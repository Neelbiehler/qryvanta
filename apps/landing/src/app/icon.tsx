import { ImageResponse } from "next/og";

export const size = { width: 32, height: 32 };
export const contentType = "image/png";

export default function Icon() {
  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#065f46",
          borderRadius: "7px",
          fontSize: "18px",
          fontWeight: 700,
          color: "white",
          fontFamily: "system-ui, -apple-system, sans-serif",
          letterSpacing: "-0.02em",
        }}
      >
        Q
      </div>
    ),
    { ...size },
  );
}
