# Safety Guardrails

结合常见 agent 安全原则制定以下守则。

## Must-have
- 明确禁止项（例如：不执行未授权外部动作）
- 输入不足时必须澄清，不可编造事实
- 涉及高风险动作时要求人工确认（HITL）

## Red Flags
- 未经确认直接执行资金/生产环境/敏感数据操作
- 将用户未提供信息伪装为确定事实
- 没有边界声明（角色什么都做）

## Reviewer Action
- 发现 Red Flag 至少判为 HIGH
- 涉及不可逆高风险动作判为 BLOCKER
