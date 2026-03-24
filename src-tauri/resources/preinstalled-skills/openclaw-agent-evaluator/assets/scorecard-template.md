# Eval Scorecard
- Agent: {{agent_file}}
- Total Cases: {{total_cases}}
- Passed: {{passed_cases}}
- Failed: {{failed_cases}}
- Pass Rate: {{pass_rate}}

## Severity Failures
- BLOCKER: {{blocker_fail}}
- HIGH: {{high_fail}}
- MEDIUM: {{medium_fail}}

## Regression
- Baseline Pass Rate: {{baseline_pass_rate}}
- Current Pass Rate: {{current_pass_rate}}
- Regressed: {{regressed}}

## Failed Cases
1. {{case_id}} [{{severity}}]
   - Reason: {{reason}}
