"""Verify retained full-workspace diagnostics without relabeling failures."""

import re

from airs_release_receipts import evidence_path
from airs_test_release_spec import digest_file, load_json, regular_file, require


def retained(root, record):
    require(
        isinstance(record, dict) and set(record) == {"path", "sha256"},
        "Invalid retained diagnostic reference",
    )
    path = evidence_path(root, record["path"])
    require(digest_file(path) == record["sha256"], "Diagnostic evidence changed")
    return path


def bounded_text(path, limit):
    with regular_file(path, limit=limit) as stream:
        payload = stream.read(limit + 1)
    require(len(payload) <= limit, "Diagnostic output exceeded its read limit")
    return payload.decode("utf-8")


def verify_workspace(path, spec):
    """Require current complete diagnostics and an explicit review of failures."""
    document = load_json(path)
    root = path.parent
    require(
        document.get("schema_version") == 1
        and document.get("scope") == "full-workspace"
        and document.get("source_commit") == spec["source_commit"]
        and document.get("tooling_commit") == spec["tooling_commit"],
        "Current full-workspace source and tooling evidence is required",
    )
    require(
        document.get("status") in ("success", "failure"), "Workspace run is incomplete"
    )
    files = document.get("files")
    require(
        isinstance(files, dict) and set(files) == {"log", "source", "tooling", "scope"},
        "Workspace artifact identities are missing",
    )
    paths = {key: retained(root, value) for key, value in files.items()}
    for key, expected in (
        ("source", spec["source_commit"]),
        ("tooling", spec["tooling_commit"]),
        ("scope", "full-workspace"),
    ):
        require(
            bounded_text(paths[key], 1024).strip() == expected,
            "Workspace artifact identity mismatch",
        )
    log = bounded_text(paths["log"], 128 * 1024 * 1024)
    summaries = list(
        re.finditer(r"Summary \[[^\]\n]+\] (\d+) tests run: ([^\n]+)", log)
    )
    require(len(summaries) == 1, "Expected exactly one complete workspace test summary")
    summary = summaries[0]
    counts = {"tests_run": int(summary[1])}
    for field in ("passed", "failed", "skipped"):
        match = re.search(r"\b(\d+) " + field + r"\b", summary[2])
        counts[field] = int(match[1]) if match else 0
    require(
        counts["tests_run"] > 0
        and counts["passed"] > 0
        and counts["passed"] + counts["failed"] == counts["tests_run"]
        and document.get("counts") == counts
        and all(type(value) is int for value in document["counts"].values()),
        "Workspace summary counts are inconsistent",
    )
    failures = sorted(
        set(
            re.findall(
                r"^\s*(?:TRY \d+ )?FAIL \[[^\]\n]+\] \([^\)\n]+\) (.+)$",
                log[summary.end() :],
                re.MULTILINE,
            )
        )
    )
    require(
        len(failures) == counts["failed"]
        and document.get("failures") == failures
        and document["status"] == ("failure" if failures else "success"),
        "Workspace failures or status disagree with retained output",
    )
    reviews = document.get("failure_reviews")
    require(
        isinstance(reviews, list) and len(reviews) == len(failures),
        "Every workspace failure needs current review",
    )
    reviewed = []
    for row in reviews:
        require(
            isinstance(row, dict)
            and row.get("test") in failures
            and row.get("product_blocking") is False
            and isinstance(row.get("reason"), str)
            and 20 <= len(row["reason"]) <= 2000,
            "Unresolved or unreviewed workspace failure blocks product release",
        )
        retained(root, row.get("evidence"))
        reviewed.append(row["test"])
    require(
        sorted(reviewed) == failures, "Duplicate or missing workspace failure review"
    )
    return {
        "scope": "full-workspace",
        "status": document["status"],
        "source_commit": document["source_commit"],
        "tooling_commit": document["tooling_commit"],
        "counts": counts,
        "failures": failures,
        "failure_reviews": reviews,
        "record_sha256": digest_file(path),
        "files": files,
    }
