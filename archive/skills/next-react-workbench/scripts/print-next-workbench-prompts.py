#!/usr/bin/env python3
"""Print recommended prompts for Next React Workbench."""

from __future__ import annotations

from pathlib import Path


def main() -> int:
    """Print the three recommended $next-react-workbench prompts.

    Prints the route-fix, surface-refactor, and polish prompts to stdout
    for operator reference. Returns 0 on success.

    Returns:
        int: Exit code (0 on success).
    """
    root = Path(".").resolve()
    print(f"Root: {root}")
    print(
        "1. Use $next-react-workbench to fix this Next route, "
        "improve the loading and error states, "
        "and verify the result in the browser."
    )
    print(
        "2. Use $next-react-workbench to refactor this React surface "
        "for cleaner composition and better runtime behavior."
    )
    print(
        "3. Use $next-react-workbench to polish this page's hierarchy, "
        "spacing, theme, and interaction quality, "
        "then run browser verification."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
