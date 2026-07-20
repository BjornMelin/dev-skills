# tooling-bunfig

## Why

`bunfig.toml` is Bun's optional project configuration. Centralizing install, run, and
test defaults there keeps behavior consistent across machines and CI without repeating
flags on every command.

## Do

- Place `bunfig.toml` in the project root alongside `package.json` (optional; Bun works
  without it).
- Configure the settings you rely on repeatedly, for example:
  ```toml
  [install]
  linker = "isolated"   # or "hoisted"; see pm-linker-and-streaming-install

  [run]
  bun = true            # alias node -> bun for scripts; see runtime-bun-run-bun-flag

  [test]
  coverage = true
  ```
- Keep runtime-affecting choices (linker, `run.bun`) in `bunfig.toml` rather than
  scattering them across scripts.

## Don't

- Don't duplicate the same flags in every script when a `bunfig.toml` default will do.
- Don't commit machine-specific or secret values; `bunfig.toml` is shared config.

## Reference

See `references/ref-bun-cli-cheatsheet.md` and the official docs
(<https://bun.com/docs/runtime/bunfig>) for the full option set.
