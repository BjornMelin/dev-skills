#!/usr/bin/env python3
"""
Analyze Jupyter notebook structure and detect anti-patterns.

Usage:
    python analyze_notebook.py <notebook.ipynb> [--output json|text]

Outputs:
    - Cell counts by type
    - Import statements
    - Function/class definitions
    - Detected issues and anti-patterns
    - Section structure
"""

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

try:
    import nbformat
except ImportError:
    print("Error: nbformat not installed. Run: pip install nbformat")
    sys.exit(1)


def analyze_notebook(notebook_path: str) -> dict[str, Any]:
    """Parse notebook and extract structure information."""
    path = Path(notebook_path)
    if not path.exists():
        raise FileNotFoundError(f"Notebook not found: {notebook_path}")

    nb = nbformat.read(path, as_version=4)

    analysis = {
        "path": str(path),
        "metadata": {
            "kernel": nb.metadata.get("kernelspec", {}).get("display_name", "Unknown"),
            "language": nb.metadata.get("kernelspec", {}).get("language", "Unknown"),
            "nbformat": f"{nb.nbformat}.{nb.nbformat_minor}",
        },
        "cell_counts": {"code": 0, "markdown": 0, "raw": 0, "total": 0},
        "execution_info": {
            "max_execution_count": 0,
            "cells_with_output": 0,
            "execution_order_issues": [],
        },
        "imports": [],
        "functions": [],
        "classes": [],
        "sections": [],
        "issues": [],
    }

    last_execution_count = 0
    import_pattern = re.compile(r"^(?:from\s+(\S+)\s+)?import\s+(\S+)", re.MULTILINE)
    function_pattern = re.compile(r"^def\s+(\w+)\s*\(", re.MULTILINE)
    class_pattern = re.compile(r"^class\s+(\w+)\s*[\(:]", re.MULTILINE)
    seed_pattern = re.compile(
        r"(?:np\.random\.seed|random\.seed|torch\.manual_seed|tf\.random\.set_seed)\s*\(",
        re.MULTILINE,
    )
    absolute_path_pattern = re.compile(r"['\"](?:/[^'\"]+|[A-Z]:\\[^'\"]+)['\"]")

    has_seed = False
    has_train_test_split = False
    imports_after_code = False
    first_code_cell = True

    for idx, cell in enumerate(nb.cells):
        cell_type = cell.cell_type
        analysis["cell_counts"][cell_type] = (
            analysis["cell_counts"].get(cell_type, 0) + 1
        )
        analysis["cell_counts"]["total"] += 1

        if cell_type == "markdown":
            # Extract section headers
            source = cell.source
            for line in source.split("\n"):
                if line.startswith("#"):
                    header_level = len(line) - len(line.lstrip("#"))
                    header_text = line.lstrip("#").strip()
                    if header_text:
                        analysis["sections"].append(
                            {
                                "level": header_level,
                                "title": header_text,
                                "cell_index": idx,
                            }
                        )

        elif cell_type == "code":
            source = cell.source
            execution_count = cell.get("execution_count")

            # Check execution order
            if execution_count is not None:
                analysis["execution_info"]["max_execution_count"] = max(
                    analysis["execution_info"]["max_execution_count"], execution_count
                )
                if execution_count < last_execution_count:
                    analysis["execution_info"]["execution_order_issues"].append(
                        {
                            "cell_index": idx,
                            "execution_count": execution_count,
                            "expected_after": last_execution_count,
                        }
                    )
                last_execution_count = execution_count

            # Check for outputs
            if cell.get("outputs"):
                analysis["execution_info"]["cells_with_output"] += 1

            # Extract imports
            for match in import_pattern.finditer(source):
                module = match.group(1) or match.group(2)
                if module:
                    module_name = module.split(".")[0]
                    if module_name not in analysis["imports"]:
                        analysis["imports"].append(module_name)

                    # Check if imports come after code
                    if not first_code_cell and module_name not in [
                        "warnings",
                        "logging",
                    ]:
                        imports_after_code = True

            # Extract functions
            for match in function_pattern.finditer(source):
                analysis["functions"].append(
                    {"name": match.group(1), "cell_index": idx}
                )

            # Extract classes
            for match in class_pattern.finditer(source):
                analysis["classes"].append({"name": match.group(1), "cell_index": idx})

            # Check for seeds
            if seed_pattern.search(source):
                has_seed = True

            # Check for train_test_split
            if "train_test_split" in source:
                has_train_test_split = True

            # Check for absolute paths
            if absolute_path_pattern.search(source):
                analysis["issues"].append(
                    {
                        "severity": "HIGH",
                        "type": "absolute_path",
                        "cell_index": idx,
                        "message": "Absolute file path detected. Use relative paths for portability.",
                    }
                )

            first_code_cell = False

    # Post-analysis checks

    # Check for missing seeds
    if not has_seed and analysis["cell_counts"]["code"] > 3:
        analysis["issues"].append(
            {
                "severity": "HIGH",
                "type": "missing_seed",
                "message": "No random seed setting found. Add np.random.seed() for reproducibility.",
            }
        )

    # Check for missing train/test split
    ml_imports = {"sklearn", "torch", "tensorflow", "keras", "xgboost", "lightgbm"}
    if ml_imports.intersection(set(analysis["imports"])) and not has_train_test_split:
        analysis["issues"].append(
            {
                "severity": "MEDIUM",
                "type": "no_train_test_split",
                "message": "ML imports found but no train_test_split detected. Ensure proper data splitting.",
            }
        )

    # Check for imports after code
    if imports_after_code:
        analysis["issues"].append(
            {
                "severity": "LOW",
                "type": "scattered_imports",
                "message": "Imports found after code cells. Move all imports to the beginning.",
            }
        )

    # Check execution order
    if analysis["execution_info"]["execution_order_issues"]:
        analysis["issues"].append(
            {
                "severity": "HIGH",
                "type": "execution_order",
                "message": f"Cells executed out of order. Found {len(analysis['execution_info']['execution_order_issues'])} issues.",
            }
        )

    # Check for no functions (lack of modularization)
    if analysis["cell_counts"]["code"] > 10 and len(analysis["functions"]) == 0:
        analysis["issues"].append(
            {
                "severity": "MEDIUM",
                "type": "no_modularization",
                "message": "No functions defined in a notebook with 10+ code cells. Consider refactoring.",
            }
        )

    # Check filename
    if "Untitled" in path.name:
        analysis["issues"].append(
            {
                "severity": "LOW",
                "type": "default_name",
                "message": "Notebook has default name. Use descriptive naming.",
            }
        )

    if "Copy" in path.name:
        analysis["issues"].append(
            {
                "severity": "LOW",
                "type": "copy_name",
                "message": "Notebook appears to be a copy. Rename appropriately.",
            }
        )

    return analysis


def format_text_output(analysis: dict[str, Any]) -> str:
    """Format analysis results as human-readable text."""
    lines = []
    lines.append(f"Notebook Analysis: {analysis['path']}")
    lines.append("=" * 60)

    # Metadata
    lines.append("\nMetadata:")
    lines.append(f"  Kernel: {analysis['metadata']['kernel']}")
    lines.append(f"  Language: {analysis['metadata']['language']}")
    lines.append(f"  Format: {analysis['metadata']['nbformat']}")

    # Cell counts
    lines.append("\nCell Counts:")
    for cell_type, count in analysis["cell_counts"].items():
        lines.append(f"  {cell_type}: {count}")

    # Execution info
    lines.append("\nExecution Info:")
    lines.append(
        f"  Max execution count: {analysis['execution_info']['max_execution_count']}"
    )
    lines.append(
        f"  Cells with output: {analysis['execution_info']['cells_with_output']}"
    )

    # Imports
    if analysis["imports"]:
        lines.append(f"\nImports ({len(analysis['imports'])}):")
        lines.append(f"  {', '.join(sorted(analysis['imports']))}")

    # Functions
    if analysis["functions"]:
        lines.append(f"\nFunctions ({len(analysis['functions'])}):")
        for func in analysis["functions"]:
            lines.append(f"  - {func['name']} (cell {func['cell_index']})")

    # Classes
    if analysis["classes"]:
        lines.append(f"\nClasses ({len(analysis['classes'])}):")
        for cls in analysis["classes"]:
            lines.append(f"  - {cls['name']} (cell {cls['cell_index']})")

    # Sections
    if analysis["sections"]:
        lines.append(f"\nSections ({len(analysis['sections'])}):")
        for section in analysis["sections"]:
            indent = "  " * section["level"]
            lines.append(f"{indent}{section['title']}")

    # Issues
    if analysis["issues"]:
        lines.append(f"\nIssues Found ({len(analysis['issues'])}):")
        # Sort by severity
        severity_order = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}
        sorted_issues = sorted(
            analysis["issues"], key=lambda x: severity_order.get(x["severity"], 4)
        )
        for issue in sorted_issues:
            lines.append(f"  [{issue['severity']}] {issue['type']}")
            lines.append(f"    {issue['message']}")
    else:
        lines.append("\nNo issues found!")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Analyze Jupyter notebook structure")
    parser.add_argument("notebook", help="Path to notebook file")
    parser.add_argument(
        "--output",
        "-o",
        choices=["json", "text"],
        default="text",
        help="Output format (default: text)",
    )

    args = parser.parse_args()

    try:
        analysis = analyze_notebook(args.notebook)

        if args.output == "json":
            print(json.dumps(analysis, indent=2))
        else:
            print(format_text_output(analysis))

    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error analyzing notebook: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
