# Handoff Contract

## Intake -> Generator

输入推荐使用 `role_spec.json`：

```json
{
  "schema_version": "1.0",
  "ready_for_generation": true,
  "profile": {"name": "", "description": "", "color": "", "emoji": "", "vibe": ""},
  "semantics": {
    "mission": [],
    "must_rules": [],
    "must_not_rules": [],
    "workflow_steps": [],
    "success_metrics": []
  },
  "quality": {"missing_fields": [], "assumptions": [], "confidence": 0.0}
}
```

`ready_for_generation=false` 时，generator 应回退给 intake 补齐。

## Generator -> Reviewer

输出至少包含：
- `agent_draft_path`
- `workspace_root`（若启用工作区骨架）
- `workspace_artifacts`（文件列表）
- `selected_patterns`
- `assumptions`
- `open_risks`

示例：

```json
{
  "agent_draft_path": "/tmp/academic-historian.agent.md",
  "workspace_root": "/tmp/academic-historian-workspace",
  "workspace_artifacts": [
    "AGENTS.md",
    "BOOTSTRAP.md",
    "HEARTBEAT.md",
    "TOOLS.md",
    "skills/task-planner/SKILL.md",
    "templates/daily-plan-template.md"
  ],
  "selected_patterns": ["Inversion", "Generator", "Pipeline"],
  "assumptions": [{"field": "vibe", "assumption": "延用输入描述"}],
  "open_risks": ["成功指标仍偏定性"]
}
```
