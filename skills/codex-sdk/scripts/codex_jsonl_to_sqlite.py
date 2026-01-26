#!/usr/bin/env python3

import argparse
import datetime as dt
import json
import os
import sqlite3
import sys
import uuid


def utc_now_iso() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


def read_lines(path: str):
    if path == "-":
        for line in sys.stdin:
            yield line
        return
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            yield line


def init_db(conn: sqlite3.Connection):
    schema_path = os.path.join(
        os.path.dirname(__file__), "..", "assets", "templates", "sqlite", "schema.sql"
    )
    with open(schema_path, "r", encoding="utf-8") as f:
        conn.executescript(f.read())
    conn.commit()


def ensure_run(conn: sqlite3.Connection, run_id: str, label: str | None, repo_root: str | None):
    cur = conn.execute("SELECT 1 FROM runs WHERE id = ?", (run_id,))
    if cur.fetchone():
        return
    conn.execute(
        "INSERT INTO runs (id, created_at, label, repo_root, metadata_json) VALUES (?, ?, ?, ?, ?)",
        (run_id, utc_now_iso(), label, repo_root, None),
    )
    conn.commit()


def extract_fields(event: dict):
    event_type = str(event.get("type") or "")
    thread_id = None
    item_id = None
    item_type = None

    if event_type == "thread.started":
        thread_id = event.get("thread_id")
    if event_type.startswith("item.") and isinstance(event.get("item"), dict):
        item = event["item"]
        item_id = item.get("id")
        item_type = item.get("type")
        # Some item payloads also include thread IDs in nested params in other surfaces; keep raw JSON anyway.

    return event_type, thread_id, item_id, item_type


def main():
    parser = argparse.ArgumentParser(
        description="Ingest Codex JSONL events (from `codex exec --json`) into SQLite for auditability and reuse."
    )
    parser.add_argument("--db", required=True, help="Path to SQLite DB file.")
    parser.add_argument(
        "--init",
        action="store_true",
        help="Initialize schema (safe to run multiple times).",
    )
    parser.add_argument(
        "--input",
        default="-",
        help="JSONL input file path, or '-' for stdin (default).",
    )
    parser.add_argument(
        "--run-id",
        default=None,
        help="Run ID (uuid). If omitted, a new run is created.",
    )
    parser.add_argument(
        "--run-label",
        default=None,
        help="Optional human label for the run (e.g. 'ci-autofix').",
    )
    parser.add_argument(
        "--repo-root",
        default=None,
        help="Optional repo root path to associate with the run.",
    )
    args = parser.parse_args()

    conn = sqlite3.connect(args.db)
    conn.row_factory = sqlite3.Row

    if args.init:
        init_db(conn)

    run_id = args.run_id or str(uuid.uuid4())
    ensure_run(conn, run_id=run_id, label=args.run_label, repo_root=args.repo_root)

    inserted = 0
    for raw_line in read_lines(args.input):
        line = raw_line.strip()
        if not line:
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            # Keep ingest robust: ignore non-JSON lines (some runners can mix logs).
            continue
        if not isinstance(event, dict):
            continue

        event_type, thread_id, item_id, item_type = extract_fields(event)
        payload_json = json.dumps(event, separators=(",", ":"), ensure_ascii=False)

        conn.execute(
            "INSERT INTO events (run_id, ingested_at, event_type, thread_id, item_id, item_type, payload_json) "
            "VALUES (?, ?, ?, ?, ?, ?, ?)",
            (run_id, utc_now_iso(), event_type, thread_id, item_id, item_type, payload_json),
        )
        inserted += 1

    conn.commit()

    sys.stdout.write(json.dumps({"db": args.db, "run_id": run_id, "inserted": inserted}) + "\n")


if __name__ == "__main__":
    main()

