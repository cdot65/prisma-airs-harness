import argparse, hashlib, json, os, platform, subprocess, sys
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--packages', type=Path, required=True)
parser.add_argument('--scripts', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--start-at')
args = parser.parse_args()
bundle, scripts, output = args.packages.resolve(strict=True), args.scripts.resolve(strict=True), args.output.resolve()
output.mkdir(parents=True, exist_ok=args.start_at is not None)
resume_at = args.start_at
environment = {key: value for key, value in os.environ.items() if not key.startswith(('AIRS_', 'OPENAI_')) and key not in ('CODEX_HOME', 'CODEX_SQLITE_HOME', 'NO_COLOR', 'CODEX_CA_CERTIFICATE')}
environment['TERM'] = 'xterm-256color'
def run(arguments, name, extra=None, timeout=900):
    global resume_at
    if resume_at and name != resume_at:
        return
    resume_at = None
    print('Running '+name, flush=True)
    with (output/(name+'.log')).open('w') as log:
        subprocess.run(arguments, env=environment | (extra or {}), stdout=log, stderr=subprocess.STDOUT, check=True, timeout=timeout)
prefix=output/'prefix'
run([sys.executable, str(scripts/'validate_airs_npm.py'), '--packages', str(bundle), '--prefix', str(prefix)], 'install')
command=prefix/'bin/airs'
os_name={'Linux':'linux','Darwin':'darwin'}[platform.system()]
arch={'x86_64':'x64','aarch64':'arm64','arm64':'arm64'}[platform.machine()]
launcher=prefix/'lib/node_modules/airs-harness'
native=launcher/'node_modules'/f'airs-harness-{os_name}-{arch}'/'bin/airs-harness'
info=json.loads((native.parent.parent/'BUILD-INFO.json').read_text())
assert info['source_commit']==json.loads((bundle/'NPM-PACKAGES.json').read_text())['source_commit']
assert info['version']=='0.1.0-alpha.22.onboarding.4'
assert hashlib.sha256(native.read_bytes()).hexdigest()==info['binary_sha256']
identity_script='validate_airs_onboarding_macos.py' if os_name=='darwin' else 'validate_airs_onboarding.py'
identity=[sys.executable,str(scripts/identity_script),'--binary',str(command),'--native-binary',str(native),'--output',str(output/'onboarding')]
if os_name=='linux':identity=['dbus-run-session','--',*identity]
run(identity,'onboarding',timeout=600)
terminal=[sys.executable,str(scripts/'validate_airs_onboarding_terminals.py'),'--binary',str(command),'--native-binary',str(native),'--output',str(output/'terminals')]
if os_name=='darwin':terminal+=['--shells','bash','zsh']
run(terminal,'terminals',timeout=180)
run([sys.executable,'-m','unittest','discover','-s',str(scripts),'-p','test_airs_harness*.py','-v'],'installed-regressions',{'AIRS_HARNESS_BIN':str(command),'AIRS_MANAGED_CLI_ACCEPTANCE':'1'})
run([sys.executable,str(scripts/'validate_prisma_cli.py'),'--launcher',str(launcher/'bin/airs.js'),'--receipt',str(output/'MANAGED-CLI.json')],'managed-cli')
run([sys.executable,str(scripts/'validate_airs_npm_upgrade.py'),'--packages',str(bundle),'--previous','0.1.0-alpha.22.onboarding.3','--output',str(output/'upgrade-onboarding3')],'upgrade-onboarding3')
run([sys.executable,str(Path(__file__).with_name('check-command-output.py')),'--binary',str(command),'--native',str(native),'--scripts',str(scripts),'--receipt',str(output/'COMMAND-OUTPUT.json')],'command-output',timeout=180)
receipt={'passed':True,'source_commit':info['source_commit'],'version':info['version'],'binary_sha256':info['binary_sha256'],'platform':platform.system(),'architecture':platform.machine(),'entrypoint':str(command),'checks':['self-contained review installation','native HTTPS OAuth and OS credential store','shells and terminal restoration','installed executable regressions','managed CLI contract and native image dependencies','upgrade from onboarding.3 with environment preservation','public help, recovery and actual session exit resume guidance'],'production_sso':False,'production_servicenow':False}
(output/'INSTALLED-CANDIDATE.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2))
