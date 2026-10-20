import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Tauri bundles the exported files; there is no Node.js server at runtime.
  output: "export",
  images: { unoptimized: true },
  assetPrefix:
    process.env.NODE_ENV === "development"
      ? "http://127.0.0.1:3000"
      : undefined,
};

export default nextConfig;
