# Release Policy

## Exit Codes
- `0`: 可发布
- `1`: 存在 BLOCKER
- `2`: 开启 `--strict-high` 且存在 HIGH
- `4`: intake 未就绪（字段缺口）
- `10+`: 子流程运行失败

## Decision Rule
- `review.BLOCKER == 0`
- `eval.fail_counts.BLOCKER == 0`
- 如果 strict-high:
  - `review.HIGH == 0`
  - `eval.fail_counts.HIGH == 0`
