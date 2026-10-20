import assert from "node:assert/strict";
import test from "node:test";

import { createMuseaVirtualTransformer, museaVirtualTransformId } from "./virtual-transform.ts";

void test("Musea virtual transforms inherit the authored art tsconfig", () => {
  assert.equal(
    museaVirtualTransformId("\0musea-art:/repo/packages/ui/src/MyButton.art.vue?musea-virtual"),
    "/repo/packages/ui/src/MyButton.art.vue.ts",
  );
  assert.equal(
    museaVirtualTransformId("\0musea:/repo/packages/ui/src/MyButton.vue?musea-virtual"),
    "/repo/packages/ui/src/MyButton.vue.ts",
  );
  assert.equal(museaVirtualTransformId("\0musea-shared:/repo/src/MyButton.art.vue"), null);
});

void test("Musea virtual TS transform prefers Vite OXC when available", async () => {
  let used = "";
  const transform = createMuseaVirtualTransformer({
    transformWithOxc: async (code, id, options) => {
      used = "oxc";
      assert.equal(code, "const value: number = 1");
      assert.equal(id, "/repo/.musea-virtual.ts");
      assert.deepEqual(options, { lang: "ts", sourcemap: true, target: "esnext" });
      return { code: "const value = 1;" };
    },
    transformWithEsbuild: async () => {
      used = "esbuild";
      return { code: "" };
    },
  });

  const result = await transform("const value: number = 1", "/repo/.musea-virtual.ts", true);
  assert.equal(used, "oxc");
  assert.equal(result.code, "const value = 1;");
});

void test("Musea virtual TS transform falls back to esbuild on Vite 7", async () => {
  let used = "";
  const transform = createMuseaVirtualTransformer({
    transformWithEsbuild: async (code, id, options) => {
      used = "esbuild";
      assert.equal(code, "const value: number = 1");
      assert.equal(id, "/repo/.musea-virtual.ts");
      assert.deepEqual(options, {
        loader: "ts",
        format: "esm",
        sourcemap: false,
        target: "esnext",
      });
      return { code: "const value = 1;" };
    },
  });

  const result = await transform("const value: number = 1", "/repo/.musea-virtual.ts", false);
  assert.equal(used, "esbuild");
  assert.equal(result.code, "const value = 1;");
});
