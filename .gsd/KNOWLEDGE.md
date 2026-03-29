# 项目知识库

项目级规则、模式与经验教训的追加式登记。
每次开始新工作单元前应先阅读。发现可复用规律后及时补充。

## Rules

| # | Scope | Rule | Why | Added |
|---|-------|------|-----|-------|
| K001 | architecture | `src/components/SpriteDesktopPetPage.vue` 与 `src-tauri/src/lib.rs` 属于高风险超大文件；优先采用小范围改动并配套直接验证。 | 两个文件体量大且跨域职责多，回归风险高。 | 2026-03-19 |
| K002 | refactor-strategy | chat-first 重构必须“先立壳层，再迁移模块”，禁止一次性全量替换。 | 渐进迁移可保留可运行主线，降低发布中断风险。 | 2026-03-25 |
| K003 | onboarding | 安装、登录、模型接入需归并在同一 onboarding 流程里统一收口。 | 分散流程会显著提高首用流失与配置错误概率。 | 2026-03-25 |

## Patterns

| # | Pattern | Where | Notes |
|---|---------|-------|-------|
| P001 | 前端服务层 + Tauri 命令桥接 | `src/services/*` + `src-tauri/src/lib.rs` | UI 通过 Tauri 命令执行高权限操作；需保持传输层与命令契约一致。 |
| P002 | 运行态快照聚合可直接支撑敏捷面板与报告页 | `load_staff_snapshot` / `load_task_snapshot` / `load_openclaw_resource_snapshot` | 可优先复用现有命令，减少新建后端接口数量。 |
| P003 | 系统级快捷呼出能力已具备基础 | `tauri_plugin_global_shortcut`（`Ctrl+\``、`Alt+\``） | 可作为“快捷聊天呼出框”实现底座。 |

## Lessons Learned

| # | What Happened | Root Cause | Fix | Scope |
|---|--------------|------------|-----|-------|
| L001 | 控制台能力快速叠加后，主组件职责过度集中。 | 缺少壳层边界与模块化约束。 | 在 M001 将壳层、面板、服务职责拆分并建立统一路由。 | front-end architecture |
