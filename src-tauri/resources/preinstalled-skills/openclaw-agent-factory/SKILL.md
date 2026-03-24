---
name: openclaw-agent-factory
description: OpenClaw agent 创建主 skill（单入口）。当用户说“请根据以下角色信息创建 agent”时，调用本 skill 走完 intake、generator、reviewer、evaluator 流水线，输出 agent 文件、工作区骨架（BOOTSTRAP/HEARTBEAT/TOOLS）、子 skills、模板与完整质量报告。
user-invocable: true
disable-model-invocation: false
metadata:
  {
    "openclaw":
      {
        "emoji": "🏭",
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

# OpenClaw Agent Factory

## Overview
这是唯一对外入口 skill。你可提供角色描述文件，或直接提供一段“角色信息 Markdown 文本”，它会自动调用 4 个子 skill 完成采集、生成、审查和回归评测。

## Internal Sub-skills
- `openclaw-agent-intake`: 抽取 `role_spec`
- `openclaw-agent-generator`: 生成 agent 草稿并预检
- `openclaw-agent-reviewer`: 质量与安全门禁
- `openclaw-agent-evaluator`: 回归评测与漂移检测

子 skill 作为内部模块存在，不建议直接作为默认入口使用。

## One-shot Workflow

```bash
python3 scripts/run_factory.py \
  --role-md academic/academic-historian.md \
  --out /tmp/academic-historian.agent.md \
  --workspace-root /tmp/academic-historian-workspace \
  --summary-out /tmp/academic-historian.summary.json
```

当用户在对话里直接说“请根据以下角色信息创建 agent: ...”时，使用：

```bash
python3 scripts/run_factory.py \
  --role-text "<完整角色 Markdown 文本>" \
  --out /tmp/new-agent.agent.md \
  --workspace-root /tmp/new-agent-workspace \
  --summary-out /tmp/new-agent.summary.json
```

## Release Policy
- `BLOCKER` 问题必须为 0
- 如开启 `--strict-high`，则 `HIGH` 也必须为 0
- role_spec 若 `ready_for_generation=false`，默认中止并返回缺口字段

## Output
- Agent 文件（markdown）
- 工作区骨架文件
  - `AGENTS.md`
  - `BOOTSTRAP.md`
  - `HEARTBEAT.md`
  - `TOOLS.md`
  - `skills/*/SKILL.md`
  - `templates/*.md`
- 汇总报告（json）
  - intake 缺口
  - 生成假设
  - reviewer findings
  - evaluator score
  - release 决策

## Resources
- `references/pipeline-architecture.md`: 主 skill 与子 skill 拓扑
- `references/release-policy.md`: 放行规则与退出码
- `references/clawhub-release.md`: ClawHub 发布与版本策略
- `assets/factory-summary-template.json`: 汇总报告模板
- `scripts/run_factory.py`: 一键 orchestrator
