"""Small, lossless overrides for disposable AIRS test configuration only."""

from pathlib import Path
import re
import tomllib


def set_mcp_store(path: Path, mode: str) -> None:
    if mode not in {"file", "auto", "keyring"}:
        raise ValueError("Unsupported fixture MCP credential store")
    original = path.read_text()
    before = tomllib.loads(original)
    key = "mcp_oauth_credentials_store"
    assignment = re.compile(
        r'(?m)^(?P<prefix>[ \t]*(?:mcp_oauth_credentials_store|"mcp_oauth_credentials_store"|\'mcp_oauth_credentials_store\')[ \t]*=[ \t]*)'
        r'(?P<value>"(?:file|auto|keyring)"|\'(?:file|auto|keyring)\')'
        r"(?P<suffix>[ \t]*(?:#[^\n]*)?)$"
    )
    # Generated AIRS root settings precede tables. Refuse unfamiliar layouts;
    # never try to rewrite a user's general-purpose TOML document.
    table = re.search(r"(?m)^[ \t]*\[", original)
    boundary = table.start() if table else len(original)
    candidates = list(assignment.finditer(original, 0, boundary))
    if key in before:
        if len(candidates) != 1 or before[key] not in ("file", "auto", "keyring"):
            raise ValueError("Ambiguous fixture MCP credential store assignment")
        match = candidates[0]
        updated = (
            original[: match.start("value")]
            + f'"{mode}"'
            + original[match.end("value") :]
        )
    else:
        if candidates:
            raise ValueError("Fixture assignment is not a top-level setting")
        updated = f'{key} = "{mode}"\n' + original
    expected = dict(before, **{key: mode})
    if tomllib.loads(updated) != expected:
        raise ValueError("Fixture override would alter unrelated configuration")
    path.write_text(updated)
