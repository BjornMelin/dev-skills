import { describe, expect, test } from "bun:test";
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const script = path.resolve(import.meta.dir, "..", "zod-run.ts");

function createRoot(): string {
  const root = mkdtempSync(path.join(tmpdir(), "zod-run-"));
  writeFileSync(path.join(root, "package.json"), "{}");
  mkdirSync(path.join(root, "node_modules", "zod"), { recursive: true });
  writeFileSync(
    path.join(root, "node_modules", "zod", "package.json"),
    JSON.stringify({ main: "index.cjs" }),
  );
  writeFileSync(
    path.join(root, "node_modules", "zod", "index.cjs"),
    [
      "exports.z = {",
      "  toJSONSchema(schema, params) { return { title: schema.title, params }; },",
      "  prettifyError() { return 'pretty error'; }",
      "};",
    ].join("\n"),
  );
  writeFileSync(
    path.join(root, "schema.ts"),
    [
      "export const schema = {",
      "  title: 'Fixture',",
      "  safeParse(value) { return { success: true, data: value }; },",
      "  encode(value) { return `encoded:${value}`; },",
      "  safeEncode(value) { return { success: true, data: `safe:${value}` }; },",
      "};",
    ].join("\n"),
  );
  return root;
}

function runZod(root: string, ...args: string[]) {
  const result = Bun.spawnSync({
    cmd: ["bun", script, "--root", root, "--module", "schema.ts", ...args],
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(result.stderr.toString());
  }
  return result.stdout.toString();
}

describe("zod-run", () => {
  test("runs encode mode against a schema export", () => {
    const root = createRoot();
    try {
      const stdout = runZod(root, "--mode", "encode", "--input", '"x"');
      expect(JSON.parse(stdout)).toBe("encoded:x");
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  test("passes JSON Schema options through to z.toJSONSchema", () => {
    const root = createRoot();
    try {
      const stdout = runZod(
        root,
        "--mode",
        "toJSONSchema",
        "--target",
        "openapi-3.0",
        "--io",
        "input",
      );
      expect(JSON.parse(stdout)).toEqual({
        title: "Fixture",
        params: { target: "openapi-3.0", io: "input" },
      });
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
