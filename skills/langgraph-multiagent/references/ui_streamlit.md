# UI integration: Streamlit (Python)

This guide complements `streamlit-master-architect` by focusing specifically on **LangGraph/LangChain multi-agent integration** patterns inside Streamlit apps.

## Load Streamlit Master Architect (recommended)

This skill intentionally stays focused on agent integration; for Streamlit APIs, evergreen upgrade rules, and production hardening, load:

- Streamlit Master Architect skill: `/home/bjorn/.codex/skills/streamlit-master-architect/SKILL.md`
- Evergreen audit + upgrade playbook: `/home/bjorn/.codex/skills/streamlit-master-architect/references/evergreen_audit_upgrade.md`
- Official doc URLs: `/home/bjorn/.codex/skills/streamlit-master-architect/references/official_urls.md`
- Caching/fragments: `/home/bjorn/.codex/skills/streamlit-master-architect/references/caching_and_fragments.md`
- Widget keys/reruns: `/home/bjorn/.codex/skills/streamlit-master-architect/references/widget_keys_and_reruns.md`
- AppTest: `/home/bjorn/.codex/skills/streamlit-master-architect/references/testing_apptest.md`

## Evergreen “ground truth” loop (don’t guess Streamlit APIs)

1. Detect the project’s Streamlit version:
   - `python3 -c "import streamlit as st; print(st.__version__)"`
2. Audit the Streamlit project (deprecations + risky patterns):
   - `python3 /home/bjorn/.codex/skills/streamlit-master-architect/scripts/audit_streamlit_project.py --root <PROJECT_ROOT> --format md`
3. If needed, sync Streamlit docs index:
   - `python3 /home/bjorn/.codex/skills/streamlit-master-architect/scripts/sync_streamlit_docs.py --out /tmp/streamlit-docs`

## Core architecture

Recommended layering:

1. **Pure agent logic** (tools, graphs, prompts) in importable modules
2. **UI wiring** (Streamlit) that only handles input/output and state
3. **Persistence** via checkpointers/stores (not `st.session_state` only)

## Streamlit state model

Use `st.session_state` for:

- user-visible chat history (messages to render)
- UI state (selected thread, pending interrupt decisions)
- lightweight identifiers (thread_id)

Use LangGraph persistence for:

- durable short-term memory (thread checkpoints)
- HITL pause/resume (required)

## Backend choices for Streamlit

- **In-process (simplest)**: import your agent/graph and call `.stream()`/`.astream()` directly.
- **Agent Server (most scalable)**: call a local/deployed Agent Server using the LangGraph SDK; still render streaming in Streamlit.

## Streaming tokens into Streamlit

Typical pattern:

- render an assistant chat message container
- update a placeholder as tokens arrive
- keep the final assistant message in `st.session_state.messages`

Prefer `agent.stream(..., stream_mode=["messages","updates","custom"])` for:

- tokens (`messages`)
- interrupt detection (`updates`)
- progress events (`custom`)

## Human-in-the-loop UI

When interrupts occur:

1. surface the pending actions (tool name + args)
2. let the user approve/edit/reject
3. resume the run with a `Command(resume=...)` using the same `thread_id`

Keep the interrupt UI explicit and visually distinct (danger/warning styling).

## Performance basics

- Cache heavy resources with `st.cache_resource` (models, embeddings, vector clients).
- Cache data reads with `st.cache_data`.
- Avoid import-time IO; initialize lazily in cached functions.

## Template shipped with this skill

- `assets/templates/streamlit/streamlit_chat_app.py`:
  - streaming tokens into `st.chat_message`
  - maintaining a `thread_id`
  - placeholder UI for interrupts

- `assets/templates/streamlit/tests/test_smoke_apptest.py`:
  - minimal AppTest smoke test (offline-first pattern)
