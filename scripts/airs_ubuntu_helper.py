"""The Ubuntu preparation helper must name the launcher it ships with, exactly once."""

import re

HELPER_MEMBER = "package/scripts/prepare_airs_ubuntu.sh"
MAX_HELPER = 64 * 1024
PACKAGE_LINE = re.compile(r"^package=(\S+)$", re.MULTILINE)
VERSION_LINE = re.compile(r"^version=\$\{AIRS_TEST_VERSION:-(\S+)\}$", re.MULTILINE)
# Every installed path and registry request must derive from the one declaration.
DERIVED_USES = (
    '"$package@$version"',
    '"$prefix/lib/node_modules/$package/package.json"',
    '"$registry/$package_url/$version"',
)
LITERAL_PATH = re.compile(r"node_modules/(?!\$package/)[^\s\"']+")


def verify_ubuntu_helper(text, launcher, version):
    """Raise ValueError unless the helper installs, checks and fetches exactly ``launcher``
    at ``version`` through its single package declaration."""
    if isinstance(text, bytes):
        text = text.decode("utf-8", errors="strict")
    if len(text) > MAX_HELPER:
        raise ValueError("Ubuntu helper exceeds size limit")
    declared = PACKAGE_LINE.findall(text)
    if declared != [launcher]:
        raise ValueError("Ubuntu helper must declare the shipped launcher package once")
    defaults = VERSION_LINE.findall(text)
    if defaults != [version]:
        raise ValueError("Ubuntu helper default version must match the release")
    for use in DERIVED_USES:
        if text.count(use) != 1:
            raise ValueError(
                "Ubuntu helper must derive every package path from $package"
            )
    without_declaration = PACKAGE_LINE.sub("", text)
    bare = launcher.split("/")[-1]
    if bare in without_declaration:
        raise ValueError("Ubuntu helper names the launcher outside its declaration")
    if LITERAL_PATH.search(text):
        raise ValueError("Ubuntu helper contains a literal node_modules path")
    return {"launcher": launcher, "version": version, "derived_uses": len(DERIVED_USES)}
