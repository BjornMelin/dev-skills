#!/usr/bin/env python3
"""DMC Component Search - Search Dash Mantine Components documentation

Usage:
    component_search.py <query> [options]

Options:
    --category CATEGORY    Filter by component category
    --props               Search prop definitions only
    --limit N             Max results (default: 10)
"""

import argparse
import re
import sys
from pathlib import Path


def find_docs_directory() -> Path | None:
    """Find DMC documentation directory.

    Searches in common locations for DMC docs.

    Returns:
        Path to docs directory if found, None otherwise
    """
    possible_paths = [
        Path.home() / ".agent" / "dash-mantine-components",
        Path.home() / ".claude" / "skills" / "dmc-py" / "docs",
        Path.cwd() / "docs" / "dash-mantine-components",
        Path.cwd() / ".agent" / "dash-mantine-components",
    ]

    for path in possible_paths:
        if path.exists() and path.is_dir():
            return path

    return None


def search_files(
    query: str,
    docs_dir: Path,
    category: str | None = None,
    props_only: bool = False,
    limit: int = 10,
) -> list[tuple[Path, int, str]]:
    """Search documentation files for query.

    Args:
        query: Search query string
        docs_dir: Documentation directory path
        category: Optional category filter
        props_only: Search only prop definitions
        limit: Maximum number of results

    Returns:
        List of tuples (file_path, line_number, line_content)
    """
    results = []
    pattern = re.compile(re.escape(query), re.IGNORECASE)

    # Determine file pattern
    if category:
        file_pattern = f"**/{category}/*.md"
    else:
        file_pattern = "**/*.md"

    # Search files
    for file_path in docs_dir.glob(file_pattern):
        if not file_path.is_file():
            continue

        try:
            with open(file_path, encoding="utf-8") as f:
                in_props_section = False

                for line_num, line in enumerate(f, 1):
                    # Track if we're in a props section
                    if props_only:
                        if re.match(r"#+\s*(Props|Properties)", line, re.IGNORECASE):
                            in_props_section = True
                        elif line.startswith("#"):
                            in_props_section = False

                        # Skip if not in props section
                        if not in_props_section:
                            continue

                    # Search for pattern
                    if pattern.search(line):
                        results.append((file_path, line_num, line.strip()))

                        # Stop if we've reached the limit
                        if len(results) >= limit:
                            return results

        except (OSError, UnicodeDecodeError) as e:
            print(f"Warning: Could not read {file_path}: {e}", file=sys.stderr)
            continue

    return results


def format_result(
    file_path: Path,
    line_num: int,
    line_content: str,
    docs_dir: Path,
    query: str,
) -> str:
    """Format a search result for display.

    Args:
        file_path: Path to the file
        line_num: Line number
        line_content: Line content
        docs_dir: Documentation directory (for relative path)
        query: Original search query (for highlighting)

    Returns:
        Formatted result string
    """
    # Get relative path
    try:
        rel_path = file_path.relative_to(docs_dir)
    except ValueError:
        rel_path = file_path

    # Highlight query in content (simple approach)
    highlighted = re.sub(
        f"({re.escape(query)})",
        r"[\1]",
        line_content,
        flags=re.IGNORECASE,
    )

    return f"{rel_path}:{line_num}\n  {highlighted}"


def main() -> None:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="DMC Component Search - Search Dash Mantine Components documentation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument("query", help="Search query string")
    parser.add_argument(
        "--category",
        help="Filter by component category",
    )
    parser.add_argument(
        "--props",
        action="store_true",
        help="Search prop definitions only",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=10,
        help="Maximum results (default: 10)",
    )
    parser.add_argument(
        "--docs-dir",
        type=Path,
        help="Custom documentation directory path",
    )

    args = parser.parse_args()

    # Find docs directory
    docs_dir = args.docs_dir or find_docs_directory()

    if not docs_dir:
        print(
            "Error: Could not find DMC documentation directory.\n"
            "Searched in:\n"
            "  - ~/.agent/dash-mantine-components\n"
            "  - ~/.claude/skills/dmc-py/docs\n"
            "  - ./docs/dash-mantine-components\n"
            "  - ./.agent/dash-mantine-components\n\n"
            "Use --docs-dir to specify a custom path.",
            file=sys.stderr,
        )
        sys.exit(1)

    # Perform search
    results = search_files(
        args.query,
        docs_dir,
        args.category,
        args.props,
        args.limit,
    )

    # Display results
    if not results:
        print(f"No results found for '{args.query}'")
        return

    print(f"Found {len(results)} result(s) for '{args.query}':\n")

    for file_path, line_num, line_content in results:
        print(format_result(file_path, line_num, line_content, docs_dir, args.query))
        print()  # Blank line between results


if __name__ == "__main__":
    main()
