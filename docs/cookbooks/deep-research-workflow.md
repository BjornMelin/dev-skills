# Cookbook: Deep Research Workflow

Use this when a decision needs current, defensible, multi-source evidence.

## Scenario

You need to decide whether to adopt or update a library, tool, pattern, or
agent workflow. The answer depends on current docs, source behavior, GitHub
history, and possibly rendered web pages.

## Steps

1. Inspect readiness:

   ```bash
   codex-research doctor
   ```

2. Plan depth:

   ```bash
   codex-research plan "Should this repo adopt <technology/pattern>?" --profile deep
   codex-research run init "Should this repo adopt <technology/pattern>?" --profile deep --topic dependency --out .codex/research/run.json
   ```

3. Use Codex-native web tools first for current official docs and high-level
   facts.

4. Query Context7 for library docs:

   ```bash
   codex-research context7 search --library "<library>" --query "<specific question>" --run .codex/research/run.json
   codex-research context7 context --library-id "/org/project" --query "<specific question>" --run .codex/research/run.json
   ```

5. Search GitHub narrowly:

   ```bash
   codex-research github search-repos "<library> official repo in:name" --per-page 5 --run .codex/research/run.json
   codex-research github search-code 'repo:owner/repo <symbol-or-config> in:file' --per-page 5 --run .codex/research/run.json
   codex-research github search-issues 'repo:owner/repo "<error or behavior>" is:issue' --per-page 5 --run .codex/research/run.json
   ```

6. Validate implementation:

   - use `$opensrc` for package source;
   - use `codex-research github file` for exact files;
   - clone and `rg` only when API evidence is insufficient.

7. Probe rendered pages before escalating:

   ```bash
   codex-research fetch probe "https://example.com/docs/page" --run .codex/research/run.json
   ```

8. Use Firecrawl only when the probe or task warrants it:

   ```bash
   codex-research fetch firecrawl "https://example.com/docs/page" --fresh --privacy public --run .codex/research/run.json
   ```

9. Record evidence:

   ```bash
   codex-research ledger init
   codex-research ledger add-source --from-cache <source-id>
   codex-research ledger add-source --provider <provider> --url <url> --title "<title>" --route <route>
   codex-research ledger add-claim --text "<claim>" --confidence 0.85 --source <source-id>
   ```

10. Render report:

    ```bash
    codex-research report --out .codex/research/report.md
    ```

## Completion Criteria

- Material claims have source IDs.
- Source freshness matches the claim.
- Contradictions are recorded.
- Provider gaps are marked `UNVERIFIED`.
- Private content was not sent externally without permission.
- Final answer separates facts, recommendations, and residual risk.

## Escalation

Use `$subspawn` when independent lanes can run in parallel:

- official docs lane;
- GitHub source/history lane;
- package implementation lane;
- citation audit lane.

After spawning, wait for all agents before continuing.
