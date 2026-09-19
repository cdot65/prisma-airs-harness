#!/usr/bin/env python3
"""Controlled synthetic fixture: compare modern native credential read to doctor."""
import ctypes
import ctypes.util
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import time

BINARY=Path('/tmp/airs-autonomous-20260919/focus2/bin/airs')
OUTPUT=Path('/tmp/airs-autonomous-20260919/evidence/focus2-native-service.json')

def deny_kernel_credentials():
    lib=ctypes.CDLL(ctypes.util.find_library('seccomp'),use_errno=True)
    lib.seccomp_init.argtypes=[ctypes.c_uint32]
    lib.seccomp_init.restype=ctypes.c_void_p
    lib.seccomp_syscall_resolve_name.argtypes=[ctypes.c_char_p]
    lib.seccomp_rule_add.argtypes=[ctypes.c_void_p,ctypes.c_uint32,ctypes.c_int,ctypes.c_uint]
    lib.seccomp_load.argtypes=[ctypes.c_void_p]
    lib.seccomp_release.argtypes=[ctypes.c_void_p]
    context=lib.seccomp_init(0x7fff0000)
    assert context
    for name in [b'keyctl',b'add_key',b'request_key']:
        number=lib.seccomp_syscall_resolve_name(name)
        assert number >= 0
        assert lib.seccomp_rule_add(context,0x00050001,number,0)==0
    assert lib.seccomp_load(context)==0
    lib.seccomp_release(context)

def main():
    assert os.environ.get('DBUS_SESSION_BUS_ADDRESS'), 'Run inside dbus-run-session'
    with tempfile.TemporaryDirectory(prefix='airs-doctor-backend-') as temporary:
        root=Path(temporary)
        env={k:v for k,v in os.environ.items() if not k.startswith(('AIRS_','OPENAI_')) and k not in ('CODEX_HOME','CODEX_SQLITE_HOME','CODEX_CA_CERTIFICATE','GNOME_KEYRING_CONTROL')}
        for variable,name in [('XDG_DATA_HOME','data'),('XDG_CONFIG_HOME','config'),('XDG_RUNTIME_DIR','runtime'),('GNOME_KEYRING_CONTROL','keyring'),('AIRS_HARNESS_HOME','harness')]:
            path=root/name
            path.mkdir(mode=0o700)
            env[variable]=str(path)
        command=['dbus-send','--session','--print-reply','--dest=org.freedesktop.DBus','/org/freedesktop/DBus','org.freedesktop.DBus.NameHasOwner','string:org.freedesktop.secrets']
        owner=subprocess.run(command,env=env,capture_output=True,text=True,check=True,timeout=5)
        assert 'boolean false' in owner.stdout, 'Must not use an existing Secret Service'
        subprocess.run(['dbus-update-activation-environment','XDG_DATA_HOME','XDG_CONFIG_HOME','XDG_RUNTIME_DIR','GNOME_KEYRING_CONTROL'],env=env,capture_output=True,check=True,timeout=5)
        daemon=subprocess.Popen(['gnome-keyring-daemon','--foreground','--unlock','--components=secrets','--control-directory',env['GNOME_KEYRING_CONTROL']],env=env,stdin=subprocess.PIPE,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        daemon.stdin.write(secrets.token_urlsafe(32).encode())
        daemon.stdin.close()
        try:
            for _ in range(100):
                owner=subprocess.run(command,env=env,capture_output=True,text=True,check=True,timeout=5)
                if 'boolean true' in owner.stdout: break
                time.sleep(.05)
            assert 'boolean true' in owner.stdout
            token='synthetic-doctor-backend-fixture'
            env['AIRS_FIXTURE_CREDENTIAL']=token
            def run(*args,input=None,extra=None):
                return subprocess.run([str(BINARY),*args],env=dict(env,**(extra or {})),cwd=root,input=input,text=True,capture_output=True,timeout=20,preexec_fn=deny_kernel_credentials)
            created=run('env','create','fixture','--gateway-url','https://fixture.invalid/v1','--credential-env','AIRS_FIXTURE_CREDENTIAL')
            assert created.returncode==0, f'create failed with exit {created.returncode}'
            saved=run('login','--with-api-key',input=token)
            assert saved.returncode==0, f'save failed with exit {saved.returncode}'
            registry=json.loads((root/'harness/environments.json').read_text())
            home=root/'harness/environments'/registry['environments']['fixture']['id']
            binding_path=home/'credential-binding.json'
            before=binding_path.read_bytes()
            binding=json.loads(before)
            assert binding['source']['kind']=='keyring-v2'
            loaded=run('credential','--home',str(home),'--binding',binding['id'])
            assert loaded.returncode==0 and loaded.stdout.strip()==token, 'native direct Secret Service read failed'
            probe=run('doctor','--json',extra={'AIRS_DOCTOR_STORAGE_PROBE':'1'})
            assert probe.returncode==0
            passed,detail=json.loads(probe.stdout)
            assert binding_path.read_bytes()==before
            loaded_after=run('credential','--home',str(home),'--binding',binding['id'])
            assert loaded_after.returncode==0 and loaded_after.stdout.strip()==token
            version=run('--version')
            receipt={'schema_version':1,'source':'focus2 frozen development binary','version':version.stdout.strip(),'binary_sha256':hashlib.sha256(BINARY.read_bytes()).hexdigest(),'fixture':'isolated dbus-run-session and encrypted disposable gnome-keyring collection','syscalls_denied':['keyctl','add_key','request_key'],'syscall_error':'EPERM','workspace_v2_save_and_verified_read':True,'separate_process_native_read_before_doctor':True,'doctor_native_service_passed':passed,'doctor_native_service_detail':detail,'separate_process_native_read_after_doctor':True,'binding_preserved':True,'divergence_reproduced':not passed,'owner_session_accessed':False,'production_acceptance':False}
            OUTPUT.write_text(json.dumps(receipt,indent=2)+'\n')
            print(json.dumps(receipt,indent=2))
            # A successful probe disproves the proposed divergence for this fixture.
            # Record the observation honestly; no owner-session conclusion follows.
        finally:
            daemon.terminate()
            try: daemon.wait(timeout=5)
            except subprocess.TimeoutExpired: daemon.kill(); daemon.wait()

if __name__=='__main__': main()
