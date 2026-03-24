---
name: openclaw-agent-evaluator
description: 运行 OpenClaw agent 的回归评测与评分卡，比较版本之间的质量漂移，并输出可追踪的 eval 报告。用于持续优化阶段：每次角色改动后验证是否引入回归、是否满足既定质量门禁。
user-invocable: false
disable-model-invocation: true
metadata:
  {
    "openclaw":
      {
        "emoji": "📈",
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

# OpenClaw Agent Evaluator

## Overview
该 skill 负责“持续评测”，不是一次性 review。通过 case 套件对 agent 文本做回归检查，并可与 baseline 比较检测漂移。

## Boundaries
- Do:
  - 执行标准化 eval 套件
  - 输出 scorecard 与回归列表
  - 给出是否可发布的结论
- Do not:
  - 不替代 reviewer 的细粒度审查
  - 不在无案例集时给出“通过”结论

## Required Workflow

### Step 1: Select Eval Suite
- 使用默认样例 [assets/eval-cases.sample.jsonl](assets/eval-cases.sample.jsonl) 或项目自定义 case。
- case 需声明 `severity`（BLOCKER/HIGH/MEDIUM）。

### Step 2: Run Eval

```bash
python3 scripts/run_eval_suite.py \
  --agent <agent_md> \
  --cases <cases.jsonl> \
  --out /tmp/eval_report.json
```

### Step 3: Compare Baseline (Optional)

```bash
python3 scripts/run_eval_suite.py \
  --agent <agent_md> \
  --cases <cases.jsonl> \
  --baseline /tmp/prev_eval_report.json \
  --out /tmp/eval_report.json
```

### Step 4: Release Decision
- `BLOCKER` fail > 0 -> 不可发布
- 若开启 `--fail-on-high` 且 `HIGH` fail > 0 -> 不可发布
- 有 baseline 且 pass rate 下降 -> 标记 regression

## Report Format
见 [assets/scorecard-template.md](assets/scorecard-template.md)

## Resources
- `references/eval-strategy.md`: 评测策略与迭代节奏
- `references/trace-grading-lite.md`: trace 维度打分建议
- `assets/eval-cases.sample.jsonl`: 样例 case 套件
- `assets/scorecard-template.md`: 评分卡模板
- `scripts/run_eval_suite.py`: 回归评测脚本
