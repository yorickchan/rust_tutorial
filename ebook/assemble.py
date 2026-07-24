#!/usr/bin/env python3
"""Assemble the Rust tutorial into a single Markdown file for pandoc EPUB conversion.

Fixes applied:
- TUTORIAL.md mermaid block -> ASCII preformatted text (EPUB readers don't run JS)
- <details>/<summary> -> plain h3 + paragraph (poor e-reader support)
- Emoji removed from headings (render inconsistently in EPUB)
- Relative file links removed (pandoc TOC handles navigation)
"""
import re
import sys
from pathlib import Path

ROOT = Path("/Users/yorick/Projects/rust_tutorial")

CHAPTERS = [
    "chapters/ch00-basics/README.md",
    "chapters/ch01-cli/README.md",
    "chapters/ch02-tui/README.md",
    "chapters/ch03-async/README.md",
    "chapters/ch04-networking/README.md",
    "chapters/ch05-sqlite/README.md",
    "chapters/ch06-web/README.md",
    "chapters/ch07-macros/README.md",
]

MERMAID_REPLACEMENT = """```text
ch00 基礎 ──► ch01 CLI ──► ch02 TUI ──┐
                                       │
                       ch03 async ─────┼──► ch04 networking ──┐
                                       │                        │
                       ch05 sqlite ────┘                        │
                                                                ▼
                                                        ch06 web（壓軸）
                                                                │
                                                                ▼
                                                          ch07 巨集

  黃色 = 起點（必讀）  綠色 = 終點（壓軸整合）
  箭頭 = 「先學 A 才好學 B」
```"""


def strip_emoji_ish(text: str) -> str:
    """Remove common emoji from markdown headings for EPUB compatibility.
    Keeps CJK, punctuation, ascii. Targets the specific emoji used in TUTORIAL.md."""
    # These are the emoji present in TUTORIAL.md headings
    emoji = [
        "📖", "🗺️", "📚", "🧭", "🚀", "🧩", "❓", "📝",
        "🔑", "🏃", "✏️", "⚠️", "🟡", "🟢",
    ]
    for e in emoji:
        text = text.replace(e, "")
    # Clean up leftover spaces before heading text
    text = re.sub(r"^(#+)\s+", r"\1 ", text, flags=re.MULTILINE)
    return text


def fix_tutorial(content: str) -> str:
    """Adapt TUTORIAL.md for EPUB: mermaid -> ASCII, details -> headings, emoji strip."""
    # 1. Replace mermaid block with ASCII diagram
    # Match the whole ```mermaid ... ``` block including surrounding context lines
    content = re.sub(
        r"```mermaid\n.*?```",
        MERMAID_REPLACEMENT,
        content,
        flags=re.DOTALL,
    )

    # 2. Convert <details><summary><b>Question</b></summary> ... </details>
    #    to ### 問：Question  / answer paragraphs
    def details_to_heading(m: re.Match) -> str:
        full = m.group(0)
        # Extract the summary text (strip <b> tags)
        summary_match = re.search(
            r"<summary>\s*(?:<b>)?(.*?)(?:</b>)?\s*</summary>", full, re.DOTALL
        )
        summary = summary_match.group(1).strip() if summary_match else "問題"
        # The rest after </summary>
        rest = full[summary_match.end():].strip() if summary_match else full
        # Remove a trailing </details> if present
        rest = rest.replace("</details>", "").strip()
        return f"### 問：{summary}\n\n{rest}"

    content = re.sub(
        r"<details>\s*<summary>.*?</summary>(.*?)</details>",
        details_to_heading,
        content,
        flags=re.DOTALL,
    )

    # 3. Remove emoji from headings
    content = strip_emoji_ish(content)

    # 4. Remove relative file links: [text](chapters/...) -> just **text**
    #    Keep external http links and anchor links
    def unlink_relative(m: re.Match) -> str:
        text = m.group(1)
        url = m.group(2)
        if url.startswith(("http", "#", "mailto:")):
            return m.group(0)
        return f"**{text}**"

    content = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", unlink_relative, content)

    return content


def assemble() -> str:
    parts = []

    # Front matter from TUTORIAL.md (the intro/navigation sections)
    tutorial = (ROOT / "TUTORIAL.md").read_text(encoding="utf-8")
    tutorial = fix_tutorial(tutorial)
    # TUTORIAL.md title becomes the book's main title; rest is front matter
    parts.append(tutorial)

    # Separator between front matter and chapter content
    parts.append("\n\n---\n\n")

    # Each chapter README - these have # 第 N 章 headings = chapter breaks
    for chap_path in CHAPTERS:
        chap = (ROOT / chap_path).read_text(encoding="utf-8")
        # Strip emoji from chapter content too (minimal - chapters are mostly clean)
        chap = strip_emoji_ish(chap)
        # Remove relative links in chapters (if any)
        chap = re.sub(
            r"\[([^\]]+)\]\(([^)]+)\)",
            lambda m: m.group(0)
            if m.group(2).startswith(("http", "#", "mailto:"))
            else f"**{m.group(1)}**",
            chap,
        )
        parts.append(chap)
        parts.append("\n\n")

    return "\n".join(parts)


def main() -> None:
    output = assemble()
    out_path = ROOT / "ebook" / "book.md"
    out_path.write_text(output, encoding="utf-8")
    print(f"Written {out_path} ({len(output)} bytes, {output.count(chr(10))} lines)")


if __name__ == "__main__":
    main()
