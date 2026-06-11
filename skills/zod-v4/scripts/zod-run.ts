#!/usr/bin/env bun
/**
 * Load a schema from a module and run common Zod v4 operations against it.
 *
 * Notes:
 * - This script executes the target module. Avoid modules with side effects.
 * - Zod is resolved from `--root` (default: cwd).
 */

import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";

type Mode =
  | "parse"
  | "safeParse"
  | "parseAsync"
  | "safeParseAsync"
  | "decode"
  | "safeDecode"
  | "decodeAsync"
  | "safeDecodeAsync"
  | "encode"
  | "safeEncode"
  | "encodeAsync"
  | "safeEncodeAsync"
  | "toJSONSchema";

type Args = Readonly<{
  root: string;
  modulePath: string;
  exportName?: string | undefined;
  mode: Mode;
  inputJson?: string | undefined;
  inputFile?: string | undefined;
  target?: string | undefined;
  io?: "input" | "output" | undefined;
  unrepresentable?: "throw" | "any" | undefined;
  cycles?: "ref" | "throw" | undefined;
  reused?: "inline" | "ref" | undefined;
}>;

const MODES: readonly Mode[] = [
  "parse",
  "safeParse",
  "parseAsync",
  "safeParseAsync",
  "decode",
  "safeDecode",
  "decodeAsync",
  "safeDecodeAsync",
  "encode",
  "safeEncode",
  "encodeAsync",
  "safeEncodeAsync",
  "toJSONSchema",
];

function printHelp(): void {
  process.stdout.write(
    [
      "zod-run",
      "",
      "Usage:",
      "  bun <skill>/scripts/zod-run.ts --module <file> [--export Name] [--mode safeParse] (--input <json> | --input-file <path>)",
      "",
      "Options:",
      "  --root <dir>              Project root for resolving zod + module (default: cwd)",
      "  --module <file>           Module file to import (ts/js), relative to --root",
      "  --export <name>           Export name (default: auto: default|schema|Schema)",
      `  --mode ${MODES.join("|")}`,
      "  --input <json>            JSON string input",
      "  --input-file <path>       Read JSON input from file",
      "",
      "toJSONSchema options:",
      "  --target <value>          draft-2020-12|draft-7|draft-4|openapi-3.0|...",
      "  --io input|output         Default output",
      "  --unrepresentable throw|any",
      "  --cycles ref|throw",
      "  --reused inline|ref",
      "",
    ].join("\n"),
  );
}

function parseArgs(argv: readonly string[]): Args {
  const args = [...argv];

  const take = (flag: string): string | undefined => {
    const idx = args.indexOf(flag);
    if (idx === -1) return undefined;
    const val = args[idx + 1];
    if (!val || val.startsWith("--")) return undefined;
    return val;
  };

  const root = path.resolve(take("--root") ?? process.cwd());
  const modulePath = take("--module");
  if (!modulePath) throw new Error("Missing required flag: --module");

  const modeRaw = take("--mode") ?? "safeParse";
  if (!MODES.includes(modeRaw as Mode)) {
    throw new Error(`Invalid --mode: ${modeRaw}`);
  }

  const ioRaw = take("--io");
  const io = ioRaw === "input" || ioRaw === "output" ? ioRaw : undefined;

  const unrepRaw = take("--unrepresentable");
  const unrepresentable =
    unrepRaw === "throw" || unrepRaw === "any" ? unrepRaw : undefined;

  const cyclesRaw = take("--cycles");
  const cycles =
    cyclesRaw === "ref" || cyclesRaw === "throw" ? cyclesRaw : undefined;

  const reusedRaw = take("--reused");
  const reused =
    reusedRaw === "inline" || reusedRaw === "ref" ? reusedRaw : undefined;

  return {
    root,
    modulePath,
    exportName: take("--export"),
    mode: modeRaw as Mode,
    inputJson: take("--input"),
    inputFile: take("--input-file"),
    target: take("--target"),
    io,
    unrepresentable,
    cycles,
    reused,
  };
}

function needsInput(mode: Mode): boolean {
  return mode !== "toJSONSchema";
}

function readInput(args: Args): unknown {
  if (!needsInput(args.mode)) return undefined;

  const raw =
    args.inputJson ??
    (args.inputFile
      ? readFileSync(path.resolve(args.root, args.inputFile), "utf8")
      : undefined);

  if (!raw) {
    throw new Error(
      "Missing input. Provide --input or --input-file (not needed for --mode toJSONSchema).",
    );
  }

  return JSON.parse(raw);
}

function pickSchemaExport(
  mod: Record<string, unknown>,
  exportName?: string,
): unknown {
  if (exportName) return mod[exportName];
  if ("schema" in mod) return mod.schema;
  if ("Schema" in mod) return mod.Schema;
  if ("default" in mod) return mod.default;
  return undefined;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function requireMethod<T extends (...args: never[]) => unknown>(
  owner: Record<string, unknown>,
  name: string,
): T {
  const method = owner[name];
  if (typeof method !== "function") {
    throw new Error(`Schema export does not have ${name}(...)`);
  }
  return method as T;
}

async function runSchemaMode(schema: Record<string, unknown>, mode: Mode, input: unknown): Promise<unknown> {
  switch (mode) {
    case "parse":
      return requireMethod<(value: unknown) => unknown>(schema, "parse").call(
        schema,
        input,
      );
    case "safeParse":
      return requireMethod<(value: unknown) => unknown>(
        schema,
        "safeParse",
      ).call(schema, input);
    case "parseAsync":
      return await requireMethod<(value: unknown) => Promise<unknown>>(
        schema,
        "parseAsync",
      ).call(schema, input);
    case "safeParseAsync":
      return await requireMethod<(value: unknown) => Promise<unknown>>(
        schema,
        "safeParseAsync",
      ).call(schema, input);
    case "decode":
      return requireMethod<(value: unknown) => unknown>(schema, "decode").call(
        schema,
        input,
      );
    case "safeDecode":
      return requireMethod<(value: unknown) => unknown>(
        schema,
        "safeDecode",
      ).call(schema, input);
    case "decodeAsync":
      return await requireMethod<(value: unknown) => Promise<unknown>>(
        schema,
        "decodeAsync",
      ).call(schema, input);
    case "safeDecodeAsync":
      return await requireMethod<(value: unknown) => Promise<unknown>>(
        schema,
        "safeDecodeAsync",
      ).call(schema, input);
    case "encode":
      return requireMethod<(value: unknown) => unknown>(schema, "encode").call(
        schema,
        input,
      );
    case "safeEncode":
      return requireMethod<(value: unknown) => unknown>(
        schema,
        "safeEncode",
      ).call(schema, input);
    case "encodeAsync":
      return await requireMethod<(value: unknown) => Promise<unknown>>(
        schema,
        "encodeAsync",
      ).call(schema, input);
    case "safeEncodeAsync":
      return await requireMethod<(value: unknown) => Promise<unknown>>(
        schema,
        "safeEncodeAsync",
      ).call(schema, input);
    case "toJSONSchema":
      throw new Error("toJSONSchema is handled separately");
  }
}

async function main(): Promise<void> {
  const raw = process.argv.slice(2);
  if (raw.includes("--help") || raw.includes("-h")) {
    printHelp();
    return;
  }

  const args = parseArgs(raw);
  const requireFromRoot = createRequire(path.join(args.root, "package.json"));
  const zod = requireFromRoot("zod") as unknown;
  if (!isObject(zod) || !("z" in zod)) {
    throw new Error(
      `Could not resolve Zod from --root (${args.root}). Is 'zod' installed?`,
    );
  }
  const z = (zod as { z: unknown }).z;

  const absModulePath = path.resolve(args.root, args.modulePath);
  const mod = (await import(pathToFileURL(absModulePath).href)) as Record<
    string,
    unknown
  >;
  const schema = pickSchemaExport(mod, args.exportName);
  if (!schema) {
    const keys = Object.keys(mod).sort().join(", ");
    throw new Error(
      `Could not find schema export. Provided --export ${String(
        args.exportName,
      )}. Available exports: ${keys}`,
    );
  }

  if (!isObject(schema)) {
    throw new Error("Export is not a schema object.");
  }

  if (args.mode === "toJSONSchema") {
    if (
      !isObject(z) ||
      !("toJSONSchema" in z) ||
      typeof (z as { toJSONSchema: unknown }).toJSONSchema !== "function"
    ) {
      throw new Error("Resolved Zod does not expose z.toJSONSchema");
    }

    const params: Record<string, unknown> = {};
    if (args.target) params.target = args.target;
    if (args.io) params.io = args.io;
    if (args.unrepresentable) params.unrepresentable = args.unrepresentable;
    if (args.cycles) params.cycles = args.cycles;
    if (args.reused) params.reused = args.reused;

    const jsonSchema = (
      z as { toJSONSchema: (s: unknown, p?: unknown) => unknown }
    ).toJSONSchema(schema, Object.keys(params).length > 0 ? params : undefined);
    process.stdout.write(`${JSON.stringify(jsonSchema, null, 2)}\n`);
    return;
  }

  const result = await runSchemaMode(schema, args.mode, readInput(args));
  process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);

  if (
    (args.mode === "safeParse" || args.mode === "safeParseAsync") &&
    isObject(result) &&
    result.success === false &&
    "error" in result &&
    isObject((result as { error: unknown }).error) &&
    isObject(z) &&
    "prettifyError" in z &&
    typeof (z as { prettifyError: unknown }).prettifyError === "function"
  ) {
    const pretty = (z as { prettifyError: (e: unknown) => string }).prettifyError(
      (result as { error: unknown }).error,
    );
    process.stdout.write(`\n--- prettifyError ---\n${pretty}\n`);
  }
}

main().catch((err) => {
  // eslint-disable-next-line no-console
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
