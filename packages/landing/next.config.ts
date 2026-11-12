import type { NextConfig } from 'next';

const NODE_ENV = process.env.NODE_ENV ?? 'development';

const nextConfig: NextConfig = {
  output: 'export',
  trailingSlash: true,
  // GitHub Pages project site lives under //tracers/
  basePath: NODE_ENV === 'production' ? '/tracers' : '',
  assetPrefix: NODE_ENV === 'production' ? '/tracers' : '',
  images: { unoptimized: true },
};

export default nextConfig;
