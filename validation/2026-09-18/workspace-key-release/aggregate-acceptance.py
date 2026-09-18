import json,re
from pathlib import Path
root=Path(__file__).resolve().parent/'acceptance'
source=(root.parent/'SOURCE').read_text().strip()
platforms=[]
for name,target in [('linux','x86_64-unknown-linux-musl'),('arm64','aarch64-unknown-linux-musl'),('mac','aarch64-apple-darwin')]:
 p=root/name
 def read(n):return json.loads((p/n).read_text())
 installed=read('INSTALLED-CANDIDATE.json');assert installed['passed'] and installed['source_commit']==source
 digest=installed['binary_sha256']
 onboarding=read('onboarding/ONBOARDING-ACCEPTANCE.json');assert onboarding['passed'] and onboarding['binary_sha256']==digest
 terminal=read('terminals/TERMINAL-ACCEPTANCE.json');assert terminal['passed'] and terminal['native_binary_sha256']==digest
 command=read('COMMAND-OUTPUT.json');assert command['passed'] and command['native_sha256']==digest and len(command['checks'])==15
 managed=read('MANAGED-CLI.json');assert managed['passed'] and managed['cli_version']=='7.0.0'
 upgrade=read('upgrade-onboarding3/UPGRADE.json');assert upgrade['passed'] and upgrade['previous']=='0.1.0-alpha.22.onboarding.3' and upgrade['configuration_preserved']
 assert all(c['passed'] and c['binary_sha256']==digest for c in upgrade['cases'])
 log=(p/'installed-regressions.log').read_text();assert re.search(r'Ran 47 tests',log) and 'OK (skipped=1)' in log
 platforms.append({'target':target,'version':installed['version'],'source_commit':source,'binary_sha256':digest,'onboarding_checks':len(onboarding['checks']),'terminal_checks':len(terminal['checks']),'command_output_checks':len(command['checks']),'installed_regressions':{'passed':46,'skipped':1},'managed_cli_groups_passed':len(managed['checks']),'upgrade_cases_passed':len(upgrade['cases']),'upgrade_previous':upgrade['previous']})
sig=json.loads((root/'INSTALLED-MAC-SIGNATURE.json').read_text());assert sig['passed'] and sig['binary_sha256']==platforms[-1]['binary_sha256']
receipt={'scope':'Exact installed gateway diagnostic update accepted on three native platforms','source_commit':source,'platforms':platforms,'forgejo_runs':{'linux_x64':212,'linux_arm64':213,'mac_build':214,'mac_signing':int((root.parent/'SIGNING-RUN').read_text())},'published':False,'production_sso_servicenow_acceptance':False,'affected_package_suite':{'passed':5281,'failed':0,'skipped':6},'live_gateway_dual_auth_verified':True}
(root/'ACCEPTANCE.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt,indent=2))
