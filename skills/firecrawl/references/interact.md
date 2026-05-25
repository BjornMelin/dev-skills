# Interact

Use `firecrawl interact` only after scraping a page. It controls a live browser
session tied to a scrape ID.

## Quick Start

```bash
firecrawl scrape "https://example.com/login" --profile example-app
firecrawl interact "Click the sign in button"
firecrawl interact "Extract the pricing table"
firecrawl interact stop
```

Explicit scrape ID:

```bash
firecrawl interact -s <scrape-id> "Click the pricing tab"
firecrawl interact <scrape-id> "What is the Pro plan price?"
```

Code execution:

```bash
firecrawl interact -c "await page.title()" --node
firecrawl interact -c "print(await page.title())" --python
firecrawl interact -c "snapshot" --bash
```

## Key Flags

- `-p, --prompt <text>`: AI instruction.
- `-c, --code <code>`: execute browser-session code.
- `-s, --scrape-id <id>`: target a specific scrape.
- `--node`, `--python`, `--bash`: code language.
- `--timeout <seconds>`: 1-300 seconds.
- `-o, --output <path>` and `--json`: save results.

## Output Shape

Prompt interactions are usually text unless `--json` and `-o` are used. Save
results that may be needed later:

```bash
firecrawl interact "Extract visible pricing rows as JSON" --json -o .firecrawl/interact-pricing.json
jq 'keys' .firecrawl/interact-pricing.json
jq '.data // .result // .' .firecrawl/interact-pricing.json
```

For code execution, stdout is the command result. Keep code probes narrow and
save anything large to `.firecrawl/`.

## Profiles

Use `--profile` on scrape to persist cookies and local storage. Use
`--no-save-changes` to read existing profile state without updating it:

```bash
firecrawl scrape "https://app.example.com/dashboard" --profile example-app --no-save-changes
```

## Boundaries

Never use interact for search/discovery. Use `search` or `map` first.
Stop sessions with `firecrawl interact stop` when finished.
