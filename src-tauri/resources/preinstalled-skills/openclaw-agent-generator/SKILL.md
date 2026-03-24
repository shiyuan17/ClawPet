---
name: openclaw-agent-generator
description: 基于结构化 role_spec 生成 OpenClaw agent 草稿，并按领域动态产出工作区骨架文件（BOOTSTRAP/HEARTBEAT/TOOLS）、子 skills 与模板（优先复用角色内置模板代码块）。用于 4-skill 链路中的中间层：intake、generator、reviewer、evaluator。
user-invocable: false
disable-model-invocation: true
metadata:
  {
    "openclaw":
      {
        "emoji": "🛠️",
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

# OpenClaw Agent Generator

## Overview
该 skill 专注“把 role_spec 变成 agent 草稿”。它不负责完整采访（由 intake 完成）也不负责最终放行（由 reviewer/evaluator 完成）。

## Required Input
- 首选输入：`role_spec.json`（见 [references/handoff-contract.md](references/handoff-contract.md)）
- 兼容输入：原始角色 markdown（会触发最小抽取，但建议先经过 intake）

## Boundaries
- Do:
  - 根据 schema 填充模板
  - 将抽象语义转为可执行规则
  - 输出 draft + 预检结果
- Do not:
  - 不绕过字段缺口直接“拍脑袋补全”
  - 不在 BLOCKER 未清零时宣告最终可发布

## Required Workflow

### Step 1: Input Readiness Check
- 检查 `ready_for_generation`。
- 若为 false，优先回退到 intake 补齐字段。

### Step 2: Pattern Selection
- 按 [references/pattern-selection.md](references/pattern-selection.md) 选择模式。
- 默认组合：`Generator + Pipeline`，必要时嵌入 `Reviewer` 回路。

### Step 3: Draft Generation
- 使用 [assets/openclaw-agent-template.md](assets/openclaw-agent-template.md) 生成草稿。
- 规则必须可执行、可观测。
- 成功指标至少包含 2 条可量化条目。
- 若传入 `--workspace-root`，同时生成：
  - `AGENTS.md`（主角色）
  - `BOOTSTRAP.md` / `HEARTBEAT.md` / `TOOLS.md`
  - `skills/*/SKILL.md`（按角色职责拆分）
  - `templates/*.md`（高频交付模板）

### Step 4: Preflight Validation
- 运行快速结构检查：

```bash
python3 scripts/validate_openclaw_agent.py <agent_file> --strict
```

- 若退出码非 0，修复后再继续。
- 若有工作区产物，建议追加：

```bash
python3 scripts/validate_openclaw_agent.py <agent_file> --strict \
  --workspace-root <workspace_root> --require-workspace
```

### Step 5: Handoff to Reviewer
- 将草稿与生成备注交给 `openclaw-agent-reviewer`。
- 输出必须包含：`selected_patterns`、`assumptions`、`open_risks`。

## Output Contract
- 详见 [references/openclaw-agent-contract.md](references/openclaw-agent-contract.md)
- 同时遵守 [references/handoff-contract.md](references/handoff-contract.md)

## Resources
- `references/pattern-selection.md`: 模式选型矩阵
- `references/openclaw-agent-contract.md`: agent 结构契约
- `references/handoff-contract.md`: intake/generator/reviewer 交接格式
- `assets/openclaw-agent-template.md`: 角色模板
- `scripts/validate_openclaw_agent.py`: 预检脚本
