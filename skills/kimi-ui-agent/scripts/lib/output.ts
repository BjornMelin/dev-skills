import type { CliResult, JsonValue } from "./types";

export function ok(command: string, result?: JsonValue, message?: string): CliResult {
  const output: CliResult = { ok: true, command };
  if (result !== undefined) output.result = result;
  if (message !== undefined) output.message = message;
  return output;
}

export function fail(command: string, message: string, result?: JsonValue): CliResult {
  const output: CliResult = { ok: false, command, message };
  if (result !== undefined) output.result = result;
  return output;
}

export function printResult(result: CliResult, json: boolean): void {
  if (json) {
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    return;
  }
  if (result.ok) {
    process.stdout.write(`${result.message || "ok"}\n`);
  } else {
    process.stderr.write(`error: ${result.message || "command failed"}\n`);
  }
  if (result.result && typeof result.result === "object") {
    process.stdout.write(`${JSON.stringify(result.result, null, 2)}\n`);
  }
}
