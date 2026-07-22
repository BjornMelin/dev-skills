#!/usr/bin/env python3
"""Regression checks for Codex subagent configuration compatibility."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO_ROOT / "subagents/codex/scripts"))
sys.path.insert(0, str(REPO_ROOT / "skills/subagent-creator/scripts"))

from render_agents import load_local_roles  # noqa: E402
from subagent_creator import validate_agent_file  # noqa: E402


class SubagentCompatibilityTests(unittest.TestCase):
    """Verify compatibility with current and legacy subagent configuration."""

    def test_local_role_without_model_uses_sol_default(self) -> None:
        """Default a missing local role model to GPT-5.6 Sol."""

        with tempfile.TemporaryDirectory() as temp_dir:
            manifest = Path(temp_dir) / "roles.local.json"
            manifest.write_text(
                json.dumps(
                    {
                        "roles": [
                            {
                                "name": "private_reviewer",
                                "description": "Private review role.",
                                "effort": "medium",
                                "sandbox": "read-only",
                                "family": "private",
                                "body": (
                                    "Review only the assigned private "
                                    "scope."
                                ),
                            }
                        ]
                    }
                ),
                encoding="utf-8",
            )

            roles = load_local_roles(manifest)

        self.assertEqual(roles[0].model, "gpt-5.6-sol")

    def test_validator_accepts_none_and_ultra_effort(self) -> None:
        """Accept the current lowest and highest reasoning efforts."""

        with tempfile.TemporaryDirectory() as temp_dir:
            for effort in ("none", "ultra"):
                path = Path(temp_dir) / f"reviewer_{effort}.toml"
                path.write_text(
                    f'name = "reviewer_{effort}"\n'
                    'description = "Compatibility check."\n'
                    'developer_instructions = "Return format:\\n'
                    '- Status\\n- Risks/blockers"\n'
                    f'model_reasoning_effort = "{effort}"\n',
                    encoding="utf-8",
                )
                messages = [
                    issue.message
                    for issue in validate_agent_file(path)
                ]
                self.assertFalse(
                    any(
                        "model_reasoning_effort" in message
                        for message in messages
                    ),
                    messages,
                )


if __name__ == "__main__":
    unittest.main()
