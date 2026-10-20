import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

// Reuse the exact native TypeScript compiler already pinned for the Corsa runtime.
// This checks owned private tooling; it neither emits nor changes public packages.
const root = fileURLToPath(new URL("../../../", import.meta.url));
const require = createRequire(new URL("../../../package.json", import.meta.url));
const packagePath = require.resolve(
  `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
);
const compiler = join(
  dirname(packagePath),
  "lib",
  process.platform === "win32" ? "tsc.exe" : "tsc",
);

// With no argument, check every project the root solution references. `tsc -b`
// would write a .tsbuildinfo beside each non-composite project.
function solutionProjects(solution = join(root, "tsconfig.json")): string[] {
  const config: unknown = JSON.parse(readFileSync(solution, "utf8"));
  const references =
    config !== null && typeof config === "object" && "references" in config
      ? config.references
      : undefined;
  const projects = Array.isArray(references)
    ? references.map((reference: unknown) =>
        reference !== null &&
        typeof reference === "object" &&
        "path" in reference &&
        typeof reference.path === "string"
          ? reference.path
          : undefined,
      )
    : [];
  if (projects.length === 0 || projects.some((project) => project === undefined))
    throw new Error(`${solution} must reference projects by path`);
  return projects.filter((project) => project !== undefined);
}

function check(project: string): Promise<number> {
  return new Promise((resolve, reject) => {
    const child = spawn(compiler, ["--noEmit", "--pretty", "false", "--project", project], {
      cwd: root,
      stdio: "inherit",
      shell: false,
    });
    child.once("error", reject);
    child.once("close", (status) => resolve(status ?? 1));
  });
}

const argument = process.argv[2];
const projects = argument ? [argument] : solutionProjects();
const statuses = await Promise.all(projects.map(check));
process.exit(statuses.find((status) => status !== 0) ?? 0);
