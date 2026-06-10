import { describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const script = path.resolve(import.meta.dir, "..", "zod-audit.ts");

function runAudit(root: string, ...args: string[]) {
  const result = Bun.spawnSync({
    cmd: ["bun", script, "--root", root, "--format", "json", ...args],
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(result.stderr.toString());
  }
  return JSON.parse(result.stdout.toString()) as {
    findings: Array<{ ruleId: string; severity: string; message: string }>;
  };
}

describe("zod-audit", () => {
  test("classifies invalid, deprecated, and advisory Zod v4 findings", () => {
    const root = mkdtempSync(path.join(tmpdir(), "zod-audit-"));
    try {
      writeFileSync(
        path.join(root, "schema.ts"),
        [
          'import * as z from "zod";',
          "const Url = z.url({ protocols: ['http'] });",
          'const Email = z.email({ pattern: "gmail" });',
          "const Legacy = z.string().uuidv7();",
          "const Native = z.nativeEnum({ A: 'a' });",
          "const Rec = z.record(z.string());",
          "const Bool = z.coerce.boolean();",
          "err.flatten();",
          "error.errors;",
        ].join("\n"),
      );

      const findings = runAudit(root).findings;
      const severities = new Map(findings.map((f) => [f.ruleId, f.severity]));

      expect(severities.get("object-url-protocols-option-ignored")).toBe(
        "error",
      );
      expect(severities.get("schema-email-pattern-must-be-regexp")).toBe(
        "error",
      );
      expect(severities.get("migrate-nativeenum-to-enum")).toBe("warn");
      expect(severities.get("migrate-error-format-flatten")).toBe("warn");
      expect(severities.get("migrate-import-namespace-root")).toBe("info");
      expect(severities.get("migrate-record-value-only-signature")).toBe(
        "info",
      );
      expect(severities.get("schema-use-stringbool-for-boolish")).toBe("info");
      expect(severities.get("migrate-top-level-string-formats")).toBe("info");
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  test("distinguishes documented rules from implemented checks", () => {
    const rules = Bun.spawnSync({
      cmd: ["bun", script, "--list-rules"],
      stdout: "pipe",
    }).stdout.toString();
    const checks = Bun.spawnSync({
      cmd: ["bun", script, "--list-checks"],
      stdout: "pipe",
    }).stdout.toString();

    expect(rules).toContain("jsonschema-tojsonschema-options");
    expect(checks).toContain("migrate-top-level-string-formats");
    expect(checks).not.toContain("jsonschema-tojsonschema-options");
  });
});
