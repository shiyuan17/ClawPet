#!/usr/bin/env python3
"""Extract a normalized role_spec JSON from role markdown."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


DOMAIN_HINTS = {
    "legal-contract": ["合同", "条款", "仲裁", "诉讼", "违约", "民法典", "法务", "合规", "电子签章", "知识产权"],
    "task-management": ["任务", "计划", "提醒", "跟进", "排期", "待办", "复盘"],
    "education": ["学习", "课程", "考试", "作业", "知识点", "教学"],
    "engineering": ["代码", "开发", "测试", "架构", "部署", "缺陷", "性能"],
}


def parse_frontmatter(text: str) -> dict[str, str]:
    m = re.match(r"\A---\s*\n(.*?)\n---\s*\n", text, re.DOTALL)
    if not m:
        return {}
    result: dict[str, str] = {}
    for line in m.group(1).splitlines():
        line = line.strip()
        if not line or line.startswith("#") or ":" not in line:
            continue
        k, v = line.split(":", 1)
        result[k.strip()] = v.strip().strip('"').strip("'")
    return result


def extract_first_paragraph_after_title(text: str) -> str:
    lines = text.splitlines()
    title_seen = False
    buf: list[str] = []
    for line in lines:
        s = line.strip()
        if s.startswith("# "):
            title_seen = True
            continue
        if not title_seen:
            continue
        if s.startswith("## "):
            if buf:
                break
            continue
        if s:
            buf.append(s)
            if len(buf) >= 2:
                break
    return " ".join(buf).strip()


def find_section_block(text: str, heading_keywords: list[str]) -> str:
    lines = text.splitlines()
    start = None
    for i, line in enumerate(lines):
        if line.startswith("## ") and any(k in line for k in heading_keywords):
            start = i + 1
            break
    if start is None:
        return ""

    end = len(lines)
    for j in range(start, len(lines)):
        if lines[j].startswith("## "):
            end = j
            break
    return "\n".join(lines[start:end]).strip()


def extract_bullets_from_block(block: str) -> list[str]:
    out: list[str] = []
    if not block:
        return out
    for line in block.splitlines():
        s = line.strip()
        if s.startswith("- "):
            out.append(s[2:].strip())
        elif re.match(r"\d+\.\s+", s):
            out.append(re.sub(r"^\d+\.\s+", "", s).strip())
    return out


def extract_h3_headings(block: str) -> list[str]:
    out: list[str] = []
    if not block:
        return out
    for line in block.splitlines():
        s = line.strip()
        if s.startswith("### "):
            out.append(s[4:].strip())
    return out


def normalize_name(s: str) -> str:
    return re.sub(r"\s+", " ", s).strip()


def template_filename(title: str, idx: int) -> str:
    t = title.lower()
    if any(k in title for k in ["意见书", "审查意见"]):
        return "contract-review-opinion-template.md"
    if any(k in title for k in ["风险评估", "风险矩阵"]):
        return "contract-risk-matrix-template.md"
    if any(k in title for k in ["谈判", "协商"]):
        return "negotiation-brief-template.md"
    slug = re.sub(r"[^a-z0-9]+", "-", t).strip("-")
    if not slug:
        slug = f"template-{idx}"
    return f"{slug}.md"


def extract_templates(block: str) -> list[dict]:
    templates: list[dict] = []
    if not block:
        return templates

    lines = block.splitlines()
    current_h3 = ""
    in_code = False
    code_lang = ""
    code_lines: list[str] = []

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("### ") and (not in_code):
            current_h3 = normalize_name(stripped[4:])
            continue

        if stripped.startswith("```"):
            if not in_code:
                in_code = True
                code_lang = stripped[3:].strip().lower()
                code_lines = []
            else:
                if code_lang in ("", "md", "markdown"):
                    title = current_h3 or f"模板{len(templates) + 1}"
                    body = "\n".join(code_lines).strip()
                    if body:
                        templates.append(
                            {
                                "title": title,
                                "filename": template_filename(title, len(templates) + 1),
                                "content": body,
                            }
                        )
                in_code = False
                code_lang = ""
                code_lines = []
            continue

        if in_code:
            code_lines.append(line)

    return templates


def dedupe(items: list[str]) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for item in items:
        key = normalize_name(item)
        if not key:
            continue
        if key in seen:
            continue
        out.append(key)
        seen.add(key)
    return out


def classify_domain(text: str) -> list[str]:
    hay = text.lower()
    scores: list[tuple[int, str]] = []
    for name, kws in DOMAIN_HINTS.items():
        score = 0
        for kw in kws:
            if kw.lower() in hay:
                score += 1
        scores.append((score, name))
    scores.sort(reverse=True)

    tags: list[str] = []
    for score, name in scores:
        if score <= 0:
            continue
        tags.append(name)
        if len(tags) >= 2:
            break
    if not tags:
        tags = ["general"]
    return tags


def list_missing(spec: dict) -> list[str]:
    missing: list[str] = []
    profile = spec["profile"]
    sem = spec["semantics"]
    if not profile["name"]:
        missing.append("profile.name")
    if not profile["description"]:
        missing.append("profile.description")
    if not sem["mission"]:
        missing.append("semantics.mission")
    if not sem["must_rules"]:
        missing.append("semantics.must_rules")
    if not sem["must_not_rules"]:
        missing.append("semantics.must_not_rules")
    if len(sem["workflow_steps"]) < 4:
        missing.append("semantics.workflow_steps(>=4)")
    return missing


def compute_confidence(spec: dict) -> float:
    missing = len(spec["quality"]["missing_fields"])
    base = 1.0 - min(0.8, missing * 0.1)
    if spec["semantics"]["success_metrics"]:
        base += 0.05
    if spec.get("artifacts", {}).get("templates"):
        base += 0.05
    return round(max(0.0, min(1.0, base)), 2)


def extract_role_spec(md_path: Path) -> dict:
    text = md_path.read_text(encoding="utf-8")
    fm = parse_frontmatter(text)

    role_scope = extract_first_paragraph_after_title(text)

    mission_block = find_section_block(text, ["核心使命"])
    rules_block = find_section_block(text, ["必须遵守", "关键规则", "规则"])
    deliverables_block = find_section_block(text, ["专业能力与交付物", "技术交付物", "交付物"])
    workflow_block = find_section_block(text, ["工作流程"])
    style_block = find_section_block(text, ["沟通风格"])
    memory_block = find_section_block(text, ["学习与记忆", "记忆"])
    success_block = find_section_block(text, ["成功指标"])
    identity_block = find_section_block(text, ["身份与角色", "身份与记忆"])

    mission = dedupe(extract_bullets_from_block(mission_block))
    rules = dedupe(extract_bullets_from_block(rules_block))

    deliverables = dedupe(extract_h3_headings(deliverables_block))
    deliverables += dedupe(extract_bullets_from_block(deliverables_block))
    deliverables = dedupe(deliverables)

    workflow = dedupe(extract_bullets_from_block(workflow_block))
    style = dedupe(extract_bullets_from_block(style_block))

    memory = dedupe(extract_bullets_from_block(memory_block))
    if not memory:
        identity_memory = [x for x in extract_bullets_from_block(identity_block) if ("记忆" in x or "追踪" in x)]
        memory = dedupe(identity_memory)

    success = dedupe(extract_bullets_from_block(success_block))

    must_not_rules = [x for x in rules if ("禁止" in x or "不要" in x or "不得" in x)]
    must_rules = [x for x in rules if x not in must_not_rules and ("必须" in x or "应" in x)]
    if rules and not must_rules:
        must_rules = [x for x in rules if x not in must_not_rules]

    templates = extract_templates(deliverables_block)

    domain_text = "\n".join(
        [
            fm.get("name", ""),
            fm.get("description", ""),
            role_scope,
            mission_block,
            rules_block,
            deliverables_block,
            workflow_block,
        ]
    )

    spec = {
        "schema_version": "1.1",
        "ready_for_generation": False,
        "profile": {
            "name": fm.get("name", ""),
            "description": fm.get("description", ""),
            "color": fm.get("color", ""),
            "emoji": fm.get("emoji", ""),
            "vibe": fm.get("vibe", ""),
        },
        "semantics": {
            "role_scope": role_scope,
            "mission": mission,
            "must_rules": must_rules,
            "must_not_rules": must_not_rules,
            "deliverables": deliverables,
            "workflow_steps": workflow,
            "communication_style": style,
            "memory_policy": memory,
            "success_metrics": success,
        },
        "artifacts": {
            "templates": templates,
            "domain_tags": classify_domain(domain_text),
            "source_role_file": str(md_path),
        },
        "quality": {
            "missing_fields": [],
            "assumptions": [],
            "confidence": 0.0,
        },
    }

    if (not spec["semantics"]["deliverables"]) and templates:
        spec["semantics"]["deliverables"] = [x["title"] for x in templates]

    spec["quality"]["missing_fields"] = list_missing(spec)
    spec["ready_for_generation"] = len(spec["quality"]["missing_fields"]) == 0
    spec["quality"]["confidence"] = compute_confidence(spec)
    return spec


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract role_spec from a markdown role file.")
    parser.add_argument("role_md", help="Path to role markdown file")
    parser.add_argument("--out", help="Output JSON path")
    parser.add_argument("--pretty", action="store_true", help="Pretty-print JSON")
    args = parser.parse_args()

    spec = extract_role_spec(Path(args.role_md))
    indent = 2 if args.pretty or args.out else None

    if args.out:
        Path(args.out).write_text(json.dumps(spec, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"Wrote {args.out}")
    else:
        print(json.dumps(spec, ensure_ascii=False, indent=indent))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
