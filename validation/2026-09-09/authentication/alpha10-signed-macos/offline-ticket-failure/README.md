The fresh Mac runner passed the strict signature and team checks, but its explicit notarized requirement failed before native execution. This result does not establish whether Apple accepted the owner-reported submission; no submission API was queried.

The next validator requests online ticket lookup while retaining the same notarization requirement. The Apple-authored [codesign(1) manual](https://keith.github.io/xcode-man-pages/codesign.1.html) documents `--check-notarization` as the online lookup option. A passing native result is still required; the failure is preserved unchanged here.
