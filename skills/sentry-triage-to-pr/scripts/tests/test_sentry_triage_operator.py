#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import io
import json
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "sentry_triage_operator.py"
SPEC = importlib.util.spec_from_file_location("sentry_triage_operator", SCRIPT)
assert SPEC and SPEC.loader
operator = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(operator)


class SentryTriageOperatorTests(unittest.TestCase):
    def sample_bundle(self) -> dict:
        return {
            "schema": operator.BUNDLE_SCHEMA,
            "generated_at": "2026-05-12T12:00:00+00:00",
            "target": "acme/",
            "query": "is:unresolved",
            "period": "7d",
            "commands": [],
            "issue_contexts": {},
            "issues": [
                {
                    "id": "1",
                    "shortId": "WEB-123",
                    "title": "TypeError cannot read properties of undefined",
                    "culprit": "apps/web/src/auth/session.ts",
                    "count": "1200",
                    "userCount": 80,
                    "lastSeen": "2026-05-12T11:30:00+00:00",
                    "level": "error",
                    "status": "unresolved",
                    "substatus": "regressed",
                    "priority": "high",
                    "platform": "javascript",
                    "permalink": "https://sentry.example/issues/1",
                    "project": {"slug": "web"},
                    "isUnhandled": True,
                    "seerFixabilityScore": 0.84,
                },
                {
                    "id": "2",
                    "shortId": "WEB-456",
                    "title": "Minor warning in analytics",
                    "culprit": "apps/web/src/analytics.ts",
                    "count": "4",
                    "userCount": 1,
                    "lastSeen": "2026-04-01T00:00:00+00:00",
                    "level": "warning",
                    "status": "unresolved",
                    "priority": "low",
                    "platform": "javascript",
                    "project": {"slug": "web"},
                    "isUnhandled": False,
                },
            ],
        }

    def test_triage_group_render_and_worktree_plan(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temp = Path(raw)
            source = temp / "capture.json"
            triaged = temp / "triaged.json"
            grouped = temp / "groups.json"
            github_dir = temp / "github"
            worktrees = temp / "worktrees.json"
            operator.write_json(source, self.sample_bundle())

            self.assertEqual(operator.main(["triage", str(source), "--out", str(triaged)]), 0)
            ranked = json.loads(triaged.read_text())["ranked_issues"]
            self.assertEqual(ranked[0]["issue"]["shortId"], "WEB-123")
            self.assertGreater(ranked[0]["score"], ranked[1]["score"])

            self.assertEqual(operator.main(["group", str(triaged), "--out", str(grouped)]), 0)
            groups = json.loads(grouped.read_text())["groups"]
            self.assertEqual(groups[0]["branch"], "fix/sentry-web-123-typeerror-cannot-read-properties-undefined")

            self.assertEqual(
                operator.main(
                    [
                        "render-github",
                        str(grouped),
                        "--repo",
                        "acme/web",
                        "--out-dir",
                        str(github_dir),
                    ]
                ),
                0,
            )
            body = (github_dir / "sentry-group-001.md").read_text()
            self.assertIn("sentry-triage-to-pr:v1", body)
            self.assertIn("WEB-123", body)
            self.assertNotIn("user@example.com", body)

            self.assertEqual(
                operator.main(
                    [
                        "plan-worktrees",
                        str(grouped),
                        "--repo-root",
                        str(temp),
                        "--out",
                        str(worktrees),
                    ]
                ),
                0,
            )
            plan = json.loads(worktrees.read_text())["worktree_plan"]
            self.assertEqual(plan[0]["branch"], groups[0]["branch"])
            self.assertIn("subspawn_prompt", plan[0])

    def test_redaction_and_validation(self) -> None:
        value = operator.redact(
            {
                "headers": {"authorization": "Bearer abcdefghijklmnopqrstuvwxyz1234567890"},
                "message": "contact user@example.com from 192.168.1.10",
                "userCount": 5,
            }
        )
        self.assertEqual(value["headers"], "[REDACTED]")
        self.assertIn("[REDACTED_EMAIL]", value["message"])
        self.assertIn("[REDACTED_IP]", value["message"])
        self.assertEqual(value["userCount"], 5)

        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "bad.json"
            operator.write_json(
                path,
                {
                    "schema": operator.BUNDLE_SCHEMA,
                    "generated_at": "2026-05-12T12:00:00+00:00",
                    "message": "email user@example.com leaked",
                },
            )
            stderr = io.StringIO()
            with redirect_stderr(stderr):
                self.assertEqual(operator.main(["validate-bundle", str(path)]), 1)
            self.assertIn("sensitive pattern found", stderr.getvalue())

    def test_explicit_issue_capture_does_not_list_by_default(self) -> None:
        calls: list[list[str]] = []

        def fake_run_sentry(args: list[str], timeout: int) -> tuple[dict, object]:
            calls.append(args)
            if args[:3] == ["issue", "view", "WEB-123"]:
                return {"args": ["sentry", *args], "returncode": 0}, {
                    "shortId": "WEB-123",
                    "title": "Explicit issue",
                    "project": {"slug": "web"},
                }
            return {"args": ["sentry", *args], "returncode": 0}, []

        with tempfile.TemporaryDirectory() as raw, patch.object(operator, "run_sentry", fake_run_sentry):
            out = Path(raw) / "capture.json"
            self.assertEqual(
                operator.main(
                    [
                        "capture",
                        "--issue",
                        "WEB-123",
                        "--event-limit",
                        "1",
                        "--no-traces",
                        "--no-replays",
                        "--out",
                        str(out),
                    ]
                ),
                0,
            )
            self.assertFalse(any(call[:2] == ["issue", "list"] for call in calls))
            self.assertTrue(any(call[:3] == ["issue", "view", "WEB-123"] for call in calls))

    def test_context_only_bundle_can_be_ranked(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temp = Path(raw)
            source = temp / "context.json"
            triaged = temp / "triaged.json"
            operator.write_json(
                source,
                {
                    "schema": operator.BUNDLE_SCHEMA,
                    "generated_at": "2026-05-12T12:00:00+00:00",
                    "commands": [],
                    "issues": [],
                    "issue_contexts": {
                        "WEB-123": {
                            "view": {
                                "shortId": "WEB-123",
                                "title": "Hydrated issue only",
                                "culprit": "src/session.ts",
                                "count": "100",
                                "userCount": 10,
                                "lastSeen": "2026-05-12T11:30:00+00:00",
                                "level": "error",
                                "priority": "high",
                                "project": {"slug": "web"},
                                "isUnhandled": True,
                            }
                        }
                    },
                },
            )
            self.assertEqual(operator.main(["triage", str(source), "--out", str(triaged)]), 0)
            ranked = json.loads(triaged.read_text())["ranked_issues"]
            self.assertEqual(ranked[0]["issue"]["shortId"], "WEB-123")

    def test_tag_values_extract_trace_and_replay_tags(self) -> None:
        context = {
            "events": [
                {
                    "tags": [
                        {"key": "trace", "value": "trace-123"},
                        {"key": "replayId", "value": "replay-456"},
                    ]
                }
            ]
        }
        self.assertEqual(operator.tag_values(context, {"trace"}), ["trace-123"])
        self.assertEqual(operator.tag_values(context, {"replayid"}), ["replay-456"])

    def test_validate_bundle_reports_invalid_json_without_traceback(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "not-json.md"
            path.write_text("# not json\n")
            stderr = io.StringIO()
            with redirect_stderr(stderr):
                self.assertEqual(operator.main(["validate-bundle", str(path)]), 2)
            self.assertIn("not valid JSON", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
