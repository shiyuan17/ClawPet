# Pipeline Architecture

主 skill 负责编排，子 skill 负责专业处理：

1. Intake
- 输入：角色 Markdown
- 输出：`role_spec.json`

2. Generator
- 输入：`role_spec.json`
- 输出：`agent.md`（草稿）

3. Reviewer
- 输入：`agent.md`
- 输出：`review.json`

4. Evaluator
- 输入：`agent.md` + `cases.jsonl`
- 输出：`eval.json`

5. Factory Summary
- 聚合上面所有结果
- 输出 release 结论
