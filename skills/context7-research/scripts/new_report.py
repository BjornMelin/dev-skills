#!/usr/bin/env python3
import argparse
from datetime import datetime, timezone
from pathlib import Path


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def read_template() -> str:
    template_path = Path(__file__).resolve().parent.parent / "assets" / "report-template.md"
    return template_path.read_text(encoding="utf-8")


def render(template: str, *, library_name: str, library_id: str, question: str, gaps: str) -> str:
    return (
        template.replace("{{date}}", now_iso())
        .replace("{{libraryName}}", library_name)
        .replace("{{libraryId}}", library_id)
        .replace("{{question}}", question)
        .replace("{{gaps}}", gaps)
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate a Context7 research report skeleton.")
    parser.add_argument("--out", required=True, help="Output markdown file path")
    parser.add_argument("--library-name", required=True, help="Library name (human-readable)")
    parser.add_argument("--library-id", default="UNRESOLVED", help="Context7 library ID (/org/project[/version])")
    parser.add_argument("--question", required=True, help="The research question/objective")
    parser.add_argument("--gaps", default="UNVERIFIED: not researched yet", help="Initial gaps/assumptions")
    args = parser.parse_args()

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    content = render(
        read_template(),
        library_name=args.library_name,
        library_id=args.library_id,
        question=args.question,
        gaps=args.gaps,
    )
    out_path.write_text(content, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

