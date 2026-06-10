#!/usr/bin/env bun
/**
 * Zod v4 audit script (report-only).
 *
 * The audit checks are intentionally heuristic. They flag migration and
 * correctness prompts, not automatic fixes.
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Severity = "error" | "warn" | "info";
type OutputFormat = "text" | "json" | "md";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const skillRoot = path.resolve(__dirname, "..");
const rulesDir = path.join(skillRoot, "rules");

type Check = Readonly<{
  ruleId: string;
  severity: Severity;
  message: string;
  replacementHint: string;
  docsRef?: string | undefined;
  match: (filePath: string, content: string) => readonly Finding[];
}>;

type Finding = Readonly<{
  ruleId: string;
  severity: Severity;
  file: string;
  line: number;
  column: number;
  message: string;
  snippet: string;
  replacementHint: string;
  docsRef?: string | undefined;
}>;

type Args = Readonly<{
  root: string;
  format: OutputFormat;
  failOn?: Severity | undefined;
  includeExts: readonly string[];
  excludeDirs: readonly string[];
  explain?: string | undefined;
  listRules: boolean;
  listChecks: boolean;
}>;

function parseArgs(argv: readonly string[]): Args {
  const args = [...argv];
  const out: {
    root: string;
    format: OutputFormat;
    failOn?: Severity | undefined;
    includeExts: readonly string[];
    excludeDirs: readonly string[];
    explain?: string | undefined;
    listRules: boolean;
    listChecks: boolean;
  } = {
    root: process.cwd(),
    format: "text",
    listRules: false,
    listChecks: false,
    includeExts: ["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"],
    excludeDirs: [
      "node_modules",
      ".next",
      "dist",
      "build",
      "out",
      "coverage",
      ".turbo",
      ".git",
      "opensrc",
    ],
  };

  const take = (flag: string): string | undefined => {
    const idx = args.indexOf(flag);
    if (idx === -1) return undefined;
    const val = args[idx + 1];
    if (!val || val.startsWith("--")) return undefined;
    return val;
  };

  const root = take("--root");
  if (root) out.root = path.resolve(root);

  const format = take("--format");
  if (format === "text" || format === "json" || format === "md") {
    out.format = format;
  }

  const failOn = take("--fail-on");
  if (failOn === "error" || failOn === "warn" || failOn === "info") {
    out.failOn = failOn;
  }

  const include = take("--include-exts");
  if (include) {
    const exts = include
      .split(",")
      .map((x) => x.trim().replace(/^\./, ""))
      .filter((x) => x.length > 0);
    if (exts.length > 0) out.includeExts = exts;
  }

  const exclude = take("--exclude-dirs");
  if (exclude) {
    const dirs = exclude
      .split(",")
      .map((x) => x.trim())
      .filter((x) => x.length > 0);
    if (dirs.length > 0) out.excludeDirs = dirs;
  }

  const explain = take("--explain");
  if (explain) out.explain = explain.trim();

  out.listRules = args.includes("--list-rules");
  out.listChecks = args.includes("--list-checks");

  return out;
}

function severityRank(sev: Severity): number {
  switch (sev) {
    case "error":
      return 3;
    case "warn":
      return 2;
    case "info":
      return 1;
  }
}

function* walkFiles(
  root: string,
  excludeDirs: readonly string[],
): Generator<string> {
  const entries = readdirSync(root, { withFileTypes: true });
  for (const ent of entries) {
    const full = path.join(root, ent.name);
    if (ent.isDirectory()) {
      if (excludeDirs.includes(ent.name)) continue;
      yield* walkFiles(full, excludeDirs);
      continue;
    }
    if (ent.isFile()) yield full;
  }
}

function isIncluded(filePath: string, includeExts: readonly string[]): boolean {
  const ext = path.extname(filePath).toLowerCase().replace(/^\./, "");
  return includeExts.includes(ext);
}

function hasZodImport(content: string): boolean {
  return /\bfrom\s+["']zod(?:\/[^"']*)?["']/.test(content);
}

function lineColFromIndex(content: string, idx: number): Readonly<{
  line: number;
  column: number;
  lineText: string;
}> {
  let line = 1;
  let lastLineStart = 0;
  for (let i = 0; i < idx; i++) {
    if (content.charCodeAt(i) === 10) {
      line++;
      lastLineStart = i + 1;
    }
  }
  const column = idx - lastLineStart + 1;
  const lineEnd = content.indexOf("\n", lastLineStart);
  const lineText =
    lineEnd === -1
      ? content.slice(lastLineStart)
      : content.slice(lastLineStart, lineEnd);
  return { line, column, lineText };
}

function finding(
  filePath: string,
  content: string,
  idx: number,
  meta: Omit<Finding, "file" | "line" | "column" | "snippet">,
): Finding {
  const { line, column, lineText } = lineColFromIndex(content, idx);
  return {
    ...meta,
    file: filePath,
    line,
    column,
    snippet: lineText.trimEnd(),
  };
}

function findAll(
  filePath: string,
  content: string,
  re: RegExp,
  meta: Omit<Finding, "file" | "line" | "column" | "snippet">,
): readonly Finding[] {
  const out: Finding[] = [];
  const rx = re.global ? re : new RegExp(re.source, `${re.flags}g`);
  rx.lastIndex = 0;
  for (;;) {
    const m = rx.exec(content);
    if (!m) break;
    out.push(finding(filePath, content, m.index, meta));
    if (m[0].length === 0) rx.lastIndex++;
  }
  return out;
}

function findObjectOption(
  filePath: string,
  content: string,
  callee: string,
  option: string,
  optionValue: RegExp,
  meta: Omit<Finding, "file" | "line" | "column" | "snippet">,
): readonly Finding[] {
  const out: Finding[] = [];
  const callRe = new RegExp(`\\b${callee.replace(".", "\\.")}\\s*\\(\\s*\\{`, "g");
  for (;;) {
    const m = callRe.exec(content);
    if (!m) break;
    const start = m.index;
    const close = content.indexOf("}", callRe.lastIndex);
    if (close === -1) continue;
    const body = content.slice(callRe.lastIndex, close);
    const optionRe = new RegExp(`\\b${option}\\s*:\\s*${optionValue.source}`, "m");
    const optionMatch = optionRe.exec(body);
    if (optionMatch) {
      out.push(finding(filePath, content, callRe.lastIndex + optionMatch.index, meta));
    }
  }
  return out;
}

function hasTopLevelComma(argumentList: string): boolean {
  let depth = 0;
  let inString: "'" | '"' | "`" | null = null;
  let escaped = false;
  for (const ch of argumentList) {
    if (inString) {
      if (escaped) {
        escaped = false;
        continue;
      }
      if (ch === "\\") {
        escaped = true;
        continue;
      }
      if (ch === inString) inString = null;
      continue;
    }
    if (ch === "'" || ch === '"' || ch === "`") {
      inString = ch;
      continue;
    }
    if (ch === "(" || ch === "[" || ch === "{") depth++;
    if (ch === ")" || ch === "]" || ch === "}") depth--;
    if (ch === "," && depth === 0) return true;
  }
  return false;
}

function singleArgRecordFindings(
  filePath: string,
  content: string,
  meta: Omit<Finding, "file" | "line" | "column" | "snippet">,
): readonly Finding[] {
  const out: Finding[] = [];
  const needle = "z.record(";
  let idx = 0;
  for (;;) {
    const start = content.indexOf(needle, idx);
    if (start === -1) break;
    idx = start + needle.length;

    let i = idx;
    let depth = 0;
    let inString: "'" | '"' | "`" | null = null;
    let escaped = false;
    for (; i < content.length; i++) {
      const ch = content[i] ?? "";
      if (inString) {
        if (escaped) {
          escaped = false;
          continue;
        }
        if (ch === "\\") {
          escaped = true;
          continue;
        }
        if (ch === inString) inString = null;
        continue;
      }
      if (ch === "'" || ch === '"' || ch === "`") {
        inString = ch;
        continue;
      }
      if (ch === "(" || ch === "[" || ch === "{") depth++;
      if (ch === ")") {
        if (depth === 0) break;
        depth--;
        continue;
      }
      if (ch === "]" || ch === "}") depth--;
    }
    if (i >= content.length) continue;
    if (!hasTopLevelComma(content.slice(idx, i))) {
      out.push(finding(filePath, content, start, meta));
    }
  }
  return out;
}

const checks: readonly Check[] = [
  {
    ruleId: "migrate-import-default",
    severity: "info",
    message: "Default import of Zod root detected; named import is preferred by this skill.",
    replacementHint:
      'Prefer `import { z } from "zod"` for consistency. Default import is valid in Zod 4.4.3.',
    match: (filePath, content) =>
      findAll(filePath, content, /\bimport\s+z\s+from\s+["']zod["']/g, {
        ruleId: "migrate-import-default",
        severity: "info",
        message:
          "Default import of Zod root detected; named import is preferred by this skill.",
        replacementHint:
          'Prefer `import { z } from "zod"` for consistency. Default import is valid in Zod 4.4.3.',
        docsRef: path.join(rulesDir, "migrate-import-default.md"),
      }),
  },
  {
    ruleId: "migrate-import-namespace-root",
    severity: "info",
    message: "Namespace import of Zod root detected; named import is the local convention.",
    replacementHint:
      'Prefer `import { z } from "zod"` for app code. Namespace root imports are valid in Zod 4.4.3.',
    match: (filePath, content) =>
      findAll(filePath, content, /\bimport\s+\*\s+as\s+z\s+from\s+["']zod["']/g, {
        ruleId: "migrate-import-namespace-root",
        severity: "info",
        message:
          "Namespace import of Zod root detected; named import is the local convention.",
        replacementHint:
          'Prefer `import { z } from "zod"` for app code. Namespace root imports are valid in Zod 4.4.3.',
        docsRef: path.join(rulesDir, "migrate-import-namespace-root.md"),
      }),
  },
  {
    ruleId: "migrate-nativeenum-to-enum",
    severity: "warn",
    message: "`z.nativeEnum()` is deprecated in Zod v4.",
    replacementHint: "Use `z.enum(MyEnumLike)`.",
    match: (filePath, content) =>
      findAll(filePath, content, /\bz\.nativeEnum\s*\(/g, {
        ruleId: "migrate-nativeenum-to-enum",
        severity: "warn",
        message: "`z.nativeEnum()` is deprecated in Zod v4.",
        replacementHint: "Use `z.enum(MyEnumLike)`.",
        docsRef: path.join(rulesDir, "migrate-nativeenum-to-enum.md"),
      }),
  },
  {
    ruleId: "migrate-object-strict-passthrough-strip",
    severity: "warn",
    message:
      "Legacy object unknown-key method detected (`.strict()` / `.passthrough()` / `.strip()`).",
    replacementHint:
      "Prefer `z.strictObject(shape)` / `z.looseObject(shape)` / default `z.object(shape)` stripping.",
    match: (filePath, content) =>
      hasZodImport(content)
        ? findAll(filePath, content, /\.(strict|passthrough|strip)\s*\(\s*\)/g, {
            ruleId: "migrate-object-strict-passthrough-strip",
            severity: "warn",
            message:
              "Legacy object unknown-key method detected (`.strict()` / `.passthrough()` / `.strip()`).",
            replacementHint:
              "Prefer `z.strictObject(shape)` / `z.looseObject(shape)` / default `z.object(shape)` stripping.",
            docsRef: path.join(rulesDir, "migrate-object-strict-passthrough-strip.md"),
          })
        : [],
  },
  {
    ruleId: "migrate-object-merge-to-extend-shape",
    severity: "warn",
    message: "Legacy `.merge(...)` detected on a file importing Zod.",
    replacementHint:
      "Prefer `.extend(other.shape)` or `z.object({ ...a.shape, ...b.shape })`; use `.safeExtend()` for refined objects.",
    match: (filePath, content) =>
      hasZodImport(content)
        ? findAll(filePath, content, /\.merge\s*\(/g, {
            ruleId: "migrate-object-merge-to-extend-shape",
            severity: "warn",
            message: "Legacy `.merge(...)` detected on a file importing Zod.",
            replacementHint:
              "Prefer `.extend(other.shape)` or `z.object({ ...a.shape, ...b.shape })`; use `.safeExtend()` for refined objects.",
            docsRef: path.join(rulesDir, "migrate-object-merge-to-extend-shape.md"),
          })
        : [],
  },
  {
    ruleId: "object-url-protocols-option-ignored",
    severity: "error",
    message: "`protocols:` passed to `z.url({ ... })` is ignored.",
    replacementHint:
      "Use `z.httpUrl()` for HTTP(S), or `z.url({ protocol: /^https?$/ })`.",
    match: (filePath, content) =>
      findObjectOption(filePath, content, "z.url", "protocols", /[^,}]+/, {
        ruleId: "object-url-protocols-option-ignored",
        severity: "error",
        message: "`protocols:` passed to `z.url({ ... })` is ignored.",
        replacementHint:
          "Use `z.httpUrl()` for HTTP(S), or `z.url({ protocol: /^https?$/ })`.",
        docsRef: path.join(rulesDir, "object-url-protocols-option-ignored.md"),
      }),
  },
  {
    ruleId: "object-url-protocols-option-ignored",
    severity: "error",
    message: "`z.url({ protocol/hostname })` expects RegExp values.",
    replacementHint:
      "Use `protocol: /^https?$/` and `hostname: z.regexes.domain`, or use `z.httpUrl()`.",
    match: (filePath, content) => [
      ...findObjectOption(filePath, content, "z.url", "protocol", /["'][^"']+["']/, {
        ruleId: "object-url-protocols-option-ignored",
        severity: "error",
        message: "`z.url({ protocol/hostname })` expects RegExp values.",
        replacementHint:
          "Use `protocol: /^https?$/` and `hostname: z.regexes.domain`, or use `z.httpUrl()`.",
        docsRef: path.join(rulesDir, "object-url-protocols-option-ignored.md"),
      }),
      ...findObjectOption(filePath, content, "z.url", "hostname", /["'][^"']+["']/, {
        ruleId: "object-url-protocols-option-ignored",
        severity: "error",
        message: "`z.url({ protocol/hostname })` expects RegExp values.",
        replacementHint:
          "Use `protocol: /^https?$/` and `hostname: z.regexes.domain`, or use `z.httpUrl()`.",
        docsRef: path.join(rulesDir, "object-url-protocols-option-ignored.md"),
      }),
    ],
  },
  {
    ruleId: "schema-email-pattern-must-be-regexp",
    severity: "error",
    message: "String `pattern: \"...\"` detected for `z.email({ pattern })`.",
    replacementHint:
      "Use `pattern: /.../` or `pattern: z.regexes.html5Email` / `z.regexes.rfc5322Email`.",
    match: (filePath, content) =>
      findObjectOption(filePath, content, "z.email", "pattern", /["'][^"']+["']/, {
        ruleId: "schema-email-pattern-must-be-regexp",
        severity: "error",
        message: "String `pattern: \"...\"` detected for `z.email({ pattern })`.",
        replacementHint:
          "Use `pattern: /.../` or `pattern: z.regexes.html5Email` / `z.regexes.rfc5322Email`.",
        docsRef: path.join(rulesDir, "schema-email-pattern-must-be-regexp.md"),
      }),
  },
  {
    ruleId: "migrate-z-promise-deprecated",
    severity: "warn",
    message: "`z.promise()` is deprecated in Zod v4.",
    replacementHint:
      "Prefer awaiting the value before validation, then parse the resolved type.",
    match: (filePath, content) =>
      findAll(filePath, content, /\bz\.promise\s*\(/g, {
        ruleId: "migrate-z-promise-deprecated",
        severity: "warn",
        message: "`z.promise()` is deprecated in Zod v4.",
        replacementHint:
          "Prefer awaiting the value before validation, then parse the resolved type.",
        docsRef: path.join(rulesDir, "migrate-z-promise-deprecated.md"),
      }),
  },
  {
    ruleId: "migrate-top-level-string-formats",
    severity: "info",
    message:
      "Deprecated `z.string().<format>()` usage detected; prefer top-level v4 helpers.",
    replacementHint:
      "Use `z.email()`, `z.url()`, `z.uuidv4()`, `z.iso.datetime()`, `z.cidrv4()`, etc.",
    match: (filePath, content) =>
      findAll(
        filePath,
        content,
        /\bz\.string\s*\([^)]*\)\s*\.\s*(base64|base64url|cidr|cidrv4|cidrv6|cuid|cuid2|date|datetime|duration|e164|email|emoji|guid|hex|hostname|ip|ipv4|ipv6|jwt|ksuid|nanoid|time|ulid|url|uuid|uuidv4|uuidv6|uuidv7|xid)\s*\(/g,
        {
          ruleId: "migrate-top-level-string-formats",
          severity: "info",
          message:
            "Deprecated `z.string().<format>()` usage detected; prefer top-level v4 helpers.",
          replacementHint:
            "Use `z.email()`, `z.url()`, `z.uuidv4()`, `z.iso.datetime()`, `z.cidrv4()`, etc.",
          docsRef: path.join(rulesDir, "migrate-top-level-string-formats.md"),
        },
      ),
  },
  {
    ruleId: "migrate-error-format-flatten",
    severity: "warn",
    message:
      "Deprecated ZodError formatting detected (`format()` / `flatten()` / `z.formatError`).",
    replacementHint:
      "Use `z.treeifyError(error)`, `z.flattenError(error)`, or `z.prettifyError(error)`.",
    match: (filePath, content) => [
      ...findAll(filePath, content, /\bz\.formatError\s*\(/g, {
        ruleId: "migrate-error-format-flatten",
        severity: "warn",
        message:
          "Deprecated ZodError formatting detected (`format()` / `flatten()` / `z.formatError`).",
        replacementHint:
          "Use `z.treeifyError(error)`, `z.flattenError(error)`, or `z.prettifyError(error)`.",
        docsRef: path.join(rulesDir, "migrate-error-format-flatten.md"),
      }),
      ...findAll(
        filePath,
        content,
        /\b(?:err|error|zodError|validationError|result\.error|parsed\.error)\s*\.\s*(format|flatten)\s*\(/g,
        {
          ruleId: "migrate-error-format-flatten",
          severity: "warn",
          message:
            "Deprecated ZodError formatting detected (`format()` / `flatten()` / `z.formatError`).",
          replacementHint:
            "Use `z.treeifyError(error)`, `z.flattenError(error)`, or `z.prettifyError(error)`.",
          docsRef: path.join(rulesDir, "migrate-error-format-flatten.md"),
        },
      ),
    ],
  },
  {
    ruleId: "migrate-error-format-flatten",
    severity: "warn",
    message: "`ZodError#errors` was removed; use `.issues`.",
    replacementHint: "Use `error.issues`.",
    match: (filePath, content) =>
      findAll(filePath, content, /\b(?:err|error|zodError|validationError|result\.error|parsed\.error)\s*\.\s*errors\b/g, {
        ruleId: "migrate-error-format-flatten",
        severity: "warn",
        message: "`ZodError#errors` was removed; use `.issues`.",
        replacementHint: "Use `error.issues`.",
        docsRef: path.join(rulesDir, "migrate-error-format-flatten.md"),
      }),
  },
  {
    ruleId: "migrate-error-customization-to-unified-error",
    severity: "warn",
    message:
      "Legacy Zod v3 error customization detected (`invalid_type_error`, `required_error`, `errorMap`, or `{ message }`).",
    replacementHint:
      "Use the v4 unified `error` parameter (string or function).",
    match: (filePath, content) => {
      const docsRef = path.join(
        rulesDir,
        "migrate-error-customization-to-unified-error.md",
      );
      return [
        ...findAll(filePath, content, /\binvalid_type_error\b/g, {
          ruleId: "migrate-error-customization-to-unified-error",
          severity: "warn",
          message:
            "Legacy Zod v3 error customization detected (`invalid_type_error`, `required_error`, `errorMap`, or `{ message }`).",
          replacementHint:
            "Use the v4 unified `error` parameter (string or function).",
          docsRef,
        }),
        ...findAll(filePath, content, /\brequired_error\b/g, {
          ruleId: "migrate-error-customization-to-unified-error",
          severity: "warn",
          message:
            "Legacy Zod v3 error customization detected (`invalid_type_error`, `required_error`, `errorMap`, or `{ message }`).",
          replacementHint:
            "Use the v4 unified `error` parameter (string or function).",
          docsRef,
        }),
        ...findAll(filePath, content, /\berrorMap\b/g, {
          ruleId: "migrate-error-customization-to-unified-error",
          severity: "warn",
          message:
            "Legacy Zod v3 error customization detected (`invalid_type_error`, `required_error`, `errorMap`, or `{ message }`).",
          replacementHint:
            "Use the v4 unified `error` parameter (string or function).",
          docsRef,
        }),
        ...findAll(filePath, content, /\.\w+\s*\(\s*[^)]*,\s*\{[^}]*\bmessage\s*:/g, {
          ruleId: "migrate-error-customization-to-unified-error",
          severity: "warn",
          message:
            "Legacy Zod v3 error customization detected (`invalid_type_error`, `required_error`, `errorMap`, or `{ message }`).",
          replacementHint:
            "Use the v4 unified `error` parameter (string or function).",
          docsRef,
        }),
      ];
    },
  },
  {
    ruleId: "migrate-record-value-only-signature",
    severity: "info",
    message:
      "One-argument `z.record(valueSchema)` detected; explicit key schemas are preferred.",
    replacementHint:
      "Prefer `z.record(z.string(), valueSchema)`. One-arg record remains compatible in Zod 4.4.3.",
    match: (filePath, content) =>
      singleArgRecordFindings(filePath, content, {
        ruleId: "migrate-record-value-only-signature",
        severity: "info",
        message:
          "One-argument `z.record(valueSchema)` detected; explicit key schemas are preferred.",
        replacementHint:
          "Prefer `z.record(z.string(), valueSchema)`. One-arg record remains compatible in Zod 4.4.3.",
        docsRef: path.join(rulesDir, "migrate-record-value-only-signature.md"),
      }),
  },
  {
    ruleId: "schema-use-stringbool-for-boolish",
    severity: "info",
    message: "`z.coerce.boolean()` detected; it treats most non-empty strings as true.",
    replacementHint:
      "Use `z.stringbool()` for env vars, query strings, and boolish string inputs.",
    match: (filePath, content) =>
      findAll(filePath, content, /\bz\.coerce\.boolean\s*\(/g, {
        ruleId: "schema-use-stringbool-for-boolish",
        severity: "info",
        message:
          "`z.coerce.boolean()` detected; it treats most non-empty strings as true.",
        replacementHint:
          "Use `z.stringbool()` for env vars, query strings, and boolish string inputs.",
        docsRef: path.join(rulesDir, "schema-use-stringbool-for-boolish.md"),
      }),
  },
  {
    ruleId: "schema-avoid-z-any",
    severity: "info",
    message: "`z.any()` detected.",
    replacementHint:
      "Prefer `z.unknown()` unless callers truly need `any` to flow through TypeScript.",
    match: (filePath, content) =>
      findAll(filePath, content, /\bz\.any\s*\(/g, {
        ruleId: "schema-avoid-z-any",
        severity: "info",
        message: "`z.any()` detected.",
        replacementHint:
          "Prefer `z.unknown()` unless callers truly need `any` to flow through TypeScript.",
        docsRef: path.join(rulesDir, "schema-avoid-z-any.md"),
      }),
  },
];

function listRuleIds(): readonly string[] {
  return readdirSync(rulesDir)
    .filter((f) => f.endsWith(".md"))
    .filter((f) => f !== "_index.md")
    .map((f) => f.replace(/\.md$/, ""))
    .sort((a, b) => a.localeCompare(b));
}

function formatText(findings: readonly Finding[], root: string): string {
  if (findings.length === 0) return "No findings.\n";
  const lines: string[] = [];
  for (const f of findings) {
    const rel = path.relative(root, f.file);
    lines.push(
      `${rel}:${f.line}:${f.column} [${f.severity}] ${f.ruleId}: ${f.message}`,
    );
    lines.push(`  ${f.snippet.trim()}`);
    lines.push(`  Fix: ${f.replacementHint}`);
    if (f.docsRef) lines.push(`  Docs: ${f.docsRef}`);
  }
  return `${lines.join("\n")}\n`;
}

function escapeMd(s: string): string {
  return s.replace(/\|/g, "\\|");
}

function formatMd(findings: readonly Finding[], root: string): string {
  if (findings.length === 0) return "No findings.\n";
  const lines: string[] = [];
  lines.push("| Location | Severity | Rule | Message | Fix |");
  lines.push("| --- | --- | --- | --- | --- |");
  for (const f of findings) {
    const rel = path.relative(root, f.file);
    const loc = `${rel}:${f.line}:${f.column}`;
    lines.push(
      `| \`${escapeMd(loc)}\` | \`${f.severity}\` | \`${f.ruleId}\` | ${escapeMd(
        f.message,
      )} | ${escapeMd(f.replacementHint)} |`,
    );
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

function printHelp(): void {
  process.stdout.write(
    [
      "zod-audit (report-only)",
      "",
      "Usage:",
      "  bun <skill>/scripts/zod-audit.ts --root <dir> --format text",
      "",
      "Options:",
      "  --root <dir>             Root directory to scan (default: cwd)",
      "  --format text|json|md     Output format (default: text)",
      "  --fail-on error|warn|info Exit 1 if any finding at/above severity",
      "  --include-exts ts,tsx,... Comma-separated extensions to scan",
      "  --exclude-dirs a,b,c      Comma-separated directory names to skip",
      "  --explain <ruleId>        Print rule doc from this skill and exit",
      "  --list-rules              Print all documented rule IDs and exit",
      "  --list-checks             Print rule IDs implemented by this scanner and exit",
      "  --help, -h                Show this help",
      "",
    ].join("\n"),
  );
}

function run(args: Args): void {
  if (args.listRules) {
    process.stdout.write(`${listRuleIds().join("\n")}\n`);
    return;
  }

  if (args.listChecks) {
    const ids = Array.from(new Set(checks.map((check) => check.ruleId))).sort(
      (a, b) => a.localeCompare(b),
    );
    process.stdout.write(`${ids.join("\n")}\n`);
    return;
  }

  if (args.explain) {
    const p = path.join(rulesDir, `${args.explain}.md`);
    try {
      const content = readFileSync(p, "utf8");
      process.stdout.write(content.endsWith("\n") ? content : `${content}\n`);
    } catch {
      // eslint-disable-next-line no-console
      console.error(`Rule doc not found: ${p}`);
      process.exit(2);
    }
    return;
  }

  const files = Array.from(walkFiles(args.root, args.excludeDirs)).filter((f) =>
    isIncluded(f, args.includeExts),
  );

  const findings: Finding[] = [];
  for (const filePath of files) {
    let content: string;
    try {
      content = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }

    for (const check of checks) {
      findings.push(...check.match(filePath, content));
    }
  }

  findings.sort((a, b) => {
    const sr = severityRank(b.severity) - severityRank(a.severity);
    if (sr !== 0) return sr;
    if (a.file !== b.file) return a.file.localeCompare(b.file);
    if (a.line !== b.line) return a.line - b.line;
    return a.column - b.column;
  });

  if (args.format === "json") {
    process.stdout.write(`${JSON.stringify({ findings }, null, 2)}\n`);
  } else if (args.format === "md") {
    process.stdout.write(formatMd(findings, args.root));
  } else {
    process.stdout.write(formatText(findings, args.root));
  }

  if (args.failOn) {
    const threshold = severityRank(args.failOn);
    if (findings.some((f) => severityRank(f.severity) >= threshold)) {
      process.exit(1);
    }
  }
}

function ensureRootExists(root: string): void {
  const st = statSync(root, { throwIfNoEntry: false });
  if (!st || !st.isDirectory()) {
    // eslint-disable-next-line no-console
    console.error(`Invalid --root (not a directory): ${root}`);
    process.exit(2);
  }
}

{
  const raw = process.argv.slice(2);
  if (raw.includes("--help") || raw.includes("-h")) {
    printHelp();
    process.exit(0);
  }

  const args = parseArgs(raw);
  if (!args.explain && !args.listRules && !args.listChecks) {
    ensureRootExists(args.root);
  }
  run(args);
}
