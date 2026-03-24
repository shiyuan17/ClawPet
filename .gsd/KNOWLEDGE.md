# 项目知识库

项目级规则、模式与经验教训的追加式登记。
每次开始新工作单元前应先阅读。发现可复用规律后及时补充。

## Rules

| # | Scope | Rule | Why | Added |
|---|-------|------|-----|-------|
| K001 | architecture | `src/components/SpriteDesktopPetPage.vue` 与 `src-tauri/src/lib.rs` 属于高风险超大文件；优先采用小范围改动并配套直接验证。 | 两个文件体量大且跨域职责多，回归风险高。 | 2026-03-19 |

## Patterns

| # | Pattern | Where | Notes |
|---|---------|-------|-------|
| P001 | 前端服务层 + Tauri 命令桥接 | `src/services/*` + `src-tauri/src/lib.rs` | UI 通过 Tauri 命令执行高权限操作；需保持传输层与命令契约一致。 |

## Lessons Learned

| # | What Happened | Root Cause | Fix | Scope |
|---|--------------|------------|-----|-------|
