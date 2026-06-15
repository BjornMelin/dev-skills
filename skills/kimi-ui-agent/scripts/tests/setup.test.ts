import { afterEach, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { scanProject, profileWrites } from "../lib/setup";
import { defaultConfig, writeManaged } from "../lib/config";

const temps: string[] = [];

function tempProject(): string {
  const dir = mkdtempSync(join(tmpdir(), "kimi-ui-agent-test-"));
  temps.push(dir);
  return dir;
}

afterEach(() => {
  for (const dir of temps.splice(0)) rmSync(dir, { recursive: true, force: true });
});

describe("project setup", () => {
  test("detects common frontend stack", () => {
    const root = tempProject();
    writeFileSync(
      join(root, "package.json"),
      JSON.stringify({
        scripts: { lint: "eslint .", typecheck: "tsc --noEmit" },
        dependencies: { react: "latest", next: "latest", "@radix-ui/react-slot": "latest" },
        devDependencies: { tailwindcss: "latest" },
      }),
    );
    writeFileSync(join(root, "bun.lock"), "");
    const scan = scanProject(root);
    expect(scan.packageManager).toBe("bun");
    expect(scan.frameworks).toContain("Next.js");
    expect(scan.frameworks).toContain("React");
    expect(scan.styling).toContain("Tailwind CSS");
    expect(scan.validationCommands).toContain("bun run lint");
  });

  test("uses packageManager when lockfiles are absent", () => {
    const cases = [
      ["pnpm@9.15.0", "pnpm", "pnpm lint"],
      ["yarn@4.7.0", "yarn", "yarn lint"],
      ["npm@10.9.0", "npm", "npm run lint"],
      ["bun@1.3.14", "bun", "bun run lint"],
    ] as const;

    for (const [packageManager, detected, validationCommand] of cases) {
      const root = tempProject();
      writeFileSync(
        join(root, "package.json"),
        JSON.stringify({
          packageManager,
          scripts: { lint: "eslint ." },
        }),
      );

      const scan = scanProject(root);
      expect(scan.packageManager).toBe(detected);
      expect(scan.validationCommands).toContain(validationCommand);
    }
  });

  test("scan skips symlinked frontend directories", () => {
    const root = tempProject();
    const outside = tempProject();
    mkdirSync(join(outside, "components"), { recursive: true });
    symlinkSync(join(outside, "components"), join(root, "components"), "dir");

    const scan = scanProject(root);
    expect(scan.componentDirs).not.toContain("components");
  });

  test("scan skips local agent tooling directories", () => {
    const root = tempProject();
    mkdirSync(join(root, "src", "components"), { recursive: true });
    mkdirSync(join(root, ".agents", "skills", "generated", "components"), { recursive: true });
    mkdirSync(join(root, ".kimi-code", "skills", "generated", "ui"), { recursive: true });

    const scan = scanProject(root);
    expect(scan.componentDirs).toContain("src/components");
    expect(scan.componentDirs).not.toContain(".agents/skills/generated/components");
    expect(scan.componentDirs).not.toContain(".kimi-code/skills/generated/ui");
  });

  test("profile writes include config and lean context pack", () => {
    const root = tempProject();
    const writes = profileWrites(root);
    expect(writes.map((write) => write.path)).toContain(".agents/kimi-ui-agent/config.json");
    expect(writes.map((write) => write.path)).toContain(".agents/kimi-ui-agent/project-profile.md");
    expect(writes.map((write) => write.path)).toContain(".agents/kimi-ui-agent/profile.lock.json");
  });

  test("protected paths markdown mirrors configured protected paths", () => {
    const root = tempProject();
    const config = defaultConfig(root);
    const protectedPaths = profileWrites(root, config).find((write) => write.path === ".agents/kimi-ui-agent/protected-paths.md");

    expect(protectedPaths?.content).toBeDefined();
    for (const protectedPath of config.protectedPaths) {
      expect(protectedPaths?.content).toContain(`- ${protectedPath}`);
    }
    expect(protectedPaths?.content).toContain("- **/bun.lock");
    expect(protectedPaths?.content).toContain("- **/bun.lockb");
  });

  test("profile lock omits workstation-local root paths", () => {
    const root = tempProject();
    const lock = profileWrites(root).find((write) => write.path === ".agents/kimi-ui-agent/profile.lock.json");
    expect(lock?.content).toBeDefined();
    expect(lock?.content).not.toContain(root);

    const parsed = JSON.parse(lock?.content || "{}");
    expect(parsed.projectRoot).toBeUndefined();
    expect(parsed.gitRoot).toBeUndefined();
    expect(parsed.scan.projectRoot).toBeUndefined();
  });

  test("profile refresh preserves existing editable markdown", () => {
    const root = tempProject();
    const profileDir = join(root, ".agents", "kimi-ui-agent");
    const profilePath = join(profileDir, "project-profile.md");
    mkdirSync(profileDir, { recursive: true });
    writeFileSync(profilePath, "curated project facts\n", "utf8");

    const writes = profileWrites(root);
    const profileWrite = writes.find((write) => write.path === ".agents/kimi-ui-agent/project-profile.md");
    expect(profileWrite?.action).toBe("skip");

    writeManaged(root, writes, true);
    expect(readFileSync(profilePath, "utf8")).toBe("curated project facts\n");
  });

  test("managed writes reject symlinked ancestors", () => {
    const root = tempProject();
    symlinkSync(join(root, "outside"), join(root, ".agents"), "dir");
    expect(() =>
      writeManaged(
        root,
        [
          {
            path: ".agents/kimi-ui-agent/config.json",
            action: "create",
            reason: "test write",
            content: "{}\n",
          },
        ],
        true,
      ),
    ).toThrow(/refusing symlinked path/);
  });
});
