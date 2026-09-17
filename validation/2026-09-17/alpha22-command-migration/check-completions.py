import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
for shell in ['bash','zsh','fish']:
 text=subprocess.check_output([a.binary,'completion',shell],text=True);file=a.output/('completion.'+shell);file.write_text(text);subprocess.run([shell,'-n',str(file)],check=True)
script=f'source {a.output}/completion.bash\nCOMP_WORDS=(airs ""); COMP_CWORD=1; _airs airs "" airs; printf "%s\\n" "${{COMPREPLY[@]}}"'
root=set(subprocess.check_output(['bash','-c',script],text=True).splitlines());assert {'env','mcp','cli','login'}<=root and 'runtime' not in root
script=f'source {a.output}/completion.bash\nCOMP_WORDS=(airs cli runtime ""); COMP_CWORD=3; _airs airs "" runtime; printf "%s\\n" "${{COMPREPLY[@]}}"'
product=set(subprocess.check_output(['bash','-c',script],text=True).splitlines());assert {'scan','profiles','dlp'}<=product and 'env' not in product
script=f'autoload -Uz compinit; compinit -d {a.output}/zcompdump; source {a.output}/completion.zsh; compadd() {{ print -l -- "$@"; }}; words=(airs cli runtime ""); CURRENT=4; _airs'
zsh=set(subprocess.check_output(['zsh','-c',script],text=True).splitlines());assert {'scan','profiles','dlp'}<=zsh and 'env' not in zsh
for line,expected in [('airs ',{'env','cli','mcp'}),('airs cli runtime ',{'scan','profiles','dlp'})]:
 out=subprocess.check_output(['fish','-c',f'source {a.output}/completion.fish; complete -C "{line}"'],text=True);tokens={x.split('\t')[0] for x in out.splitlines()};assert expected<=tokens
 if line=='airs ':assert 'runtime' not in tokens
(a.output/'COMPLETIONS.json').write_text(json.dumps({'passed':True,'installed_binary':a.binary,'checks':['bash native root and nested CLI','zsh nested CLI','fish native root and nested CLI','shell syntax']},indent=2)+'\n');print('Installed shell completions passed')
