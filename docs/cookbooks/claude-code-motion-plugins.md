# Claude Code Motion Plugins

The `web-motion` plugin directory is installable in Claude Code through the
repository marketplace at `.claude-plugin/marketplace.json`. It also keeps its
Codex manifest under `.codex-plugin/`.

> **Native motion moved (expo-motion).** The former `native-motion` plugin and
> its nine Expo/React Native skills (Reanimated, gestures, transitions, Skia,
> NativeWind, validation, Lottie/Rive/R3F) were consolidated into the standalone
> `expo-motion` skill under `skills/expo-motion`, installed via the `skills` CLI
> rather than this plugin marketplace. The retired plugin skill source is
> preserved under `archive/skills/`. (GSAP similarly moved to the standalone
> `gsap` skill — see the migration note below.)

## Install

Add the marketplace from GitHub with sparse checkout so Claude Code fetches only
the marketplace catalog and plugin directories:

```bash
claude plugin marketplace add BjornMelin/dev-skills --sparse .claude-plugin plugins/web-motion plugins/claude-core plugins/design-motion
claude plugin install web-motion@bjorn-dev-skills
```

If Claude Code is already running, reload components after installation:

```text
/reload-plugins
```

Plugin skills are namespaced by plugin name, for example
`/web-motion:web-motion-react`. The standalone `expo-motion` and `gsap` skills
are installed with the `skills` CLI (`skills add BjornMelin/dev-skills -g -s expo-motion`).

> **GSAP migration (web-motion 0.2.0).** This release removed the eight
> `gsap-*` skills (`gsap-core`, `gsap-frameworks`, `gsap-performance`,
> `gsap-plugins`, `gsap-react`, `gsap-scrolltrigger`, `gsap-timeline`, and
> `gsap-utils`) from the `web-motion` plugin. GSAP now lives in the standalone
> `gsap` skill under `skills/gsap`, installed via the `skills` CLI rather than
> this plugin marketplace. The retired plugin skill source is preserved under
> `archive/skills/`. Install GSAP with the `skills` CLI; do not expect it from
> `web-motion` anymore.

## Local Development

Load the source directory directly while iterating:

```bash
claude --plugin-dir ./plugins/web-motion
```

In an active Claude Code session, run `/reload-plugins` after changing plugin
manifests, hooks, agents, MCP/LSP config, or other non-`SKILL.md` components.

## Validate

Run Claude Code validation for the marketplace and the plugin:

```bash
claude plugin validate . --strict
claude plugin validate ./plugins/web-motion --strict
```

Run the repo-native motion plugin gate:

```bash
node plugins/web-motion/scripts/validate-atomic-skills.mjs
for d in plugins/web-motion/skills/*; do
  [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"
done
```

## Release Notes

The Claude manifests use explicit semver versions so
`claude plugin validate --strict` passes. Bump the changed plugin's
`.claude-plugin/plugin.json` version on every Claude Code plugin release, or
users will keep the cached copy even after marketplace updates.
