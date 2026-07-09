#!/usr/bin/env node
// Single source of truth = the canonical flat skills under repo `skills/`.
// This plugin re-bundles three of them; the mirror under `plugins/design-motion/skills/`
// is GENERATED, never hand-edited.
//
//   node scripts/sync-skills.mjs           # regenerate the mirror from canonical skills
//   node scripts/sync-skills.mjs --check   # CI guard: exit 1 if the mirror has drifted
//
import {
  readdirSync, readFileSync, statSync, mkdirSync, copyFileSync, rmSync, existsSync,
} from 'node:fs';
import { join, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const PLUGIN = join(HERE, '..');          // plugins/design-motion
const REPO = join(PLUGIN, '..', '..');    // repo root
const CANON = join(REPO, 'skills');
const MIRROR = join(PLUGIN, 'skills');
const SKILLS = ['design-motion-system', 'design-motion-audit', 'r3f-scene-polish'];
const check = process.argv.includes('--check');

const skip = (name) => name === '__pycache__' || name.endsWith('.pyc');

function walk(dir) {
  const out = [];
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    if (skip(e.name)) continue;
    const p = join(dir, e.name);
    if (e.isDirectory()) out.push(...walk(p));
    else out.push(p);
  }
  return out;
}

let drift = 0;
for (const skill of SKILLS) {
  const src = join(CANON, skill);
  const dst = join(MIRROR, skill);
  if (!existsSync(src) || !statSync(src).isDirectory()) {
    console.error(`missing canonical skill: skills/${skill}`);
    process.exit(1);
  }
  const wantRel = new Set(walk(src).map((f) => relative(src, f)));
  for (const rel of wantRel) {
    const from = join(src, rel);
    const to = join(dst, rel);
    const toExists = existsSync(to);
    const same = toExists && statSync(to).isFile() && readFileSync(to).equals(readFileSync(from));
    if (check) {
      if (!same) { console.error(`DRIFT: ${skill}/${rel}`); drift++; }
    } else if (!same) {
      if (toExists && !statSync(to).isFile()) rmSync(to, { recursive: true, force: true });
      mkdirSync(dirname(to), { recursive: true });
      copyFileSync(from, to);
    }
  }
  if (existsSync(dst)) {
    for (const f of walk(dst)) {
      const rel = relative(dst, f);
      if (!wantRel.has(rel)) {
        if (check) { console.error(`EXTRA (not in canonical): ${skill}/${rel}`); drift++; }
        else rmSync(f);
      }
    }
  }
}

if (check) {
  if (drift) {
    console.error(`\n${drift} file(s) out of sync. Regenerate with: node plugins/design-motion/scripts/sync-skills.mjs`);
    process.exit(1);
  }
  console.log('design-motion plugin skills mirror is in sync with canonical skills/ ✓');
} else {
  console.log(`Synced ${SKILLS.length} skills → ${relative(REPO, MIRROR)}/`);
}
