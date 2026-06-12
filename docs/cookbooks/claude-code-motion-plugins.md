# Claude Code Motion Plugins

The `web-motion` and `native-motion` plugin directories are installable in
Claude Code through the repository marketplace at `.claude-plugin/marketplace.json`.
The same plugin directories still keep their Codex manifests under
`.codex-plugin/`.

## Install

Add the marketplace from GitHub with sparse checkout so Claude Code fetches only
the marketplace catalog and motion plugin directories:

```bash
claude plugin marketplace add BjornMelin/dev-skills --sparse .claude-plugin plugins/web-motion plugins/native-motion
claude plugin install web-motion@bjorn-dev-skills
claude plugin install native-motion@bjorn-dev-skills
```

If Claude Code is already running, reload components after installation:

```text
/reload-plugins
```

Plugin skills are namespaced by plugin name, for example
`/web-motion:gsap-core` and `/native-motion:native-motion-core`.

## Local Development

Load the source directories directly while iterating:

```bash
claude --plugin-dir ./plugins/web-motion --plugin-dir ./plugins/native-motion
```

In an active Claude Code session, run `/reload-plugins` after changing plugin
manifests, hooks, agents, MCP/LSP config, or other non-`SKILL.md` components.

## Validate

Run Claude Code validation for the marketplace and both plugins:

```bash
claude plugin validate . --strict
claude plugin validate ./plugins/web-motion --strict
claude plugin validate ./plugins/native-motion --strict
```

Run the repo-native motion plugin gates:

```bash
node plugins/web-motion/scripts/validate-atomic-skills.mjs
node plugins/native-motion/scripts/validate-atomic-skills.mjs
for d in plugins/web-motion/skills/* plugins/native-motion/skills/*; do
  [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"
done
```

## Release Notes

The Claude manifests use explicit semver versions so
`claude plugin validate --strict` passes. Bump each changed plugin's
`.claude-plugin/plugin.json` version on every Claude Code plugin release, or
users will keep the cached copy even after marketplace updates.
