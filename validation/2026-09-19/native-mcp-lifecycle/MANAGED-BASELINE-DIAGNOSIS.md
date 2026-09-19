# Independent diagnosis: managed CLI baseline failure

The original `BASELINE-INSTALLED.json` remains an honest non-green run: 48 tests executed, 46 passed, one failed, one macOS-specific skip. The failing test was enabled with `AIRS_MANAGED_CLI_ACCEPTANCE=1` while the selected executable was the raw native package binary.

`npm/airs-harness/lib/launcher.js::managedEnvironment` supplies `AIRS_MANAGED_CLI` and adds the managed directory to PATH. The Rust binary alone does not implement the npm bundle setup. The test executes `"$AIRS_MANAGED_CLI" --version`, so the raw-native context is missing a declared test prerequisite. This is not evidence that the published npm launcher is broken.

The unchanged test was rerun in two isolated, otherwise equivalent contexts, using the same installed package prefix. Direct native failed again with empty `managed-version.txt`; `prefix/bin/airs` passed and verified bundled CLI 7.0.0, managed doctor JSON, missing-credential results, dotenv isolation and all eight embedded skill names/body. No expectation, source, package or test was changed. There was no additional managed environment injection beyond what the actual npm launcher normally performs.

Evidence: `MANAGED-BASELINE-REPRODUCTION.json`, `managed-baseline-direct-native.log`, `managed-baseline-installed-launcher.log`; reproduction script `../reproduce-managed-baseline.py`. Native SHA256 is 774c47315cacb3836ad6461d061974f3ed32e7a7762dba293b1dd9e3de154222. Launcher entrypoint SHA256 is 255a9e3dbe5eaaa9a06ce941decaca6571b5ebc18e6876fa9acdf9343d869ad5.

Report 46 passing original native cases plus the corrected launcher case and one platform skip, retaining the original failed run. Do not rewrite the first suite receipt as green or imply this establishes real-account, owner Ubuntu, or production gateway acceptance. Future whole npm-installed runs should use `prefix/bin/airs`, as the reusable release acceptance runner already does; native-only runs should not enable the npm-managed prerequisite flag.

Follow-up: root requested one complete correct-context run for a self-contained baseline. `BASELINE-INSTALLED-LAUNCHER.json` and `baseline-installed-launcher.log` record all48tests:47passed, one macOSSeatbelt skip, zero failures/errors. The original non-green receipt remains preserved.
