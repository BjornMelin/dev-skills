#!/usr/bin/env python3
"""Regression tests for upgrade-pack-generator."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import textwrap
import unittest
import json
from pathlib import Path
from typing import Any
from unittest.mock import patch


SKILL_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = SKILL_ROOT / "scripts"
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

import bootstrap_manifest  # type: ignore  # noqa: E402
import common  # type: ignore  # noqa: E402
import enrich_manifest  # type: ignore  # noqa: E402
import qualify_upgrade_pack  # type: ignore  # noqa: E402
import research_upgrade_pack  # type: ignore  # noqa: E402


def write_text(path: Path, content: str) -> None:
    """Write a UTF-8 fixture file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(content).strip() + "\n", encoding="utf-8")


def make_signr_like_repo(root: Path) -> None:
    """Create a minimal monorepo fixture covering Next, Expo, Convex, and Turborepo."""
    write_text(
        root / "package.json",
        """
        {
          "name": "signr",
          "private": true,
          "packageManager": "bun@1.3.12",
          "workspaces": ["apps/*", "packages/backend", "packages/shared"],
          "scripts": {
            "lint": "bun run lint:root && turbo run lint",
            "typecheck": "bun run scripts/run-typecheck.mjs",
            "test": "turbo run test",
            "build": "turbo run build",
            "validate:local:agent": "bun run scripts/validate-local-agent.mjs",
            "convex:verify:release": "bun run --filter @signr/backend convex:verify:release"
          },
          "devDependencies": {
            "turbo": "2.9.6",
            "convex": "1.35.1",
            "expo-doctor": "^1.18.18"
          }
        }
        """,
    )
    write_text(
        root / "turbo.json",
        """
        {
          "$schema": "https://turborepo.dev/schema.v2.json",
          "tasks": {
            "build": { "dependsOn": ["^build"], "outputs": [".next/**", "dist/**"] },
            "lint": { "outputs": [] },
            "test": { "outputs": [] },
            "typecheck": { "outputs": [] }
          },
          "futureFlags": {
            "affectedUsingTaskInputs": true
          }
        }
        """,
    )
    write_text(
        root / "apps/web/package.json",
        """
        {
          "name": "@signr/web",
          "private": true,
          "scripts": {
            "typegen": "next typegen",
            "typecheck": "bun run typegen && tsc --noEmit",
            "lint": "bun run typegen && eslint . --max-warnings=0",
            "test": "vitest run",
            "build": "next build"
          },
          "dependencies": {
            "next": "16.2.4",
            "react": "19.2.0",
            "react-dom": "19.2.0"
          }
        }
        """,
    )
    write_text(
        root / "apps/web/next.config.ts",
        """
        export default {
          cacheComponents: true,
          typedRoutes: true,
          turbopack: { root: __dirname }
        };
        """,
    )
    write_text(root / "apps/web/app/page.tsx", "export default function Page() { return null; }")
    write_text(root / "apps/web/proxy.ts", "export function proxy() { return null; }")

    write_text(
        root / "apps/mobile/package.json",
        """
        {
          "name": "@signr/mobile",
          "private": true,
          "main": "expo-router/entry",
          "scripts": {
            "doctor": "expo-doctor",
            "doctor:ci": "bun ./scripts/run-mobile-doctor.ts",
            "deps:check": "expo install --check",
            "workflows:validate": "bun ./scripts/validate-eas-workflows.mjs",
            "typecheck": "tsc --noEmit",
            "lint": "eslint . --max-warnings=0",
            "test": "vitest run",
            "build:smoke:android": "bun ./scripts/build-smoke-android.mjs"
          },
          "dependencies": {
            "expo": "~55.0.15",
            "expo-router": "~55.0.12",
            "expo-updates": "~55.0.20",
            "expo-dev-client": "~55.0.27",
            "react-native": "0.83.4",
            "react-native-web": "~0.21.0",
            "react": "19.2.0",
            "react-dom": "19.2.0"
          },
          "devDependencies": {
            "expo-doctor": "^1.18.18"
          }
        }
        """,
    )
    write_text(root / "apps/mobile/app.config.ts", "export default { name: 'mobile' };")
    write_text(
        root / "apps/mobile/eas.json",
        """
        {
          "build": {
            "preview": {},
            "production": {}
          }
        }
        """,
    )

    write_text(
        root / "packages/backend/package.json",
        """
        {
          "name": "@signr/backend",
          "private": true,
          "scripts": {
            "lint": "eslint . --max-warnings=0",
            "typecheck": "tsc --noEmit",
            "test": "vitest run",
            "convex:codegen:strict": "convex codegen",
            "convex:dev:once:strict": "convex dev --once",
            "convex:verify:release": "bun run convex:codegen:strict && bun run convex:dev:once:strict"
          },
          "dependencies": {
            "convex": "1.35.1",
            "convex-helpers": "0.1.114",
            "@convex-dev/workflow": "^0.3.9"
          }
        }
        """,
    )
    write_text(root / "packages/backend/convex/schema.ts", "export const tables = defineTable({});")
    write_text(root / "packages/backend/convex/_generated/api.d.ts", "export {};")

    write_text(root / "packages/shared/package.json", '{"name":"@signr/shared","private":true}')
    write_text(root / "apps/web/turbo.json", '{"extends":["//"],"tasks":{"lint":{}}}')
    write_text(root / "apps/mobile/turbo.json", '{"extends":["//"],"tasks":{"lint":{}}}')
    write_text(root / "packages/backend/turbo.json", '{"extends":["//"],"tasks":{"typecheck":{}}}')
    write_text(root / "packages/shared/turbo.json", '{"extends":["//"],"tasks":{"build":{}}}')

    write_text(root / ".agents/skills/template/package.json", '{"name":"ignored-template","dependencies":{"next":"0.0.0"}}')
    write_text(root / ".agents/skills/next-upgrade/SKILL.md", "---\nname: next-upgrade\n---\n# Next upgrade\n")
    write_text(root / ".agents/skills/convex-best-practices/SKILL.md", "---\nname: convex-best-practices\n---\n# Convex best practices\n")
    write_text(root / ".agents/skills/monorepo-management/SKILL.md", "---\nname: monorepo-management\n---\n# Monorepo management\n")
    write_text(root / ".codex/.tmp/plugins/example/package.json", '{"name":"ignored-plugin","dependencies":{"expo":"0.0.0"}}')


def make_single_next_repo(root: Path) -> None:
    """Create a minimal single-app Next.js repo."""
    write_text(
        root / "package.json",
        """
        {
          "name": "single-next",
          "private": true,
          "packageManager": "pnpm@10.28.0",
          "scripts": {
            "lint": "biome check .",
            "typecheck": "tsc --noEmit",
            "test": "vitest run",
            "build": "next build"
          },
          "dependencies": {
            "next": "16.2.4",
            "react": "19.2.0",
            "react-dom": "19.2.0"
          }
        }
        """,
    )
    write_text(root / "next.config.ts", "export default { typedRoutes: true };")
    write_text(root / "app/page.tsx", "export default function Page() { return null; }")


class UpgradePackGeneratorTests(unittest.TestCase):
    """Regression coverage for owner selection and rendering."""

    def build_manifest(self, repo_root: Path, anchor_package: str) -> dict:
        """Create a finalized manifest without running the CLI wrappers."""
        repo_context = common.detect_repo_context(repo_root)
        override = bootstrap_manifest.match_override(anchor_package, Path(bootstrap_manifest.__file__))
        family_slug = (override or {}).get("family_slug") or common.normalize_slug(anchor_package)
        manifest = bootstrap_manifest.generic_manifest(anchor_package, repo_context, family_slug)
        if override:
            manifest = common.recursive_merge(manifest, override)
        return bootstrap_manifest.finalize_manifest(manifest)

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_detect_repo_context_skips_internal_manifests(self, _mock_fetch) -> None:
        """Framework detection should ignore `.agents` and `.codex` package manifests."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            make_signr_like_repo(root)
            context = common.detect_repo_context(root)
            self.assertIn("nextjs", context["frameworks_detected"])
            self.assertIn("expo", context["frameworks_detected"])
            self.assertIn("convex", context["frameworks_detected"])
            self.assertNotIn(".agents/skills/template/package.json", context["package_json_paths"])
            self.assertNotIn(".codex/.tmp/plugins/example/package.json", context["package_json_paths"])

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_next_owner_workspace_and_naming(self, _mock_fetch) -> None:
        """Next.js monorepo packs should anchor on the owning workspace."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            make_signr_like_repo(root)
            manifest = self.build_manifest(root, "next")
            enriched = enrich_manifest.enrich_next_manifest(manifest, root)
            self.assertEqual(enriched["current_version"], "16.2.4")
            self.assertEqual(enriched["target_surface"]["workspace_path"], "apps/web")
            self.assertEqual(enriched["plan_basename"], "nextjs-apps-web-v16-upgrade-and-optimization")
            self.assertIn("bun run --filter @signr/web typecheck", enriched["verification_commands"])

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_expo_convex_and_turborepo_owner_selection(self, _mock_fetch) -> None:
        """Expo, Convex, and Turborepo should resolve the expected owner surfaces."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            make_signr_like_repo(root)

            expo_manifest = enrich_manifest.enrich_expo_manifest(self.build_manifest(root, "expo"), root)
            self.assertEqual(expo_manifest["target_surface"]["workspace_path"], "apps/mobile")
            self.assertEqual(expo_manifest["plan_basename"], "expo-eas-apps-mobile-sdk55-upgrade-and-optimization")

            convex_manifest = enrich_manifest.enrich_convex_manifest(self.build_manifest(root, "convex"), root)
            self.assertEqual(convex_manifest["target_surface"]["workspace_path"], "packages/backend")
            self.assertEqual(convex_manifest["plan_basename"], "convex-packages-backend-upgrade-and-optimization")

            turbo_manifest = enrich_manifest.enrich_turborepo_manifest(self.build_manifest(root, "turbo"), root)
            self.assertEqual(turbo_manifest["target_surface"]["workspace_path"], ".")
            self.assertEqual(turbo_manifest["target_surface"]["workspace_slug"], "root")
            self.assertEqual(turbo_manifest["target_surface"]["verification_strategy"], "root-plus-package-configs")

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_next_root_owner_for_single_app(self, _mock_fetch) -> None:
        """Single-app Next.js repos should stay root-owned."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            make_single_next_repo(root)
            manifest = self.build_manifest(root, "next")
            enriched = enrich_manifest.enrich_next_manifest(manifest, root)
            self.assertEqual(enriched["target_surface"]["workspace_path"], ".")
            self.assertEqual(enriched["target_surface"]["workspace_slug"], "root")
            self.assertEqual(enriched["plan_basename"], "nextjs-v16-upgrade-and-optimization")

    def test_override_aliases_route_to_family(self) -> None:
        """Related package anchors should still resolve to the intended family overrides."""
        expo_override = bootstrap_manifest.match_override("expo-router", Path(bootstrap_manifest.__file__))
        self.assertIsNotNone(expo_override)
        self.assertEqual(expo_override["family_slug"], "expo-eas")

        convex_override = bootstrap_manifest.match_override("@convex-dev/workflow", Path(bootstrap_manifest.__file__))
        self.assertIsNotNone(convex_override)
        self.assertEqual(convex_override["family_slug"], "convex")

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_enriched_manifest_includes_qualification_plan_and_overlays(self, _mock_fetch) -> None:
        """Family enrichment should populate qualification plans and local overlay notes."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            make_signr_like_repo(root)

            next_manifest = enrich_manifest.enrich_next_manifest(self.build_manifest(root, "next"), root)
            next_plan = next_manifest["qualification_plan"]
            next_research = next_manifest["research_plan"]
            self.assertEqual(next_plan["snapshot_filename"], "qualification-snapshot.json")
            self.assertTrue(any(check["label"] == "Next.js codemod help" for check in next_plan["cli_checks"]))
            self.assertIn("next@16.2.4", next_plan["source_specs"])
            self.assertTrue(next_manifest["repo_local_skill_overlays"])
            self.assertEqual(next_research["snapshot_filename"], "research-snapshot.json")
            self.assertIn("release_history", next_research["required_categories"])
            self.assertIn("docs home", next_research["official_docs"])
            self.assertIn("github releases", next_research["release_history"])
            self.assertTrue(next_research["repo_usage_queries"])

            convex_manifest = enrich_manifest.enrich_convex_manifest(self.build_manifest(root, "convex"), root)
            convex_plan = convex_manifest["qualification_plan"]
            self.assertTrue(any(check["label"] == "Convex CLI help" for check in convex_plan["cli_checks"]))
            self.assertTrue(any(overlay["skill_name"] == "convex-best-practices" for overlay in convex_manifest["repo_local_skill_overlays"]))
            self.assertNotIn("Repo-local skill overlays", convex_manifest.get("repo_probes", {}))

            turbo_manifest = enrich_manifest.enrich_turborepo_manifest(self.build_manifest(root, "turbo"), root)
            turbo_plan = turbo_manifest["qualification_plan"]
            self.assertTrue(any(check["label"] == "Turbo query help" for check in turbo_plan["cli_checks"]))
            self.assertTrue(any(overlay["skill_name"] == "monorepo-management" for overlay in turbo_manifest["repo_local_skill_overlays"]))

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_generic_override_family_keeps_override_source_specs(self, _mock_fetch) -> None:
        """Generic override families should merge repo-version and override source specs."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_text(
                root / "package.json",
                """
                {
                  "name": "ui-upgrades",
                  "private": true,
                  "packageManager": "pnpm@10.28.0",
                  "dependencies": {
                    "lucide-react": "1.8.0",
                    "radix-ui": "1.4.2"
                  }
                }
                """,
            )

            lucide_manifest = enrich_manifest.enrich_generic_manifest(self.build_manifest(root, "lucide-react"), root)
            self.assertIn("lucide-react@1.8.0", lucide_manifest["qualification_plan"]["source_specs"])
            self.assertIn("lucide-icons/lucide", lucide_manifest["qualification_plan"]["source_specs"])
            self.assertIn("lucide-react@1.8.0", lucide_manifest["research_plan"]["source_specs"])
            self.assertIn("lucide-icons/lucide", lucide_manifest["research_plan"]["source_specs"])

            shadcn_manifest = enrich_manifest.enrich_generic_manifest(self.build_manifest(root, "radix-ui"), root)
            self.assertIn("radix-ui@1.4.2", shadcn_manifest["qualification_plan"]["source_specs"])
            self.assertIn("shadcn-ui/ui", shadcn_manifest["qualification_plan"]["source_specs"])
            self.assertIn("radix-ui/primitives", shadcn_manifest["qualification_plan"]["source_specs"])
            self.assertIn("radix-ui@1.4.2", shadcn_manifest["research_plan"]["source_specs"])
            self.assertIn("shadcn-ui/ui", shadcn_manifest["research_plan"]["source_specs"])
            self.assertIn("radix-ui/primitives", shadcn_manifest["research_plan"]["source_specs"])

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    @patch.object(research_upgrade_pack, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_research_snapshot_complete_for_next_family(self, _research_fetch, _enrich_fetch) -> None:
        """Research stage should produce a complete snapshot for built-in family packs."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            make_signr_like_repo(root)
            manifest = enrich_manifest.enrich_next_manifest(self.build_manifest(root, "next"), root)
            opensrc_root = root / "opensrc-cache" / "next"
            write_text(opensrc_root / "CHANGELOG.md", "# Changelog")
            write_text(opensrc_root / "MIGRATION.md", "# Migration")
            write_text(opensrc_root / "examples" / "app" / "README.md", "# Example")

            def fake_run_shell(command: str, _cwd: Path) -> dict[str, Any]:
                if command.startswith("opensrc path "):
                    return {
                        "exit_code": 0,
                        "status": "ok",
                        "stdout_excerpt": [str(opensrc_root)],
                        "stderr_excerpt": [],
                        "summary": [str(opensrc_root)],
                    }
                return {
                    "exit_code": 0,
                    "status": "ok",
                    "stdout_excerpt": ["ok"],
                    "stderr_excerpt": [],
                    "summary": ["ok"],
                }

            with patch.object(research_upgrade_pack, "run_shell", side_effect=fake_run_shell):
                snapshot = research_upgrade_pack.generate_snapshot(manifest, root)

            self.assertEqual(snapshot["research_status"], "complete")
            self.assertEqual(snapshot["category_status"]["release_history"], "ok")
            self.assertEqual(snapshot["category_status"]["repo_usage_mapping"], "ok")
            self.assertEqual(snapshot["summary"]["missing_categories"], 0)
            self.assertTrue(snapshot["source_evidence"][0]["release_note_files"])

    def test_repo_usage_rg_no_matches_is_not_a_failure(self) -> None:
        """Read-only grep probes should treat exit code 1 as valid no-match evidence."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)

            def fake_run_shell(_command: str, _cwd: Path) -> dict[str, Any]:
                return {
                    "exit_code": 1,
                    "status": "failed",
                    "stdout_excerpt": [],
                    "stderr_excerpt": [],
                    "summary": [],
                }

            with patch.object(research_upgrade_pack, "run_shell", side_effect=fake_run_shell):
                entries = research_upgrade_pack.repo_usage_entries(
                    root,
                    [{"label": "No matches", "cwd": ".", "command": "rg -n 'missing-pattern' ."}],
                )

            self.assertEqual(entries[0]["status"], "ok")
            self.assertEqual(entries[0]["summary"], ["no matches"])
            self.assertEqual(entries[0]["exit_code"], 1)

    def test_qualification_status_requires_complete_research(self) -> None:
        """Qualification should not mark a pack ready when research is incomplete."""
        summary = {
            "doc_checks": 2,
            "doc_failures": 0,
            "source_checks": 1,
            "source_failures": 0,
            "cli_checks": 2,
            "cli_failures": 0,
            "repo_local_overlays": 0,
            "research_status": "partial",
        }
        status, caveats = qualify_upgrade_pack.qualification_status(
            summary,
            {"research_status": "partial"},
        )
        self.assertEqual(status, "ready-with-caveats")
        self.assertTrue(any("research stage" in caveat for caveat in caveats))

        ready_status, ready_caveats = qualify_upgrade_pack.qualification_status(
            summary | {"research_status": "complete"},
            {"research_status": "complete"},
        )
        self.assertEqual(ready_status, "ready")
        self.assertEqual(ready_caveats, [])

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_rendered_trigger_prompt_wraps_long_lines(self, _mock_fetch) -> None:
        """Rendered packs should use the grouped playbook and linked thin launcher outputs."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            out = Path(tmp) / "rendered"
            make_signr_like_repo(root)
            manifest = self.build_manifest(root, "expo")
            enriched = enrich_manifest.enrich_expo_manifest(manifest, root)
            manifest_path = Path(tmp) / "upgrade-pack.yaml"
            common.dump_yaml(manifest_path, enriched)
            research_snapshot = {
                "schema_version": 1,
                "generated_at": "2026-04-19T00:00:00Z",
                "family_slug": enriched["family_slug"],
                "anchor_package": enriched["anchor_package"],
                "repo_root": str(root),
                "snapshot_filename": "research-snapshot.json",
                "research_status": "complete",
                "current_version": enriched["current_version"],
                "target_version": enriched["validated_upstream_version"],
                "target_version_policy": "latest-compatible-stable",
                "compatibility_rationale": "Use the latest compatible stable release.",
                "release_range": f"{enriched['current_version']} -> {enriched['validated_upstream_version']}",
                "required_categories": enriched["research_plan"]["required_categories"],
                "category_status": {category: "ok" for category in enriched["research_plan"]["required_categories"]},
                "summary": {
                    "required_categories": len(enriched["research_plan"]["required_categories"]),
                    "ok_categories": len(enriched["research_plan"]["required_categories"]),
                    "partial_categories": 0,
                    "failed_categories": 0,
                    "missing_categories": 0,
                    "official_docs": len(enriched["research_plan"]["official_docs"]),
                    "api_reference": len(enriched["research_plan"]["api_reference"]),
                    "migration_guides": len(enriched["research_plan"]["migration_guides"]),
                    "release_history": len(enriched["research_plan"]["release_history"]),
                    "examples_cookbooks": len(enriched["research_plan"]["examples_cookbooks"]),
                    "source_evidence": len(enriched["research_plan"]["source_specs"]),
                    "repo_usage_mapping": len(enriched["research_plan"]["repo_usage_queries"]),
                },
                "official_docs": [],
                "api_reference": [],
                "migration_guides": [],
                "release_history": [],
                "examples_cookbooks": [],
                "source_evidence": [],
                "repo_usage_mapping": [],
                "caveats": [],
            }
            research_path = Path(tmp) / "research-snapshot.json"
            research_path.write_text(json.dumps(research_snapshot), encoding="utf-8")
            qualification_snapshot = {
                "schema_version": 1,
                "generated_at": "2026-04-19T00:00:00Z",
                "family_slug": enriched["family_slug"],
                "anchor_package": enriched["anchor_package"],
                "repo_root": str(root),
                "snapshot_filename": "qualification-snapshot.json",
                "qualification_status": "ready",
                "summary": {
                    "doc_checks": 2,
                    "doc_failures": 0,
                    "source_checks": 1,
                    "source_failures": 0,
                    "cli_checks": 2,
                    "cli_failures": 0,
                    "repo_local_overlays": 0,
                },
                "doc_checks": [],
                "source_checks": [],
                "cli_checks": [],
                "repo_local_skill_overlays": [],
                "caveats": [],
            }
            qualification_path = Path(tmp) / "qualification-snapshot.json"
            qualification_path.write_text(json.dumps(qualification_snapshot), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPTS_DIR / "render_upgrade_pack.py"),
                    "--manifest",
                    str(manifest_path),
                    "--research-snapshot",
                    str(research_path),
                    "--qualification-snapshot",
                    str(qualification_path),
                    "--output-dir",
                    str(out),
                ],
                check=True,
            )
            trigger_path = out / enriched["trigger_filename"]
            playbook_path = out / enriched["playbook_filename"]
            operator_path = out / enriched["operator_filename"]
            playbook_text = playbook_path.read_text(encoding="utf-8")
            operator_text = operator_path.read_text(encoding="utf-8")
            trigger_text = trigger_path.read_text(encoding="utf-8")
            self.assertTrue((out / "research-snapshot.json").exists())
            self.assertTrue((out / "qualification-snapshot.json").exists())
            self.assertIn("### Research Coverage", playbook_text)
            self.assertIn("status: `complete`", playbook_text)
            self.assertIn("status: `ready`", playbook_text)
            self.assertIn("## Pack Map", playbook_text)
            self.assertIn("## Current State And Evidence", playbook_text)
            self.assertIn("## Decisions And End State", playbook_text)
            self.assertIn("## Execution And Verification", playbook_text)
            self.assertIn("## Live Tracker And Closeout", playbook_text)
            self.assertIn("### Findings Matrix", playbook_text)
            self.assertIn("### Decision Log", playbook_text)
            self.assertIn("### Affected Files Map", playbook_text)
            self.assertIn("### Change Checklist", playbook_text)
            self.assertIn("### Verification Evidence", playbook_text)
            self.assertIn("### Residual Risks / Defers", playbook_text)
            self.assertIn("## Read This First", operator_text)
            self.assertIn("## Non-Negotiable Guardrails", operator_text)
            self.assertNotIn("## Family Profile", operator_text)
            self.assertNotIn("## Target Surface", operator_text)
            self.assertIn("Research status for this pack: `complete`.", operator_text)
            self.assertIn(f"./{enriched['playbook_filename']}#current-state-and-evidence", operator_text)
            self.assertIn(f"./{enriched['playbook_filename']}#live-tracker-and-closeout", operator_text)
            self.assertIn("Repo-specific summary:", trigger_text)
            self.assertNotIn("Goals:", trigger_text)
            self.assertNotIn("Required decisions:", trigger_text)
            self.assertNotIn("- -", trigger_text)
            self.assertIn(f"PLAYBOOK=./{enriched['playbook_filename']}", trigger_text)
            self.assertIn("${PLAYBOOK}#pack-map", trigger_text)
            self.assertIn("${PLAYBOOK}#execution-and-verification", trigger_text)
            self.assertIn("${PLAYBOOK}#live-tracker-and-closeout", trigger_text)
            self.assertTrue(
                playbook_text.startswith(
                    "<!-- markdownlint-disable MD013 -->\n"
                )
            )
            self.assertTrue(
                operator_text.startswith(
                    "<!-- markdownlint-disable MD013 -->\n"
                )
            )
            self.assertTrue(
                trigger_text.startswith(
                    "<!-- markdownlint-disable MD013 -->\n"
                )
            )

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_rendered_playbook_does_not_duplicate_repo_local_overlays(self, _mock_fetch) -> None:
        """Repo-local overlays should render once in their dedicated section."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            out = Path(tmp) / "rendered"
            make_signr_like_repo(root)
            manifest = self.build_manifest(root, "convex")
            enriched = enrich_manifest.enrich_convex_manifest(manifest, root)
            manifest_path = Path(tmp) / "upgrade-pack.yaml"
            common.dump_yaml(manifest_path, enriched)
            research_snapshot = {
                "schema_version": 1,
                "generated_at": "2026-04-19T00:00:00Z",
                "family_slug": enriched["family_slug"],
                "anchor_package": enriched["anchor_package"],
                "repo_root": str(root),
                "snapshot_filename": "research-snapshot.json",
                "research_status": "complete",
                "current_version": enriched["current_version"],
                "target_version": enriched["validated_upstream_version"],
                "target_version_policy": "latest-compatible-stable",
                "compatibility_rationale": "Use the latest compatible stable release.",
                "release_range": f"{enriched['current_version']} -> {enriched['validated_upstream_version']}",
                "required_categories": enriched["research_plan"]["required_categories"],
                "category_status": {category: "ok" for category in enriched["research_plan"]["required_categories"]},
                "summary": {
                    "required_categories": len(enriched["research_plan"]["required_categories"]),
                    "ok_categories": len(enriched["research_plan"]["required_categories"]),
                    "partial_categories": 0,
                    "failed_categories": 0,
                    "missing_categories": 0,
                    "official_docs": len(enriched["research_plan"]["official_docs"]),
                    "api_reference": len(enriched["research_plan"]["api_reference"]),
                    "migration_guides": len(enriched["research_plan"]["migration_guides"]),
                    "release_history": len(enriched["research_plan"]["release_history"]),
                    "examples_cookbooks": len(enriched["research_plan"]["examples_cookbooks"]),
                    "source_evidence": len(enriched["research_plan"]["source_specs"]),
                    "repo_usage_mapping": len(enriched["research_plan"]["repo_usage_queries"]),
                },
                "official_docs": [],
                "api_reference": [],
                "migration_guides": [],
                "release_history": [],
                "examples_cookbooks": [],
                "source_evidence": [],
                "repo_usage_mapping": [],
                "caveats": [],
            }
            research_path = Path(tmp) / "research-snapshot.json"
            research_path.write_text(json.dumps(research_snapshot), encoding="utf-8")
            qualification_snapshot = {
                "schema_version": 1,
                "generated_at": "2026-04-19T00:00:00Z",
                "family_slug": enriched["family_slug"],
                "anchor_package": enriched["anchor_package"],
                "repo_root": str(root),
                "snapshot_filename": "qualification-snapshot.json",
                "qualification_status": "ready",
                "summary": {
                    "doc_checks": 2,
                    "doc_failures": 0,
                    "source_checks": 1,
                    "source_failures": 0,
                    "cli_checks": 2,
                    "cli_failures": 0,
                    "repo_local_overlays": len(enriched["repo_local_skill_overlays"]),
                },
                "doc_checks": [],
                "source_checks": [],
                "cli_checks": [],
                "repo_local_skill_overlays": enriched["repo_local_skill_overlays"],
                "caveats": [],
            }
            qualification_path = Path(tmp) / "qualification-snapshot.json"
            qualification_path.write_text(json.dumps(qualification_snapshot), encoding="utf-8")

            subprocess.run(
                [
                    "python3",
                    str(SCRIPTS_DIR / "render_upgrade_pack.py"),
                    "--manifest",
                    str(manifest_path),
                    "--research-snapshot",
                    str(research_path),
                    "--qualification-snapshot",
                    str(qualification_path),
                    "--output-dir",
                    str(out),
                ],
                check=True,
            )

            playbook_text = (out / enriched["playbook_filename"]).read_text(encoding="utf-8")
            self.assertEqual(playbook_text.count("## Repo-Local Skill Overlays"), 1)
            self.assertNotIn("### Repo-local skill overlays", playbook_text)


if __name__ == "__main__":
    unittest.main()
