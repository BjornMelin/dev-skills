/** JSON-compatible value emitted by the CLI or persisted in generated artifacts. */
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

/** JSON object record used for diagnostics and structured command results. */
export type JsonRecord = { [key: string]: JsonValue };

/** Shared runtime context passed to command handlers. */
export type CommandContext = {
  skillDir: string;
  cwd: string;
  json: boolean;
};

/** Standard command response shape for human and JSON output modes. */
export type CliResult = {
  ok: boolean;
  command: string;
  message?: string;
  result?: JsonValue;
  diagnostics?: JsonRecord[];
};

/** Project-local Kimi UI Agent policy written by setup. */
export type ProjectConfig = {
  schema: "kimi_ui_agent.project.v1";
  projectName: string;
  createdAt: string;
  updatedAt: string;
  protectedPaths: string[];
  branchPrefix: string;
  redaction: {
    extraPatterns: string[];
  };
};

/** Repository scan result used to generate project setup context. */
export type ProjectScan = {
  projectRoot: string;
  projectName: string;
  packageManager: string | null;
  frameworks: string[];
  uiLibraries: string[];
  styling: string[];
  appDirs: string[];
  componentDirs: string[];
  validationCommands: string[];
  protectedPathHints: string[];
};

/** Planned file mutation returned by dry-run setup, install, and run commands. */
export type ManagedWrite = {
  path: string;
  action: "create" | "update" | "skip";
  reason: string;
  content?: string;
};

/** Durable parent-controller state for a delegated Kimi UI Agent run. */
export type RunRecord = {
  schema: "kimi_ui_agent.run.v1";
  runId: string;
  createdAt: string;
  updatedAt: string;
  state: "planned" | "waiting" | "ready" | "finalized" | "aborted";
  projectRoot: string;
  worktreePath: string;
  branchName: string;
  task: string;
  artifactDir: string;
};
