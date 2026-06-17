import { describe, expect, test } from "bun:test";
import { commandExists, resolveInside, safeSegment, slugify } from "../lib/paths";
import { redact } from "../lib/redact";

describe("path safety", () => {
  test("safeSegment rejects traversal", () => {
    expect(() => safeSegment("../bad", "run id")).toThrow();
    expect(() => safeSegment("bad/segment", "run id")).toThrow();
    expect(safeSegment("run-20260615-abc123", "run id")).toBe("run-20260615-abc123");
  });

  test("resolveInside rejects escaped paths", () => {
    expect(resolveInside("/tmp/root", "a/b")).toBe("/tmp/root/a/b");
    expect(() => resolveInside("/tmp/root", "../escape")).toThrow();
  });

  test("slugify keeps stable shell-friendly output", () => {
    expect(slugify("Improve Dashboard UI!")).toBe("improve-dashboard-ui");
  });

  test("commandExists reports available and missing commands", () => {
    expect(commandExists("git")).toBe(true);
    expect(commandExists("definitely-not-a-real-kimi-ui-agent-command")).toBe(false);
  });
});

describe("redaction", () => {
  test("redacts common token shapes", () => {
    const bareSecret = "sk-abcdefghijklmnopqrstuvwxyz";
    expect(redact(`token ${bareSecret}`)).toBe("token [REDACTED]");
    expect(redact(`token ${bareSecret}`)).not.toContain(bareSecret);
    expect(redact("TOKEN=abc123456789secret")).toContain("TOKEN=[REDACTED]");
    expect(redact("Authorization: Bearer abc123456789secret")).toContain("Bearer [REDACTED]");
  });

  test("treats extra redaction patterns as literals", () => {
    expect(redact("literal (a+)+ value", ["(a+)+"])).toBe("literal [REDACTED] value");
    expect(redact("aaaaab", ["(a+)+$"])).toBe("aaaaab");
  });
});
