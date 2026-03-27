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
- Source: inherited
- Primary owning slice: M001-S04
- Supporting slices: M001-S03
- Validation: planned
- Notes: 包含动画/音效反馈与自主状态切换。

### R002 — 桌面 UI 发起 OpenClaw 对话
- Class: core-capability
- Status: active
- Description: 用户必须能够从 UI 发送消息，并通过已配置的 OpenClaw 通道接收模型回复。
- Why it matters: 没有对话能力，产品将退化为纯视觉桌宠。
- Source: inherited
- Primary owning slice: M001-S04
- Supporting slices: M002-S02
- Validation: planned
- Notes: 需覆盖 Tauri invoke 路径与兼容传输兜底。

### R003 — 可配置的提供商/平台路由
- Class: integration
- Status: active
- Description: 用户必须能够管理模型平台/提供商配置，并在 OpenClaw 请求中生效。
- Why it matters: 多平台配置是实际使用中的成本、可用性与策略基础。
- Source: inherited
- Primary owning slice: M002-S02
- Supporting slices: M004-S02
- Validation: planned
- Notes: 包含协议、Base URL、模型、路径等配置项。

### R004 — OpenClaw 运行时引导与维护
- Class: operability
- Status: active
- Description: 应用必须在桌面端提供 OpenClaw 的安装、健康检查、重启与修复流程。
- Why it matters: 降低上手门槛，保障非命令行用户可持续使用。
- Source: inherited
- Primary owning slice: M002-S01
- Supporting slices: M006-S01
- Validation: planned
- Notes: 包含 gateway 检查、安装向导与维护动作。

### R005 — 配置与运行活动可观测性
- Class: failure-visibility
- Status: active
- Description: 用户必须可在 UI 中查看请求日志与运行快照（platform/channel/staff/task/resource）。
- Why it matters: 可排障与可解释性依赖透明的运行态信息。
- Source: inherited
- Primary owning slice: M004-S02
- Supporting slices: M004-S01
- Validation: planned
- Notes: 需提供足够信号定位请求失败与配置错误。

### R006 — 主界面改造为微信式聊天承载结构
- Class: information-architecture
- Status: active
- Description: 主界面必须采用聊天优先布局，功能菜单固定在最左侧并承载核心功能切换。
- Why it matters: 统一入口降低认知成本，提升长期可扩展性。
- Source: user-directive (2026-03-25)
- Primary owning slice: M001-S01
- Supporting slices: M001-S02
- Validation: planned
- Notes: 菜单至少覆盖聊天、仪表盘、员工管理、员工招募、技能市场、任务管理。

### R007 — 安装界面优化
- Class: onboarding-experience
- Status: active
- Description: 安装界面必须重构为更清晰、低负担的步骤化体验，并提供关键状态反馈。
- Why it matters: 首次体验质量直接影响留存与信任。
- Source: user-directive (2026-03-25)
- Primary owning slice: M002-S01
- Supporting slices: M006-S01
- Validation: planned
- Notes: 需要统一文案、步骤和异常提示风格。

### R008 — Codex GPT 自动接入与可自定义替换
- Class: model-access
- Status: active
- Description: 默认流程必须可自动接入 Codex GPT，并支持后续切换到自定义模型提供商。
- Why it matters: 缩短可用路径，同时保留高级用户灵活性。
- Source: user-directive (2026-03-25)
- Primary owning slice: M002-S02
- Supporting slices: M004-S02
- Validation: planned
- Notes: 自动接入需要可回退，不得阻塞手动配置路径。

### R009 — 多渠道登录流程
- Class: authentication
- Status: active
- Description: 需要构建登录流程，支持微信扫码、飞书扫码、支付宝扫码、手机号登录。
- Why it matters: 多渠道登录是企业/个人用户落地前提。
- Source: user-directive (2026-03-25)
- Primary owning slice: M002-S03
- Supporting slices: M002-S01
- Validation: planned
- Notes: 需定义统一登录态模型与失败回退策略。

### R010 — 安装流程简化（参考 PetClaw）
- Class: onboarding-flow
- Status: active
- Description: 安装流程必须简化为最短可用路径，并参考 PetClaw 的低步骤设计。
- Why it matters: 减少中途流失，提升安装成功率。
- Source: user-directive (2026-03-25)
- Primary owning slice: M002-S01
- Supporting slices: M002-S03
- Validation: planned
- Notes: 以“可用优先”而非“配置完整优先”。

### R011 — 交互动画统一体系
- Class: interaction-design
- Status: active
- Description: 增加并统一交互动画规范（特别是按钮和关键操作反馈）。
- Why it matters: 统一反馈提升可理解性与品牌一致性。
- Source: user-directive (2026-03-25)
- Primary owning slice: M001-S03
- Supporting slices: M006-S01
- Validation: planned
- Notes: 需定义动效 token（时长、曲线、层级、触发条件）。

### R012 — 技能安装集成
- Class: skills-platform
- Status: active
- Description: 集成角色安装 skills、安全审查 skills、记忆管理 skills（字节 OpenViking、Turix、操作电脑界面、网站 CLI/OpenCLI）。
- Why it matters: 技能能力决定 agent 可执行任务的上限。
- Source: user-directive (2026-03-25)
- Primary owning slice: M003-S01
- Supporting slices: M003-S03
- Validation: planned
- Notes: 需明确技能来源、权限边界、启用开关与风险提示。

### R013 — 角色安装命令化与测试优化
- Class: install-automation
- Status: active
- Description: 支持通过 `/dragonclaw-agent-skill-install xxx` 按角色信息执行安装，并优化角色安装测试。
- Why it matters: 角色批量装配是规模化交付基础。
- Source: user-directive (2026-03-25)
- Primary owning slice: M003-S02
- Supporting slices: M003-S03
- Validation: planned
- Notes: 需覆盖参数校验、失败重试、幂等与回滚。

### R014 — 敏捷面板
- Class: planning-ops
- Status: active
- Description: 提供敏捷面板展示当前迭代的核心目标、优先级和执行状态。
- Why it matters: 提升团队节奏感与执行透明度。
- Source: user-directive (2026-03-25)
- Primary owning slice: M004-S01
- Supporting slices: M004-S02
- Validation: planned
- Notes: 至少包含任务状态、阻塞项和负责人视图。

### R015 — 可视化报告与去重汇总
- Class: reporting-analytics
- Status: active
- Description: 增加可视化报告页面，对报告信息去重汇总，并可查看任务/agent 的执行与完成状态。
- Why it matters: 决策质量依赖准确、可追踪、无重复噪声的数据。
- Source: user-directive (2026-03-25)
- Primary owning slice: M004-S02
- Supporting slices: M004-S01
- Validation: planned
- Notes: 需覆盖“当前任务、执行中 agent、已完成任务”的统一视图。

### R016 — 快捷呼出框聊天
- Class: quick-access
- Status: active
- Description: 提供快呼弹框进行快捷聊天（参考元宝），支持系统级快速唤起。
- Why it matters: 降低交互摩擦，提升高频使用效率。
- Source: user-directive (2026-03-25)
- Primary owning slice: M005-S01
- Supporting slices: M001-S04
- Validation: planned
- Notes: 需保证与主窗口会话状态可同步。

### R017 — 官网制作
- Class: growth-surface
- Status: active
- Description: 完成官网制作，用于产品介绍、能力展示、下载引导与更新信息发布。
- Why it matters: 官网是外部获客与品牌认知入口。
- Source: user-directive (2026-03-25)
- Primary owning slice: M005-S02
- Supporting slices: none
- Validation: planned
- Notes: 需与桌面端品牌与文案体系一致。

### R021 — 前端控制台超大组件拆分
- Class: quality-attribute
- Status: active
- Description: 将超大主 Vue 组件重构为按功能划分的模块/组合式逻辑。
- Why it matters: 降低回归风险并提升迭代效率。
- Source: reactivated (2026-03-25)
- Primary owning slice: M001-S02
- Supporting slices: M006-S01
- Validation: planned
- Notes: 以“壳层 + 领域面板 + 服务层”分层实施，避免一次性大迁移。

## Validated

（暂无）

## Deferred

### R020 — 后端超大命令模块拆分
- Class: quality-attribute
- Status: deferred
- Description: 将超大 Rust 命令模块按领域拆分为边界清晰的子模块。
- Why it matters: 提升可维护性与变更安全性。
- Source: inherited
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: 前端重构与业务闭环先落地，再切入 Rust 大模块拆分。

## Out of Scope

### R030 — 云端多租户后端服务
- Class: out-of-scope
- Status: out-of-scope
- Description: 集中托管的 SaaS 后端（账号租户、远程共享状态）。
- Why it matters: 防止范围偏移，保持桌面优先架构焦点。
- Source: inherited
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: 当前产品定位是本地桌面 + 本地/自托管 OpenClaw 运行时。

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | primary-user-loop | active | M001-S04 | M001-S03 | planned |
| R002 | core-capability | active | M001-S04 | M002-S02 | planned |
| R003 | integration | active | M002-S02 | M004-S02 | planned |
| R004 | operability | active | M002-S01 | M006-S01 | planned |
| R005 | failure-visibility | active | M004-S02 | M004-S01 | planned |
| R006 | information-architecture | active | M001-S01 | M001-S02 | planned |
| R007 | onboarding-experience | active | M002-S01 | M006-S01 | planned |
| R008 | model-access | active | M002-S02 | M004-S02 | planned |
| R009 | authentication | active | M002-S03 | M002-S01 | planned |
| R010 | onboarding-flow | active | M002-S01 | M002-S03 | planned |
| R011 | interaction-design | active | M001-S03 | M006-S01 | planned |
| R012 | skills-platform | active | M003-S01 | M003-S03 | planned |
| R013 | install-automation | active | M003-S02 | M003-S03 | planned |
| R014 | planning-ops | active | M004-S01 | M004-S02 | planned |
| R015 | reporting-analytics | active | M004-S02 | M004-S01 | planned |
| R016 | quick-access | active | M005-S01 | M001-S04 | planned |
| R017 | growth-surface | active | M005-S02 | none | planned |
| R021 | quality-attribute | active | M001-S02 | M006-S01 | planned |
| R020 | quality-attribute | deferred | none | none | unmapped |
| R030 | out-of-scope | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 18
- Mapped to slices: 18
- Validated: 0
- Unmapped active requirements: 0
