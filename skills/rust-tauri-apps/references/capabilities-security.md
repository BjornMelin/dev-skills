# Capabilities and Security

## Least Privilege

Capabilities are part of the app security boundary. Keep them narrow:

- Limit filesystem scope to required directories.
- Avoid shell permissions unless there is a clear user workflow and strict argument validation.
- Prefer opener/dialog plugins over custom shell calls.
- Gate dangerous commands behind explicit UI actions and clear confirmation.

## Input Validation

Treat frontend input as untrusted:

- Normalize and validate paths.
- Reject traversal outside allowed roots.
- Validate URLs, schemes, and hosts.
- Validate command arguments as data, not strings to concatenate.
- Enforce file size and content type limits when reading user-selected files.

## Secrets

Do not send secrets to the frontend unless the frontend must display them. Prefer platform keychain/secure storage plugins for durable secrets and short-lived in-memory tokens for command work.

## Updater

Updater configuration should be explicit:

- signed update artifacts
- stable channels
- clear rollback plan
- environment-specific endpoints
- tests or smoke scripts for update metadata

## Audit Checklist

Before shipping risky permissions, answer:

- Which user action requires this capability?
- What exact paths, commands, or hosts are allowed?
- What happens if the frontend is compromised?
- Is there an official plugin with a smaller permission surface?
