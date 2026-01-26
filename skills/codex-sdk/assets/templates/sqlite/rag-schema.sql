PRAGMA foreign_keys = ON;

-- Optional extension schema for SQLite-first “memory” / RAG.
-- This is intentionally minimal and provider-agnostic.
--
-- Suggested usage:
-- - Store source documents (ADRs, design notes, docs) in `documents`
-- - Store chunked text in `chunks`
-- - Store structured run-scoped notes in `notes`
--
-- If you need vector search, you can add an embedding column (JSON or BLOB) to `chunks`
-- and implement retrieval in application code, or move to a dedicated vector store.

CREATE TABLE IF NOT EXISTS documents (
  id TEXT PRIMARY KEY,
  source TEXT,
  uri TEXT,
  title TEXT,
  text TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chunks (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL,
  text TEXT NOT NULL,
  token_count INTEGER,
  updated_at TEXT NOT NULL,
  UNIQUE(document_id, chunk_index)
);

CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  run_id TEXT,
  scope TEXT,
  key TEXT NOT NULL,
  value_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_notes_run_id ON notes(run_id);
CREATE INDEX IF NOT EXISTS idx_notes_scope_key ON notes(scope, key);

