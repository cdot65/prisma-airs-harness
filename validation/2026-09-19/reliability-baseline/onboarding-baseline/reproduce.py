#!/usr/bin/env python3
"""Isolated published-package reproduction; run inside fresh dbus-run-session."""
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile
import time

REPO = Path('/home/cdot/development/cdot65/airs-session-doctor')
sys.path.insert(0, str(REPO / 'scripts'))
from airs_onboarding_fixture import IdentityFixture
from validate_airs_onboarding_preview import Preview

BINARY = Path('/tmp/airs-doctor-publish-20260918/linux-registry/prefix/bin/airs')
NATIVE = BINARY.parent.parent / 'lib/node_modules/airs-harness/node_modules/airs-harness-linux-x64/bin/airs-harness'
OUT = Path(__file__).resolve().parent

with tempfile.TemporaryDirectory(prefix='airs-onboarding-baseline-') as temporary:
    root = Path(temporary)
    fixture = IdentityFixture(root)
    fixture.gateway_status = 403
    env = {key: value for key, value in os.environ.items()
           if not key.startswith(('AIRS_', 'OPENAI_')) and key not in
           ('CODEX_HOME', 'CODEX_SQLITE_HOME', 'CODEX_CA_CERTIFICATE', 'NO_COLOR')}
    env.update(AIRS_HARNESS_HOME=str(root / 'harness'),
               XDG_DATA_HOME=str(root / 'data'), XDG_CONFIG_HOME=str(root / 'config'),
               GNOME_KEYRING_CONTROL=str(root / 'keyring'),
               SSL_CERT_FILE=str(fixture.certificate))
    for key in ('AIRS_HARNESS_HOME', 'XDG_DATA_HOME', 'XDG_CONFIG_HOME', 'GNOME_KEYRING_CONTROL'):
        Path(env[key]).mkdir(mode=0o700)
    owner_command = ['dbus-send', '--session', '--print-reply', '--dest=org.freedesktop.DBus',
                     '/org/freedesktop/DBus', 'org.freedesktop.DBus.NameHasOwner',
                     'string:org.freedesktop.secrets']
    owner = subprocess.run(owner_command, env=env, capture_output=True, text=True, check=True)
    assert 'boolean false' in owner.stdout, 'Requires an empty private D-Bus session'
    subprocess.run(['dbus-update-activation-environment', 'XDG_DATA_HOME',
                    'XDG_CONFIG_HOME', 'GNOME_KEYRING_CONTROL'], env=env, check=True,
                   capture_output=True)
    with (root / 'keyring.log').open('wb') as log:
        daemon = subprocess.Popen(['gnome-keyring-daemon', '--foreground', '--unlock',
                                   '--components=secrets', '--control-directory', env['GNOME_KEYRING_CONTROL']],
                                  env=env, stdin=subprocess.PIPE, stdout=log, stderr=log)
        daemon.stdin.write(secrets.token_urlsafe(32).encode())
        daemon.stdin.close()
    terminal = None
    try:
        for _ in range(100):
            owner = subprocess.run(owner_command, env=env, capture_output=True, text=True, check=True)
            if 'boolean true' in owner.stdout:
                break
            time.sleep(0.05)
        assert 'boolean true' in owner.stdout
        def cli(*arguments):
            return subprocess.run([str(BINARY), *arguments], env=env, cwd=root,
                                  capture_output=True, text=True, check=True, timeout=30).stdout
        for name in ('work', 'staging'):
            cli('env', 'create', name, '--gateway-url', fixture.issuer + '/v1')
        cli('env', 'use', 'work')
        registry_path = Path(env['AIRS_HARNESS_HOME']) / 'environments.json'
        before = registry_path.read_bytes()
        registry = json.loads(before)
        homes = {name: registry_path.parent / 'environments' / value['id']
                 for name, value in registry['environments'].items()}
        (homes['work'] / 'history.jsonl').write_text('preserved fixture history\n')
        terminal = Preview(BINARY, arguments=['--environment', 'staging', 'login'],
                           environment=env, directory=root, columns=100, rows=40)
        terminal.expect('Sign in to continue')
        terminal.send(b'2')
        terminal.expect('Workspace API key (input hidden')
        key = b'synthetic-onboarding-baseline-key'
        fixture.access_tokens.add(key.decode())
        terminal.send(b'\x1b[200~' + key + b'\x1b[201~\r')
        terminal.expect('gateway access needs attention', timeout=20)
        screen = '\n'.join(terminal.screen.display)
        compact = ' '.join(screen.split())
        assert 'Environment staging' in compact
        assert 'HTTP 403' in compact
        assert 'Retry: airs doctor --verify-access (select the same environment).' in compact
        assert 'airs --environment staging doctor' not in compact
        assert key not in terminal.transcript
        (OUT / 'rejected-probe-screen.txt').write_text(screen + '\n')
        terminal.send(b'\x1b')
        terminal.finish('', status=1)
        terminal_text = '\n'.join(terminal.screen.display)
        assert 'Start AIRS when ready, or inspect access with airs doctor --verify-access.' in ' '.join(terminal_text.split())
        (OUT / 'cancelled-after-save-screen.txt').write_text(terminal_text + '\n')
        assert registry_path.read_bytes() == before
        assert (homes['staging'] / 'credential-binding.json').exists()
        assert not (homes['work'] / 'credential-binding.json').exists()
        assert (homes['work'] / 'history.jsonl').read_text() == 'preserved fixture history\n'
        (OUT / 'RESULT.json').write_text(json.dumps({
            'reproduced': True,
            'version': cli('--version').strip(),
            'binary': str(BINARY), 'native_binary': str(NATIVE),
            'native_sha256': hashlib.file_digest(NATIVE.open('rb'), 'sha256').hexdigest(),
            'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=REPO, text=True).strip(),
            'selected_environment': 'staging', 'saved_default': 'work',
            'gateway_http_status': 403,
            'displayed_retry': 'airs doctor --verify-access (select the same environment)',
            'displayed_exit': 'Start AIRS when ready, or inspect access with airs doctor --verify-access.',
            'registry_and_default_preserved': True,
            'default_history_preserved': True,
            'credential_saved_only_in_selected_environment': True,
            'terminal_restored': True,
            'secret_not_echoed': True,
            'production_access': False,
            'node18_execution': 'Not performed: no locally available Node18 located; installed Node is v22.23.2.'
        }, indent=2) + '\n')
        print('Reproduced: rejected explicit staging login recommends unscoped doctor while saved default stays work.')
    finally:
        if terminal:
            terminal.close()
        fixture.close()
        daemon.terminate()
        daemon.wait(timeout=10)
