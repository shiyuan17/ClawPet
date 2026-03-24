# 需求契约

本文档是项目能力范围与覆盖状态的显式契约。

用于跟踪：哪些需求处于当前范围、哪些已通过实际工作验证、哪些被有意延期、哪些明确不在范围内。

规范：
- 需求应面向能力，不写成无边界功能清单。
- 需求应原子化、可验证、可用自然语言说明。
- 每条 **Active** 需求都应映射到切片、延期、阻塞（含原因）或移出范围。
- 每条需求应有一个主责任切片，可有辅助切片。
- 研究可以提出需求，但不应默认自动生效为契约。
- “已验证”必须由完成工作和验证证据支持，不能只停留在讨论。

## Active

### R001 — 桌宠核心交互循环
- Class: primary-user-loop
- Status: active
- Description: 应用必须提供可交互桌宠，并对悬停、点击、拖拽、待机行为做出响应。
- Why it matters: 这是产品的核心交互与辨识度来源。
- Source: inferred
- Primary owning slice: none yet
- Supporting slices: none
- Validation: unmapped
- Notes: 包含动画/音效反馈与自主状态切换。

### R002 — 桌面 UI 发起 OpenClaw 对话
- Class: core-capability
- Status: active
- Description: 用户必须能够从 UI 发送消息，并通过已配置的 OpenClaw 通道接收模型回复。
- Why it matters: 没有对话能力，产品将退化为纯视觉桌宠。
- Source: inferred
- Primary owning slice: none yet
- Supporting slices: none
- Validation: unmapped
- Notes: 需覆盖 Tauri invoke 路径与兼容传输兜底。

### R003 — 可配置的提供商/平台路由
- Class: integration
- Status: active
- Description: 用户必须能够管理模型平台/提供商配置，并在 OpenClaw 请求中生效。
- Why it matters: 多平台配置是实际使用中的成本、可用性与策略基础。
- Source: inferred
- Primary owning slice: none yet
- Supporting slices: none
- Validation: unmapped
- Notes: 包含协议、Base URL、模型、路径等配置项。

### R004 — OpenClaw 运行时引导与维护
- Class: operability
- Status: active
- Description: 应用必须在桌面端提供 OpenClaw 的安装、健康检查、重启与修复流程。
- Why it matters: 降低上手门槛，保障非命令行用户可持续使用。
- Source: inferred
- Primary owning slice: none yet
- Supporting slices: none
- Validation: unmapped
- Notes: 包含 gateway 检查、安装向导与维护动作。

### R005 — 配置与运行活动可观测性
- Class: failure-visibility
- Status: active
- Description: 用户必须可在 UI 中查看请求日志与运行快照（platform/channel/staff/task/resource）。
- Why it matters: 可排障与可解释性依赖透明的运行态信息。
- Source: inferred
- Primary owning slice: none yet
- Supporting slices: none
- Validation: unmapped
- Notes: 需提供足够信号定位请求失败与配置错误。

## Validated

（暂无）

## Deferred

### R020 — 后端超大命令模块拆分
- Class: quality-attribute
- Status: deferred
- Description: 将超大 Rust 命令模块按领域拆分为边界清晰的子模块。
- Why it matters: 提升可维护性与变更安全性。
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: 延期到功能稳定性工作明确后推进。

### R021 — 前端控制台超大组件拆分
- Class: quality-attribute
- Status: deferred
- Description: 将超大主 Vue 组件重构为按功能划分的模块/组合式逻辑。
- Why it matters: 降低回归风险并提升迭代效率。
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: 为避免基线期高风险抖动，暂缓实施。

## Out of Scope

### R030 — 云端多租户后端服务
- Class: out-of-scope
- Status: out-of-scope
- Description: 集中托管的 SaaS 后端（账号租户、远程共享状态）。
- Why it matters: 防止范围偏移，保持桌面优先架构焦点。
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: 当前产品定位是本地桌面 + 本地/自托管 OpenClaw 运行时。

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | primary-user-loop | active | none yet | none | unmapped |
| R002 | core-capability | active | none yet | none | unmapped |
| R003 | integration | active | none yet | none | unmapped |
| R004 | operability | active | none yet | none | unmapped |
| R005 | failure-visibility | active | none yet | none | unmapped |
| R020 | quality-attribute | deferred | none | none | unmapped |
| R021 | quality-attribute | deferred | none | none | unmapped |
| R030 | out-of-scope | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 5
- Mapped to slices: 0
- Validated: 0
- Unmapped active requirements: 5
