import argparse,hashlib,json,tarfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--input',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--platform',required=True);a=p.parse_args()
root=a.input.resolve();excluded={'prefix','npm managed prefix','legacy prefix','preserved-harness-state'}
with tarfile.open(a.output,'w:gz') as tar:
 for path in sorted(root.rglob('*')):
  relative=path.relative_to(root)
  if path.is_file() and not path.is_symlink() and not any(part in excluded for part in relative.parts) and path.suffix in {'.json','.log','.html'}:
   tar.add(path,arcname=str(relative),recursive=False)
 for name in ['INSTALL-VERIFICATION.json','INSTALL-NETWORK.json','npm-install.log']:
  tar.add(root/'prefix'/name,arcname='install/'+name,recursive=False)
 native=root/'prefix/lib/node_modules/airs-harness/node_modules'/('airs-harness-'+a.platform)
 for name in ['BUILD-INFO.json','SIGNING.json']:
  if (native/name).exists():tar.add(native/name,arcname='native/'+name,recursive=False)
print(json.dumps({'output':str(a.output),'sha256':hashlib.sha256(a.output.read_bytes()).hexdigest()}))
