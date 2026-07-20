# runtime-bun-shell

## Why

Bun Shell (`Bun.$`) is a cross-platform, JavaScript-native shell built into Bun. It
replaces brittle `child_process` + shell-string plumbing and behaves the same on Windows,
macOS, and Linux without a system POSIX shell.

## Do

- Use `Bun.$` for scripting and command orchestration inside Bun programs.
- Interpolate values as arguments; Bun Shell escapes them (no manual quoting, no shell
  injection).
- Capture output with `.text()`, `.json()`, or `.quiet()`, or stream it.

## Don't

- Don't hand-build shell command strings by concatenation and pass them to
  `child_process`; Bun Shell escapes interpolations for you.
- Don't assume a system shell is installed; `Bun.$` does not need one.

## Examples

```ts
import { $ } from "bun";

const branch = (await $`git rev-parse --abbrev-ref HEAD`.text()).trim();

const dir = "my files";
await $`ls ${dir}`; // dir is safely escaped as a single argument

const pkg = await $`cat package.json`.json();
```
