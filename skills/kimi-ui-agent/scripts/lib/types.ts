export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export type JsonRecord = { [key: string]: JsonValue };

export type CommandContext = {
  skillDir: string;
  cwd: string;
  json: boolean;
};

export type CliResult = {
  ok: boolean;
  command: string;
  message?: string;
  result?: JsonValue;
  diagnostics?: JsonRecord[];
};

export type ProjectConfig = {
  schema: "kimi_ui_agent.project.v1";
  projectName: string;
  createdAt: string;
  updatedAt: string;
  protectedPaths: string[];
  allowedRoots: string[];
  branchPrefix: string;
  artifactDir: string;
  stateScope: "xdg";
  adapters: {
    codex: boolean;
    kimiCode: boolean;
    claudeCode: boolean;
  };
  redaction: {
    extraPatterns: string[];
  };
};

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

export type ManagedWrite = {
  path: string;
  action: "create" | "update" | "skip";
  reason: string;
  content?: string;
};

export type RunRecord = {
  schema: "kimi_ui_agent.run.v1";
  runId: string;
  createdAt: string;
  updatedAt: string;
  state: "planned" | "started" | "waiting" | "ready" | "finalized" | "aborted";
  projectRoot: string;
  worktreePath: string;
  branchName: string;
  task: string;
  artifactDir: string;
};
