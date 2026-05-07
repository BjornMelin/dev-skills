# Deep Researcher Architecture

## Locked Decisions

| Branch | Decision | Score | Rationale |
| --- | --- | ---: | --- |
| Engine | GPT-5.5/Codex-native agentic research plus Rust CLI | 9.7 | Keeps reasoning and native tools in Codex while making evidence, cache, and provider calls replayable. |
| Context7 | Direct REST API only | 9.6 | Avoids removed legacy research mode and gives control over search, context, refresh, versions, 202/301/429 handling. |
| Render routing | Predictive router | 9.7 | Reduces failed fetch cascades by classifying before escalating to browser or Firecrawl. |
| Search mix | Codex web first, Exa for breadth | 9.6 | Native web is best for narrow current checks; Exa is reserved for semantic expansion and broad exploration. |
| Firecrawl | Hobby capacity under classified policy | 9.6 | Paid capacity is useful for rendered/blocked/crawl-heavy pages, but should not replace primary sources. |
| GitHub | Hybrid app plus REST/gh fallback | 9.7 | Uses existing Codex app where available and supports standalone CLI replay. |
| Repo shape | Cargo workspace now, Turborepo later | 9.3 | Avoids Node package-manager churn until multiple packages or JS adapters justify task-graph caching. |
| Output | JSONL/JSON plus Markdown | 9.6 | Supports human review, replay, evals, stale checks, and automation. |

## Provider Responsibilities

Codex native web:

- current official docs and narrow fact checks;
- line-oriented source reading with `web.open` and `web.find`;
- low-overhead verification before paid/broad providers.

Context7 REST:

- library ID search;
- version-pinned documentation snippets;
- manual refresh when release timing matters;
- source IDs and snippet metadata for citation ledgers.

GitHub:

- repositories, code search, issues, PRs, releases, tags, raw files, manifests,
  changelogs, Actions logs when relevant;
- hydration of search hits before citation;
- local clone/sparse checkout only when APIs cannot prove source claims.

Exa:

- broad semantic search;
- GitHub/repo inspiration outside known repositories;
- filtered deep exploration when native web is too narrow.

Direct fetch:

- static HTML, markdown, text, JSON, raw source files;
- route probes and cacheable source records.

Agent browser:

- local/public pages that need JavaScript rendering but do not justify external
  scraping.

Firecrawl:

- JS-heavy public docs, blocked public pages, broad crawl/search, PDFs, and
  content extraction where local rendering/direct fetch is weak.

Opensrc:

- installed package source, version diffs, internal implementation proof, and
  dependency upgrade verification.

## Config And Run State

`codex-research` owns replayable run state outside the Codex conversation:

- TOML config defines profile budgets, provider defaults, privacy posture, and
  cache policy.
- `run init` materializes a JSON state file for one research question.
- provider commands with `--run` debit budgets before network calls;
  `run debit --provider codex-web` records native Codex web calls manually.
- source-cache rows store normalized metadata for direct fetch, Context7,
  GitHub, and Firecrawl results; raw bodies are stored only when policy allows.
- route memory records successful domain/provider outcomes so later probes can
  skip repeated weak routes.

Treat the run file and source cache as audit support, not as a substitute for
claim-level citation and synthesis.

## Predictive Router

The router should avoid blind fetch cascades:

1. Classify by source class: GitHub URL, package docs, official docs, raw file,
   rendered app, PDF, search result, or unknown.
2. Probe cheaply: HEAD plus small GET with byte cap.
3. Detect app shells: low text density, high script count, `__NEXT_DATA__`,
   `#__next`, `window.__NUXT__`, generic root-only shells, Cloudflare/block
   signatures, or JavaScript-required copy.
4. Use site adapters before rendering: GitHub API, Context7, llms.txt, raw
   markdown, Docusaurus/VitePress/MkDocs/Mintlify pages.
5. Remember domain route outcomes in SQLite so repeated failures escalate
   directly to the better route.
6. Record route, freshness, cache settings, and provider errors in evidence.

## Evidence Model

Each run should separate:

- source records: provider, URL, title, fetched time, freshness, route, hash;
- claims: text, confidence, source IDs, note, status;
- route stats: attempted provider, failure reason, escalation, cost class;
- report: human-readable synthesis with citations and remaining risks.

Confidence is claim-level, not report-level. High confidence requires primary
source support and freshness appropriate to the claim.
