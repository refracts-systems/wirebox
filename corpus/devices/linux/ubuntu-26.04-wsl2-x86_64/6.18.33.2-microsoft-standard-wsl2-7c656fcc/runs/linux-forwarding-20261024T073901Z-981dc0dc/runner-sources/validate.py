"""Validate corpus schema, content hashes, scope, and replayed comparisons.

Uses the distro-provided jsonschema package. Validation of evidence is distinct
from the claim that the underlying comparison passed.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import sys

from jsonschema import Draft202012Validator, FormatChecker
from compare import compare_case


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def safe_path(base, relative):
    path = PurePosixPath(relative)
    if not relative or "\\" in relative or path.is_absolute() or ".." in path.parts or ":" in relative:
        raise ValueError(f"unsafe artifact path: {relative}")
    resolved = (base / relative).resolve()
    if not resolved.is_relative_to(base.resolve()):
        raise ValueError(f"artifact escapes its root: {relative}")
    return resolved


def require(ok, message):
    if not ok:
        raise ValueError(message)


def check_hash(base, item):
    path = safe_path(base, item["path"])
    require(path.is_file(), f"missing artifact: {item['path']}")
    require(hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"], f"hash mismatch: {item['path']}")
    if "bytes" in item:
        require(path.stat().st_size == item["bytes"], f"size mismatch: {item['path']}")
    return path


def schema(repo, kind, value):
    definition = read(repo / f"corpus/schema/{kind}-v1.schema.json")
    Draft202012Validator.check_schema(definition)
    Draft202012Validator(definition, format_checker=FormatChecker()).validate(value)


def validate_record(repo, record_path, require_pass=False):
    folder = record_path.parent
    record = read(record_path)
    schema(repo, "record", record)
    require(folder.name == record["record_id"], "record id and directory differ")
    target = read(check_hash(repo, record["target"]))
    scenario = read(check_hash(repo, record["scenario"]))
    schema(repo, "target", target)
    schema(repo, "scenario", scenario)
    require(record["target_id"] == target["target_id"], "target identity mismatch")
    require(record["scenario_id"] == scenario["scenario_id"], "scenario identity mismatch")
    require(read(folder / "scenario.json") == scenario, "archived scenario differs")
    require(set(scenario["cases"]) == set(scenario["expected"]), "scenario cases and expectations differ")
    require(record["limits"] == scenario["comparison_policy"]["excluded"], "scope exclusions changed")
    paths = [item["path"] for item in record["artifacts"]]
    require(len(paths) == len(set(paths)), "duplicate artifact path")
    actual = {p.relative_to(folder).as_posix() for p in folder.rglob("*") if p.is_file() and p != record_path}
    require(set(paths) == actual, "unindexed or missing run files")
    for item in record["artifacts"]:
        check_hash(folder, item)
    for item in record["runner_sources"]:
        check_hash(folder, item)
        require(item["path"] in paths, "runner source not indexed")
    comparison = read(safe_path(folder, record["comparison"]))
    schema(repo, "comparison", comparison)
    require(comparison["scenario_id"] == record["scenario_id"], "comparison scenario mismatch")
    expected_pairs = {(case, rep) for case in scenario["cases"] for rep in range(1, record["repetitions"] + 1)}
    pairs = [(c["case"], c["repetition"]) for c in comparison["comparisons"]]
    require(set(pairs) == expected_pairs and len(pairs) == len(expected_pairs), "missing or duplicate case repetitions")
    required_checks = set(scenario["comparison_policy"]["required"]) | {"fixture_outcome"}
    for result in comparison["comparisons"]:
        names = [c["name"] for c in result["checks"]]
        require(set(names) == required_checks and len(names) == len(required_checks), "missing or duplicate invariant check")
        case_dir = folder / f"{result['case']}-r{result['repetition']}"
        captures, stats = [], []
        for role in ("source", "router"):
            rows = [json.loads(line) for line in (case_dir / f"{role}.jsonl").read_text().splitlines()]
            require(all(r["role"] == role for r in rows), "capture role mismatch")
            captures.extend(rows)
            stats.append(read(case_dir / f"{role}.stats.json"))
        cleanup = read(case_dir / "cleanup.json")
        require(cleanup == {"complete": True, "failed_namespaces": []}, "namespace cleanup incomplete")
        rebuilt = compare_case(result["case"], read(folder / f"{result['case']}.json"), captures, stats, scenario)
        rebuilt["repetition"] = result["repetition"]
        require(rebuilt == result, "comparison does not reproduce from captured artifacts")
    status = "pass" if all(r["status"] == "pass" for r in comparison["comparisons"]) else "fail"
    require(record["status"] == comparison["status"] == status, "false aggregate result")
    if require_pass:
        require(status == "pass", "comparison has known mismatches")
    return {"record_id": record["record_id"], "integrity": "pass", "comparison": status, "cases": len(pairs)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--record", type=Path)
    parser.add_argument("--require-pass", action="store_true")
    args = parser.parse_args()
    repo = args.repo.resolve()
    records = [args.record.resolve()] if args.record else sorted((repo / "corpus/devices").rglob("record.json"))
    require(bool(records), "no corpus records")
    results = [validate_record(repo, path, args.require_pass) for path in records]
    incomplete = [p.relative_to(repo).as_posix() for p in (repo / "corpus/devices").rglob("failure.json") if not (p.parent / "record.json").exists()]
    print(json.dumps({"records": results, "incomplete_attempts": incomplete}, indent=2))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"corpus validation failed: {error}", file=sys.stderr)
        sys.exit(1)
