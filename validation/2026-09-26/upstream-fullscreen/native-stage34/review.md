# First native fullscreen checkpoint

Source 5d22c3fc0c995196e5c5f492d18191030e27f249 was synchronized to the Apple
Silicon host with all 7,402 tracked Rust files matching. The full CLI/TUI/config/
features run executed 6,250 tests: **6,249 passed, one failed**, seven existing
skips, no retries. Total 698.387 seconds; test execution 359.053 seconds.

The only failure is the Mac-specific paginated-history snapshot: partial and
complete footer rows still expected `esc edit previous` instead of the already
implemented `f3 find` and `esc browse prompts`. Both exact differences were
reviewed. The corrected snapshot is carried in the subsequent recap checkpoint;
final-source native validation remains required. No production fix is implied by
this stale expectation.

The actual Mac SGR mouse/terminal clipboard test and three-process `/tui` restart
test both passed, as did the remaining application, routing, private-input and
rendering checks. This receipt is not described as an all-green native run, a
signed build or a published package. No keychain unlock action is pending.
