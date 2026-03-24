# OpenClaw Agent Output Contract

生成结果默认支持两层产物：
1) 主角色文件（`agent.md` 或 `AGENTS.md`）
2) 可选工作区骨架（传入 `--workspace-root` 时必须产出）

## 1) 主角色文件契约

### Frontmatter

```yaml
---
name: <角色名>
description: <一句话定位>
color: "#RRGGBB"
emoji: <emoji>
vibe: <角色语气>
---
```

### 语义章节

按以下语义顺序输出（标题可轻微变体）：
1. `身份与角色`
2. `核心使命`
3. `必须遵守的规则`
4. `专业能力与交付物`
5. `工作流程`
6. `沟通风格`
7. `学习与记忆`
8. `成功指标`
9. `子技能分工`
10. `工作区文件约定`

### 质量规则
- 规则使用可执行语言（“必须/禁止/当...则...”）
- 工作流程至少 4 步
- 成功指标至少 4 条，且至少 2 条可量化
- 禁止保留占位符（`{{...}}`、`[TODO]`）

## 2) 工作区骨架契约（`--workspace-root`）

必须产出：
- `AGENTS.md`
- `BOOTSTRAP.md`
- `HEARTBEAT.md`
- `TOOLS.md`
- `skills/*/SKILL.md`（至少 2 个）
- `templates/*.md`（至少 2 个）

建议同时创建目录：
- `inbox/`
- `logs/`
- `memory/`
- `reports/`

## 3) Hard Fail Conditions

出现以下任一项即视为不合格：
- 缺少 `name` 或 `description`
- 缺少关键语义章节
- 存在未替换占位符
- 规则冲突（既要求又禁止同一行为）
- 启用工作区模式时缺少 `BOOTSTRAP.md` / `HEARTBEAT.md` / `TOOLS.md`
