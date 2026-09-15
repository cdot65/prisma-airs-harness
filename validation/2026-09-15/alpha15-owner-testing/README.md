# Alpha.15 owner testing publication — September 15, 2026

The owner explicitly requested publication to test on a remote Mac. The exact
frozen alpha.15 Linux x64 and signed Apple Silicon binaries are published under
`gateway-validation` at `https://npm.cdot.io`. `latest` and `alpha` remain alpha.14.

```sh
npm install -g airs-harness@0.1.0-alpha.15 --registry=https://npm.cdot.io
airs-harness --version
```

This is a testing publication, not passed production lifecycle acceptance.
Native package receipts retain `passed: false`, `release_ready: false` and zero
completed frontend expiry cycles. The owner's explicit publication authorization
is recorded separately. Previous failed MCP receipts remain in each native
package. Ordinary upgrade, installed executable and Apple signing evidence binds
the unchanged native hashes; no additional OAuth flow is required to publish.

The previous SCM API denial is out of scope following the owner's removal of
SCM tools. The replacement server exposes eight local utility tools. Active
renewal, idle return and reauthentication still require production acceptance
against that inventory. The 30-minute idle policy remains unchanged.

The staging and publication scripts are specific records of this authorized
release. The normal complete-lifecycle promotion gate is unchanged. The scripts
contain private filesystem paths but no credentials; registry authentication
remains in an external npm configuration. Registry receipts will record fresh
anonymous installations and signature verification independently of live OAuth.
