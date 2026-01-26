import path from "node:path";
import { Agent, run, MCPServerStdio } from "@openai/agents";

// Minimal MCP stdio example (TypeScript):
// - Connect to a local MCP server (filesystem shown here)
// - Attach it to an agent
// - Run a query that uses tools

const dataDir = path.join(process.cwd(), "data");

const filesystemServer = new MCPServerStdio({
  name: "Filesystem MCP Server",
  fullCommand: `npx -y @modelcontextprotocol/server-filesystem ${dataDir}`,
});

await filesystemServer.connect();

try {
  const agent = new Agent({
    name: "File Assistant",
    instructions: "Use available tools to read files and answer questions.",
    mcpServers: [filesystemServer],
  });

  const result = await run(agent, "List files under ./data and summarize markdown contents.");
  console.log(result.finalOutput);
} finally {
  await filesystemServer.close();
}

