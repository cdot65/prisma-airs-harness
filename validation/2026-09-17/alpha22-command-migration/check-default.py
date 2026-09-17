import argparse,hashlib,json,os,subprocess,sys
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--prefix',type=Path,required=True);p.add_argument('--scripts',type=Path,required=True);p.add_argument('--sha',required=True);p.add_argument('--receipt',type=Path,required=True);a=p.parse_args();sys.path.insert(0,str(a.scripts.resolve()));from airs_npm_registry import install_environment
prefix=a.prefix.resolve();env=install_environment(prefix,'https://npm.cdot.io',False);env.update(AIRS_HARNESS_HOME=str(prefix/'version-probe-state'),NPM_CONFIG_PREFER_ONLINE='true')
with a.receipt.with_suffix('.log').open('w') as log:subprocess.run(['npm','install','-g','--prefix',str(prefix),'--registry=https://npm.cdot.io','--ignore-scripts','--no-audit','--no-fund','airs-harness'],env=env,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=180)
cmd=prefix/'bin/airs';version=subprocess.check_output([str(cmd),'--version'],env=env,text=True).strip();assert version=='airs 0.1.0-alpha.22'
manifest=prefix/'lib/node_modules/airs-harness/package.json';native_manifest=subprocess.check_output(['node','-e','const {createRequire}=require("module");const r=createRequire(process.argv[1]);console.log(r.resolve("airs-harness-"+process.platform+"-"+process.arch+"/package.json"));',str(manifest)],env=env,text=True).strip();native=Path(native_manifest).parent/'bin/airs-harness';sha=hashlib.sha256(native.read_bytes()).hexdigest();assert sha==a.sha
subprocess.run([str(cmd),'env','create','--help'],env=env,stdout=subprocess.DEVNULL,check=True)
a.receipt.write_text(json.dumps({'passed':True,'unversioned_install':True,'version':version,'binary_sha256':sha,'environment_create_help':True},indent=2)+'\n');print(version)
