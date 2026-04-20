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
import solve_constraints  # type: ignore  # noqa: E402


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


def make_generic_repo(root: Path) -> None:
    """Create a minimal single-package repo for generic package-lane tests."""
    write_text(
        root / "package.json",
        """
        {
          "name": "generic-upgrade-repo",
          "private": true,
          "packageManager": "pnpm@10.28.0",
          "scripts": {
            "lint": "biome check .",
            "typecheck": "tsc --noEmit",
            "test": "vitest run",
            "build": "tsc --noEmit"
          },
          "dependencies": {
            "zod": "^3.25.76"
          }
        }
        """,
    )
    write_text(
        root / "src/schemas.ts",
        """
        import { z } from "zod";

        export const UserSchema = z.object({
          email: z.string().email()
        });
        """,
    )


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
    @patch.object(research_upgrade_pack, "probe_url", side_effect=lambda url: {"url": url, "status": "ok", "final_url": url, "title": "Doc", "last_updated": "April 1, 2026"})
    @patch.object(research_upgrade_pack, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_research_snapshot_complete_for_next_family(self, _research_fetch, _probe_fetch, _enrich_fetch) -> None:
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
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": f"{opensrc_root}\n",
                        "stdout_excerpt": [str(opensrc_root)],
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": [str(opensrc_root)],
                    }
                if command.startswith("npm view next@16.2.4 "):
                    text = json.dumps(
                        {
                            "peerDependencies": {"react": "^18.2.0 || ^19.0.0", "react-dom": "^18.2.0 || ^19.0.0"},
                            "engines": {"node": ">=20.9.0"},
                            "repository": {"type": "git", "url": "git+https://github.com/vercel/next.js.git"},
                            "homepage": "https://nextjs.org",
                            "bugs": {"url": "https://github.com/vercel/next.js/issues"},
                        }
                    )
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stdout_excerpt": [text],
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("npm view next "):
                    text = json.dumps(
                        {
                            "repository": {"type": "git", "url": "git+https://github.com/vercel/next.js.git"},
                            "homepage": "https://nextjs.org",
                            "bugs": {"url": "https://github.com/vercel/next.js/issues"},
                            "dist-tags": {"latest": "16.2.4"},
                            "versions": ["16.2.2", "16.2.3", "16.2.4"],
                            "peerDependencies": {"react": "^18.2.0 || ^19.0.0", "react-dom": "^18.2.0 || ^19.0.0"},
                            "engines": {"node": ">=20.9.0"},
                            "name": "next",
                        }
                    )
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stdout_excerpt": [text],
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command == "gh api repos/vercel/next.js":
                    text = json.dumps(
                        {
                            "full_name": "vercel/next.js",
                            "default_branch": "canary",
                            "html_url": "https://github.com/vercel/next.js",
                        }
                    )
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stdout_excerpt": [text],
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command == "gh api 'repos/vercel/next.js/releases?per_page=5'":
                    text = json.dumps([{"tag_name": "v16.2.4"}, {"tag_name": "v16.2.3"}])
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stdout_excerpt": [text],
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("ctx7 library next "):
                    stdout = "\n".join(
                        [
                            "1. Title: Next.js",
                            "   Context7-compatible library ID: /vercel/next.js",
                            "   Description: Next.js app router framework.",
                        ]
                    )
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": stdout,
                        "stdout_excerpt": stdout.splitlines(),
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("ctx7 docs /vercel/next.js "):
                    stdout = "\n".join(
                        [
                            "### Next.js docs",
                            "Source: https://nextjs.org/docs",
                            "Use App Router, upgrade guides, and API references.",
                        ]
                    )
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": stdout,
                        "stdout_excerpt": stdout.splitlines(),
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("rg -n "):
                    stdout = "apps/web/package.json:1:\"next\": \"16.2.4\"\napps/web/next.config.ts:1:export default { typedRoutes: true }\n"
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": stdout,
                        "stdout_excerpt": stdout.splitlines(),
                        "stderr": "",
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                return {
                    "command": command,
                    "cwd": str(root),
                    "exit_code": 0,
                    "status": "ok",
                    "stdout": "ok\n",
                    "stdout_excerpt": ["ok"],
                    "stderr": "",
                    "stderr_excerpt": [],
                    "summary": ["ok"],
                }

            web_findings = {
                "entries": [
                    {
                        "category": "official_docs",
                        "url": manifest["research_plan"]["official_docs"]["docs home"],
                        "confirmed": True,
                        "confirmed_at": "2026-04-20T00:00:00Z",
                        "facts": ["Upgrade guide reviewed."],
                    },
                    {
                        "category": "api_reference",
                        "url": next(iter(manifest["research_plan"]["api_reference"].values())),
                        "confirmed": True,
                        "confirmed_at": "2026-04-20T00:00:00Z",
                        "facts": ["API reference reviewed."],
                    },
                ]
            }

            with patch.object(research_upgrade_pack, "run_shell", side_effect=fake_run_shell):
                snapshot, bundle = research_upgrade_pack.generate_snapshot(manifest, root, web_findings)

            self.assertEqual(snapshot["research_status"], "complete")
            self.assertEqual(snapshot["category_status"]["release_history"], "ok")
            self.assertEqual(snapshot["category_status"]["repo_usage_mapping"], "ok")
            self.assertEqual(snapshot["summary"]["missing_categories"], 0)
            self.assertEqual(snapshot["summary"]["required_web_queue"], 2)
            self.assertEqual(snapshot["summary"]["confirmed_web_queue"], 2)
            self.assertTrue(snapshot["source_evidence"][0]["release_note_files"])
            self.assertEqual(snapshot["summary"]["identity_status"], "high-confidence")
            self.assertEqual(bundle["bundle_filename"], "research-bundle.json")
            self.assertEqual(bundle["source_map_seed"]["match_status"], "fresh")

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

    def test_repo_usage_rg_missing_optional_files_is_not_a_failure(self) -> None:
        """Optional missing manifest paths should not downgrade a grep-backed usage probe."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)

            def fake_run_shell(_command: str, _cwd: Path) -> dict[str, Any]:
                return {
                    "exit_code": 2,
                    "status": "failed",
                    "stdout": "package.json:1:\"zod\": \"^4.3.6\"\n",
                    "stderr": "rg: pnpm-workspace.yaml: No such file or directory (os error 2)\n",
                    "stdout_excerpt": ['package.json:1:"zod": "^4.3.6"'],
                    "stderr_excerpt": ["rg: pnpm-workspace.yaml: No such file or directory (os error 2)"],
                    "summary": ['package.json:1:"zod": "^4.3.6"', "rg: pnpm-workspace.yaml: No such file or directory (os error 2)"],
                }

            with patch.object(research_upgrade_pack, "run_shell", side_effect=fake_run_shell):
                entries = research_upgrade_pack.repo_usage_entries(
                    root,
                    [{"label": "Manifest usage", "cwd": ".", "command": "rg -n '\"zod\"' . --glob 'package.json' --glob 'pnpm-workspace.yaml'"}],
                )

            self.assertEqual(entries[0]["status"], "ok")
            self.assertEqual(entries[0]["summary"], ['package.json:1:"zod": "^4.3.6"'])
            self.assertEqual(entries[0]["exit_code"], 2)

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
            summary | {"research_status": "complete", "identity_status": "high-confidence", "identity_confidence": 0.92},
            {"research_status": "complete", "identity": {"status": "high-confidence", "confidence": 0.92}},
        )
        self.assertEqual(ready_status, "ready")
        self.assertEqual(ready_caveats, [])

        identity_caveat_status, identity_caveats = qualify_upgrade_pack.qualification_status(
            summary | {"research_status": "complete", "identity_status": "medium-confidence", "identity_confidence": 0.62},
            {"research_status": "complete", "identity": {"status": "medium-confidence", "confidence": 0.62}},
        )
        self.assertEqual(identity_caveat_status, "ready-with-caveats")
        self.assertTrue(any("package identity" in caveat for caveat in identity_caveats))

    @patch.object(enrich_manifest, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    @patch.object(research_upgrade_pack, "probe_url", side_effect=lambda url: {"url": url, "status": "ok", "final_url": url, "title": "Doc", "last_updated": "April 1, 2026"})
    @patch.object(research_upgrade_pack, "fetch_doc_metadata", return_value=("Doc", "April 1, 2026"))
    def test_generic_research_auto_discovers_identity_and_bundle(self, _research_fetch, _probe_fetch, _enrich_fetch) -> None:
        """Generic packs should auto-discover upstream evidence and write a raw bundle."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "repo"
            make_generic_repo(root)
            manifest = enrich_manifest.enrich_generic_manifest(self.build_manifest(root, "zod"), root)

            def fake_run_shell(command: str, _cwd: Path) -> dict[str, Any]:
                if command.startswith("opensrc path "):
                    opensrc_root = root / "opensrc-cache" / "zod"
                    write_text(opensrc_root / "CHANGELOG.md", "# Changelog")
                    write_text(opensrc_root / "MIGRATION.md", "# Migration")
                    write_text(opensrc_root / "examples" / "basic.ts", "export {}")
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": f"{opensrc_root}\n",
                        "stderr": "",
                        "stdout_excerpt": [str(opensrc_root)],
                        "stderr_excerpt": [],
                        "summary": [str(opensrc_root)],
                    }
                if command.startswith("npm view zod@4.3.6 "):
                    payload = {
                        "peerDependencies": {},
                        "engines": {"node": ">=18"},
                        "repository": {"type": "git", "url": "git+https://github.com/colinhacks/zod.git"},
                        "homepage": "https://zod.dev",
                        "bugs": {"url": "https://github.com/colinhacks/zod/issues"},
                    }
                    text = json.dumps(payload)
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stderr": "",
                        "stdout_excerpt": [text],
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("npm view zod "):
                    payload = {
                        "repository": {"type": "git", "url": "git+https://github.com/colinhacks/zod.git"},
                        "homepage": "https://zod.dev",
                        "bugs": {"url": "https://github.com/colinhacks/zod/issues"},
                        "dist-tags": {"latest": "4.3.6", "canary": "4.4.0-canary.20260125T215152"},
                        "versions": ["3.25.76", "4.3.4", "4.3.5", "4.3.6"],
                        "peerDependencies": {},
                        "engines": {"node": ">=18"},
                        "name": "zod",
                    }
                    text = json.dumps(payload)
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stderr": "",
                        "stdout_excerpt": [text],
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command == "gh api repos/colinhacks/zod":
                    payload = {
                        "full_name": "colinhacks/zod",
                        "default_branch": "main",
                        "html_url": "https://github.com/colinhacks/zod",
                    }
                    text = json.dumps(payload)
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stderr": "",
                        "stdout_excerpt": [text],
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command == "gh api 'repos/colinhacks/zod/releases?per_page=5'":
                    payload = [{"tag_name": "v4.3.6"}, {"tag_name": "v4.3.5"}]
                    text = json.dumps(payload)
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": text,
                        "stderr": "",
                        "stdout_excerpt": [text],
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("ctx7 library zod "):
                    stdout = "\n".join(
                        [
                            "1. Title: Zod",
                            "   Context7-compatible library ID: /colinhacks/zod",
                            "   Description: Zod is a TypeScript-first schema validation library.",
                        ]
                    )
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": stdout,
                        "stderr": "",
                        "stdout_excerpt": stdout.splitlines(),
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("ctx7 docs /colinhacks/zod "):
                    stdout = "\n".join(
                        [
                            "### Zod API docs",
                            "Source: https://zod.dev",
                            "Use `.parse`, `.safeParse`, and schema composition helpers.",
                        ]
                    )
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": stdout,
                        "stderr": "",
                        "stdout_excerpt": stdout.splitlines(),
                        "stderr_excerpt": [],
                        "summary": ["ok"],
                    }
                if command.startswith("rg -n "):
                    stdout = "src/schemas.ts:1:import { z } from \"zod\";\npackage.json:11:\"zod\": \"^3.25.76\"\n"
                    return {
                        "command": command,
                        "cwd": str(root),
                        "exit_code": 0,
                        "status": "ok",
                        "stdout": stdout,
                        "stderr": "",
                        "stdout_excerpt": stdout.splitlines(),
                        "stderr_excerpt": [],
                        "summary": ["usage found"],
                    }
                return {
                    "command": command,
                    "cwd": str(root),
                    "exit_code": 1,
                    "status": "failed",
                    "stdout": "",
                    "stderr": "",
                    "stdout_excerpt": [],
                    "stderr_excerpt": [],
                    "summary": [],
                }

            web_findings = {
                "entries": [
                    {
                        "category": "official_docs",
                        "url": "https://zod.dev/",
                        "confirmed": True,
                        "confirmed_at": "2026-04-20T00:00:00Z",
                        "facts": ["Zod docs reviewed."],
                    },
                    {
                        "category": "api_reference",
                        "url": "https://zod.dev/api",
                        "confirmed": True,
                        "confirmed_at": "2026-04-20T00:00:00Z",
                        "facts": ["Zod API reviewed."],
                    },
                ]
            }

            with patch.object(research_upgrade_pack, "run_shell", side_effect=fake_run_shell):
                snapshot, bundle = research_upgrade_pack.generate_snapshot(manifest, root, web_findings)

            self.assertEqual(snapshot["research_status"], "complete")
            self.assertEqual(snapshot["summary"]["identity_status"], "high-confidence")
            self.assertEqual(snapshot["identity"]["repository_slug"], "colinhacks/zod")
            self.assertEqual(snapshot["target_version"], "4.3.6")
            self.assertEqual(snapshot["target_resolution"]["selected_status"], "compatible")
            self.assertEqual(snapshot["category_status"]["official_docs"], "ok")
            self.assertEqual(snapshot["category_status"]["api_reference"], "ok")
            self.assertEqual(snapshot["category_status"]["migration_guides"], "ok")
            self.assertEqual(snapshot["category_status"]["release_history"], "ok")
            self.assertEqual(snapshot["category_status"]["examples_cookbooks"], "ok")
            self.assertEqual(snapshot["bundle_filename"], "research-bundle.json")
            self.assertIn("ctx7", bundle)
            self.assertIn("github", bundle)
            self.assertEqual(bundle["identity"]["status"], "high-confidence")
            self.assertEqual(bundle["source_map_seed"]["entry"]["packageName"], "zod")
            self.assertTrue(bundle["web_research_queue"])
            self.assertIn("category_provenance", bundle)

    def test_research_status_requires_web_confirmation(self) -> None:
        """Research cannot be complete until required official pages are web-confirmed."""
        status, caveats = research_upgrade_pack.research_status(
            ["official_docs", "api_reference"],
            {"official_docs": "ok", "api_reference": "ok"},
            {"status": "high-confidence", "confidence": 0.9},
            {"selected_status": "compatible"},
            [
                {
                    "category": "official_docs",
                    "seed_url": "https://example.com/docs",
                    "required_for_complete": True,
                },
                {
                    "category": "api_reference",
                    "seed_url": "https://example.com/api",
                    "required_for_complete": True,
                },
            ],
            {"entries": []},
        )
        self.assertEqual(status, "partial")
        self.assertTrue(any("web.run confirmation" in caveat for caveat in caveats))

    def test_semver_solver_handles_union_and_minor_ranges(self) -> None:
        """The bundled solver should handle common peer/engine range shapes."""
        self.assertTrue(solve_constraints.satisfies("19.2.0", "^18.2.0 || ^19.0.0"))
        self.assertTrue(solve_constraints.satisfies("55.0.15", "~55.0.0"))
        self.assertFalse(solve_constraints.satisfies("17.0.2", "^18.2.0 || ^19.0.0"))

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
                "bundle_filename": "research-bundle.json",
                "web_findings_filename": "web-research-findings.json",
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
                    "identity_status": "high-confidence",
                    "identity_confidence": 0.93,
                    "required_web_queue": 2,
                    "confirmed_web_queue": 2,
                },
                "identity": {
                    "status": "high-confidence",
                    "confidence": 0.93,
                    "repository_slug": "expo/expo",
                },
                "target_resolution": {
                    "selected_status": "compatible",
                    "recommended_related_packages": ["expo-router"],
                },
                "recommended_related_packages": ["expo-router"],
                "official_docs": [],
                "api_reference": [],
                "migration_guides": [],
                "release_history": [],
                "examples_cookbooks": [],
                "source_evidence": [],
                "repo_usage_mapping": [],
                "web_research_queue": [],
                "web_research_findings": {"entries": []},
                "caveats": [],
            }
            research_path = Path(tmp) / "research-snapshot.json"
            research_path.write_text(json.dumps(research_snapshot), encoding="utf-8")
            research_bundle = {
                "schema_version": 1,
                "generated_at": "2026-04-19T00:00:00Z",
                "family_slug": enriched["family_slug"],
                "anchor_package": enriched["anchor_package"],
                "repo_root": str(root),
                "snapshot_filename": "research-snapshot.json",
                "bundle_filename": "research-bundle.json",
                "web_findings_filename": "web-research-findings.json",
                "identity": research_snapshot["identity"],
                "source_map_seed": {"match_status": "fresh"},
                "target_resolution": research_snapshot["target_resolution"],
                "ctx7": {},
                "github": {},
                "registry": {},
                "repo_runtime": {},
                "collectors": {},
                "discovered_sources": {},
                "source_evidence": [],
                "repo_usage_mapping": [],
                "category_entries": {},
                "category_status": research_snapshot["category_status"],
                "category_provenance": {},
                "web_research_queue": [],
                "web_research_findings": {"entries": []},
                "caveats": [],
            }
            research_bundle_path = Path(tmp) / "research-bundle.json"
            research_bundle_path.write_text(json.dumps(research_bundle), encoding="utf-8")
            web_findings = {"entries": []}
            web_findings_path = Path(tmp) / "web-research-findings.json"
            web_findings_path.write_text(json.dumps(web_findings), encoding="utf-8")
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
                    "--research-bundle",
                    str(research_bundle_path),
                    "--web-findings",
                    str(web_findings_path),
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
            self.assertTrue((out / "research-bundle.json").exists())
            self.assertTrue((out / "web-research-findings.json").exists())
            self.assertTrue((out / "qualification-snapshot.json").exists())
            self.assertIn("### Research Coverage", playbook_text)
            self.assertIn("raw bundle file: `research-bundle.json`", playbook_text)
            self.assertIn("web findings file: `web-research-findings.json`", playbook_text)
            self.assertIn("package identity: `high-confidence`", playbook_text)
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
                "bundle_filename": "research-bundle.json",
                "web_findings_filename": "web-research-findings.json",
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
                    "identity_status": "high-confidence",
                    "identity_confidence": 0.91,
                    "required_web_queue": 2,
                    "confirmed_web_queue": 2,
                },
                "identity": {
                    "status": "high-confidence",
                    "confidence": 0.91,
                    "repository_slug": "get-convex/convex-backend",
                },
                "target_resolution": {
                    "selected_status": "compatible",
                    "recommended_related_packages": ["convex-helpers"],
                },
                "recommended_related_packages": ["convex-helpers"],
                "official_docs": [],
                "api_reference": [],
                "migration_guides": [],
                "release_history": [],
                "examples_cookbooks": [],
                "source_evidence": [],
                "repo_usage_mapping": [],
                "web_research_queue": [],
                "web_research_findings": {"entries": []},
                "caveats": [],
            }
            research_path = Path(tmp) / "research-snapshot.json"
            research_path.write_text(json.dumps(research_snapshot), encoding="utf-8")
            research_bundle = {
                "schema_version": 1,
                "generated_at": "2026-04-19T00:00:00Z",
                "family_slug": enriched["family_slug"],
                "anchor_package": enriched["anchor_package"],
                "repo_root": str(root),
                "snapshot_filename": "research-snapshot.json",
                "bundle_filename": "research-bundle.json",
                "web_findings_filename": "web-research-findings.json",
                "identity": research_snapshot["identity"],
                "source_map_seed": {"match_status": "fresh"},
                "target_resolution": research_snapshot["target_resolution"],
                "ctx7": {},
                "github": {},
                "registry": {},
                "repo_runtime": {},
                "collectors": {},
                "discovered_sources": {},
                "source_evidence": [],
                "repo_usage_mapping": [],
                "category_entries": {},
                "category_status": research_snapshot["category_status"],
                "category_provenance": {},
                "web_research_queue": [],
                "web_research_findings": {"entries": []},
                "caveats": [],
            }
            research_bundle_path = Path(tmp) / "research-bundle.json"
            research_bundle_path.write_text(json.dumps(research_bundle), encoding="utf-8")
            web_findings = {"entries": []}
            web_findings_path = Path(tmp) / "web-research-findings.json"
            web_findings_path.write_text(json.dumps(web_findings), encoding="utf-8")
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
                    "--research-bundle",
                    str(research_bundle_path),
                    "--web-findings",
                    str(web_findings_path),
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
