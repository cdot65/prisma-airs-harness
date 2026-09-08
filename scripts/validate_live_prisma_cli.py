#!/usr/bin/env python3
"""Ask the actual gateway-backed agent to use its embedded Prisma AIRS CLI skill.

Only inference uses real credentials. The CLI doctor receives an empty trusted
fixture config so missing scanner/management credentials can be checked safely.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--gateway-url', required=True)
    parser.add_argument('--credential-file', type=Path, required=True)
    parser.add_argument('--output-directory', type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    root = args.output_directory.resolve()
    root.mkdir(parents=True, exist_ok=False)
    work = root / 'work'
    work.mkdir()
    config = work / 'trusted-cli.json'
    config.write_text('{}')
    env = {key: value for key, value in os.environ.items()
           if not key.startswith(('PANW_', 'PRISMA_AIRS_', 'DOTENV_'))}
    env.update(AIRS_HARNESS_HOME=str(root / 'state'), PRISMA_AIRS_CONFIG_PATH=str(config))
    binary = args.binary.resolve(strict=True)

    def run(arguments, name, timeout=45):
        result = subprocess.run([str(binary), *arguments], cwd=work, env=env,
                                text=True, capture_output=True, timeout=timeout)
        (root / name).write_text(result.stdout + '\nSTDERR\n' + result.stderr)
        if result.returncode:
            raise RuntimeError(f'{name} failed with exit {result.returncode}')
        return result

    run(['setup', '--gateway-url', args.gateway_url], 'setup.log')
    run(['login', '--credential-file', str(args.credential_file.resolve(strict=True))], 'login.log')
    prompt = (
        '$prisma-airs-cli Use the built-in skill to check this environment. '
        'Run the absolute harness-managed Prisma AIRS CLI to write its actual --version output '
        'to managed-version.txt, and its actual doctor --output json stdout to managed-doctor.json. '
        'The trusted fixture config is deliberately empty: doctor exit 1 is expected. '
        'Do not change configuration or supply credentials. Report which scanner/management '
        'credential variable names are missing; do not print environment or secret values. '
        'Complete these local commands now, without a plan-only answer.'
    )
    run(['exec', '--skip-git-repo-check', '--ephemeral', '-s', 'workspace-write', prompt],
        'agent.log', timeout=240)
    assert (work / 'managed-version.txt').read_text().strip() == '5.2.0'
    doctor = json.loads((work / 'managed-doctor.json').read_text())
    checks = {row['name']: row['status'] for row in doctor}
    assert checks['Scanner credentials'] == checks['Management credentials'] == 'fail'
    assert checks['Scanner API'] == checks['Management OAuth'] == checks['AI Gateway API'] == 'warn'
    skills = list((root / 'state').rglob('skills/.system/prisma-airs-cli/SKILL.md'))
    assert skills, 'Built-in skill was not installed in the environment'
    receipt = {'passed': True, 'cli_version': '5.2.0', 'live_inference': True,
               'management_mutations': False, 'doctor_fixture_checks': checks,
               'built_in_skill_installed': True}
    (root / 'VALIDATION.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
