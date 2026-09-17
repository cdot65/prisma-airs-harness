import argparse, hashlib, json, os, platform, subprocess, tempfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--receipt',required=True);a=p.parse_args()
binary=Path(a.binary).resolve()
with tempfile.TemporaryDirectory(prefix='alpha22-environments-') as tmp:
 home=Path(tmp);env=dict(os.environ,AIRS_HARNESS_HOME=tmp,AIRS_API_KEY='synthetic-local-status-key')
 def run(*args):
  r=subprocess.run([str(binary),*args],env=env,text=True,capture_output=True,timeout=30)
  assert r.returncode==0,(args,r.stderr)
  return r.stdout
 def registry():return json.loads((home/'environments.json').read_text())
 assert run('--version').strip()=='airs 0.1.0-alpha.22'
 run('env','create','work','--gateway-url','https://gateway.example.com/v1')
 original=registry()['environments']['work'];state=home/'environments'/original['id']
 (state/'history.jsonl').write_text('preserved history\n');(state/'credential-marker').write_text('synthetic binding\n')
 before={name:(state/name).read_bytes() for name in ['config.toml','history.jsonl','credential-marker']}
 run('env','rename','work','team');assert registry()['active']=='team';assert registry()['environments']['team']==original
 run('env','create','other','--gateway-url','https://other.example.com/v1')
 assert original['id'] in run('--environment','team','env','show');assert registry()['active']=='other'
 assert 'https://gateway.example.com/v1' in run('env','status','team')
 run('env','use','team');assert registry()['active']=='team'
 run('env','remove','team');assert registry()['active'] is None
 assert all((state/name).read_bytes()==data for name,data in before.items())
 run('env','create','team','--gateway-url','https://gateway.example.com/v1');assert registry()['environments']['team']['id']!=original['id']
 run('env','list')
Path(a.receipt).write_text(json.dumps({'passed':True,'version':'0.1.0-alpha.22','platform':platform.system(),'architecture':platform.machine(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'checks':['create selects default','rename preserves state identity','one-command override preserves default','named status','saved default switch','remove retains files and clears default','recreated name gets new identity'],'network':'no external requests','scope':'exact executable environment lifecycle'},indent=2)+'\n')
print('Environment lifecycle passed')
