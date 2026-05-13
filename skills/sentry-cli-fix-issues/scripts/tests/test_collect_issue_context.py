#!/usr/bin/env python3
"""Tests for the sentry-cli-fix-issues context collector."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "collect_issue_context.py"
SPEC = importlib.util.spec_from_file_location("collect_issue_context", SCRIPT)
assert SPEC and SPEC.loader
collect_issue_context = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(collect_issue_context)


class CollectIssueContextTests(unittest.TestCase):
    def test_redact_sensitive_keys_and_values(self) -> None:
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
        values = collect_issue_context.unique(["a", "b", "a", "c"], 2)

        self.assertEqual(values, ["a", "b"])

    def test_nested_trace_and_tag_helpers_extract_linked_ids(self) -> None:
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
        self.assertEqual(collect_issue_context.tag_values(event, "trace"), ["trace-2"])
        self.assertEqual(
            collect_issue_context.tag_values(event, "replayId"),
            ["replay-1"],
        )

    def test_extract_issue_identifiers(self) -> None:
        issue_view = {
            "id": "123",
            "project": {"organization": {"slug": "acme"}},
        }

        self.assertEqual(collect_issue_context.extract_issue_id(issue_view), "123")
        self.assertEqual(collect_issue_context.extract_org(issue_view), "acme")

    def test_issue_tag_endpoint_is_relative_to_api_root(self) -> None:
        endpoint = collect_issue_context.issue_tag_values_endpoint(
            "acme org",
            "123",
            "release.stage",
        )

        self.assertEqual(
            endpoint,
            "organizations/acme%20org/issues/123/tags/release.stage/values/",
        )
        self.assertFalse(endpoint.startswith("/api/0/"))


if __name__ == "__main__":
    unittest.main()
