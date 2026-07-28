# REPO_CONTEXT

<!--
Fill this document from repository evidence.
Replace every placeholder.
If a section cannot be completed, write `Not found in repo` or `Unknown`.
Use file paths as evidence anchors throughout.
-->

## 1. Repository identity

- **Repository:** 
- **Primary purpose:** 
- **Repository type:** single app | service | library | monorepo | infra | mixed
- **Primary languages:** 
- **Primary frameworks / platforms:** 
- **Primary deployment target:** 
- **Default / main branch:** 
- **Root path analyzed:** 
- **Last analyzed:** 
- **Analyzer:** 

## 2. One-paragraph summary

Write a tight paragraph explaining what the repository does, who or what it serves, and the main technical shape of the codebase.

## 3. Top-level layout

| Path | Kind | Role | Notes |
| --- | --- | --- | --- |
|  |  |  |  |

## 4. Main runtime surfaces and entrypoints

| Surface | Entrypoint(s) | How it starts | Notes |
| --- | --- | --- | --- |
|  |  |  |  |

Examples of surfaces:

- web app
- API server
- worker
- CLI
- scheduler / cron job
- shared package with no runtime surface

## 5. Architecture summary

### 5.1 Main components

Summarize the important apps, services, packages, layers, or modules.

### 5.2 Request, job, and data flow

Describe the real flow through the system. Prefer simple prose over diagrams unless the repo already contains a canonical diagram.

### 5.3 Boundaries and coupling

Note important boundaries, shared libraries, internal APIs, and places where coupling is high.

### 5.4 External systems and integrations

| System / service | Purpose | Where referenced | Config / secret notes |
| --- | --- | --- | --- |
|  |  |  |  |

## 6. Data, state, and contracts

| Concern | Technology / format | Key paths | Notes |
| --- | --- | --- | --- |
| Database |  |  |  |
| Cache / queue |  |  |  |
| Object storage |  |  |  |
| Schemas / contracts |  |  |  |
| Auth / identity |  |  |  |

Add or remove rows based on the repo.

## 7. Build, run, and verification workflow

### 7.1 Prerequisites

List the actual prerequisites discovered in the repo.

### 7.2 Install / sync

~~~bash
# exact commands from repo, if found
~~~

### 7.3 Run locally

~~~bash
# exact commands from repo, if found
~~~

### 7.4 Lint / format / typecheck / test

~~~bash
# exact commands from repo, if found
~~~

### 7.5 Build / package

~~~bash
# exact commands from repo, if found
~~~

### 7.6 Deploy / release

~~~bash
# exact commands from repo, if found
~~~

## 8. Configuration and secrets

| Variable / file / setting | Required? | Purpose | Evidence |
| --- | --- | --- | --- |
|  |  |  |  |

## 9. Testing and quality signals

| Layer | Tooling | Key paths | Notes |
| --- | --- | --- | --- |
| Unit |  |  |  |
| Integration |  |  |  |
| E2E / UI |  |  |  |
| Lint / format |  |  |  |
| Typecheck |  |  |  |

Adapt rows to the repo.

## 10. Infrastructure, CI/CD, and operations

### 10.1 Infrastructure / platform summary

### 10.2 CI / automation summary

### 10.3 Observability, logging, and runtime support

### 10.4 Release / deployment risks

## 11. Important files to read first

| File | Why it matters |
| --- | --- |
|  |  |

Keep this list short and high-signal.

## 12. Current pain points, tech debt, and risks

- 
- 
- 

## 13. Change guidance for future agents

List implementation constraints, preferred patterns, and important do-not-break expectations that future work should respect.

- 
- 
- 

## 14. Open questions and unknowns

| Item | Why it matters | Where checked | Fastest resolution path |
| --- | --- | --- | --- |
|  |  |  |  |

## 15. Evidence log

List the highest-value files that informed this document.

- 
- 
- 
