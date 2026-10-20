import * as vite from "vite";
import type { Plugin } from "vite";

type HookFunction<T> = T extends (...args: infer Args) => infer Result
  ? (...args: Args) => Result
  : T extends { handler: infer Handler }
    ? Handler extends (...args: infer Args) => infer Result
      ? (...args: Args) => Result
      : never
    : never;
type PluginTransformReturn = Awaited<ReturnType<HookFunction<NonNullable<Plugin["transform"]>>>>;
type TransformOutput = Exclude<PluginTransformReturn, string | null | void>;
type OxcOptions = { lang: "ts"; sourcemap: boolean; target: "esnext" };
type EsbuildOptions = { loader: "ts"; format: "esm"; sourcemap: boolean; target: "esnext" };
type TransformWithOxc = (
  code: string,
  id: string,
  options: OxcOptions,
) => TransformOutput | Promise<TransformOutput>;
type TransformWithEsbuild = (
  code: string,
  id: string,
  options: EsbuildOptions,
) => TransformOutput | Promise<TransformOutput>;

interface ViteTransformApi {
  transformWithOxc?: TransformWithOxc;
  transformWithEsbuild?: TransformWithEsbuild;
}

const VIRTUAL_ART_PREFIX = "\0musea-art:";
const VIRTUAL_INLINE_ART_PREFIX = "\0musea:";

/**
 * Give Vite's TypeScript transformer a filename beside the authored art file.
 * OXC uses this filename to select a tsconfig, so a synthetic filename under
 * the Vite root can accidentally select an unrelated ancestor config.
 */
export function museaVirtualTransformId(id: string): string | null {
  const prefix = id.startsWith(VIRTUAL_ART_PREFIX)
    ? VIRTUAL_ART_PREFIX
    : id.startsWith(VIRTUAL_INLINE_ART_PREFIX)
      ? VIRTUAL_INLINE_ART_PREFIX
      : null;

  if (!prefix) return null;

  const sourceId = id.slice(prefix.length).replace(/\?musea-virtual$/, "");
  return `${sourceId}.ts`;
}

export function createMuseaVirtualTransformer(viteApi: ViteTransformApi) {
  return async (code: string, id: string, sourcemap: boolean): Promise<TransformOutput> => {
    if (typeof viteApi.transformWithOxc === "function") {
      return viteApi.transformWithOxc(code, id, {
        lang: "ts",
        sourcemap,
        target: "esnext",
      });
    }
    if (typeof viteApi.transformWithEsbuild === "function") {
      return viteApi.transformWithEsbuild(code, id, {
        loader: "ts",
        format: "esm",
        sourcemap,
        target: "esnext",
      });
    }
    throw new Error("Installed Vite does not expose transformWithOxc or transformWithEsbuild");
  };
}

export const transformMuseaVirtualModule = createMuseaVirtualTransformer(vite);
