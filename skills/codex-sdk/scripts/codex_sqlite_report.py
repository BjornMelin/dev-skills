#!/usr/bin/env python3

import argparse
import json
import sqlite3
import sys


def main():
    parser = argparse.ArgumentParser(description="Summarize Codex runs stored by codex_jsonl_to_sqlite.py")
    parser.add_argument("--db", required=True, help="Path to SQLite DB file.")
    parser.add_argument("--run-id", default=None, help="Run id to summarize.")
    parser.add_argument("--latest", action="store_true", help="Summarize the most recent run.")
    args = parser.parse_args()

    conn = sqlite3.connect(args.db)
    conn.row_factory = sqlite3.Row

    run_id = args.run_id
    if args.latest:
        row = conn.execute("SELECT id FROM runs ORDER BY created_at DESC LIMIT 1").fetchone()
        if not row:
            sys.stderr.write("No runs found.\n")
            sys.exit(1)
        run_id = row["id"]

    if not run_id:
        sys.stderr.write("Provide --run-id or --latest.\n")
        sys.exit(2)

    run = conn.execute("SELECT * FROM runs WHERE id = ?", (run_id,)).fetchone()
    if not run:
        sys.stderr.write(f"Run not found: {run_id}\n")
        sys.exit(1)

    counts = conn.execute(
        "SELECT event_type, COUNT(*) AS c FROM events WHERE run_id = ? GROUP BY event_type ORDER BY c DESC",
        (run_id,),
    ).fetchall()

    thread = conn.execute(
        "SELECT thread_id FROM events WHERE run_id = ? AND thread_id IS NOT NULL ORDER BY id ASC LIMIT 1",
        (run_id,),
    ).fetchone()

    last_agent_message = conn.execute(
        "SELECT payload_json FROM events WHERE run_id = ? AND event_type = 'item.completed' AND item_type = 'agent_message' "
        "ORDER BY id DESC LIMIT 1",
        (run_id,),
    ).fetchone()

    summary = {
        "run": dict(run),
        "thread_id": thread["thread_id"] if thread else None,
        "event_type_counts": {r["event_type"]: r["c"] for r in counts},
        "last_agent_message": json.loads(last_agent_message["payload_json"]) if last_agent_message else None,
    }

    sys.stdout.write(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()

