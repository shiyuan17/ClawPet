# Interview Schema for Role Inputs

在生成 agent 前，先按本 schema 采集或抽取字段。

## Required Fields

### A. Frontmatter
- `name`: 角色名称
- `description`: 角色一句话定位
- `color`: 角色主题色（建议 Hex）
- `emoji`: 角色图标
- `vibe`: 角色气质和表达基调

### B. Core Semantics
- 角色定位：这个角色负责什么，不负责什么
- 核心使命：角色最终要优化的结果
- 强制规则：必须遵守与绝对禁止事项
- 交付物：角色应输出的结构化成果
- 工作流程：从输入到输出的步骤
- 沟通风格：表达方式、语气、反馈策略
- 记忆策略：会话内需持续追踪的信息
- 成功指标：可衡量的质量标准

## Extraction from `academic/*.md`

优先按下列路径抽取：
1. Frontmatter 直接读取 A 类字段
2. 标题/段落中识别“身份、使命、规则、流程、指标”等语义块
3. 保留原角色术语，减少同义改写

## Missing-Field Question Order

若字段缺失，按顺序追问：
1. 核心使命和使用场景
2. 必须/禁止规则
3. 交付物格式
4. 风格与边界

每次只问最小必要问题，避免一次性过长问卷。

## Assumption Policy

- 可做最小假设：仅在不影响安全和核心行为时使用
- 必须显式标注：`假设`、`影响范围`、`确认建议`
- 高风险字段（权限、外部执行、敏感数据）禁止假设
