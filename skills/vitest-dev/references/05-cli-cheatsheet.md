# Vitest CLI cheat sheet

Primary reference: https://vitest.dev/guide/cli

## Local development

```bash
# watch mode (TTY)
npx vitest
```

## CI

```bash
# run once
npx vitest run
```

## Reporters

```bash
npx vitest run --reporter=junit
npx vitest run --reporter=json
npx vitest run --reporter=blob --outputFile=reports/blob.json
```

See reporters: https://vitest.dev/guide/reporters

## Sharding

```bash
npx vitest run --shard=1/4 --reporter=blob --outputFile=reports/blob-1.json
npx vitest run --shard=2/4 --reporter=blob --outputFile=reports/blob-2.json
# merge:
npx vitest --merge-reports=reports --reporter=default --reporter=json
```

## Debugging global leakage

```bash
npx vitest run --no-file-parallelism
```

Config reference: https://vitest.dev/config/fileparallelism
