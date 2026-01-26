#!/usr/bin/env python3
"""
Convert Jupyter notebook to Python script.

Usage:
    python convert_to_script.py notebook.ipynb output.py [options]

Options:
    --include-markdown    Include markdown cells as comments
    --group-by-sections   Create function for each markdown section
    --add-main           Add if __name__ == "__main__" block
    --strip-outputs      Remove output references from comments
"""

import argparse
import re
import sys
from pathlib import Path

try:
    import nbformat
except ImportError:
    print("Error: nbformat not installed. Run: pip install nbformat")
    sys.exit(1)


def clean_source(source: str) -> str:
    """Clean cell source code."""
    # Remove IPython magic commands that won't work in scripts
    lines = source.split("\n")
    cleaned = []
    for line in lines:
        stripped = line.strip()
        # Skip IPython magics
        if stripped.startswith("%") or stripped.startswith("!"):
            cleaned.append(f"# {line}  # IPython magic - may need adjustment")
        else:
            cleaned.append(line)
    return "\n".join(cleaned)


def markdown_to_comment(source: str, prefix: str = "# ") -> str:
    """Convert markdown to Python comments."""
    lines = source.split("\n")
    commented = []
    for line in lines:
        if line.strip():
            commented.append(f"{prefix}{line}")
        else:
            commented.append("#")
    return "\n".join(commented)


def extract_section_title(markdown: str) -> str | None:
    """Extract section title from markdown cell."""
    for line in markdown.split("\n"):
        if line.startswith("#"):
            # Extract title, convert to valid function name
            title = line.lstrip("#").strip()
            # Convert to snake_case
            title = re.sub(r"[^\w\s]", "", title)
            title = re.sub(r"\s+", "_", title.lower())
            return title[:50]  # Limit length
    return None


def convert_notebook(
    notebook_path: str,
    include_markdown: bool = False,
    group_by_sections: bool = False,
    add_main: bool = False,
) -> str:
    """Convert notebook to Python script."""
    path = Path(notebook_path)
    if not path.exists():
        raise FileNotFoundError(f"Notebook not found: {notebook_path}")

    nb = nbformat.read(path, as_version=4)

    lines = []
    lines.append('"""')
    lines.append(f"Converted from: {path.name}")
    lines.append("")
    lines.append("Auto-generated Python script from Jupyter notebook.")
    lines.append('"""')
    lines.append("")

    current_section = None
    section_code = []
    sections = []

    def flush_section():
        """Save current section code."""
        nonlocal section_code
        if section_code:
            if group_by_sections and current_section:
                sections.append((current_section, "\n".join(section_code)))
            else:
                lines.extend(section_code)
                lines.append("")
            section_code = []

    for cell in nb.cells:
        if cell.cell_type == "markdown":
            if include_markdown:
                comment = markdown_to_comment(cell.source)
                section_code.append(comment)
                section_code.append("")

            # Check for section header
            if group_by_sections:
                title = extract_section_title(cell.source)
                if title:
                    flush_section()
                    current_section = title

        elif cell.cell_type == "code":
            source = cell.source.strip()
            if not source:
                continue

            cleaned = clean_source(source)
            section_code.append(cleaned)
            section_code.append("")

    # Flush remaining code
    flush_section()

    # Build final script
    if group_by_sections and sections:
        # Create functions for each section
        lines.append("# " + "=" * 60)
        lines.append("# Section Functions")
        lines.append("# " + "=" * 60)
        lines.append("")

        main_calls = []
        for section_name, code in sections:
            # Indent code
            indented = "\n".join(
                f"    {line}" if line.strip() else "" for line in code.split("\n")
            )

            lines.append(f"def {section_name}():")
            lines.append(f'    """Section: {section_name.replace("_", " ").title()}"""')
            if indented.strip():
                lines.append(indented)
            else:
                lines.append("    pass")
            lines.append("")

            main_calls.append(f"    {section_name}()")

        if add_main:
            lines.append("")
            lines.append("# " + "=" * 60)
            lines.append("# Main Execution")
            lines.append("# " + "=" * 60)
            lines.append("")
            lines.append('if __name__ == "__main__":')
            lines.append('    print("Running notebook as script...")')
            lines.extend(main_calls)
            lines.append('    print("Done!")')

    elif add_main:
        # Wrap existing code in main block
        all_code = "\n".join(lines[6:])  # Skip header
        lines = lines[:6]  # Keep header

        lines.append("")
        lines.append('if __name__ == "__main__":')

        # Indent all code
        for line in all_code.split("\n"):
            if line.strip():
                lines.append(f"    {line}")
            else:
                lines.append("")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Convert Jupyter notebook to Python script"
    )
    parser.add_argument("notebook", help="Input notebook path")
    parser.add_argument("output", help="Output Python script path")
    parser.add_argument(
        "--include-markdown",
        action="store_true",
        help="Include markdown cells as comments",
    )
    parser.add_argument(
        "--group-by-sections",
        action="store_true",
        help="Create function for each markdown section",
    )
    parser.add_argument(
        "--add-main",
        action="store_true",
        help='Add if __name__ == "__main__" block',
    )

    args = parser.parse_args()

    try:
        script = convert_notebook(
            args.notebook,
            include_markdown=args.include_markdown,
            group_by_sections=args.group_by_sections,
            add_main=args.add_main,
        )

        output_path = Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(script)

        print(f"Script saved to: {output_path}")

        # Print summary
        with open(args.notebook) as f:
            nb = nbformat.read(f, as_version=4)
        code_cells = sum(1 for c in nb.cells if c.cell_type == "code")
        print(f"Converted {code_cells} code cells")

    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Error converting notebook: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
