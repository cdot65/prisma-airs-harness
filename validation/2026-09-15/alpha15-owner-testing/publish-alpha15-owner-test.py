"""Publish only promoted alpha.15 payloads, then advance tags after both installs pass."""
import argparse,hashlib,json,os,subprocess,time,urllib.request
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('phase',choices=['publish','tags']);a=p.parse_args()
root=Path('/home/cdot/.cache/airs-auth-recovery');stage=root/'npm-release-alpha15'
version='0.1.0-alpha.15';registry='https://npm.cdot.io'
d=json.loads((stage/'NPM-PACKAGES.json').read_text())
assert d['registry']==registry and d.get('publishable') is not False
assert d['source_commit']=='2f5a1ad306d379c23e21b60173b87876a79045c4'
records=d['publish_order']
assert [r['name'] for r in records]==['airs-harness-linux-x64','airs-harness-darwin-arm64','airs-harness']
for record in records:
 assert record['version']==version
 assert hashlib.file_digest((stage/'tarballs'/record['filename']).open('rb'),'sha256').hexdigest()==record['sha256']
 assert json.loads((stage/record['name']/'package.json').read_text()).get('private') is not True
for platform in ['linux','mac']:
 validation=json.loads((root/'owner-test-alpha15'/platform/'VALIDATION.json').read_text())
 assert validation['passed'] is False and validation['release_ready'] is False and validation['product_version']==version
 assert validation['scope']=='owner-requested-alpha15-testing' and validation['publication_authorized'] is True
 assert validation['authorization']=='publish alphas.15 so i can test on my remote mac'
 assert validation['source_commit']==d['source_commit']
 assert validation['frontend_refresh_cycles_completed']==0

def metadata(name):
 with urllib.request.urlopen(registry+'/'+name,timeout=30) as response:return json.load(response)
def run(label,cmd):
 env={k:v for k,v in os.environ.items() if not k.upper().startswith(('NPM_','NODE_AUTH_TOKEN'))}
 env.update(NPM_CONFIG_USERCONFIG=str(root/'publication.npmrc'),NPM_CONFIG_REGISTRY=registry,NPM_CONFIG_UPDATE_NOTIFIER='false')
 with (root/(label+'.log')).open('w') as log:
  result=subprocess.run(cmd,env=env,cwd=stage,stdout=log,stderr=subprocess.STDOUT,timeout=300)
 if result.returncode:raise RuntimeError(label+' failed; inspect private npm log')

receipt_path=root/'ALPHA15-PUBLICATION.json'
receipt=json.loads(receipt_path.read_text()) if receipt_path.exists() else {'version':version,'registry':registry,'started_at':time.time(),'published':False,'complete':False,'original_tags':{r['name']:metadata(r['name'])['dist-tags'] for r in records},'packages':[]}
def save():receipt_path.write_text(json.dumps(receipt,indent=2)+'\n')
save()
if a.phase=='publish':
 for record in records:
  name=record['name'];current=metadata(name)
  if version not in current['versions']:
   run('publish-'+name,['npm','publish',str(stage/'tarballs'/record['filename']),'--registry',registry,'--tag','gateway-validation','--ignore-scripts'])
  current=metadata(name)
  assert current['versions'][version]['dist']['integrity']==record['integrity']
  assert current['dist-tags']['gateway-validation']==version
  receipt['packages']=[r for r in receipt['packages'] if r['name']!=name]+[{**record,'registry_integrity_verified':True}]
  save();print(json.dumps({'published_package':name,'version':version}),flush=True)
 receipt['published']=True;receipt['dist_tag']='gateway-validation';save()
else:
 raise RuntimeError('Owner testing publication does not advance alpha/latest tags')
 assert receipt['published'] is True and len(receipt['packages'])==3
 for platform,expected in [('linux','Linux'),('mac','Darwin')]:
  path=root/('registry-alpha15-'+platform)/'REGISTRY-INSTALL.json'
  check=json.loads(path.read_text());assert check['passed'] and check['version']==version and check['platform']==expected and check['anonymous_fresh_install']
 for record in records:
  name=record['name'];assert metadata(name)['versions'][version]['dist']['integrity']==record['integrity']
  for tag in ['alpha','latest']:
   run('tag-'+name+'-'+tag,['npm','dist-tag','add',name+'@'+version,tag,'--registry',registry])
  tags=metadata(name)['dist-tags'];assert tags['alpha']==tags['latest']==version
 receipt.update(complete=True,dist_tags=['alpha','latest','gateway-validation'],completed_at=time.time());save()
 print(json.dumps({'complete':True,'version':version}),flush=True)
