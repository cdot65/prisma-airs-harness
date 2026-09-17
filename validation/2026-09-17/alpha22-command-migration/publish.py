import argparse,hashlib,json,os,subprocess,time,urllib.request
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('phase',choices=['publish','tags']);a=p.parse_args()
root=Path(__file__).resolve().parent;stage=root/'npm-release';version='0.1.0-alpha.22';registry='https://npm.cdot.io'
d=json.loads((stage/'NPM-PACKAGES.json').read_text());assert d['source_commit']=='2a787c03d1673b28fe85803c4082dd1a5e85a36e' and d.get('publishable') is not False
records=d['publish_order'];assert [r['name'] for r in records]==['airs-harness-linux-x64','airs-harness-linux-arm64','airs-harness-darwin-arm64','airs-harness']
def metadata(name):
 with urllib.request.urlopen(registry+'/'+name,timeout=30) as r:return json.load(r)
def run(label,cmd):
 env={k:v for k,v in os.environ.items() if not k.upper().startswith(('NPM_','NODE_AUTH_TOKEN'))};env.update(NPM_CONFIG_USERCONFIG=str(root/'publication.npmrc'),NPM_CONFIG_REGISTRY=registry,NPM_CONFIG_UPDATE_NOTIFIER='false')
 with (root/(label+'.log')).open('w') as log:subprocess.run(cmd,env=env,cwd=stage,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=300)
for r in records:
 assert r['version']==version and hashlib.sha256((stage/'tarballs'/r['filename']).read_bytes()).hexdigest()==r['sha256'];assert json.loads((stage/r['name']/'package.json').read_text()).get('private') is not True
for label in ['linux','arm64','mac']:
 check=json.loads((root/(label+'-COMMAND-MIGRATION.json')).read_text());assert check['passed'] and check['cli_version']=='7.0.0' and not check['force_used']
 check=json.loads((root/(label+'-INSTALLED-CLI.json')).read_text());assert check['passed'] and check['cli_version']=='7.0.0'
 check=json.loads((root/(label+'-candidate-install.json')).read_text());assert check['passed'] and check['version']=='airs '+version
 check=json.loads((root/(label+'-candidate-ENVIRONMENTS.json')).read_text());assert check['passed']
 if label=='arm64':assert check['platform']=='Linux' and check['architecture'] in ['aarch64','arm64']
receipt_path=root/'PUBLICATION.json';receipt=json.loads(receipt_path.read_text()) if receipt_path.exists() else {'version':version,'source_commit':d['source_commit'],'registry':registry,'publication_authorized':True,'authorization':'okay, begin. let me know when both the CLI and harness are published and ready for me to test end-to-end on another remote machine','published':False,'default_tags_changed':False,'native_arm64_installed_acceptance':True,'production_sso_servicenow_acceptance':False,'original_tags':{r['name']:metadata(r['name'])['dist-tags'] for r in records},'packages':[]}
def save():receipt_path.write_text(json.dumps(receipt,indent=2)+'\n')
save()
if a.phase=='publish':
 for record in records:
  name=record['name'];current=metadata(name)
  if version not in current['versions']:run('publish-'+name,['npm','publish',str(stage/'tarballs'/record['filename']),'--registry',registry,'--tag','migration','--ignore-scripts'])
  current=metadata(name);assert current['versions'][version]['dist']['integrity']==record['integrity'];assert current['dist-tags']['migration']==version
  receipt['packages']=[r for r in receipt['packages'] if r['name']!=name]+[{**record,'registry_integrity_verified':True}];save();print(name+' published',flush=True)
 receipt['published']=True;save()
else:
 assert receipt['published']
 for label,plat in [('linux','Linux'),('arm64','Linux'),('mac','Darwin')]:
  c=json.loads((root/(label+'-REGISTRY-INSTALL.json')).read_text());assert c['passed'] and c['version']==version and c['platform']==plat and c['anonymous_fresh_install']
 for r in records:
  name=r['name'];assert metadata(name)['versions'][version]['dist']['integrity']==r['integrity']
  for tag in ['alpha','latest']:run('tag-'+name+'-'+tag,['npm','dist-tag','add',name+'@'+version,tag,'--registry',registry])
  assert all(metadata(name)['dist-tags'][tag]==version for tag in ['alpha','latest'])
 receipt.update(default_tags_changed=True,completed_at=time.time());save();print('All default tags now alpha.22')
