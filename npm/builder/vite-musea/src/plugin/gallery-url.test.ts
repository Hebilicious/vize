import assert from "node:assert/strict";
import test from "node:test";
import { resolveMuseaGalleryUrls } from "./gallery-url.js";

void test("gallery URLs retain Vite's resolved host and all published addresses", () => {
  assert.deepEqual(
    resolveMuseaGalleryUrls(
      {
        local: ["http://127.0.0.1:5397/"],
        network: ["http://192.0.2.10:5397/"],
      },
      { protocol: "http", address: "127.0.0.1", port: 5397 },
      "/__musea__",
    ),
    ["http://127.0.0.1:5397/__musea__", "http://192.0.2.10:5397/__musea__"],
  );
});

void test("gallery URL fallback preserves an explicit loopback binding", () => {
  assert.deepEqual(
    resolveMuseaGalleryUrls(
      null,
      { protocol: "http", address: "127.0.0.1", port: 5397 },
      "/__musea__",
    ),
    ["http://127.0.0.1:5397/__musea__"],
  );
});
