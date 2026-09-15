"""Stage the exact alpha.15 binaries explicitly requested for owner testing.

This does not pass lifecycle validation or change the regular promotion gate.
"""
import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path('/home/cdot/.cache/airs-auth-recovery')
SOURCE = '2f5a1ad306d379c23e21b60173b87876a79045c4'
VERSION = '0.1.0-alpha.15'
AUTHORIZATION = 'publish alphas.15 so i can test on my remote mac'
LIMITATIONS = [
    'Production frontend MCP expiry and concurrent renewal validation are incomplete; zero completed expiry cycles.',
    'Graceful sign-in after the preserved 30-minute idle timeout still requires owner acceptance.',
    'Previous native runs passed initial gateway login and eight former SCM tools, then stopped on backend API errors; failed receipts are preserved.',
    'The upstream server now exposes local utility tools; authenticated acceptance against this replacement inventory is pending.',
    'Full workspace validation and independent release review are not claimed.',
]

def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

specs = [
    ('linux', 'Linux', 'x86_64-unknown-linux-musl',
     'linux-final/airs-harness-0.1.0-alpha.15-linux-x86_64-musl',
     '2b2825a80d2013ea03feae06f495f7455eac0201144c519e17ef71c5efaec0bc',
     'production-linux-coordinated/GATEWAY-MCP.json', 'linux-npm-upgrade/UPGRADE.json',
     'LINUX-INSTALLED-ACCEPTANCE.json'),
    ('mac', 'Darwin', 'aarch64-apple-darwin',
     'mac-final/airs-harness-0.1.0-alpha.15-darwin-arm64',
     'ff8a5666c1f9cef6f728bbb9c8e86e1522718e36b2a697a47e46ba85b6d56198',
     'production-mac-coordinated-retry/GATEWAY-MCP.json', 'mac-final/NPM-UPGRADE.json',
     'mac-final/MAC-INSTALLED-DESKTOP.json'),
]
for label, platform, target, release, sha, e2e_path, upgrade_path, installed_path in specs:
    src = ROOT / release
    info = json.loads((src / 'BUILD-INFO.json').read_text())
    assert info['version'] == VERSION and info['source_commit'] == SOURCE
    assert info['target'] == target and info['binary_sha256'] == digest(src / 'airs-harness') == sha
    e2e = json.loads((ROOT / e2e_path).read_text())
    assert e2e['passed'] is False and e2e['binary_sha256'] == sha and e2e['platform'] == platform
    assert e2e['endpoint'] == 'https://mcp.redtail.cdot.io/prisma-airs/mcp'
    assert e2e['routing']['mode'] == 'gateway-proxied-mcp'
    rows = {r['case']: r for r in e2e['results']}
    for case in ['inference_browser_pkce', 'mcp_browser_pkce', 'doctor_after_mcp',
                 'all_read_tools_tool_results', 'inference_history_preserved',
                 'mcp_logout', 'mcp_list_after_logout', 'inference_after_mcp_logout', 'inference_logout']:
        assert rows[case]['passed'] is True
    upgrade = json.loads((ROOT / upgrade_path).read_text())
    assert upgrade['passed'] and upgrade['version'] == VERSION and upgrade['platform'] == platform
    assert upgrade['configuration_preserved'] and upgrade['legacy_target_preserved']
    for case in upgrade['cases']:
        assert case['passed'] and case['binary_sha256'] == sha
        assert not case['uninstall_used'] and not case['manual_command_removal']
        if case['case'] == 'npm-managed-in-place':
            assert not case['force_used']
    installed = json.loads((ROOT / installed_path).read_text())
    assert installed['passed'] and installed['binary_sha256'] == sha and installed['runtime_source'] == SOURCE
    if platform == 'Darwin':
        signing = json.loads((src / 'SIGNING.json').read_text())
        assert digest(src / 'SIGNING.json') == info['signing_receipt_sha256']
        assert signing['binary_sha256'] == sha and signing['source_commit'] == SOURCE
        assert signing['team_id'] == 'G5QLZ5A8TA'
        assert all(signing[k] for k in ['codesign_verified', 'hardened_runtime', 'notarization_verified'])
    out = ROOT / 'owner-test-alpha15' / label
    shutil.copytree(src, out)
    evidence = out / 'validation-evidence'
    evidence.mkdir(exist_ok=True)
    records = []
    for name, path in [('native-mcp-e2e', ROOT / e2e_path), ('npm-upgrade', ROOT / upgrade_path),
                       ('installed-acceptance', ROOT / installed_path), ('original-validation', src / 'VALIDATION.json')]:
        dest = evidence / (name + '.json')
        shutil.copyfile(path, dest)
        records.append({'role': name, 'path': dest.name, 'sha256': digest(dest)})
    validation = dict(schema_version=1, scope='owner-requested-alpha15-testing',
                      passed=False, release_ready=False, publication_authorized=True,
                      authorization=AUTHORIZATION, authorization_date='2026-09-15',
                      product_version=VERSION, source_commit=SOURCE, target=target, binary_sha256=sha,
                      full_gateway_lifecycle_passed=False, frontend_refresh_cycles_completed=0,
                      limitations=LIMITATIONS, evidence=records)
    (out / 'VALIDATION.json').write_text(json.dumps(validation, indent=2) + '\n')
    info.pop('publishable', None)
    info.pop('release_status', None)
    info['release_scope'] = validation['scope']
    info['validation_receipt_sha256'] = digest(out / 'VALIDATION.json')
    (out / 'BUILD-INFO.json').write_text(json.dumps(info, indent=2) + '\n')
    (out / 'SHA256SUMS').write_text(''.join(
        f'{digest(p)}  {p.relative_to(out).as_posix()}\n'
        for p in sorted(out.rglob('*')) if p.is_file() and p.name != 'SHA256SUMS'))
    print(json.dumps({'staged': label, 'scope': validation['scope'], 'lifecycle_passed': False, 'binary_sha256': sha}))
