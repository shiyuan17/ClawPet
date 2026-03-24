# ClawHub 发布规范（OpenClaw Agent Factory）

## 目标
- 对外入口只保留 `openclaw-agent-factory`。
- `openclaw-agent-intake` / `openclaw-agent-generator` / `openclaw-agent-reviewer` / `openclaw-agent-evaluator` 作为内部子 skill，用于流水线复用与分层迭代。

## 目录与文件约束
- 每个 skill 根目录必须包含 `SKILL.md`。
- ClawHub 仅上传文本文件（例如 `md/json/yaml/py`）。
- 运行依赖通过 frontmatter `metadata.openclaw.requires/install` 声明。

## 发布前检查
1. 确认本地可发现：
   `openclaw skills info openclaw-agent-factory --json`
2. 跑一轮端到端：
   `python3 openclaw-agent-factory/scripts/run_factory.py --role-md academic/academic-historian.md --out /tmp/academic-historian.agent.md --summary-out /tmp/academic-historian.summary.json --strict-high`
3. 同步预览：
   `clawhub sync --root "/Users/hsy/Documents/skills 封装角色" --dry-run`

## 发布命令模板

```bash
clawhub publish "/Users/hsy/Documents/skills 封装角色/openclaw-agent-factory" \
  --slug openclaw-agent-factory \
  --name "OpenClaw Agent Factory" \
  --version 1.0.0 \
  --changelog "Initial release"
```

## 版本策略
- `patch`: 行为兼容修复
- `minor`: 新增能力且保持兼容
- `major`: 合约或输出格式不兼容变更

## Sync 建议
- 只发布主 skill 时，优先使用上面的 `clawhub publish`（避免把子 skill 一并发布）。
- 首次批量发布：
  `clawhub sync --root "/Users/hsy/Documents/skills 封装角色" --all --bump patch --changelog "Initial sync"`
- 日常更新：
  先 `--dry-run`，再 `--all` 正式同步。
