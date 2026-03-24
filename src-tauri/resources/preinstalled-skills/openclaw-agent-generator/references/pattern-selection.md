# Pattern Selection Matrix

本文件用于把角色生成任务映射到可执行的 agent 设计模式。

数据来源（截至 2026-03-20）：
- Google Developers Blog（2026-01）：8 个多智能体模式
- Ofox 文章（2026-03-18）：5 个 Agent Skill 设计模式

## Default Composition

默认组合为：
`Inversion -> Generator -> Reviewer`，并用 `Pipeline` 串联为强约束流程。

## Mapping Table

| 任务场景 | 优先模式（Ofox 5） | 对应多智能体模式（Google 8） | 使用建议 |
|---|---|---|---|
| 角色信息缺失、需求模糊 | Inversion | Coordinator/Dispatcher | 先提问后生成，避免拍脑袋补全 |
| 需要稳定输出同一结构 | Generator | Sequential Pipeline | 用固定模板和字段契约控制一致性 |
| 交付前质量把关 | Reviewer | Generator and Critic | 先产出草稿，再按清单审查并回写修复 |
| 多步骤生成并带门禁 | Pipeline | Sequential Pipeline + Composite | 每步设置 checkpoint，失败即回滚 |
| 多维度并行检查（语气/安全/一致性） | Reviewer + Pipeline | Parallel Fan-Out/Gather | 并行跑子检查，最后汇总 |
| 角色过于复杂，需拆分子任务 | Pipeline | Hierarchical Decomposition | 先拆 domain，再分别生成再汇总 |
| 涉及高风险动作（自动执行外部行为） | Pipeline + Reviewer | Human-in-the-loop | 必须引入人工审批点 |

## Anti-Patterns to Avoid

- 单一 mega-prompt：把所有规则塞进一个系统提示，导致不可维护。
- 无采访直出：输入缺失时直接生成，导致事实漂移和角色失真。
- 无退出条件循环：迭代优化没有停机条件，浪费 token 且不稳定。
- 并行写同一状态键：并行子流程覆盖彼此输出，造成竞态。
- 缺少硬门禁：`BLOCKER` 未清零就交付，后续返工成本更高。
- 子 agent 描述模糊：路由条件不清，Coordinator 随机分配任务。

## Pattern Decision Order

按以下顺序选择：
1. 先判断信息是否完整（决定是否启用 Inversion）
2. 再判断输出是否结构化（决定 Generator 强度）
3. 再判断是否需要质量门禁（决定 Reviewer 级别）
4. 最后决定编排形态（Sequential / Parallel / Hierarchical / HITL）
