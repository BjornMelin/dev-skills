from __future__ import annotations

import importlib.util
import json
import os
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path


SKILL_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = SKILL_ROOT / "scripts" / "codex_session_cleanup.py"
SPEC = importlib.util.spec_from_file_location("codex_session_cleanup", SCRIPT)
assert SPEC and SPEC.loader
cleanup = importlib.util.module_from_spec(SPEC)
sys.modules["codex_session_cleanup"] = cleanup
SPEC.loader.exec_module(cleanup)


REVIEW_ID = "11111111-1111-1111-1111-111111111111"
ACTIVE_ID = "22222222-2222-2222-2222-222222222222"


class CleanupFixture(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.codex_home = self.root / ".codex"
        self.repo = self.root / "repo"
        self.repo.mkdir(parents=True)
        (self.repo / ".git").mkdir()
        (self.codex_home / "sessions").mkdir(parents=True)
        (self.codex_home / "memories" / "rollout_summaries").mkdir(parents=True)
        self.cache_dir = self.codex_home / "tmp"
        self.cache_dir.mkdir()
        (self.cache_dir / "scratch.txt").write_text("scratch", encoding="utf8")
        self.now = int(time.time())
        self.review_session = self.codex_home / "sessions" / f"rollout-{REVIEW_ID}.jsonl"
        self.active_session = self.codex_home / "sessions" / f"rollout-{ACTIVE_ID}.jsonl"
        self.review_session.write_text(
            json.dumps({"type": "message", "role": "user", "content": [{"text": "$gh-pr-review-fix pr 1"}]}) + "\n",
            encoding="utf8",
        )
        self.active_session.write_text(
            json.dumps({"type": "message", "role": "user", "content": [{"text": "$gh-pr-review-fix active"}]}) + "\n",
            encoding="utf8",
        )
        self._write_state_db()
        self._write_logs_db()
        self._write_jsonl()
        self.memory_summary = self.codex_home / "memories" / "rollout_summaries" / "review-memory.md"
        self.memory_summary.write_text(
            f"thread_id: {REVIEW_ID}\nReusable knowledge: auth workflow runbook\n",
            encoding="utf8",
        )
        self.memory_registry = self.codex_home / "memories" / "MEMORY.md"
        self.memory_registry.write_text(
            f"- rollout_summaries/review-memory.md thread_id={REVIEW_ID} stale command docs:arch:validate\n",
            encoding="utf8",
        )

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def _write_state_db(self) -> None:
        conn = sqlite3.connect(self.codex_home / "state_5.sqlite")
        conn.executescript(
            """
            create table threads (
                id text primary key,
                rollout_path text not null,
                created_at integer not null,
                updated_at integer not null,
                source text not null default '',
                model_provider text not null default '',
                cwd text not null,
                title text not null,
                sandbox_policy text not null default '',
                approval_mode text not null default '',
                tokens_used integer not null default 0,
                has_user_event integer not null default 0,
                archived integer not null default 0,
                archived_at integer,
                git_sha text,
                git_branch text,
                git_origin_url text,
                cli_version text not null default '',
                first_user_message text not null default '',
                agent_nickname text,
                agent_role text,
                memory_mode text not null default 'enabled',
                model text,
                reasoning_effort text,
                agent_path text,
                created_at_ms integer,
                updated_at_ms integer,
                thread_source text
            );
            create table thread_goals (
                thread_id text primary key not null,
                goal_id text not null,
                objective text not null,
                status text not null,
                token_budget integer,
                tokens_used integer not null default 0,
                time_used_seconds integer not null default 0,
                created_at_ms integer not null,
                updated_at_ms integer not null
            );
            create table thread_spawn_edges (
                parent_thread_id text not null,
                child_thread_id text not null primary key,
                status text not null
            );
            create table agent_jobs (
                id text primary key,
                name text not null,
                status text not null,
                instruction text not null,
                output_schema_json text,
                input_headers_json text not null,
                input_csv_path text not null,
                output_csv_path text not null,
                auto_export integer not null default 1,
                created_at integer not null,
                updated_at integer not null,
                started_at integer,
                completed_at integer,
                last_error text,
                max_runtime_seconds integer
            );
            create table agent_job_items (
                job_id text not null,
                item_id text not null,
                row_index integer not null,
                source_id text,
                row_json text not null,
                status text not null,
                assigned_thread_id text,
                attempt_count integer not null default 0,
                result_json text,
                last_error text,
                created_at integer not null,
                updated_at integer not null,
                completed_at integer,
                reported_at integer,
                primary key (job_id, item_id)
            );
            """
        )
        old = self.now - 100 * 3600
        conn.execute(
            "insert into threads (id, rollout_path, created_at, updated_at, cwd, title, tokens_used, has_user_event, archived, first_user_message) values (?,?,?,?,?,?,?,?,?,?)",
            (REVIEW_ID, str(self.review_session), old, old, str(self.repo), "$gh-pr-review-fix pr 1", 1000, 1, 0, "$gh-pr-review-fix pr 1"),
        )
        conn.execute(
            "insert into threads (id, rollout_path, created_at, updated_at, cwd, title, tokens_used, has_user_event, archived, first_user_message) values (?,?,?,?,?,?,?,?,?,?)",
            (ACTIVE_ID, str(self.active_session), old, old, str(self.repo), "$gh-pr-review-fix active", 1000, 1, 0, "$gh-pr-review-fix active"),
        )
        conn.execute(
            "insert into thread_goals (thread_id, goal_id, objective, status, created_at_ms, updated_at_ms) values (?,?,?,?,?,?)",
            (ACTIVE_ID, "goal-1", "keep active", "active", old * 1000, old * 1000),
        )
        conn.commit()
        conn.close()

    def _write_logs_db(self) -> None:
        conn = sqlite3.connect(self.codex_home / "logs_2.sqlite")
        conn.execute("create table logs (thread_id text not null, ts integer, level text, target text, feedback_log_body text)")
        conn.execute("insert into logs (thread_id, ts, level, target, feedback_log_body) values (?,?,?,?,?)", (REVIEW_ID, self.now, "INFO", "test", "review"))
        conn.execute("insert into logs (thread_id, ts, level, target, feedback_log_body) values (?,?,?,?,?)", (ACTIVE_ID, self.now, "INFO", "test", "active"))
        conn.commit()
        conn.close()

    def _write_jsonl(self) -> None:
        (self.codex_home / "history.jsonl").write_text(
            json.dumps({"session_id": REVIEW_ID, "text": "review"}) + "\n"
            + json.dumps({"session_id": ACTIVE_ID, "text": "active"}) + "\n",
            encoding="utf8",
        )
        (self.codex_home / "session_index.jsonl").write_text(
            json.dumps({"id": REVIEW_ID, "thread_name": "review"}) + "\n"
            + json.dumps({"id": ACTIVE_ID, "thread_name": "active"}) + "\n",
            encoding="utf8",
        )

    def run_script(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--codex-home", str(self.codex_home), *args],
            text=True,
            capture_output=True,
            check=False,
        )

    def scan(self, *extra_args: str, report_name: str = "scan") -> dict:
        report_dir = self.root / "reports"
        result = self.run_script(
            "scan",
            "--cwd",
            str(self.repo),
            "--output-dir",
            str(report_dir),
            "--report-name",
            report_name,
            *extra_args,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        summary = json.loads(result.stdout)
        return json.loads(Path(summary["manifest"]).read_text())


class CleanupTests(CleanupFixture):
    def test_classifier_marks_review_high_confidence(self) -> None:
        row = cleanup.ThreadRow(
            id=REVIEW_ID,
            rollout_path=str(self.review_session),
            created_at=self.now - 100 * 3600,
            updated_at=self.now - 100 * 3600,
            cwd=str(self.repo),
            title="$gh-pr-review-fix pr 1",
            tokens_used=1000,
            has_user_event=1,
            archived=0,
            first_user_message="$gh-pr-review-fix pr 1",
        )
        candidate = cleanup.classify_thread(row, self.now, 24)
        self.assertEqual(candidate.confidence, cleanup.HIGH)
        self.assertIn("gh-pr-review-fix-title", candidate.reasons)
        self.assertLessEqual(candidate.risk_score, cleanup.AUTO_RISK_LIMIT)

    def test_classifier_selects_resolve_pr_review_prompt_despite_large_token_count(self) -> None:
        prompt = """# /resolve-pr-review-comments <PR_NUM>

---
description: Fetch unresolved GitHub PR review comments and resolve them end-to-end.
argument-hint: "<PR_NUM>"
"""
        row = cleanup.ThreadRow(
            id=REVIEW_ID,
            rollout_path=str(self.review_session),
            created_at=self.now - 100 * 3600,
            updated_at=self.now - 100 * 3600,
            cwd=str(self.repo),
            title=prompt,
            tokens_used=1_100_000,
            has_user_event=1,
            archived=0,
            first_user_message=prompt,
        )
        candidate = cleanup.classify_thread(row, self.now, 24)
        self.assertEqual(candidate.confidence, cleanup.HIGH)
        self.assertIn("resolve-pr-review-comments-title", candidate.reasons)
        self.assertNotIn("protected-main-dev-likely", candidate.protected_reasons)
        self.assertTrue(candidate.selected)

    def test_first_prompt_prefix_filter_normalizes_whitespace(self) -> None:
        row = cleanup.ThreadRow(
            id=REVIEW_ID,
            rollout_path=str(self.review_session),
            created_at=self.now - 100 * 3600,
            updated_at=self.now - 100 * 3600,
            cwd=str(self.repo),
            title="# /resolve-pr-review-comments <PR_NUM>",
            tokens_used=1000,
            has_user_event=1,
            archived=0,
            first_user_message="# /resolve-pr-review-comments <PR_NUM>\n\n---\ndescription: Fetch unresolved GitHub PR review comments and resolve them end-to-end.",
        )
        self.assertTrue(
            cleanup.thread_matches_first_prompt_prefix(
                row,
                [
                    "# /resolve-pr-review-comments <PR_NUM>---description: Fetch unresolved GitHub PR review comments"
                ],
            )
        )

    def test_session_file_user_prompt_prefix_matches_later_user_message(self) -> None:
        prompt = "# /resolve-pr-review-comments <PR_NUM>\n\n---\ndescription: Fetch unresolved GitHub PR review comments"
        self.review_session.write_text(
            json.dumps(
                {
                    "type": "response_item",
                    "payload": {
                        "type": "message",
                        "role": "user",
                        "content": [{"type": "input_text", "text": "ordinary first prompt"}],
                    },
                }
            )
            + "\n"
            + json.dumps(
                {
                    "type": "event_msg",
                    "payload": {
                        "type": "user_message",
                        "message": prompt,
                    },
                }
            )
            + "\n",
            encoding="utf8",
        )
        self.assertTrue(
            cleanup.session_file_has_user_prompt_prefix(
                self.review_session,
                [
                    "# /resolve-pr-review-comments <PR_NUM>---description: Fetch unresolved GitHub PR review comments"
                ],
            )
        )

    def test_user_prompt_prefix_filter_selects_matching_session(self) -> None:
        prompt = "# /resolve-pr-review-comments <PR_NUM>\n\n---\ndescription: Fetch unresolved GitHub PR review comments"
        self.review_session.write_text(
            json.dumps({"type": "message", "role": "user", "content": [{"text": "ordinary first prompt"}]})
            + "\n"
            + json.dumps({"type": "event_msg", "payload": {"type": "user_message", "message": prompt}})
            + "\n",
            encoding="utf8",
        )
        manifest = self.scan(
            "--user-prompt-prefix",
            "# /resolve-pr-review-comments <PR_NUM>---description: Fetch unresolved GitHub PR review comments",
            report_name="user-prompt-prefix",
        )
        self.assertEqual([item["id"] for item in manifest["selected_threads"]], [REVIEW_ID])
        self.assertIn("user-prompt-prefix-match", manifest["selected_threads"][0]["reasons"])

    def test_contains_text_filter_selects_session_and_honors_exclude_thread(self) -> None:
        self.review_session.write_text("cleanup mention: # /resolve-pr-review-comments <PR_NUM>\n", encoding="utf8")
        selected = self.scan(
            "--contains-text",
            "# /resolve-pr-review-comments <PR_NUM>",
            "--min-age-hours",
            "0",
            report_name="contains-text-selected",
        )
        self.assertEqual([item["id"] for item in selected["selected_threads"]], [REVIEW_ID])
        self.assertIn("session-file-contains-text", selected["selected_threads"][0]["reasons"])

        excluded = self.scan(
            "--contains-text",
            "# /resolve-pr-review-comments <PR_NUM>",
            "--exclude-thread-id",
            REVIEW_ID,
            "--min-age-hours",
            "0",
            report_name="contains-text-excluded",
        )
        self.assertEqual(excluded["selected_threads"], [])

    def test_exact_thread_id_filter_selects_requested_thread(self) -> None:
        manifest = self.scan(
            "--thread-id",
            REVIEW_ID,
            "--min-age-hours",
            "0",
            report_name="exact-thread-id",
        )
        self.assertEqual([item["id"] for item in manifest["selected_threads"]], [REVIEW_ID])
        self.assertIn("exact-thread-id-match", manifest["selected_threads"][0]["reasons"])

    def test_scan_selects_review_and_protects_active_goal(self) -> None:
        manifest = self.scan()
        self.assertEqual([item["id"] for item in manifest["selected_threads"]], [REVIEW_ID])
        protected_ids = {item["id"] for item in manifest["excluded_threads"]}
        self.assertIn(ACTIVE_ID, protected_ids)
        self.assertEqual(manifest["memory"]["policy_default"], "copy")
        recommendations = {item["recommendation"] for item in manifest["memory"]["triage"]}
        self.assertIn("preserve", recommendations)
        self.assertIn("linked-durable-memory", manifest["automation"]["manual_review_required_reasons"])

    def test_roots_scope_uses_explicit_root(self) -> None:
        manifest = self.scan("--scope", "roots", "--root", str(self.repo), report_name="roots")
        self.assertEqual(manifest["scope"], "roots")
        self.assertEqual(manifest["scope_roots"][0]["path"], str(self.repo))

    def test_scope_matrix_and_noop_apply(self) -> None:
        roots_file = self.root / "roots.txt"
        roots_file.write_text(str(self.repo) + "\n", encoding="utf8")
        matrix = [
            ("current", ()),
            ("root", ("--scope", "root", "--root", str(self.repo))),
            ("roots-file", ("--scope", "roots", "--roots-file", str(roots_file))),
            ("cwd-subrepos", ("--scope", "cwd-subrepos", "--cwd", str(self.root))),
            ("codex-home", ("--scope", "codex-home")),
            ("all", ("--scope", "all")),
        ]
        manifests = {
            label: self.scan(*args, report_name=f"matrix-{label}")
            for label, args in matrix
        }
        self.assertEqual(manifests["current"]["scope"], "current")
        self.assertEqual(manifests["root"]["scope"], "root")
        self.assertEqual(manifests["roots-file"]["scope"], "roots")
        self.assertEqual(manifests["cwd-subrepos"]["scope"], "cwd-subrepos")
        self.assertEqual(manifests["codex-home"]["selected_threads"], [])
        self.assertEqual([item["id"] for item in manifests["all"]["selected_threads"]], [REVIEW_ID])

        noop_manifest = manifests["codex-home"]
        result = self.run_script(
            "apply",
            "--manifest",
            noop_manifest["manifest_path"],
            "--confirm",
            noop_manifest["manifest_id"],
            "--execute",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["noop"])

    def test_codex_home_fallback_unknown_cwd_is_protected(self) -> None:
        (self.codex_home / "state_5.sqlite").unlink()
        manifest = self.scan("--scope", "codex-home", report_name="codex-home-fallback")
        self.assertEqual(manifest["selected_threads"], [])
        excluded = {item["id"]: item for item in manifest["excluded_threads"]}
        self.assertIn(REVIEW_ID, excluded)
        self.assertIn("protected-unknown-cwd-codex-home", excluded[REVIEW_ID]["protected_reasons"])

    def test_apply_copy_restore_and_purge_dry_run(self) -> None:
        manifest = self.scan()
        result = self.run_script(
            "apply",
            "--manifest",
            manifest["manifest_path"],
            "--confirm",
            manifest["manifest_id"],
            "--memory-policy",
            "copy",
            "--execute",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        quarantine = Path(payload["quarantine"])
        self.assertFalse(self.review_session.exists())
        self.assertTrue(self.active_session.exists())
        self.assertTrue(self.memory_summary.exists())
        self.assertTrue(self.memory_registry.exists())
        copied = payload["memory"]["files"][0]["copied_to"]
        self.assertTrue(Path(copied).exists())
        self.assertTrue((quarantine / "SHA256SUMS").exists())

        restore = self.run_script("restore", "--quarantine", str(quarantine), "--execute")
        self.assertEqual(restore.returncode, 0, restore.stderr)
        self.assertTrue(self.review_session.exists())

        old = self.now - (3 * 86400)
        os.utime(quarantine, (old, old))
        purge = self.run_script(
            "purge",
            "--older-than-days",
            "1",
            "--confirm",
            f"purge:{quarantine.name}",
        )
        self.assertEqual(purge.returncode, 0, purge.stderr)
        purge_payload = json.loads(purge.stdout)
        self.assertIn(str(quarantine), purge_payload["purge_candidates"])

    def test_purge_exact_quarantine_does_not_select_other_bundles(self) -> None:
        first = self.codex_home / "prune-quarantine" / "first-quarantine"
        second = self.codex_home / "prune-quarantine" / "second-quarantine"
        for quarantine in (first, second):
            (quarantine / "manifests").mkdir(parents=True)
            (quarantine / "manifests" / "result.json").write_text("{}", encoding="utf8")
            (quarantine / "SHA256SUMS").write_text("", encoding="utf8")

        purge = self.run_script(
            "purge",
            "--quarantine",
            first.name,
            "--confirm",
            f"purge:{first.name}",
        )
        self.assertEqual(purge.returncode, 0, purge.stderr)
        purge_payload = json.loads(purge.stdout)
        self.assertEqual(purge_payload["purge_candidates"], [str(first)])

    def test_memory_move_requires_extra_confirmation(self) -> None:
        manifest = self.scan()
        result = self.run_script(
            "apply",
            "--manifest",
            manifest["manifest_path"],
            "--confirm",
            manifest["manifest_id"],
            "--memory-policy",
            "move",
        )
        self.assertEqual(result.returncode, 3)
        self.assertIn("confirm-memory-move", result.stderr)

    def test_artifact_quarantine_policy_moves_and_restores_manual_candidates(self) -> None:
        manifest = self.scan(
            "--artifact-family",
            "sessions",
            "--artifact-family",
            "memory",
            "--artifact-family",
            "cache",
            "--min-age-hours",
            "0",
            report_name="artifact-policy",
        )
        cache_items = manifest["artifacts"]["cache"]["items"]
        self.assertEqual(cache_items[0]["path"], str(self.cache_dir))
        self.assertEqual(cache_items[0]["action"], "manual-quarantine-candidate")

        dry_run = self.run_script(
            "apply",
            "--manifest",
            manifest["manifest_path"],
            "--confirm",
            manifest["manifest_id"],
        )
        self.assertEqual(dry_run.returncode, 0, dry_run.stderr)
        self.assertTrue(self.cache_dir.exists())
        self.assertEqual(json.loads(dry_run.stdout)["artifact_policy"], "report")

        result = self.run_script(
            "apply",
            "--manifest",
            manifest["manifest_path"],
            "--confirm",
            manifest["manifest_id"],
            "--artifact-policy",
            "quarantine",
            "--execute",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertFalse(self.cache_dir.exists())
        artifact_rows = payload["artifacts"]["files"]
        self.assertEqual(len(artifact_rows), 1)
        self.assertTrue(Path(artifact_rows[0]["quarantine_path"]).exists())

        restore = self.run_script("restore", "--quarantine", payload["quarantine"], "--execute")
        self.assertEqual(restore.returncode, 0, restore.stderr)
        self.assertTrue((self.cache_dir / "scratch.txt").exists())


if __name__ == "__main__":
    unittest.main()
