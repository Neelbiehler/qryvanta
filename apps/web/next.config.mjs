/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ["@qryvanta/ui", "@qryvanta/api-types"],
  experimental: {
    optimizePackageImports: ["lucide-react"],
  },
};

export default nextConfig;
