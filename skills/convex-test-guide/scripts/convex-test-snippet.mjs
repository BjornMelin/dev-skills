#!/usr/bin/env node

import process from 'node:process';

const SNIPPETS = {
  base: `import { convexTest } from 'convex-test';
import { describe, expect, it } from 'vitest';

import { api } from '../../../convex/_generated/api';
import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';

describe('feature', () => {
  it('enforces behavior', async () => {
    const t = convexTest(schema, convexModules);

    await t.run(async (ctx) => {
      await ctx.db.insert('users', {
        clerkUserId: 'user_test',
        displayName: 'Test User',
      });
    });

    const result = await t.query(api.someModule.someQuery, {});

    expect(result).toMatchObject({});
  });
});
`,
  authz: `import { convexTest } from 'convex-test';
import { describe, expect, it } from 'vitest';

import { api } from '../../../convex/_generated/api';
import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';

describe('protected function', () => {
  it('allows owner and rejects stranger in separate harnesses', async () => {
    const allowed = convexTest(schema, convexModules);
    const owner = allowed.withIdentity({ subject: 'user_owner', name: 'Owner' });

    await allowed.run(async (ctx) => {
      await ctx.db.insert('users', { clerkUserId: 'user_owner' });
    });

    await expect(owner.query(api.someModule.protectedQuery, {})).resolves.toBeDefined();

    const denied = convexTest(schema, convexModules);
    const stranger = denied.withIdentity({ subject: 'user_stranger', name: 'Stranger' });

    await expect(stranger.query(api.someModule.protectedQuery, {})).rejects.toThrow();
  });
});
`,
  scheduled: `import { convexTest } from 'convex-test';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { api } from '../../../convex/_generated/api';
import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';

describe('scheduled work', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('runs scheduled functions deterministically', async () => {
    const t = convexTest(schema, convexModules);

    await t.mutation(api.someModule.enqueueWork, {});
    await t.finishAllScheduledFunctions(vi.runAllTimers);

    const result = await t.query(api.someModule.workStatus, {});

    expect(result).toMatchObject({ done: true });
  });
});
`,
  http: `import { convexTest } from 'convex-test';
import { describe, expect, it } from 'vitest';

import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';

describe('HTTP action', () => {
  it('handles the request', async () => {
    const t = convexTest(schema, convexModules);

    const response = await t.fetch('/api/example', {
      body: JSON.stringify({ ok: true }),
      headers: { 'content-type': 'application/json' },
      method: 'POST',
    });

    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toMatchObject({});
  });
});
`,
  workpool: `import { convexTest } from 'convex-test';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { api } from '../../../convex/_generated/api';
import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';

describe('Workpool component wrapper', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('drains queued component work through app wrappers', async () => {
    const t = convexTest(schema, convexModules);

    await t.mutation(api.someModule.enqueueComponentWork, {});
    await t.finishAllScheduledFunctions(vi.runAllTimers);

    const status = await t.query(api.someModule.componentWorkStatus, {});

    expect(status).toMatchObject({ pending: 0 });
  });
});
`,
};

function printHelp() {
  console.log(`Usage: convex-test-snippet.mjs <kind>

Print Signr-shaped convex-test boilerplate.

Kinds:
  ${Object.keys(SNIPPETS).join(', ')}
  all
`);
}

function main() {
  const kind = process.argv[2] ?? 'base';
  if (kind === '--help' || kind === '-h') {
    printHelp();
    return;
  }
  if (kind === 'all') {
    for (const [name, snippet] of Object.entries(SNIPPETS)) {
      process.stdout.write(`// ${name}\n${snippet}\n`);
    }
    return;
  }
  const snippet = SNIPPETS[kind];
  if (!snippet) {
    console.error(`Unknown snippet: ${kind}`);
    printHelp();
    process.exit(2);
  }
  process.stdout.write(snippet);
}

main();
