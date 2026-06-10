#!/usr/bin/env python3
"""Validate pi/Claude skill frontmatter for this repo.

Checks every SKILL.md under .claude/skills and .pi/skills for:
- YAML frontmatter delimiters
- parseable YAML
- required name + description
- description length <= 1024 chars

Also emits style warnings for punctuation-heavy plain-scalar descriptions; use
--strict-style to make those warnings fail CI/local validation.
"""
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:  # pragma: no cover - environment guard
    print("ERROR: PyYAML is required (python3 -m pip install PyYAML)", file=sys.stderr)
    sys.exit(2)

ROOT = Path(__file__).resolve().parents[2]
SKILL_GLOBS = (".claude/skills/*/SKILL.md", ".pi/skills/*/SKILL.md")
MAX_DESCRIPTION_CHARS = 1024
YAML_PLAIN_SCALAR_RISK_RE = re.compile(r":\s")


@dataclass
class Finding:
    severity: str
    path: Path
    message: str


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def frontmatter(text: str) -> tuple[str | None, str | None]:
    if not text.startswith("---\n"):
        return None, "missing YAML frontmatter opening delimiter"
    end = text.find("\n---", 4)
    if end < 0:
        return None, "missing YAML frontmatter closing delimiter"
    return text[4:end], None


def raw_description_style_warning(raw_fm: str) -> str | None:
    lines = raw_fm.splitlines()
    for line in lines:
        stripped = line.strip()
        if not stripped.startswith("description:"):
            continue
        after = stripped[len("description:") :].strip()
        if after in {"|", ">", "|-", "|+", ">-", ">+"}:
            return None
        if not after:
            return None
        if len(after) > 120 or YAML_PLAIN_SCALAR_RISK_RE.search(after):
            return "prefer folded block style (`description: >`) for long descriptions or YAML-sensitive punctuation such as `: `"
        return None
    return None


def validate_skill(path: Path) -> list[Finding]:
    findings: list[Finding] = []
    text = path.read_text(encoding="utf-8", errors="replace")
    raw_fm, fm_error = frontmatter(text)
    if fm_error:
        return [Finding("error", path, fm_error)]
    assert raw_fm is not None

    try:
        data: Any = yaml.safe_load(raw_fm) or {}
    except Exception as exc:  # noqa: BLE001 - surface parser class/message exactly
        return [Finding("error", path, f"YAML parse error: {exc.__class__.__name__}: {exc}")]

    if not isinstance(data, dict):
        findings.append(Finding("error", path, "frontmatter must be a YAML mapping"))
        return findings

    name = data.get("name")
    if not isinstance(name, str) or not name.strip():
        findings.append(Finding("error", path, "name is required"))

    description = data.get("description")
    if not isinstance(description, str) or not description.strip():
        findings.append(Finding("error", path, "description is required"))
    elif len(description) > MAX_DESCRIPTION_CHARS:
        findings.append(
            Finding(
                "error",
                path,
                f"description exceeds {MAX_DESCRIPTION_CHARS} characters ({len(description)})",
            )
        )

    style_warning = raw_description_style_warning(raw_fm)
    if style_warning:
        findings.append(Finding("warning", path, style_warning))

    return findings


def discover_skills() -> list[Path]:
    paths: list[Path] = []
    for pattern in SKILL_GLOBS:
        paths.extend(ROOT.glob(pattern))
    return sorted(set(paths))


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate Brehon pi/Claude skill frontmatter")
    parser.add_argument("--strict-style", action="store_true", help="treat style warnings as errors")
    args = parser.parse_args()

    skills = discover_skills()
    findings: list[Finding] = []
    for skill in skills:
        findings.extend(validate_skill(skill))

    errors = [f for f in findings if f.severity == "error"]
    warnings = [f for f in findings if f.severity == "warning"]

    for finding in findings:
        print(f"{finding.severity.upper()}: {rel(finding.path)}: {finding.message}")

    print(f"Checked {len(skills)} skill files: {len(errors)} error(s), {len(warnings)} warning(s)")

    if errors or (args.strict_style and warnings):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
