import argparse,hashlib,json,os,platform,subprocess,sys,urllib.request
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--scripts',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--spec',default='airs-harness@0.1.0-alpha.22.onboarding.4');a=p.parse_args()
sys.path.insert(0,str(a.scripts));from airs_npm_registry import install_environment
root=a.output.resolve();root.mkdir(parents=True,exist_ok=False);prefix=root/'prefix';prefix.mkdir();env=install_environment(prefix,'https://npm.cdot.io',False)
with (root/'install.log').open('w') as log:
 subprocess.run(['npm','install','-g','--prefix',str(prefix),'--no-audit','--no-fund','--registry=https://npm.cdot.io',a.spec],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=300)
command=prefix/'bin/airs';v=subprocess.check_output([str(command),'--version'],env=env,text=True).strip();assert v=='airs 0.1.0-alpha.22.onboarding.4'
arch={'x86_64':'x64','arm64':'arm64','aarch64':'arm64'}[platform.machine()];osname={'Linux':'linux','Darwin':'darwin'}[platform.system()]
name=f'airs-harness-{osname}-{arch}';native=prefix/'lib/node_modules/airs-harness/node_modules'/name/'bin/airs-harness'
sha=hashlib.sha256(native.read_bytes()).hexdigest();assert sha==a.expected_sha
info=json.loads((native.parent.parent/'BUILD-INFO.json').read_text());assert info['binary_sha256']==sha
assert subprocess.check_output([str(command),'cli','--version'],env=env,text=True).strip()=='7.0.0'
env.update(AIRS_MANAGED_CLI_ACCEPTANCE='1',AIRS_HARNESS_BIN=str(command))
with (root/'installed-tests.log').open('w') as log:
 subprocess.run([sys.executable,'-m','unittest','discover','-s',str(a.scripts),'-p','test_airs_harness*.py','-v'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=600)
subprocess.run([sys.executable,str(Path(__file__).with_name('check-environments.py')),'--binary',str(native),'--receipt',str(root/'ENVIRONMENTS.json')],check=True)
subprocess.run([sys.executable,str(Path(__file__).with_name('check-command-output.py')),'--binary',str(command),'--native',str(native),'--scripts',str(a.scripts),'--receipt',str(root/'COMMAND-OUTPUT.json')],check=True,env=env,timeout=180)
if platform.system()=='Darwin':
 subprocess.run(['codesign','--verify','--strict',str(native)],check=True)
 subprocess.run([sys.executable,str(a.scripts/'validate_airs_macos_keychain.py'),'--binary',str(command),'--receipt',str(root/'KEYCHAIN.json')],check=True,timeout=180)
receipt={'passed':True,'version':'0.1.0-alpha.22.onboarding.4','version_output':v,'platform':platform.system(),'architecture':platform.machine(),'anonymous_fresh_install':True,'binary_sha256':sha,'source_commit':info['source_commit'],'prefix':str(prefix),'native_fixture_suite':True,'environment_lifecycle':True,'production_sso_or_servicenow_call':False}
(root/'REGISTRY-INSTALL.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
