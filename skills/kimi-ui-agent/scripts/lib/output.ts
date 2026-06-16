import type { CliResult, JsonValue } from "./types";

/**
 * Builds a successful CLI result envelope.
 *
 * @param command - Command name being reported.
 * @param result - Optional structured command payload.
 * @param message - Optional human-readable message.
 * @returns Standard successful CLI result.
 */
export function ok(command: string, result?: JsonValue, message?: string): CliResult {
  const output: CliResult = { ok: true, command };
  if (result !== undefined) output.result = result;
  if (message !== undefined) output.message = message;
  return output;
}

/**
 * Builds a failed CLI result envelope.
 *
 * @param command - Command name being reported.
 * @param message - Human-readable failure reason.
 * @param result - Optional structured failure payload.
 * @returns Standard failed CLI result.
 */
export function fail(command: string, message: string, result?: JsonValue): CliResult {
  const output: CliResult = { ok: false, command, message };
  if (result !== undefined) output.result = result;
  return output;
}

/**
 * Writes a CLI result in JSON or human-readable form.
 *
 * @param result - Result envelope to print.
 * @param json - Whether to emit machine-readable JSON.
 */
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
