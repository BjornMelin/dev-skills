#!/usr/bin/env python3
"""Tests for the sentry-cli-fix-issues context collector."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "collect_issue_context.py"
SPEC = importlib.util.spec_from_file_location("collect_issue_context", SCRIPT)
assert SPEC is not None
assert SPEC.loader is not None
collect_issue_context = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(collect_issue_context)


class CollectIssueContextTests(unittest.TestCase):
    """Tests for collect_issue_context behavior."""

    def test_redact_sensitive_keys_and_values(self) -> None:
        """Redact sensitive keys and sensitive scalar patterns."""
        payload = {
            "Authorization": "Bearer abcdefghijklmnopqrstuvwxyz123456",
            "email": "user@example.com",
            "dsn": "https://public@example.ingest.sentry.io/123",
            "safe": "request had expected shape",
        }

        redacted = collect_issue_context.redact(payload)

        self.assertEqual(redacted["Authorization"], "[REDACTED]")
        self.assertEqual(redacted["email"], "[REDACTED_EMAIL]")
        self.assertEqual(redacted["dsn"], "[REDACTED]")
        self.assertEqual(redacted["safe"], "request had expected shape")

    def test_unique_preserves_order_and_limit(self) -> None:
        """Preserve first-seen ordering when de-duplicating values."""
        values = collect_issue_context.unique(["a", "b", "a", "c"], 2)

        self.assertEqual(values, ["a", "b"])

    def test_nested_trace_and_tag_helpers_extract_linked_ids(self) -> None:
        """Extract trace and replay identifiers from nested event data."""
        event = {
            "contexts": {"trace": {"trace_id": "trace-1"}},
            "tags": [
                {"key": "trace", "value": "trace-2"},
                {"key": "replayId", "value": "replay-1"},
            ],
        }

        self.assertEqual(
            collect_issue_context.nested_values(event, {"trace_id"}),
            ["trace-1"],
        )
        self.assertEqual(
            collect_issue_context.tag_values(event, "trace"),
            ["trace-2"],
        )
        self.assertEqual(
            collect_issue_context.tag_values(event, "replayId"),
            ["replay-1"],
        )
        self.assertEqual(
            collect_issue_context.tag_values(event, "replayid"),
            ["replay-1"],
        )

    def test_extract_issue_identifiers(self) -> None:
        """Extract organization and issue identifiers from issue payloads."""
        issue_view = {
            "id": "123",
            "project": {"organization": {"slug": "acme"}},
        }

        self.assertEqual(
            collect_issue_context.extract_issue_id(issue_view),
            "123",
        )
        self.assertEqual(collect_issue_context.extract_org(issue_view), "acme")

    def test_issue_tag_endpoint_is_relative_to_api_root(self) -> None:
        """Build relative tag-value API paths for `sentry api`."""
        endpoint = collect_issue_context.issue_tag_values_endpoint(
            "acme/org",
            "123/456",
            "release/stage",
        )

        self.assertEqual(
            endpoint,
            "organizations/acme%2Forg/issues/123%2F456/"
            "tags/release%2Fstage/values/",
        )
        self.assertFalse(endpoint.startswith("/api/0/"))

    def test_positive_int_rejects_non_positive_values(self) -> None:
        """Reject non-positive values before subprocess timeouts or limits."""
        self.assertEqual(collect_issue_context.positive_int("3"), 3)

        for flag in ("--timeout", "--limit-events", "--max-string"):
            with contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit):
                    collect_issue_context.build_parser().parse_args(
                        ["WEB-1", flag, "0"]
                    )


if __name__ == "__main__":
    unittest.main()
