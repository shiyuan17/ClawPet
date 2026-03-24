# Role Spec Schema

`role_spec` 是 intake 到 generator 的唯一标准输入。

## Top-level

```json
{
  "schema_version": "1.1",
  "ready_for_generation": true,
  "profile": {},
  "semantics": {},
  "artifacts": {},
  "quality": {}
}
```

## profile
- `name`: string
- `description`: string
- `color`: string (`#RRGGBB`)
- `emoji`: string
- `vibe`: string

## semantics
- `role_scope`: string
- `mission`: string[]
- `must_rules`: string[]
- `must_not_rules`: string[]
- `deliverables`: string[]
- `workflow_steps`: string[]
- `communication_style`: string[]
- `memory_policy`: string[]
- `success_metrics`: string[]

## quality
- `missing_fields`: string[]
- `assumptions`:
  - `field`: string
  - `assumption`: string
  - `impact`: string
- `confidence`: number (0-1)

## artifacts
- `templates`: object[]
  - `title`: string
  - `filename`: string
  - `content`: string
- `domain_tags`: string[]
- `source_role_file`: string

## Readiness Rules
- `ready_for_generation=true` 需要满足：
  - `profile.name` 与 `profile.description` 有值
  - `mission` 至少 1 条
  - `must_rules` 与 `must_not_rules` 都至少 1 条
  - `workflow_steps` 至少 4 条
