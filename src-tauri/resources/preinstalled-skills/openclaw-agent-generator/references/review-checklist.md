# Review Checklist

在交付前按本清单审查。严重级别分为：`BLOCKER` / `HIGH` / `MEDIUM`。

## BLOCKER

- frontmatter 必填字段齐全（`name`, `description`；严格模式含 `color`, `emoji`, `vibe`）
- 输出不含未替换占位符（`{{`, `}}`, `[TODO]`）
- 存在明确边界：角色不做什么、何时拒绝
- 没有明显规则冲突
- 核心流程存在并可执行

## HIGH

- 每条关键规则可操作，可观察，不空泛
- 交付物模板与角色使命一致
- 有至少 1 条风险处理策略（例如输入信息不足时的处理）
- 成功指标可用于复盘，不是纯口号

## MEDIUM

- 术语与语气保持统一
- 段落层级清晰，便于后续维护
- 与输入角色设定保持一致，不无端扩写设定

## Review Output Format

使用以下格式输出审查结论：

```text
Review Summary
- BLOCKER: <数量>
- HIGH: <数量>
- MEDIUM: <数量>

Findings
1. [SEVERITY] <问题>
   - Evidence: <证据>
   - Fix: <修复建议>
```

`BLOCKER > 0` 时，禁止交付最终 agent。
