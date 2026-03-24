#!/usr/bin/env python3
"""Review OpenClaw agent markdown and optional workspace bundle."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path


@dataclass
class Finding:
    severity: str
    title: str
    evidence: str
    fix: str


SECTION_ALIASES = {
    "identity": ["身份与角色", "身份与记忆", "你的身份与记忆"],
    "mission": ["核心使命", "你的核心使命"],
    "rules": ["必须遵守的规则", "关键规则", "你必须遵守的关键规则"],
    "deliverables": ["专业能力与交付物", "技术交付物", "交付物"],
    "workflow": ["工作流程", "你的工作流程"],
    "style": ["沟通风格", "你的沟通风格"],
    "memory": ["学习与记忆", "记忆"],
    "success": ["成功指标", "你的成功指标"],
}
WORKSPACE_REQUIRED_FILES = ["AGENTS.md", "BOOTSTRAP.md", "HEARTBEAT.md", "TOOLS.md"]


def parse_frontmatter(text: str) -> dict[str, str]:
    match = re.match(r"\A---\s*\n(.*?)\n---\s*\n", text, re.DOTALL)
    if not match:
        return {}
    out: dict[str, str] = {}
    for line in match.group(1).splitlines():
        line = line.strip()
        if not line or ":" not in line:
            continue
        k, v = line.split(":", 1)
        out[k.strip()] = v.strip().strip('"').strip("'")
    return out


def has_section(text: str, aliases: list[str]) -> bool:
    for alias in aliases:
        if re.search(rf"^##\s+.*{re.escape(alias)}.*$", text, re.MULTILINE):
            return True
    return False


def extract_rule_lines(text: str) -> tuple[list[str], list[str]]:
    must: list[str] = []
    must_not: list[str] = []
    for line in text.splitlines():
        s = line.strip()
        if not s.startswith("- "):
            continue
        content = s[2:].strip()
        if "必须" in content:
            must.append(content)
        if "禁止" in content or "不要" in content or "不得" in content:
            must_not.append(content)
    return must, must_not


def normalize_rule(rule: str) -> str:
    rule = re.sub(r"^(必须|禁止|不要|不得)[:：]?", "", rule)
    rule = re.sub(r"\s+", "", rule)
    return rule


def quantified_metric_count(text: str) -> int:
    success_start = re.search(r"^##\s+.*成功指标.*$", text, re.MULTILINE)
    if not success_start:
        return 0
    part = text[success_start.start() :]
    lines = [ln.strip() for ln in part.splitlines() if ln.strip().startswith("- ")]
    return sum(1 for ln in lines if re.search(r"\d|%|>=|<=|>|<", ln))


def workflow_step_count(text: str) -> int:
    return len(re.findall(r"^\s*\d+\.\s+", text, re.MULTILINE))


def review_text(text: str, filename: str) -> dict:
    findings: list[Finding] = []

    fm = parse_frontmatter(text)
    if not fm:
        findings.append(Finding("BLOCKER", "缺少 frontmatter", "未找到 ---...--- 区块", "补充 frontmatter 并至少包含 name/description"))
    else:
        for key in ("name", "description"):
            if not fm.get(key):
                findings.append(Finding("BLOCKER", f"frontmatter 缺少 {key}", f"{key} 为空", f"补充 {key} 字段"))

    for section_name, aliases in SECTION_ALIASES.items():
        if not has_section(text, aliases):
            findings.append(Finding("BLOCKER", f"缺少关键章节: {section_name}", f"未匹配 {aliases}", "补充该语义章节"))

    if re.search(r"\{\{.+?\}\}|\[TODO", text):
        findings.append(Finding("BLOCKER", "存在未替换占位符", "检测到 {{...}} 或 [TODO]", "替换所有占位符为真实内容"))

    must_rules, must_not_rules = extract_rule_lines(text)
    if not must_not_rules:
        findings.append(Finding("HIGH", "缺少明确禁止项", "未发现包含 禁止/不要/不得 的规则", "补充禁止类边界规则"))

    # Simple contradiction: normalized string overlap.
    must_norm = {normalize_rule(x): x for x in must_rules}
    not_norm = {normalize_rule(x): x for x in must_not_rules}
    overlap = sorted(set(must_norm.keys()) & set(not_norm.keys()))
    for key in overlap:
        findings.append(
            Finding(
                "BLOCKER",
                "规则冲突",
                f"同时出现: {must_norm[key]} / {not_norm[key]}",
                "删除冲突项或改为条件化规则",
            )
        )

    steps = workflow_step_count(text)
    if steps < 4:
        findings.append(Finding("HIGH", "工作流程步骤不足", f"仅检测到 {steps} 步", "补充为至少 4 个步骤"))

    quant = quantified_metric_count(text)
    if quant < 2:
        findings.append(Finding("HIGH", "可量化指标不足", f"仅检测到 {quant} 条可量化指标", "补充数字/百分比阈值指标"))

    blocker = sum(1 for f in findings if f.severity == "BLOCKER")
    high = sum(1 for f in findings if f.severity == "HIGH")
    medium = sum(1 for f in findings if f.severity == "MEDIUM")

    return {
        "file": filename,
        "pass": blocker == 0,
        "counts": {"BLOCKER": blocker, "HIGH": high, "MEDIUM": medium},
        "findings": [asdict(f) for f in findings],
    }


def review_workspace(workspace_root: Path, require_workspace: bool) -> list[Finding]:
    findings: list[Finding] = []
    missing_sev = "BLOCKER" if require_workspace else "HIGH"

    if not workspace_root.exists():
        findings.append(
            Finding(
                missing_sev,
                "工作区目录不存在",
                str(workspace_root),
                "确认 --workspace-root 路径正确，或先生成工作区骨架文件",
            )
        )
        return findings

    for filename in WORKSPACE_REQUIRED_FILES:
        path = workspace_root / filename
        if not path.exists():
            findings.append(Finding(missing_sev, f"缺少工作区文件: {filename}", str(path), "补充并定义该文件的行为约束"))
            continue
        text = path.read_text(encoding="utf-8")
        if not text.strip():
            findings.append(Finding("HIGH", f"工作区文件为空: {filename}", str(path), "补充最小可执行定义"))
        if re.search(r"\{\{.+?\}\}|\[TODO", text):
            findings.append(Finding(missing_sev, f"工作区文件存在占位符: {filename}", str(path), "替换所有占位符"))

    skills_dir = workspace_root / "skills"
    if not skills_dir.exists():
        findings.append(Finding(missing_sev, "缺少 skills 目录", str(skills_dir), "创建 skills 目录并至少拆分 2 个子 skill"))
    else:
        skills = sorted(skills_dir.glob("*/SKILL.md"))
        if len(skills) < 2:
            findings.append(
                Finding(
                    missing_sev,
                    "子 skills 拆分不足",
                    f"仅发现 {len(skills)} 个 SKILL.md",
                    "按角色职责至少拆分 2 个子 skill，并写明输入/输出",
                )
            )
        for skill_file in skills:
            text = skill_file.read_text(encoding="utf-8")
            if "# Skill:" not in text:
                findings.append(
                    Finding(
                        "HIGH",
                        "子 skill 缺少标准标题",
                        str(skill_file),
                        "使用 '# Skill: <name>' 作为首个标题",
                    )
                )

    templates_dir = workspace_root / "templates"
    if not templates_dir.exists():
        findings.append(Finding(missing_sev, "缺少 templates 目录", str(templates_dir), "补充模板目录并放入高频交付模板"))
    else:
        templates = sorted(templates_dir.glob("*.md"))
        if len(templates) < 2:
            findings.append(
                Finding(
                    missing_sev,
                    "模板数量不足",
                    f"仅发现 {len(templates)} 个模板",
                    "至少提供 2 个可直接复用的 markdown 模板",
                )
            )

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Review OpenClaw agent markdown")
    parser.add_argument("agent_md", help="Path to agent markdown file")
    parser.add_argument("--json", action="store_true", help="Print JSON report")
    parser.add_argument("--out", help="Write JSON report to path")
    parser.add_argument("--strict-high", action="store_true", help="Return non-zero if HIGH exists")
    parser.add_argument("--workspace-root", help="Optional workspace root for bundle review")
    parser.add_argument("--require-workspace", action="store_true", help="Treat missing workspace artifacts as BLOCKER")
    args = parser.parse_args()

    text = Path(args.agent_md).read_text(encoding="utf-8")
    report = review_text(text, args.agent_md)
    findings = report["findings"]

    if args.workspace_root:
        ws_findings = review_workspace(Path(args.workspace_root), require_workspace=args.require_workspace)
        findings.extend(asdict(x) for x in ws_findings)

        blocker = sum(1 for f in findings if f["severity"] == "BLOCKER")
        high = sum(1 for f in findings if f["severity"] == "HIGH")
        medium = sum(1 for f in findings if f["severity"] == "MEDIUM")
        report["counts"] = {"BLOCKER": blocker, "HIGH": high, "MEDIUM": medium}
        report["pass"] = blocker == 0
        report["findings"] = findings

    if args.out:
        Path(args.out).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"Wrote {args.out}")
    elif args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        print(f"Review {report['file']}")
        print(f"- Pass: {report['pass']}")
        print(f"- BLOCKER: {report['counts']['BLOCKER']}")
        print(f"- HIGH: {report['counts']['HIGH']}")
        print(f"- MEDIUM: {report['counts']['MEDIUM']}")
        for i, f in enumerate(report["findings"], start=1):
            print(f"{i}. [{f['severity']}] {f['title']}")
            print(f"   Evidence: {f['evidence']}")
            print(f"   Fix: {f['fix']}")

    if report["counts"]["BLOCKER"] > 0:
        return 1
    if args.strict_high and report["counts"]["HIGH"] > 0:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
