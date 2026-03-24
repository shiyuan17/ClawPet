---
name: openclaw-agent-reviewer
description: 审查 OpenClaw agent 文件的结构、规则冲突、质量门禁与安全约束，并输出结构化审查报告。用于生成后质检、重构回归检查、上线前 gate，以及与 evaluator 协同做长期质量跟踪。
user-invocable: false
disable-model-invocation: true
metadata:
  {
    "openclaw":
      {
        "emoji": "🛡️",
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

# OpenClaw Agent Reviewer

## Overview
该 skill 是 OpenClaw 角色文件的质量闸门。默认执行 `BLOCKER -> HIGH -> MEDIUM` 三级检查，并产出可追踪报告。

## Boundaries
- Do:
  - 按 checklist 报告问题与证据
  - 对 BLOCKER 给出可执行修复建议
  - 运行脚本并输出可机读结果
- Do not:
  - 不跳过 BLOCKER 直接放行
  - 不用“感觉”替代证据

## Required Workflow

### Step 1: Contract Check
- 读取目标 agent 文件。
- 按 [references/quality-gates.md](references/quality-gates.md) 校验结构和字段。

### Step 2: Safety Check
- 按 [references/safety-guardrails.md](references/safety-guardrails.md) 检查高风险行为。
- 发现高风险自动执行、权限越界或敏感数据误用时，至少给出 HIGH。

### Step 3: Consistency Check
- 检查规则冲突（同一行为同时“必须/禁止”）。
- 检查流程是否完整（>=4 步）。
- 检查指标是否可量化。

### Step 4: Structured Reporting
- 输出 JSON 报告（机器可读）和文本摘要（人可读）。
- `BLOCKER > 0` 时必须标记 `pass=false`。

## CLI Helpers

```bash
python3 scripts/review_openclaw_agent.py <agent_md> --json
python3 scripts/review_openclaw_agent.py <agent_md> --out /tmp/review.json
python3 scripts/review_openclaw_agent.py <agent_md> --strict-high
```

## Report Format
参考 [assets/review-report-template.md](assets/review-report-template.md)

## Resources
- `references/quality-gates.md`: BLOCKER/HIGH/MEDIUM 判定标准
- `references/safety-guardrails.md`: 安全与审批门禁
- `assets/review-report-template.md`: 审查报告模板
- `scripts/review_openclaw_agent.py`: 自动化审查脚本
