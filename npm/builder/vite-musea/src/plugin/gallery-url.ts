import type { ViteDevServer } from "vite";

export function resolveMuseaGalleryUrls(
  resolvedUrls: ViteDevServer["resolvedUrls"],
  fallback: { protocol: "http" | "https"; address: string; port: number },
  basePath: string,
): string[] {
  const origins = [...(resolvedUrls?.local ?? []), ...(resolvedUrls?.network ?? [])];
  if (origins.length === 0) {
    const host = ["::", "0.0.0.0"].includes(fallback.address)
      ? "localhost"
      : fallback.address === "::1"
        ? "[::1]"
        : fallback.address;
    origins.push(`${fallback.protocol}://${host}:${fallback.port}/`);
  }

  return [...new Set(origins)].map((origin) => {
    const url = new URL(origin);
    url.pathname = basePath;
    url.search = "";
    url.hash = "";
    return url.href;
  });
}
