"""Exact released-version parsing and historical AIRS command compatibility."""

from dataclasses import dataclass
import re

NUMBER = r"(?:0|[1-9]\d*)"
EXACT = re.compile(
    rf"(?P<major>{NUMBER})\.(?P<minor>{NUMBER})\.(?P<patch>{NUMBER})"
    rf"(?:-alpha\.(?P<alpha>{NUMBER})(?:\.(?:onboarding|mcp)\.{NUMBER})?)?\Z"
)


@dataclass(frozen=True)
class ReleasedVersion:
    value: str
    alpha: int | None
    base: tuple[int, int, int]

    @property
    def command(self):
        return (
            "airs-harness"
            if self.base == (0, 1, 0) and self.alpha is not None and self.alpha < 22
            else "airs"
        )

    @property
    def setup(self):
        return (
            ["setup"]
            if self.base == (0, 1, 0) and self.alpha is not None and self.alpha < 21
            else ["env", "create", "work"]
        )

    @property
    def stable(self):
        return self.alpha is None


def released_version(value):
    matched = EXACT.fullmatch(value) if isinstance(value, str) else None
    if matched is None:
        raise ValueError("Expected an explicit immutable stable or alpha version")
    alpha = matched["alpha"]
    return ReleasedVersion(
        value,
        int(alpha) if alpha is not None else None,
        tuple(int(matched[key]) for key in ("major", "minor", "patch")),
    )
