#!/usr/bin/env python3
"""Validate OpenClaw agent markdown structure and optional workspace bundle."""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


@dataclass
class ValidationIssue:
    level: str
    message: str


REQUIRED_FRONTMATTER_BASE = {"name", "description"}
REQUIRED_FRONTMATTER_STRICT = REQUIRED_FRONTMATTER_BASE | {"color", "emoji", "vibe"}

SECTION_ALIASES = {
    "identity": ["身份与角色", "身份与记忆", "你的身份与记忆"],
    "mission": ["核心使命", "你的核心使命"],
    "rules": ["必须遵守的规则", "你必须遵守的关键规则", "关键规则"],
    "deliverables": ["专业能力与交付物", "技术交付物", "交付物"],
    "workflow": ["工作流程", "你的工作流程"],
    "style": ["沟通风格", "你的沟通风格"],
    "memory": ["学习与记忆", "记忆"],
    "success": ["成功指标", "你的成功指标"],
}

PLACEHOLDER_PATTERNS = [r"\[TODO", r"\{\{.+?\}\}"]
WORKSPACE_REQUIRED_FILES = ["AGENTS.md", "BOOTSTRAP.md", "HEARTBEAT.md", "TOOLS.md"]


def parse_frontmatter(text: str) -> dict[str, str]:
    match = re.match(r"\A---\s*\n(.*?)\n---\s*\n", text, re.DOTALL)
    if not match:
        return {}

    frontmatter: dict[str, str] = {}
    for raw_line in match.group(1).splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or ":" not in line:
            continue
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip().strip('"').strip("'")
        frontmatter[key] = value
    return frontmatter


def has_semantic_section(text: str, aliases: Iterable[str]) -> bool:
    for alias in aliases:
        pattern = rf"^##\s+.*{re.escape(alias)}.*$"
        if re.search(pattern, text, re.MULTILINE):
            return True
    return False


def count_quantified_metrics(text: str) -> int:
    # Treat digits, percentages, and comparison symbols as measurable hints.
    metric_lines = [
        line
        for line in text.splitlines()
        if line.strip().startswith(("-", "1.", "2.", "3.", "4."))
    ]
    quantified = 0
    for line in metric_lines:
        if re.search(r"\d|%|>=|<=|>|<", line):
            quantified += 1
    return quantified


def validate_agent_file(path: Path, strict: bool) -> list[ValidationIssue]:
    issues: list[ValidationIssue] = []

    try:
        text = path.read_text(encoding="utf-8")
    except Exception as exc:  # pragma: no cover
        return [ValidationIssue("BLOCKER", f"读取失败: {exc}")]

    fm = parse_frontmatter(text)
    required_fm = REQUIRED_FRONTMATTER_STRICT if strict else REQUIRED_FRONTMATTER_BASE
    missing_fm = sorted(k for k in required_fm if not fm.get(k))
    for key in missing_fm:
        issues.append(ValidationIssue("BLOCKER", f"frontmatter 缺少字段: {key}"))

    if not fm:
        issues.append(ValidationIssue("BLOCKER", "缺少 frontmatter（--- ... ---）"))

    for name, aliases in SECTION_ALIASES.items():
        if not has_semantic_section(text, aliases):
            issues.append(ValidationIssue("BLOCKER", f"缺少语义章节: {name}"))

    for pattern in PLACEHOLDER_PATTERNS:
        if re.search(pattern, text):
            issues.append(ValidationIssue("BLOCKER", f"发现未替换占位符: /{pattern}/"))

    workflow_steps = len(re.findall(r"^\s*\d+\.\s+", text, re.MULTILINE))
    if workflow_steps < 4:
        issues.append(ValidationIssue("HIGH", "工作流程步骤少于 4 步"))

    metric_section_match = re.search(r"^##\s+.*成功指标.*$", text, re.MULTILINE)
    if metric_section_match:
        quantified = count_quantified_metrics(text[metric_section_match.start() :])
        if quantified < 2:
            issues.append(ValidationIssue("HIGH", "成功指标中可量化条目少于 2 条"))

    if "禁止" not in text:
        issues.append(ValidationIssue("HIGH", "缺少明确禁止项"))

    return issues


def workspace_level(require_workspace: bool, default: str = "HIGH") -> str:
    if require_workspace:
        return "BLOCKER"
    return default


def has_placeholder(text: str) -> bool:
    for pattern in PLACEHOLDER_PATTERNS:
        if re.search(pattern, text):
            return True
    return False


def validate_workspace_bundle(workspace_root: Path, require_workspace: bool) -> list[ValidationIssue]:
    issues: list[ValidationIssue] = []

    if not workspace_root.exists():
        issues.append(ValidationIssue(workspace_level(require_workspace), f"工作区目录不存在: {workspace_root}"))
        return issues

    for filename in WORKSPACE_REQUIRED_FILES:
        path = workspace_root / filename
        if not path.exists():
            issues.append(ValidationIssue(workspace_level(require_workspace), f"缺少工作区文件: {filename}"))
            continue
        text = path.read_text(encoding="utf-8")
        if not text.strip():
            issues.append(ValidationIssue(workspace_level(require_workspace, "MEDIUM"), f"工作区文件为空: {filename}"))
        if has_placeholder(text):
            issues.append(ValidationIssue(workspace_level(require_workspace), f"工作区文件存在占位符: {filename}"))

    skills_dir = workspace_root / "skills"
    if not skills_dir.exists():
        issues.append(ValidationIssue(workspace_level(require_workspace), "缺少 skills 目录"))
    else:
        skill_defs = sorted(skills_dir.glob("*/SKILL.md"))
        if len(skill_defs) < 2:
            issues.append(ValidationIssue(workspace_level(require_workspace), f"子 skills 数量不足: {len(skill_defs)}"))
        for skill_file in skill_defs:
            text = skill_file.read_text(encoding="utf-8")
            if "# Skill:" not in text:
                issues.append(ValidationIssue("HIGH", f"子 skill 缺少标准标题: {skill_file}"))
            if has_placeholder(text):
                issues.append(ValidationIssue(workspace_level(require_workspace), f"子 skill 存在占位符: {skill_file}"))

    templates_dir = workspace_root / "templates"
    if not templates_dir.exists():
        issues.append(ValidationIssue(workspace_level(require_workspace), "缺少 templates 目录"))
    else:
        templates = sorted(templates_dir.glob("*.md"))
        if len(templates) < 2:
            issues.append(ValidationIssue(workspace_level(require_workspace), f"模板数量不足: {len(templates)}"))
        for tpl in templates:
            if not tpl.read_text(encoding="utf-8").strip():
                issues.append(ValidationIssue("HIGH", f"模板文件为空: {tpl}"))

    return issues


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate OpenClaw agent markdown structure.")
    parser.add_argument("files", nargs="+", help="Path(s) to markdown agent files")
    parser.add_argument("--strict", action="store_true", help="Require full frontmatter fields")
    parser.add_argument("--workspace-root", help="Optional workspace root to validate bundle files")
    parser.add_argument("--require-workspace", action="store_true", help="Treat missing workspace bundle artifacts as BLOCKER")
    args = parser.parse_args()

    has_blocker = False
    has_high = False

    for file_arg in args.files:
        path = Path(file_arg)
        issues = validate_agent_file(path, strict=args.strict)
        if not issues:
            print(f"PASS {path}")
            continue

        print(f"FAIL {path}")
        for issue in issues:
            print(f"- [{issue.level}] {issue.message}")
            if issue.level == "BLOCKER":
                has_blocker = True
            if issue.level == "HIGH":
                has_high = True

    if args.workspace_root:
        workspace_issues = validate_workspace_bundle(Path(args.workspace_root), require_workspace=args.require_workspace)
        if not workspace_issues:
            print(f"PASS_WORKSPACE {args.workspace_root}")
        else:
            print(f"FAIL_WORKSPACE {args.workspace_root}")
            for issue in workspace_issues:
                print(f"- [{issue.level}] {issue.message}")
                if issue.level == "BLOCKER":
                    has_blocker = True
                if issue.level == "HIGH":
                    has_high = True

    if has_blocker:
        return 1
    if args.strict and has_high:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
