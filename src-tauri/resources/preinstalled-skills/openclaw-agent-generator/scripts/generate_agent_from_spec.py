#!/usr/bin/env python3
"""Generate OpenClaw agent markdown and workspace bundle from role_spec JSON."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


DOMAIN_PROFILES = {
    "legal-contract": {
        "keywords": ["合同", "条款", "仲裁", "诉讼", "违约", "法务", "电子签章", "争议"],
        "skills": [
            {
                "name": "contract-intake",
                "keywords": ["主体", "资格", "背景", "交易", "签约"],
                "purpose": "收集并校验合同背景、签约主体与授权材料",
                "inputs": ["合同文本", "交易背景", "主体资质/授权文件"],
                "outputs": ["审查范围确认", "主体与授权风险清单", "缺失材料列表"],
            },
            {
                "name": "clause-risk-scanner",
                "keywords": ["条款", "风险", "违约", "赔偿", "免责", "知识产权", "保密"],
                "purpose": "按条款维度识别风险并完成高/中/低分级",
                "inputs": ["合同条款全文", "条款审查清单"],
                "outputs": ["逐条风险发现", "风险等级", "红线问题汇总"],
            },
            {
                "name": "redline-drafter",
                "keywords": ["修改", "建议", "替代", "谈判", "红线"],
                "purpose": "给出可执行的红线修改建议和谈判替代条款",
                "inputs": ["风险条款", "业务可接受边界"],
                "outputs": ["修订条款建议", "替代方案", "谈判要点"],
            },
            {
                "name": "dispute-resolution-designer",
                "keywords": ["争议", "仲裁", "诉讼", "管辖", "调解"],
                "purpose": "设计争议解决路径与管辖策略",
                "inputs": ["合同争议解决条款", "地域与交易信息"],
                "outputs": ["争议解决方案", "管辖条款优化建议", "执行风险提示"],
            },
            {
                "name": "esign-compliance-checker",
                "keywords": ["电子签章", "电子签名", "存证", "实名认证"],
                "purpose": "核查电子签章流程与证据链合规性",
                "inputs": ["签署平台信息", "签署流程记录"],
                "outputs": ["合规检查结论", "证据链缺口", "整改建议"],
            },
        ],
        "heartbeat_fields": [
            "待审合同数",
            "高风险条款数",
            "待补主体资质项",
            "争议解决条款缺失数",
            "48 小时内到期合同数",
        ],
        "template_pack": {
            "contract-review-opinion-template.md": """# 合同审查意见书模板

## 基本信息
- 合同名称：
- 合同编号：
- 对方主体：
- 合同金额：
- 审查日期：
- 审查人：

## 风险总评
- 整体风险等级：🔴高 / 🟡中 / 🟢低
- 核心风险点数量：高_个 / 中_个 / 低_个
- 审查建议：可签署 / 修改后签署 / 不建议签署

## 逐条审查意见
### 🔴 高风险条款（必须修改）
| 条款位置 | 原文摘要 | 风险说明 | 修改建议 |
|---|---|---|---|
|  |  |  |  |

### 🟡 中风险条款（建议修改）
| 条款位置 | 原文摘要 | 风险说明 | 修改建议 |
|---|---|---|---|
|  |  |  |  |

### 🟢 低风险/提示事项
| 条款位置 | 提示内容 |
|---|---|
|  |  |
""",
            "contract-risk-matrix-template.md": """# 合同风险评估矩阵模板

## 评估维度
| 维度 | 评分(1-5) | 风险说明 |
|---|---|---|
| 主体资信风险 |  |  |
| 条款完备性 |  |  |
| 权利义务对等性 |  |  |
| 违约救济充分性 |  |  |
| 争议解决合理性 |  |  |
| 法律合规性 |  |  |
| 商业可行性 |  |  |

## 综合评分
- 总分：__/35
- 结论：低风险 / 中风险 / 高风险
""",
            "clause-redline-log-template.md": """# 条款修订跟踪模板

| 条款位置 | 原条款摘要 | 修订建议 | 业务反馈 | 当前状态 |
|---|---|---|---|---|
|  |  |  |  | 待确认 |
""",
        },
    },
    "task-management": {
        "keywords": ["任务", "提醒", "排期", "计划", "跟进", "待办"],
        "skills": [
            {
                "name": "task-planner",
                "keywords": ["计划", "拆解", "任务", "优先级", "排期"],
                "purpose": "将模糊目标拆解为可执行任务并排序优先级",
                "inputs": ["用户目标 / 待办列表 / 截止日期"],
                "outputs": ["结构化任务清单", "优先级顺序", "排期建议"],
            },
            {
                "name": "followup-manager",
                "keywords": ["跟进", "逾期", "阻塞", "催办", "状态"],
                "purpose": "跟进等待中的任务，识别逾期与阻塞并推动闭环",
                "inputs": ["进行中任务与状态记录", "等待事项清单"],
                "outputs": ["待跟进事项", "风险级别", "推荐跟进动作"],
            },
            {
                "name": "reminder-assistant",
                "keywords": ["提醒", "截止", "时间点", "提醒机制"],
                "purpose": "根据任务节奏生成提醒策略与执行触发点",
                "inputs": ["任务清单", "关键时间节点"],
                "outputs": ["提醒计划", "截止前动作清单", "下一步提醒文案"],
            },
        ],
        "heartbeat_fields": ["活跃任务数", "高优先级任务数", "逾期任务数", "等待中事项数"],
        "template_pack": {
            "daily-plan-template.md": """# 每日计划模板

## 今日目标
- 

## 今日最重要 3 件事
1. 
2. 
3. 

## 次要任务
- 

## 时间块建议
- 上午：
- 下午：
- 晚上：
""",
            "followup-template.md": """# 跟进模板

## 待跟进事项
- 事项：
- 当前状态：
- 上次进展：
- 需要谁回复：
- 建议跟进时间：
""",
        },
    },
    "general": {
        "keywords": [],
        "skills": [
            {
                "name": "input-normalizer",
                "keywords": ["输入", "需求", "上下文"],
                "purpose": "整理输入上下文并定义任务边界",
                "inputs": ["用户请求", "约束条件"],
                "outputs": ["结构化任务定义", "关键假设清单"],
            },
            {
                "name": "solution-builder",
                "keywords": ["方案", "建议", "设计", "交付"],
                "purpose": "产出结构化方案与可执行交付内容",
                "inputs": ["任务定义", "参考资料"],
                "outputs": ["候选方案", "推荐方案", "执行步骤"],
            },
            {
                "name": "quality-gate",
                "keywords": ["检查", "质量", "风险", "验证"],
                "purpose": "执行交付前质量检查并输出风险提示",
                "inputs": ["方案草稿", "质量门禁清单"],
                "outputs": ["检查结论", "风险清单", "修复建议"],
            },
        ],
        "heartbeat_fields": ["活跃任务数", "阻塞项数量", "本轮关键目标"],
        "template_pack": {
            "delivery-template.md": """# 交付模板

## 背景
- 

## 分析
- 

## 建议
- 

## 风险与下一步
- 
""",
            "review-template.md": """# 复核模板

## 检查项
- 

## 发现问题
- 

## 修复建议
- 
""",
        },
    },
}


def pick(items: list[str], idx: int, default: str) -> str:
    if idx < len(items) and items[idx].strip():
        return items[idx].strip()
    return default


def ensure_len(items: list[str], n: int, filler: list[str]) -> list[str]:
    out = [x.strip() for x in items if x and x.strip()]
    i = 0
    while len(out) < n and i < len(filler):
        out.append(filler[i])
        i += 1
    return out[:n]


def count_quantifiable(items: list[str]) -> int:
    count = 0
    for item in items:
        if any(token in item for token in [">=", "<=", "%", ">", "<"]):
            count += 1
            continue
        if any(ch.isdigit() for ch in item):
            count += 1
    return count


def normalize_line(item: str) -> str:
    s = item.strip()
    s = re.sub(r"^(必须|禁止|不要|不得)[:：]\s*", "", s)
    return s.strip()


def score_keywords(keywords: list[str], haystack: str) -> int:
    score = 0
    for kw in keywords:
        if kw.lower() in haystack:
            score += 1
    return score


def detect_domain(spec: dict) -> str:
    artifact_tags = (spec.get("artifacts", {}) or {}).get("domain_tags", [])
    if artifact_tags:
        first = artifact_tags[0]
        if first in DOMAIN_PROFILES:
            return first

    sem = spec.get("semantics", {})
    profile = spec.get("profile", {})
    haystack = " ".join(
        [
            profile.get("name", ""),
            profile.get("description", ""),
            sem.get("role_scope", ""),
            *sem.get("mission", []),
            *sem.get("deliverables", []),
            *sem.get("workflow_steps", []),
        ]
    ).lower()

    best_domain = "general"
    best_score = 0
    for name, profile_cfg in DOMAIN_PROFILES.items():
        score = score_keywords(profile_cfg.get("keywords", []), haystack)
        if score > best_score:
            best_score = score
            best_domain = name
    return best_domain


def slugify(text: str) -> str:
    s = text.lower().strip()
    s = re.sub(r"[^a-z0-9]+", "-", s)
    s = s.strip("-")
    return s


def derive_subskills(spec: dict, domain: str) -> list[dict]:
    sem = spec.get("semantics", {})
    source_lines = [
        sem.get("role_scope", ""),
        *sem.get("mission", []),
        *sem.get("deliverables", []),
        *sem.get("workflow_steps", []),
        *sem.get("success_metrics", []),
    ]
    haystack = " ".join(normalize_line(x) for x in source_lines if x).lower()

    profile_cfg = DOMAIN_PROFILES.get(domain, DOMAIN_PROFILES["general"])
    candidates = profile_cfg.get("skills", [])
    scored: list[tuple[int, dict]] = []
    for skill in candidates:
        scored.append((score_keywords(skill.get("keywords", []), haystack), skill))
    scored.sort(key=lambda x: x[0], reverse=True)

    selected: list[dict] = []
    seen: set[str] = set()
    for score, skill in scored:
        if score <= 0:
            continue
        if skill["name"] in seen:
            continue
        selected.append(skill)
        seen.add(skill["name"])
        if len(selected) >= 4:
            break

    if len(selected) < 3:
        for skill in candidates:
            if skill["name"] in seen:
                continue
            selected.append(skill)
            seen.add(skill["name"])
            if len(selected) >= 3:
                break

    # If deliverables carry explicit labels, synthesize one skill to cover unmatched area.
    deliverables = sem.get("deliverables", [])
    if deliverables:
        top = deliverables[0]
        synthetic_name = slugify(top) or "deliverable-specialist"
        if synthetic_name not in seen:
            selected.append(
                {
                    "name": synthetic_name,
                    "purpose": f"围绕“{top}”提供专门产出与质量控制",
                    "inputs": ["任务背景", "原始材料"],
                    "outputs": ["结构化结果", "质量自检结论"],
                }
            )

    return selected[:5]


def render_skill_markdown(skill: dict) -> str:
    lines = [
        f"# Skill: {skill['name']}",
        "",
        "用途：",
        f"- {skill['purpose']}",
        "",
        "输入：",
    ]
    for item in skill.get("inputs", []):
        lines.append(f"- {item}")
    lines += ["", "输出："]
    for item in skill.get("outputs", []):
        lines.append(f"- {item}")
    lines += [
        "",
        "边界：",
        "- 不擅自更改用户已确认约束",
        "- 信息缺失时先标注不确定性并提出最小补充问题",
        "",
    ]
    return "\n".join(lines)


def build_tools_markdown(domain: str, spec: dict) -> str:
    name = (spec.get("profile", {}) or {}).get("name") or "该角色"
    if domain == "legal-contract":
        return f"""# TOOLS.md

## 工具边界
- `{name}` 仅提供合同审查与风险控制建议，不替代执业律师出具正式法律意见
- 涉及重大诉讼/仲裁策略时，必须标注“需律师复核”

## 审查工具链
- 条款结构化检查：主体、标的、价款、违约、知识产权、保密、争议解决
- 风险分级矩阵：高风险(必须修改) / 中风险(建议修改) / 低风险(提示)
- 红线修订对照：原条款、风险说明、替代条款、谈判话术

## 证据与引用规则
- 法律依据必须写明法规名称与条款号（如《民法典》第585条）
- 结论必须区分“确定事实”“推断结论”“待确认信息”

## 高风险动作审批
- 对外发送最终法律结论前，需二次确认
- 涉及盖章、签署、争议策略落地的动作必须先获授权
"""

    return """# TOOLS.md

## 工具使用原则
- 先使用本地可验证信息，再调用外部工具
- 高风险动作（外发消息、删除、覆盖、执行外部命令）需要显式确认
- 输出中必须标明关键假设与数据来源

## 默认工具策略
- 检索：优先结构化搜索，再做全文扫描
- 生成：先给可执行版本，再给可选优化方案
- 复核：关键结论至少做一次交叉检查
"""


def build_bootstrap_markdown(name: str, subskills: list[dict], workflow: list[str], domain: str) -> str:
    steps = ensure_len(
        workflow,
        5,
        [
            "读取 `AGENTS.md`，确认角色边界与目标",
            "加载 `skills/` 并确认可用子技能",
            "加载 `templates/` 并选择本次任务模板",
            "输出首轮执行计划与风险提示",
            "进入执行并持续复核",
        ],
    )

    lines = ["# BOOTSTRAP.md", "", f"{name} 启动后建议按如下顺序运行："]
    for i, step in enumerate(steps, start=1):
        lines.append(f"{i}. {step}")

    if domain == "legal-contract":
        lines += [
            "",
            "法律合同场景额外检查：",
            "- 先确认合同版本、签约主体与授权链",
            "- 高风险条款必须进入红线清单，不得口头带过",
            "- 输出结论时区分“必须改/建议改/可接受风险”",
        ]

    lines += ["", "建议优先启用的子技能："]
    for skill in subskills:
        lines.append(f"- `{skill['name']}`：{skill['purpose']}")
    lines.append("")
    return "\n".join(lines)


def build_heartbeat_markdown(metrics: list[str], domain: str) -> str:
    profile = DOMAIN_PROFILES.get(domain, DOMAIN_PROFILES["general"])
    pulse = profile.get("heartbeat_fields", ["活跃任务数", "阻塞项数量", "本轮关键目标"])
    metric_items = ensure_len(metrics, 3, ["任务完成率 >= 85%", "关键规则违规数 = 0", "返工轮次 <= 2"])

    lines = ["# HEARTBEAT.md", "", "心跳消息建议包含："]
    for item in pulse:
        lines.append(f"- {item}")
    lines += ["", "质量追踪建议："]
    for item in metric_items:
        lines.append(f"- {item}")
    lines.append("")
    return "\n".join(lines)


def build_identity_markdown(spec: dict) -> str:
    profile = spec.get("profile", {})
    name = profile.get("name") or "未命名角色"
    vibe = profile.get("vibe") or "理性、清晰、可执行"
    emoji = profile.get("emoji") or "🧩"
    color = profile.get("color") or "#4F46E5"
    description = profile.get("description") or "待补充角色定位"

    return f"""# IDENTITY.md

- **Name:** {name}
- **Emoji:** {emoji}
- **Color:** {color}
- **Vibe:** {vibe}

---

{description}
"""


def build_user_markdown(domain: str) -> str:
    if domain == "legal-contract":
        return """# USER.md

在对话中优先补充以下上下文：
- 合同类型、交易背景、合作目标
- 当前争议点与红线条款
- 期望签署时间与谈判窗口
- 对方主体信息与授权资料是否齐全
"""

    return """# USER.md

在对话中持续记录用户偏好与上下文：
- 目标与约束
- 时间优先级
- 风险偏好
- 常用交付格式
"""


def build_soul_markdown(agent_md: str) -> str:
    return "# SOUL.md\n\n" + agent_md.strip() + "\n"


def build_markdown(spec: dict, subskills: list[dict]) -> str:
    profile = spec.get("profile", {})
    sem = spec.get("semantics", {})

    name = profile.get("name") or "未命名角色"
    description = profile.get("description") or "待补充角色定位"
    color = profile.get("color") or "#4F46E5"
    emoji = profile.get("emoji") or "🧩"
    vibe = profile.get("vibe") or "理性、清晰、可执行"

    role_scope = sem.get("role_scope") or "面向复杂任务提供结构化支持的专业智能体"
    mission = ensure_len(sem.get("mission", []), 2, ["确保输出准确且可执行", "在约束下持续优化结果质量"])
    must_rules = ensure_len(sem.get("must_rules", []), 2, ["优先澄清不完整信息后再执行", "输出必须结构化且可验证"])
    must_not_rules = ensure_len(sem.get("must_not_rules", []), 2, ["禁止将未知信息当作确定事实", "禁止在缺乏授权时执行高风险动作"])
    deliverables = ensure_len(sem.get("deliverables", []), 2, ["结构化分析报告", "可执行行动清单"])
    workflow = ensure_len(
        sem.get("workflow_steps", []),
        5,
        ["澄清目标与边界", "收集并验证关键信息", "形成候选方案", "按清单自检并修订", "输出最终结论与下一步"],
    )
    comm_style = ensure_len(sem.get("communication_style", []), 2, ["先给结论，再给依据", "反馈直接但保持尊重"])
    memory = ensure_len(sem.get("memory_policy", []), 2, ["记录已确认约束与假设", "发现冲突时优先回溯来源"])

    metrics = ensure_len(
        sem.get("success_metrics", []),
        4,
        ["任务完成率 >= 85%", "关键规则违规数 = 0", "返工轮次 <= 2", "用户满意度 >= 4.5/5"],
    )
    if count_quantifiable(metrics) < 2:
        quant_fillers = ["任务完成率 >= 85%", "返工轮次 <= 2", "关键规则违规数 = 0", "用户满意度 >= 4.5/5"]
        replace_idx = len(metrics) - 1
        for q in quant_fillers:
            if count_quantifiable(metrics) >= 2:
                break
            if q in metrics:
                continue
            if replace_idx < 0:
                replace_idx = 0
            metrics[replace_idx] = q
            replace_idx -= 1

    lines = [
        "---",
        f"name: {name}",
        f"description: {description}",
        f"color: \"{color}\"",
        f"emoji: {emoji}",
        f"vibe: {vibe}",
        "---",
        "",
        f"# {name}智能体人格",
        "",
    ]

    intro = role_scope.strip()
    if not intro.startswith("你是"):
        intro = f"你是**{name}**，{intro}"
    if intro and intro[-1] not in "。！？!?":
        intro += "。"
    lines += [intro, ""]

    lines += ["## 身份与角色", f"- 角色：{role_scope}", f"- 个性：{vibe}", f"- 经验：聚焦 {pick(deliverables, 0, '结构化交付')} 与 {pick(deliverables, 1, '质量门禁')} 的落地实践", ""]
    lines += ["## 核心使命", f"- {mission[0]}", f"- {mission[1]}", ""]
    lines += ["## 必须遵守的规则", f"- 必须：{must_rules[0]}", f"- 必须：{must_rules[1]}", f"- 禁止：{must_not_rules[0]}", f"- 禁止：{must_not_rules[1]}", ""]
    lines += ["## 专业能力与交付物", f"- 能力 1：{deliverables[0]}", f"- 能力 2：{deliverables[1]}", ""]

    lines += [
        "```markdown",
        "# 交付模板",
        "## 背景",
        "- 目标与约束",
        "## 分析",
        "- 关键发现",
        "## 建议",
        "- 方案与取舍",
        "## 风险与下一步",
        "- 风险清单与行动",
        "```",
        "",
    ]

    lines += ["## 工作流程"]
    for i, step in enumerate(workflow, start=1):
        lines.append(f"{i}. {step}")
    lines += ["", "## 沟通风格", f"- {comm_style[0]}", f"- {comm_style[1]}", "", "## 学习与记忆", f"- 追踪：{memory[0]}", f"- 发现冲突时：{memory[1]}", ""]

    lines += ["## 成功指标", f"- {metrics[0]}", f"- {metrics[1]}", f"- {metrics[2]}", f"- {metrics[3]}", ""]

    lines += ["## 子技能分工"]
    for skill in subskills:
        lines.append(f"- `{skill['name']}`：{skill['purpose']}")

    lines += [
        "",
        "## 工作区文件约定",
        "- `BOOTSTRAP.md`：定义启动检查顺序与优先级",
        "- `HEARTBEAT.md`：定义心跳指标与状态报告字段",
        "- `TOOLS.md`：定义工具使用边界与审批策略",
        "- `skills/*/SKILL.md`：子职责拆分与路由说明",
        "- `templates/*.md`：高频交付模板库",
    ]

    return "\n".join(lines) + "\n"


def write_file(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content.rstrip() + "\n", encoding="utf-8")


def load_role_templates(spec: dict) -> dict[str, str]:
    template_items = (spec.get("artifacts", {}) or {}).get("templates", [])
    out: dict[str, str] = {}
    for idx, item in enumerate(template_items, start=1):
        filename = item.get("filename") or f"template-{idx}.md"
        if not filename.endswith(".md"):
            filename += ".md"
        content = (item.get("content") or "").strip()
        if not content:
            continue
        out[filename] = content
    return out


def write_workspace_bundle(
    workspace_root: Path,
    spec: dict,
    domain: str,
    agent_name: str,
    agent_md: str,
    subskills: list[dict],
    metrics: list[str],
) -> list[Path]:
    written: list[Path] = []

    for dirname in ["skills", "templates", "inbox", "logs", "memory", "reports"]:
        d = workspace_root / dirname
        d.mkdir(parents=True, exist_ok=True)
        written.append(d)

    agents_path = workspace_root / "AGENTS.md"
    write_file(agents_path, agent_md)
    written.append(agents_path)

    sem = spec.get("semantics", {})
    write_file(workspace_root / "BOOTSTRAP.md", build_bootstrap_markdown(agent_name, subskills, sem.get("workflow_steps", []), domain))
    write_file(workspace_root / "HEARTBEAT.md", build_heartbeat_markdown(metrics, domain))
    write_file(workspace_root / "TOOLS.md", build_tools_markdown(domain, spec))
    write_file(workspace_root / "IDENTITY.md", build_identity_markdown(spec))
    write_file(workspace_root / "USER.md", build_user_markdown(domain))
    write_file(workspace_root / "SOUL.md", build_soul_markdown(agent_md))
    written += [
        workspace_root / "BOOTSTRAP.md",
        workspace_root / "HEARTBEAT.md",
        workspace_root / "TOOLS.md",
        workspace_root / "IDENTITY.md",
        workspace_root / "USER.md",
        workspace_root / "SOUL.md",
    ]

    for skill in subskills:
        skill_file = workspace_root / "skills" / skill["name"] / "SKILL.md"
        write_file(skill_file, render_skill_markdown(skill))
        written.append(skill_file)

    # Role-provided templates first, then domain defaults as fallback.
    role_templates = load_role_templates(spec)
    for filename, content in role_templates.items():
        tpl_file = workspace_root / "templates" / filename
        write_file(tpl_file, content)
        written.append(tpl_file)

    defaults = DOMAIN_PROFILES.get(domain, DOMAIN_PROFILES["general"]).get("template_pack", {})
    for filename, content in defaults.items():
        tpl_file = workspace_root / "templates" / filename
        if tpl_file.exists():
            continue
        write_file(tpl_file, content)
        written.append(tpl_file)

    return written


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate OpenClaw agent markdown from role_spec JSON")
    parser.add_argument("--role-spec", required=True, help="Path to role_spec JSON")
    parser.add_argument("--out", required=True, help="Output agent markdown path")
    parser.add_argument("--workspace-root", help="Optional workspace root to emit BOOTSTRAP/HEARTBEAT/TOOLS + skills/templates")
    parser.add_argument("--no-workspace-bundle", action="store_true", help="Do not emit workspace bundle files")
    args = parser.parse_args()

    spec = json.loads(Path(args.role_spec).read_text(encoding="utf-8"))
    domain = detect_domain(spec)
    subskills = derive_subskills(spec, domain)
    md = build_markdown(spec, subskills)

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(md, encoding="utf-8")
    print(f"Wrote {out_path}")

    if args.workspace_root and (not args.no_workspace_bundle):
        metrics = ensure_len(
            spec.get("semantics", {}).get("success_metrics", []),
            4,
            ["任务完成率 >= 85%", "关键规则违规数 = 0", "返工轮次 <= 2", "用户满意度 >= 4.5/5"],
        )
        profile_name = (spec.get("profile", {}) or {}).get("name") or "Agent"
        workspace_root = Path(args.workspace_root)
        written = write_workspace_bundle(workspace_root, spec, domain, profile_name, md, subskills, metrics)
        print(f"Wrote workspace bundle: {workspace_root}")
        for item in written:
            if item.is_file():
                print(f"- {item}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
