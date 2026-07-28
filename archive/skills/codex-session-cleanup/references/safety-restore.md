# Safety And Restore

## Apply Safety

Before mutating, the script must have:

- a fresh scan manifest;
- exact `--confirm <manifest-id>`;
- `--execute`;
- readable and writable selected session files;
- writable quarantine path;
- backup inventory for existing core DB/history/index files;
- zero JSONL parse errors;
- passing SQLite integrity checks;
- no active goals, jobs, or unsafe spawn-edge references;
- no stale manifest or changed critical counts.

If any condition fails, stop.

## Memory Policy

Supported policies:

- `copy`: copy linked rollout summaries and a registry snapshot into quarantine
  and leave active memory untouched. This is the default.
- `move`: move linked rollout summaries into quarantine after explicit
  `--confirm-memory-move <manifest-id>`. The registry is still copied, not
  moved.
- `ignore`: do not preserve linked memory summaries after explicit
  `--confirm-memory-ignore <manifest-id>`.

If active developer or system instructions forbid memory mutation, use `copy`
even when a user previously preferred stronger cleanup.

## Autonomous Apply

Autonomous apply means reversible quarantine only. It is allowed only when:

- scope is narrow enough for reliable ownership;
- every selected thread is high-confidence;
- risk score is at or below the configured limit;
- age is at least the broad-scope threshold;
- memory policy is `copy`;
- no parser, backup, integrity, active-state, or durable-memory blockers exist;
- a read-only validator agrees when the report is broad or memory-linked.

Manual review is required for `--include-medium`, `--scope all`, destructive
memory policy, purge, and any disagreement or uncertainty.

## Restore

Restore with:

```bash
python3 scripts/codex_session_cleanup.py restore --quarantine <dir> --restore-db --execute
```

Default restore copies session and memory files back when missing. `--restore-db`
copies DB/history backups back over current files. Use it only when no newer
Codex state must be preserved.

## Purge

Purge is explicit and separate. First run without `--execute`; the command will
tell you the required `purge:<bundle-names>` confirmation token:

```bash
python3 scripts/codex_session_cleanup.py purge --older-than-days 30 --confirm purge:<bundle-names>
```

Do not purge a quarantine until rollback is no longer needed. Purge requires a
valid `result.json` and `SHA256SUMS` bundle.
