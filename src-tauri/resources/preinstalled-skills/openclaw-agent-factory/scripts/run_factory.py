#!/usr/bin/env python3
"""One-shot orchestrator for OpenClaw agent creation pipeline."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def run_cmd(cmd: list[str]) -> dict:
    proc = subprocess.run(cmd, capture_output=True, text=True)
    return {
        "cmd": cmd,
        "returncode": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
    }


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Run OpenClaw Agent Factory pipeline")
    parser.add_argument("--role-md", help="Input role markdown file path")
    parser.add_argument("--role-text", help="Inline role markdown text")
    parser.add_argument("--out", required=True, help="Output agent markdown")
    parser.add_argument("--workspace-root", help="Output workspace root for BOOTSTRAP/HEARTBEAT/TOOLS/skills/templates")
    parser.add_argument("--cases", help="Eval cases jsonl path")
    parser.add_argument("--tmp-dir", help="Temp working directory")
    parser.add_argument("--summary-out", help="Write summary JSON")
    parser.add_argument("--allow-incomplete", action="store_true", help="Allow generation even if intake says not ready")
    parser.add_argument("--strict-high", action="store_true", help="Fail release if HIGH findings exist")
    args = parser.parse_args()
    if (not args.role_md) and (not args.role_text):
        parser.error("one of --role-md or --role-text is required")
    if args.role_md and args.role_text:
        parser.error("--role-md and --role-text are mutually exclusive")

    workspace = Path(__file__).resolve().parents[2]
    out_md = Path(args.out)
    workspace_root = Path(args.workspace_root) if args.workspace_root else out_md.parent

    intake_script = workspace / "openclaw-agent-intake/scripts/extract_role_spec.py"
    gen_script = workspace / "openclaw-agent-generator/scripts/generate_agent_from_spec.py"
    validate_script = workspace / "openclaw-agent-generator/scripts/validate_openclaw_agent.py"
    review_script = workspace / "openclaw-agent-reviewer/scripts/review_openclaw_agent.py"
    eval_script = workspace / "openclaw-agent-evaluator/scripts/run_eval_suite.py"
    eval_cases = Path(args.cases) if args.cases else workspace / "openclaw-agent-evaluator/assets/eval-cases.sample.jsonl"

    managed_tmp = None
    if args.tmp_dir:
        tmp_dir = Path(args.tmp_dir)
        tmp_dir.mkdir(parents=True, exist_ok=True)
    else:
        managed_tmp = tempfile.TemporaryDirectory(prefix="oc_factory_")
        tmp_dir = Path(managed_tmp.name)

    if args.role_text:
        role_md = tmp_dir / "inline_role.md"
        role_md.write_text(args.role_text, encoding="utf-8")
    else:
        role_md = Path(args.role_md)

    role_spec_path = tmp_dir / "role_spec.json"
    review_path = tmp_dir / "review.json"
    eval_path = tmp_dir / "eval.json"

    summary = {
        "role_md": str(role_md),
        "role_source": "inline-text" if args.role_text else "file",
        "agent_out": str(out_md),
        "workspace_root": str(workspace_root),
        "release": False,
        "exit_code": 1,
        "steps": {},
        "intake": {},
        "review": {},
        "eval": {},
    }

    step = run_cmd([sys.executable, str(intake_script), str(role_md), "--out", str(role_spec_path)])
    summary["steps"]["intake"] = step
    if step["returncode"] != 0:
        summary["exit_code"] = 10
        if args.summary_out:
            Path(args.summary_out).write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 10

    role_spec = load_json(role_spec_path)
    summary["intake"] = {
        "ready_for_generation": role_spec.get("ready_for_generation", False),
        "missing_fields": role_spec.get("quality", {}).get("missing_fields", []),
        "confidence": role_spec.get("quality", {}).get("confidence"),
    }

    if (not role_spec.get("ready_for_generation", False)) and (not args.allow_incomplete):
        summary["exit_code"] = 4
        if args.summary_out:
            Path(args.summary_out).write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 4

    out_md.parent.mkdir(parents=True, exist_ok=True)
    step = run_cmd(
        [
            sys.executable,
            str(gen_script),
            "--role-spec",
            str(role_spec_path),
            "--out",
            str(out_md),
            "--workspace-root",
            str(workspace_root),
        ]
    )
    summary["steps"]["generator"] = step
    if step["returncode"] != 0:
        summary["exit_code"] = 11
        if args.summary_out:
            Path(args.summary_out).write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 11

    step = run_cmd(
        [
            sys.executable,
            str(validate_script),
            str(out_md),
            "--strict",
            "--workspace-root",
            str(workspace_root),
            "--require-workspace",
        ]
    )
    summary["steps"]["preflight_validate"] = step

    step = run_cmd(
        [
            sys.executable,
            str(review_script),
            str(out_md),
            "--out",
            str(review_path),
            "--workspace-root",
            str(workspace_root),
            "--require-workspace",
        ]
    )
    summary["steps"]["reviewer"] = step
    if step["returncode"] not in (0, 1, 2):
        summary["exit_code"] = 12
        if args.summary_out:
            Path(args.summary_out).write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 12

    eval_target = workspace_root / "AGENTS.md"
    if not eval_target.exists():
        eval_target = out_md

    eval_cmd = [sys.executable, str(eval_script), "--agent", str(eval_target), "--cases", str(eval_cases), "--out", str(eval_path)]
    if args.strict_high:
        eval_cmd.append("--fail-on-high")
    step = run_cmd(eval_cmd)
    summary["steps"]["evaluator"] = step
    if step["returncode"] not in (0, 1, 2, 3):
        summary["exit_code"] = 13
        if args.summary_out:
            Path(args.summary_out).write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 13

    review = load_json(review_path)
    ev = load_json(eval_path)
    summary["review"] = review.get("counts", {})
    summary["eval"] = {
        "pass_rate": ev.get("summary", {}).get("pass_rate"),
        "fail_counts": ev.get("summary", {}).get("fail_counts", {}),
    }

    review_blocker = summary["review"].get("BLOCKER", 0)
    review_high = summary["review"].get("HIGH", 0)
    eval_blocker = summary["eval"]["fail_counts"].get("BLOCKER", 0)
    eval_high = summary["eval"]["fail_counts"].get("HIGH", 0)

    exit_code = 0
    if review_blocker > 0 or eval_blocker > 0:
        exit_code = 1
    elif args.strict_high and (review_high > 0 or eval_high > 0):
        exit_code = 2

    summary["exit_code"] = exit_code
    summary["release"] = exit_code == 0

    if args.summary_out:
        Path(args.summary_out).write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print(json.dumps(summary, ensure_ascii=False, indent=2))

    if managed_tmp is not None:
        managed_tmp.cleanup()
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
