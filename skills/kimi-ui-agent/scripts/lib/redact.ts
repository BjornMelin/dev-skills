const BUILTIN_PATTERNS: { pattern: RegExp; replacement: string }[] = [
  { pattern: /\bsk-[A-Za-z0-9_-]{12,}\b/g, replacement: "[REDACTED]" },
  { pattern: /\b([A-Za-z0-9_]*TOKEN[A-Za-z0-9_]*=)[^\s]+/gi, replacement: "$1[REDACTED]" },
  { pattern: /\b([A-Za-z0-9_]*KEY[A-Za-z0-9_]*=)[^\s]+/gi, replacement: "$1[REDACTED]" },
  { pattern: /\b(Bearer\s+)[A-Za-z0-9._~+/=-]{12,}/gi, replacement: "$1[REDACTED]" },
  { pattern: /\b(api[_-]?key["']?\s*[:=]\s*["']?)[A-Za-z0-9._~+/=-]{12,}/gi, replacement: "$1[REDACTED]" },
];

export function redact(input: string, extraPatterns: string[] = []): string {
  let output = input;
  for (const { pattern, replacement } of BUILTIN_PATTERNS) {
    output = output.replace(pattern, replacement);
  }
  for (const source of extraPatterns) {
    if (!source.trim()) continue;
    output = output.replaceAll(source, "[REDACTED]");
  }
  return output;
}
