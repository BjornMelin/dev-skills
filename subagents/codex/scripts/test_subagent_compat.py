#!/usr/bin/env python3
"""Regression checks for Codex subagent configuration compatibility."""

from __future__ import annotations

import json
import os
import subprocess
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
                    'developer_instructions = "Do not spawn nested '
                    'subagents.\\n'
                    'Treat the parent prompt as the authority.\\n'
                    'Redact secrets.\\n'
                    'Return format:\\n'
                    '- Status\\n- Risks/blockers"\n'
                    f'model_reasoning_effort = "{effort}"\n',
                    encoding="utf-8",
                )
                messages = [
                    issue.message
                    for issue in validate_agent_file(path)
                ]
                self.assertEqual(messages, [], messages)

    def test_doctor_uses_only_trusted_project_config(self) -> None:
        """Report trusted project overrides and ignore untrusted ones."""

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            codex_home = root / "codex-home"
            project_dir = root / "project"
            codex_home.mkdir()
            (project_dir / ".codex").mkdir(parents=True)
            project_path = project_dir.as_posix()
            project_config = project_dir / ".codex" / "config.toml"
            project_config.write_text(
                "[features.multi_agent_v2]\n"
                "max_concurrent_threads_per_session = 9\n",
                encoding="utf-8",
            )

            def _run_doctor(trust_level: str) -> dict[str, object]:
                (codex_home / "config.toml").write_text(
                    f'[projects."{project_path}"]\n'
                    f'trust_level = "{trust_level}"\n'
                    "[features.multi_agent_v2]\n"
                    "enabled = true\n",
                    encoding="utf-8",
                )
                result = subprocess.run(
                    [
                        sys.executable,
                        str(
                            REPO_ROOT
                            / "skills/subagent-creator/scripts/"
                            "subagent_creator.py"
                        ),
                        "doctor",
                        "--codex-bin",
                        "missing-codex",
                        "--project-dir",
                        str(project_dir),
                        "--json",
                    ],
                    check=True,
                    capture_output=True,
                    text=True,
                    env={**os.environ, "CODEX_HOME": str(codex_home)},
                )
                return json.loads(result.stdout)

            key = (
                "features.multi_agent_v2."
                "max_concurrent_threads_per_session"
            )
            trusted = _run_doctor("trusted")
            self.assertEqual(trusted[key], 9)
            self.assertEqual(
                trusted["features.multi_agent_v2"],
                {
                    "enabled": True,
                    "max_concurrent_threads_per_session": 9,
                },
            )
            self.assertIsNone(_run_doctor("untrusted")[key])


if __name__ == "__main__":
    unittest.main()
