# Meaningful question labels in copy source — fullscreen prerequisite

Adapted the label-retention/question rendering portion of `321c50fc2c`. Meaningful first-row labels remain in logical copy source while hanging indentation remains display-only. Unanswered status remains attached to the question across wrapping, with source styles and byte ranges retained. AIRS cyan/dim styles remain unchanged; no unrelated accent palette dependency is imported.

Three additional regressions cover answer labels/internal whitespace/Unicode/source sharing/styles, unanswered suffixes at two widths, and secret answers masked in retained source, visible transcript and raw export. The new completed/interrupted visual snapshot preserves AIRS styles. Initial upstream RGB expectation failure was inspected and accepted as an AIRS adaptation; the initial failed receipt is retained.

Final local question/hyperlink subset:111/111 passed, zero retries. Scoped config/core/TUI lint passed without warnings or fixes. Actual selection/clipboard interaction still depends on the connected fullscreen phase; no standalone feature score or native release claim is made here.
