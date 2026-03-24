---
name: openclaw-agent-intake
description: 从原始角色资料（尤其 academic/*.md 风格的 Markdown）抽取结构化 role_spec，并在信息缺失时执行 Inversion 提问。用于 OpenClaw agent 生产链路的第 1 步输入标准化：字段归一、置信度标注、缺口清单和后续 handoff。
user-invocable: false
disable-model-invocation: true
metadata:
  {
    "openclaw":
      {
        "emoji": "🧾",
        "requires": { "bins": ["python3"] },
        "install":
          [
            {
              "id": "brew",
              "kind": "brew",
              "formula": "python",
              "bins": ["python3"],
              "label": "Install Python 3 (brew)",
            },
          ],
      },
  }
---

# OpenClaw Agent Intake

## Overview
把非结构化角色描述转成统一 `role_spec.json`。该 skill 只负责采集与标准化，不直接生成最终 agent 文件。

## Boundaries
- Do:
  - 解析 frontmatter 与正文语义块
  - 标注缺失字段与低置信度项
  - 产出可交给 `openclaw-agent-generator` 的 `role_spec`
- Do not:
  - 不直接编写最终 agent 文档
  - 不掩盖字段缺口

## Required Workflow

### Step 1: Source Detection
- 输入可以是：单个 markdown、目录（批量）、对话中的角色文本。
- 优先识别 `academic/*.md` 风格结构。

### Step 2: Structured Extraction
- 按 [references/role-spec-schema.md](references/role-spec-schema.md) 逐项填充。
- 从 frontmatter 提取 `name/description/color/emoji/vibe`。
- 从正文提取：使命、规则、交付物、流程、沟通风格、记忆策略、成功指标。

### Step 3: Inversion Interview
- 若缺字段，按 [references/interview-playbook.md](references/interview-playbook.md) 的顺序追问。
- 每次只问最小必要问题，避免一次性长问卷。
- 高风险字段（权限、敏感数据、外部执行）禁止猜测。

### Step 4: Handoff Packaging
- 输出 `role_spec.json`。
- 同时输出：
  - `missing_fields`
  - `assumptions`
  - `confidence`（0-1）

## CLI Helpers

```bash
python3 scripts/extract_role_spec.py <role_md> --out /tmp/role_spec.json
python3 scripts/extract_role_spec.py <role_md> --pretty
```

## Output Contract
- 主输出必须是 JSON，字段见 [references/role-spec-schema.md](references/role-spec-schema.md)
- 若信息不足，`ready_for_generation` 必须为 `false`
- 不允许把未知内容写成确定事实

## Resources
- `references/role-spec-schema.md`: role_spec 字段契约
- `references/interview-playbook.md`: 追问顺序与提问模板
- `assets/role-spec-template.json`: JSON 模板
- `scripts/extract_role_spec.py`: 从 Markdown 抽取 role_spec
