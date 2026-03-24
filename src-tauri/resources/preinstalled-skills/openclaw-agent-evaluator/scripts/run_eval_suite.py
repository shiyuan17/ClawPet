#!/usr/bin/env python3
"""Run a regex-based regression eval suite for OpenClaw agent files."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path


@dataclass
class EvalCase:
    id: str
    severity: str
    must_contain: list[str]
    must_not_contain: list[str]
    note: str


def load_cases(path: Path) -> list[EvalCase]:
    cases: list[EvalCase] = []
    for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        obj = json.loads(line)
        cases.append(
            EvalCase(
                id=obj["id"],
                severity=obj.get("severity", "MEDIUM"),
                must_contain=obj.get("must_contain", []),
                must_not_contain=obj.get("must_not_contain", []),
                note=obj.get("note", f"line:{i}"),
            )
        )
    return cases


def evaluate_case(text: str, case: EvalCase) -> tuple[bool, str]:
    for pattern in case.must_contain:
        if not re.search(pattern, text, re.MULTILINE):
            return False, f"missing required pattern: {pattern}"
    for pattern in case.must_not_contain:
        if re.search(pattern, text, re.MULTILINE):
            return False, f"matched forbidden pattern: {pattern}"
    return True, "pass"


def run_eval(agent_text: str, cases: list[EvalCase]) -> dict:
    results = []
    pass_count = 0

    fail_counts = {"BLOCKER": 0, "HIGH": 0, "MEDIUM": 0}

    for case in cases:
        ok, reason = evaluate_case(agent_text, case)
        if ok:
            pass_count += 1
        else:
            sev = case.severity if case.severity in fail_counts else "MEDIUM"
            fail_counts[sev] += 1
        results.append(
            {
                "id": case.id,
                "severity": case.severity,
                "pass": ok,
                "reason": reason,
                "note": case.note,
            }
        )

    total = len(cases)
    failed = total - pass_count
    pass_rate = round(pass_count / total, 4) if total else 0.0
    releasable = fail_counts["BLOCKER"] == 0

    return {
        "summary": {
            "total_cases": total,
            "passed_cases": pass_count,
            "failed_cases": failed,
            "pass_rate": pass_rate,
            "fail_counts": fail_counts,
            "releasable": releasable,
        },
        "results": results,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Run eval suite against an OpenClaw agent markdown")
    parser.add_argument("--agent", required=True, help="Path to agent markdown")
    parser.add_argument("--cases", required=True, help="Path to eval cases jsonl")
    parser.add_argument("--baseline", help="Previous eval report JSON for regression comparison")
    parser.add_argument("--out", help="Write eval report JSON path")
    parser.add_argument("--fail-on-high", action="store_true", help="Return non-zero if HIGH failures exist")
    args = parser.parse_args()

    agent_text = Path(args.agent).read_text(encoding="utf-8")
    cases = load_cases(Path(args.cases))
    report = run_eval(agent_text, cases)

    regression = None
    baseline_rate = None
    if args.baseline:
        baseline = json.loads(Path(args.baseline).read_text(encoding="utf-8"))
        baseline_rate = baseline.get("summary", {}).get("pass_rate")
        current = report["summary"]["pass_rate"]
        if isinstance(baseline_rate, (int, float)):
            regression = current < baseline_rate

    report["agent_file"] = args.agent
    report["cases_file"] = args.cases
    report["baseline_pass_rate"] = baseline_rate
    report["regression"] = regression

    if args.out:
        Path(args.out).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"Wrote {args.out}")
    else:
        print(json.dumps(report, ensure_ascii=False, indent=2))

    fail_counts = report["summary"]["fail_counts"]
    if fail_counts["BLOCKER"] > 0:
        return 1
    if args.fail_on_high and fail_counts["HIGH"] > 0:
        return 2
    if regression is True:
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
