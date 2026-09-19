#!/usr/bin/env python3
"""Run installed fixture tests with structured results; empty suites never pass."""

import argparse
from pathlib import Path
import unittest

from airs_release_receipts import atomic_json
from airs_test_release_spec import require


def test_ids(suite):
    result = []
    for item in suite:
        result.extend(
            test_ids(item) if isinstance(item, unittest.TestSuite) else [item.id()]
        )
    return result


def run_suite(scripts, pattern, receipt):
    suite = unittest.TestLoader().discover(str(scripts), pattern=pattern)
    ids = test_ids(suite)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    summary = {
        "schema_version": 1,
        "pattern": pattern,
        "test_ids": ids,
        "tests_run": result.testsRun,
        "failures": [case.id() for case, _ in result.failures],
        "errors": [case.id() for case, _ in result.errors],
        "skipped": [
            {"test": case.id(), "reason": reason[:512]}
            for case, reason in result.skipped
        ],
        "passed": result.wasSuccessful() and result.testsRun > len(result.skipped),
    }
    atomic_json(receipt, summary)
    require(
        summary["passed"],
        "Installed fixture suite failed, was empty or entirely skipped",
    )
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scripts", type=Path, required=True)
    parser.add_argument("--pattern", required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    run_suite(args.scripts, args.pattern, args.receipt)


if __name__ == "__main__":
    main()
