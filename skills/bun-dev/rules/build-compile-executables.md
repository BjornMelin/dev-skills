# build-compile-executables

## Why

`bun build --compile` can produce a standalone executable, useful for CLIs and some deployment targets.

## Do

- Use `--compile` for CLI tools and small services where a single binary is valuable.
- Cross-compile when targeting different platforms.

## Don't

- Don’t assume a compiled binary is always smaller/faster than a bundle; verify per target.

## Examples

```bash
bun build ./src/cli.ts --compile --outfile mycli
```

Cross-compile:

```bash
bun build ./src/cli.ts --compile --target=bun-linux-x64 --outfile mycli-linux
```

