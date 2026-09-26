#!/opt/homebrew/bin/python3.13
import os,sys,tempfile,runpy,json
from pathlib import Path
# Isolation is test-process-only; Cargo/Rustup retain their normal homes.
names=json.loads(Path('/Users/cdot/.cache/airs-upstream-20260926/retry-isolated-test-names.json').read_text())
if any(arg in names for arg in sys.argv[2:]):
    home=tempfile.mkdtemp(prefix='profile-',dir=os.environ['TMPDIR'])
    os.environ['HOME']=home
    os.environ['ZDOTDIR']=home
sys.argv[0]='/Users/cdot/.cache/airs-upstream-20260926/source/scripts/airs_rust_test_runner.py'
runpy.run_path(sys.argv[0],run_name='__main__')
