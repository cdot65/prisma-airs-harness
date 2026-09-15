"""Verify an anonymous installation against final promoted package receipts."""
import argparse, base64, hashlib, json, os, platform, shutil, subprocess, sys, time
from pathlib import Path
from urllib.parse import urlsplit
from urllib.request import urlopen

p=argparse.ArgumentParser()
p.add_argument('--packages-receipt',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--source',type=Path,required=True)
a=p.parse_args()
sys.path.insert(0,str(a.source/'scripts'))
from airs_bundle import verify_bundle
from plan_airs_review_publication import archive_snapshot

REGISTRY='https://npm.cdot.io'
VERSION='0.1.0-alpha.15'
SOURCE='2f5a1ad306d379c23e21b60173b87876a79045c4'
expected={'Linux':('airs-harness-linux-x64','2b2825a80d2013ea03feae06f495f7455eac0201144c519e17ef71c5efaec0bc'),'Darwin':('airs-harness-darwin-arm64','ff8a5666c1f9cef6f728bbb9c8e86e1522718e36b2a697a47e46ba85b6d56198')}
system=platform.system();native_name,native_sha=expected[system]
assert (system,platform.machine()) in [('Linux','x86_64'),('Darwin','arm64')]
receipt=json.loads(a.packages_receipt.read_text())
assert receipt['registry']==REGISTRY and receipt['source_commit']==SOURCE
assert receipt.get('publishable') is not False
records={r['name']:r for r in receipt['publish_order']}
assert set(records)=={'airs-harness','airs-harness-linux-x64','airs-harness-darwin-arm64'}
a.output.mkdir(mode=0o700,parents=True,exist_ok=False)
out=a.output.resolve();prefix=out/'prefix';archives=out/'archives';archives.mkdir()
env={k:v for k,v in os.environ.items() if not k.upper().startswith(('NPM_','NODE_AUTH_TOKEN'))}
for name in ['empty.npmrc','empty-global.npmrc']:(out/name).write_text('')
env.update(NPM_CONFIG_USERCONFIG=str(out/'empty.npmrc'),NPM_CONFIG_GLOBALCONFIG=str(out/'empty-global.npmrc'),NPM_CONFIG_CACHE=str(out/'fresh-cache'),NPM_CONFIG_UPDATE_NOTIFIER='false')
def run(label,argv,timeout=300,extra=None):
 with (out/(label+'.log')).open('w') as log:
  r=subprocess.run(argv,env=env|(extra or {}),cwd=a.source,stdout=log,stderr=subprocess.STDOUT,timeout=timeout)
 if r.returncode:raise RuntimeError(label+' failed, exit '+str(r.returncode))

downloaded=[]
for name in [native_name,'airs-harness']:
 record=records[name];assert record['version']==VERSION
 with urlopen(REGISTRY+'/'+name,timeout=30) as response: metadata=json.load(response)
 release=metadata['versions'][VERSION]
 assert metadata['dist-tags']['gateway-validation']==VERSION
 assert release['dist']['integrity']==record['integrity']
 url=release['dist']['tarball'];parsed=urlsplit(url)
 assert parsed.scheme=='https' and parsed.netloc=='npm.cdot.io' and not parsed.query and not parsed.fragment
 archive=archives/record['filename']
 with urlopen(url,timeout=90) as response,archive.open('wb') as target:
  assert urlsplit(response.url).netloc=='npm.cdot.io'
  shutil.copyfileobj(response,target)
 sha=hashlib.file_digest(archive.open('rb'),'sha256').hexdigest()
 integrity='sha512-'+base64.b64encode(hashlib.file_digest(archive.open('rb'),'sha512').digest()).decode()
 assert sha==record['sha256'] and integrity==record['integrity']
 downloaded.append({'name':name,'sha256':sha,'integrity':integrity})
run('install',['npm','install','--global','--prefix',str(prefix),'--ignore-scripts','--include=optional','--no-audit','--no-fund','--registry',REGISTRY,'airs-harness@'+VERSION])
launcher=prefix/'lib/node_modules/airs-harness';native_dir=launcher/'node_modules'/native_name
if not native_dir.exists():native_dir=prefix/'lib/node_modules'/native_name
assert native_dir.resolve().is_relative_to(prefix)
for name,directory in [('airs-harness',launcher),(native_name,native_dir)]:archive_snapshot(archives/records[name]['filename'],directory)
info=json.loads((native_dir/'BUILD-INFO.json').read_text())
native=native_dir/'bin/airs-harness'
assert info['source_commit']==SOURCE and info['version']==VERSION and info['binary_sha256']==native_sha
assert hashlib.file_digest(native.open('rb'),'sha256').hexdigest()==native_sha
inventory=(launcher/'BUNDLE-INVENTORY.json').read_bytes()
assert hashlib.sha256(inventory).hexdigest()==receipt['cli_bundle']['inventory_sha256']
bundle=verify_bundle(launcher,json.loads(inventory))
command=prefix/'bin/airs-harness'
assert command.resolve()==launcher/'bin/airs-harness.js'
assert subprocess.check_output([str(command),'--version'],env=env,text=True).strip()=='airs-harness '+VERSION
manifest=json.loads((launcher/'package.json').read_text())
assert subprocess.check_output([str(command),'airs','--version'],env=env,text=True).strip()==manifest['dependencies']['@cdot65/prisma-airs-cli']
signing=None
if system=='Darwin':
 run('codesign',['codesign','--verify','--strict','--verbose=2',str(native)])
 run('notarization',['codesign','--verify','--strict','--verbose=4','--check-notarization','-R','=notarized',str(native)])
 signing={'codesign_verified':True,'notarization_verified':True}
run('native-tests',[sys.executable,'-m','unittest','discover','-s','scripts','-p','test_airs_harness*.py','-v'],timeout=900,extra={'AIRS_HARNESS_BIN':str(command),'AIRS_MANAGED_CLI_ACCEPTANCE':'1','AIRS_HARNESS_TEST_EVIDENCE':str(out/'native-test-evidence')})
result={'passed':True,'platform':system,'version':VERSION,'binary_sha256':native_sha,'source_commit':SOURCE,'anonymous_fresh_install':True,'downloaded_archives':downloaded,'bundle':bundle,'signing':signing,'observed_at':time.time(),'prefix':str(prefix)}
(out/'REGISTRY-INSTALL.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'passed':True,'platform':system,'version':VERSION,'binary_sha256':native_sha}),flush=True)
