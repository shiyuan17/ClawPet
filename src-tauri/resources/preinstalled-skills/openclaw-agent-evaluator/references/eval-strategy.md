# Eval Strategy

## Why
评测用于发现回归，而不是证明“模型很聪明”。

## Core Rules
- 每次改动后都跑同一套基线 case
- 失败 case 必须可复现（regex/规则明确）
- 增量加入边缘 case（模糊需求、冲突规则、敏感边界）

## Metrics
- pass_rate = pass_cases / total_cases
- blocker_fail_count
- high_fail_count
- regression = 当前 pass_rate < baseline pass_rate

## Recommended Cadence
- 每次 prompt/模板/规则改动：必跑
- 每周汇总：观察长期漂移
