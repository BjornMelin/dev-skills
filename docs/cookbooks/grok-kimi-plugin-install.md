# Grok and Kimi Plugin Install

This repository ships two install surfaces beyond Claude Code and Codex: the
Grok marketplace catalog at `.grok-plugin/marketplace.json` and the Kimi
plugin manifest at `plugins/design-motion/kimi.plugin.json`. Every step below
traces to the linked official runtime docs; no wrapper-specific commands are
invented here.

## Grok Build

The catalog follows the
[xAI marketplace spec](https://github.com/xai-org/plugin-marketplace):
required `name` plus `source` on every entry, `{ "type": "local", "path": ... }`
sources for the vendored `plugins/` directories, and a recommended
`description` on each plugin. Its three entries intentionally mirror
`.claude-plugin/marketplace.json`.

1. Register this repository as a marketplace source via
   `[[marketplace.sources]]` in `~/.grok/config.toml` or
   `~/.grok/plugins/known_marketplaces.json` (source mechanism per the
   [official skills/plugins/marketplaces page](https://docs.x.ai/build/features/skills-plugins-marketplaces)).
2. Browse the catalog in the TUI Marketplace tab, or run
   `grok plugin marketplace list`.
3. Install with `grok plugin install <name> --trust`
   (or `/marketplace` then `i` inside Grok Build).
4. Validate: the `bjorn-dev-skills` entries appear in the marketplace list,
   installed files land under `~/.grok/plugins/marketplaces/`, and the
   plugin's `skills/` directory loads like any other skill source.

Note: Grok reads Claude Code marketplaces, plugins, and skills with zero
configuration, so the mirrored entries resolve the same files as their Claude
counterparts.

## Kimi Code CLI

The manifest lives at the plugin root (`plugins/design-motion/kimi.plugin.json`),
which takes precedence per the
[official plugins doc](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/plugins).
It declares `skills: ./skills/` (`design-motion-audit`, `r3f-scene-polish`),
`agents: ./agents/` (seven specialists), and the display interface block.

1. Clone the repo: `git clone https://github.com/BjornMelin/dev-skills`.
2. Install from the local directory:
   `/plugins install <clone>/plugins/design-motion`.
3. Activate with `/reload` or `/new` (required: plugin changes apply after
   reload or in new sessions).
4. Validate: `/plugins list` shows `design-motion`, and
   `/plugins info design-motion` reports no manifest diagnostics.

Notes: installs are per-user and apply to all projects; the CLI runs from the
managed copy under `$KIMI_CODE_HOME/plugins/managed/<id>/`, so reinstall after
editing the source. The plugin is a mirror: its skills route single-stack work
to `expo-motion`, `web-three-r3f`, and `gsap` from the full catalog, so
standalone Kimi installs needing those should install them separately.
