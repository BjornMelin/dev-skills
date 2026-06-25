# Research

Use `firecrawl research` for Firecrawl-native public research over arXiv papers
and GitHub issue/PR history. It is not a substitute for source verification:
inspect the cited paper, repository, issue, PR, or README before relying on an
important claim.

## Quick Start

```bash
mkdir -p .firecrawl
firecrawl research search-papers "efficient transformer attention" --limit 20 -o .firecrawl/research-papers.json
firecrawl research inspect-paper arxiv:1706.03762 -o .firecrawl/research-paper.md
firecrawl research read-paper arxiv:1706.03762 --question "What is the attention mechanism?" -o .firecrawl/research-paper-question.md
firecrawl research related-papers arxiv:1706.03762 --intent "efficient transformers" -o .firecrawl/research-related.json
firecrawl research search-github "foundationdb queue worker shutdown" --limit 10 -o .firecrawl/research-github.json
```

## Subcommands

- `search-papers <query>`: semantic arXiv discovery. Try several query
  phrasings rather than one huge query.
- `inspect-paper <paperId>`: canonical metadata for one paper.
- `related-papers <seedIds...>`: citation-graph expansion from strong hits.
- `read-paper <paperId>`: relevant full-text passages for a question.
- `search-github <query>`: GitHub issue/PR history and repository README
  matches.

## Boundaries

- Use only for public research surfaces.
- Keep outputs under `.firecrawl/` and inspect with `jq`, `head`, or `rg`.
- Do not use `research search-github` for private repo content.
- Verify claims against the source URL or paper before final answers.
