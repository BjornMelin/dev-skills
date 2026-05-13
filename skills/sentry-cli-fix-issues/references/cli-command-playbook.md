# Sentry CLI Command Playbook

Use dedicated `sentry` commands first. Fall back to `sentry schema` and
`sentry api` only when the command surface does not expose the endpoint you
need.

## Global Practices

- Prefer `--json --fields` for machine-readable, bounded output.
- Use `--limit`, `--period`, `--query`, and `--fresh` to keep evidence current
  and small.
- Let the CLI auto-detect org/project unless it reports ambiguity.
- If output is too large, rerun with fewer fields instead of pasting raw event
  payloads into the conversation.
- Never include authentication output or token values in reports.

## Issue Triage

Find candidate issues:

```bash
sentry issue list --query "is:unresolved" --period 24h --limit 10 \
  --json --fields shortId,title,priority,level,status,count,userCount,lastSeen,project,permalink
```

View the selected issue:

```bash
sentry issue view ISSUE --json \
  --fields id,shortId,title,culprit,count,userCount,firstSeen,lastSeen,level,status,substatus,priority,platform,permalink,project,metadata,assignedTo,isUnhandled,event,trace,replayIds
```

List representative events:

```bash
sentry issue events ISSUE --full --period 24h --limit 5 --json \
  --fields id,eventID,groupID,projectID,message,title,location,culprit,user,tags,platform,dateCreated,crashFile,metadata
```

Use magic selectors only when the user asks for "latest" or "most frequent":

```bash
sentry issue view @latest --json
sentry issue view @most_frequent --json
```

## Event, Trace, Span, Log, And Replay

Open a single event:

```bash
sentry event view EVENT_ID --spans --json
```

Inspect a trace and related logs:

```bash
sentry trace view TRACE_ID --full --json
sentry trace logs TRACE_ID --period 24h --limit 50 --json
```

Inspect spans directly:

```bash
sentry span list TRACE_ID --limit 50 --json
sentry span view TRACE_ID/SPAN_ID --json
```

Inspect logs without a trace when the project is known:

```bash
sentry log list ORG/PROJECT --query "severity:error" --period 24h --limit 50 --json
```

Inspect a replay when linked by an issue or event:

```bash
sentry replay view REPLAY_ID --json
```

## Explore Queries

Use `explore` to test whether a fix target is isolated or systemic.

```bash
sentry explore --dataset errors --query "issue:ISSUE_ID" --period 24h \
  --limit 20 --json

sentry explore --dataset spans --query "span.op:http.client" --period 24h \
  --sort "-span.duration" --limit 20 --json

sentry explore --dataset logs --query "severity:error" --period 24h \
  --limit 20 --json
```

Prefer trends and top offenders over raw payload dumps.

## Seer Analysis

Use Seer output as a hypothesis source:

```bash
sentry issue explain ISSUE --json
sentry issue plan ISSUE --json
```

If `issue plan` reports multiple root causes, rerun with the specific `--cause`
value after reading the explanation. Do not apply a plan until stack frames and
repo code confirm the same cause.

## API Fallbacks

Discover first:

```bash
sentry schema --search issue --json --fields method,path,operationId,summary
sentry schema --search tag --json --fields method,path,operationId,summary
sentry schema --search source-map --json --fields method,path,operationId,summary
```

Then call the endpoint through authenticated CLI API access:

```bash
sentry api organizations/ORG/issues/ISSUE_ID/tags/environment/values/ --json
sentry api projects/ORG/PROJECT/events/EVENT_ID/source-map-debug/ --json
```

Keep API responses bounded with endpoint parameters when available. Avoid
`--verbose` unless debugging CLI transport itself.

## Source Maps And Releases

For JavaScript source-map failures, first determine whether the event release,
dist, debug IDs, uploaded artifact names, and URL prefixes match the deployed
bundle.

Useful commands:

```bash
sentry sourcemap inject DIST_DIR --dry-run --json
sentry sourcemap upload DIST_DIR --release RELEASE --dist DIST --url-prefix URL_PREFIX --json
sentry api projects/ORG/PROJECT/events/EVENT_ID/source-map-debug/ --json
```

Do not upload new source maps from an unverified build directory. Confirm the
release string matches the SDK initialization value exactly.

## Mutations

Use Sentry state changes sparingly:

```bash
sentry issue resolve ISSUE --in @commit --json
sentry issue unresolve ISSUE --json
sentry issue archive ISSUE --until auto --json
sentry issue merge SOURCE_ISSUE TARGET_ISSUE --into TARGET_ISSUE --json
```

Before mutation, confirm issue identity, project, status, and intended outcome.
Prefer `--in @commit` or a release version for fixes that should regress if the
bug returns.
