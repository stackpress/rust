"""Bounded owner walkthrough of one freshly owned simulator; never a build or listener."""
import argparse
import json
import os
from pathlib import Path
import plistlib
import signal
import subprocess
import sys
import tempfile
import time
import uuid


def options(arguments):
    """Reject unbounded lifetimes and ambiguous variants before touching a device."""
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manual-review',action='store_true',required=True)
    parser.add_argument('--variant',choices=['disabled','plugin'],required=True)
    parser.add_argument('--duration-seconds',type=int,default=1800)
    parser.add_argument('--manual-worker',action='store_true',help=argparse.SUPPRESS)
    parser.add_argument('--control',choices=['restart','stop'])
    args=parser.parse_args(arguments)
    if not 30<=args.duration_seconds<=7200:
        parser.error('--duration-seconds must be 30..7200 (including setup)')
    return args


def control(logs, command=None):
    """A closed, one-shot local file supports restart/finish; it carries no app operation."""
    path=logs/'manual-control'
    if path.is_symlink() or (path.exists() and (not path.is_file() or path.stat().st_size>16)):
        raise ValueError('Invalid manual control file')
    if command is not None:
        if command not in ('restart','stop'):raise ValueError('Invalid manual command')
        # Refuse replacing an unconsumed control instead of losing owner intent.
        with tempfile.NamedTemporaryFile(mode='w',prefix='manual-control-',dir=logs,delete=False) as output:
            temporary=Path(output.name)
            output.write(command)
        try:os.link(temporary,path)
        finally:temporary.unlink()
        return
    if not path.exists():return None
    value=path.read_text();path.unlink()
    if value not in ('restart','stop'):raise ValueError('Invalid manual command')
    return value


def stop_console(host, device, child):
    """Terminate this app before forcibly reaping its attached launch process."""
    if child is None:return
    try:
        host.run('xcrun','simctl','terminate',device,host.APP_ID,allow_failure=True)
    finally:
        try:child.wait(timeout=10)
        except subprocess.TimeoutExpired:child.kill()
        finally:child.wait()


def wait_until_running(host, device, child, timeout=60):
    """Attach the display only after the app runs; Duo SpringBoard crashes on an earlier attach."""
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        # A console that already exited means the launch failed; report it, don't attach.
        if child.poll() is not None:raise RuntimeError('App console exited; see manual-app.log')
        listing=host.run('xcrun','simctl','spawn',device,'launchctl','list',allow_failure=True)
        if listing and 'UIKitApplication:'+host.APP_ID in listing:return
        time.sleep(.5)
    raise RuntimeError('App did not start before attaching the display')


def recover(host):
    """Fallback cleanup retries all owned resources, preserving a failed record for inspection."""
    if not host.OWNED.exists():return
    record=json.loads(host.OWNED.read_text())
    device=record.get('device')
    if device is None:
        # A worker can be killed after CoreSimulator creates a device but before
        # simctl returns its UDID. The pre-recorded unique name closes that gap.
        inventory=json.loads(host.run('xcrun','simctl','list','devices','--json'))
        matches=[item['udid'] for items in inventory['devices'].values() for item in items
                 if item['name']==record['name'] and item['udid'] not in record['before']]
        if len(matches)>1:raise RuntimeError('Ambiguous owned device; inspect cleanup record')
        device=next(iter(matches),None)
    try:
        if device is not None and device in host.devices():host.cleanup(device)
    finally:
        host.close_hub(record.get('display_pids',[]))
    if device is not None and device in host.devices():raise RuntimeError('Owned device remains')
    host.OWNED.unlink()


def track_display(host):
    """Claim only our launched display and observable descendants, never other displays."""
    record=json.loads(host.OWNED.read_text())
    owned=set(record.get('display_pids',[]))
    if not record.get('display_started'):return sorted(owned)
    rows=subprocess.check_output(['ps','-axo','pid=,ppid='],text=True,timeout=5)
    parents={int(pid):int(parent) for row in rows.splitlines() for pid,parent in [row.split()]}
    # Retain ancestry observed before a child can be reparented by LaunchServices.
    previous=set()
    while previous!=owned:
        previous=set(owned)
        owned.update(pid for pid,parent in parents.items() if parent in owned)
    record['display_pids']=sorted(owned & host.hub_pids() | set(record.get('display_pids',[])))
    host.OWNED.write_text(json.dumps(record))
    return record['display_pids']


def session(host, args):
    """Keep the installed app interactive until its deadline or explicit stop command."""
    build=json.loads((host.LOGS/'ios-build.json').read_text())
    if not (build['exit_code']==0 and build['closed_before_proof'] and build.get('review_variant')==args.variant):
        raise ValueError('Build receipt does not match the requested debug review variant')
    descriptor=host.ROOT/'examples/profile-ui/native/gen/apple/.tauri/cli-options-server.json'
    if descriptor.exists():raise RuntimeError('Build options service descriptor remains')
    # The ordinary proof verifies closed sockets too. Reuse that policy without
    # creating a service: a successful connection is an immediate refusal.
    for address in build['observed_addresses']:
        address_host,port=address.rsplit(':',1)
        try:
            with host.socket.create_connection((address_host,int(port)),timeout=1):
                raise RuntimeError('Build options service still active')
        except ConnectionRefusedError:pass
    bundle=Path(build['bundle'])
    if plistlib.loads((bundle/'Info.plist').read_bytes())['CFBundleIdentifier']!=host.APP_ID:
        raise ValueError('Unexpected app identity')
    # A fresh owned simulator keeps fixture preferences separate from the owner's
    # devices. Refusal never quits a pre-existing display or shuts down a device.
    before=host.devices()
    if host.hub_pids() or host.booted_devices():raise RuntimeError('An owner display/device is active')
    hub=host.display_bundle()
    info=plistlib.loads((hub/'Contents/Info.plist').read_bytes())
    if info['CFBundleIdentifier']!='com.apple.dt.Devices':raise ValueError('Unexpected simulator display')
    name='Stackpress-Review-'+uuid.uuid4().hex
    # Persist intent before the external create call, so the parent can recover
    # an interrupted creation without adopting any pre-existing simulator.
    record=dict(device=None,name=name,before=sorted(before),variant=args.variant,display_started=False,display_pids=[])
    host.OWNED.write_text(json.dumps(record))
    device=host.run('xcrun','simctl','create',name,host.DEVICE_TYPE,host.RUNTIME)
    if device in before or len(device)!=36:raise RuntimeError('Device was not freshly created')
    child=display=None
    console=None
    try:
        # Even an ownership-record I/O failure must enter device cleanup.
        record['device']=device
        host.OWNED.write_text(json.dumps(record))
        host.run('xcrun','simctl','boot',device)
        host.run('xcrun','simctl','bootstatus',device,'-b',timeout=120)
        host.run('xcrun','simctl','install',device,bundle,timeout=120)
        console=(host.LOGS/'manual-app.log').open('a')
        env=dict(os.environ,SIMCTL_CHILD_PROFILE_UI_MANUAL_REVIEW='1')
        # Old automated drivers must not manipulate or close this manual session.
        for key in ('PROFILE_UI_PROOF','PROFILE_UI_PROOF_MODE','PROFILE_UI_MOBILE_PROOF',
                    'SIMCTL_CHILD_PROFILE_UI_PROOF','SIMCTL_CHILD_PROFILE_UI_PROOF_MODE','SIMCTL_CHILD_PROFILE_UI_MOBILE_PROOF'):
            env.pop(key,None)
        def launch():
            """Attach only this app's console; no proof driver or additional command grant."""
            return subprocess.Popen(['xcrun','simctl','launch','--console',device,host.APP_ID],
                                    env=env,stdout=console,stderr=subprocess.STDOUT)
        child=launch()
        wait_until_running(host,device,child)
        if host.hub_pids():raise RuntimeError('An owner display appeared; leave it untouched')
        display=subprocess.Popen([str(hub/'Contents/MacOS'/info['CFBundleExecutable'])],
                                 stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        record=json.loads(host.OWNED.read_text())
        record.update(display_started=True,display_pids=[display.pid])
        host.OWNED.write_text(json.dumps(record))
        host.run('xcrun','simctl','io',device,'screenConfig','--display=primary','power','on')
        deadline=float(os.environ['PROFILE_UI_REVIEW_DEADLINE'])
        (host.LOGS/'manual-session.json').write_text(json.dumps(dict(variant=args.variant,device=device,
            bundle=str(bundle),deadline_monotonic=deadline,status='interactive; owner acceptance unverified'),indent=2)+'\n')
        tracked=0.0
        while time.monotonic()<deadline:
            # Process listing is slow on a loaded host: sample every two seconds, and
            # treat one slow listing as "try again", not as a failed review session.
            if time.monotonic()-tracked>=2:
                try:track_display(host)
                except subprocess.TimeoutExpired:pass
                tracked=time.monotonic()
            if child.poll() is not None:raise RuntimeError('App console exited; see manual-app.log')
            command=control(host.LOGS)
            if command=='stop':break
            if command=='restart':
                stop_console(host,device,child);child=None
                child=launch()
            time.sleep(.1)
    finally:
        # A failed app termination must never skip device or display cleanup.
        try:
            stop_console(host,device,child)
        finally:
            if console is not None:console.close()
            try:host.cleanup(device)
            finally:
                if display is not None:
                    try:host.close_hub(track_display(host))
                    finally:
                        try:display.wait(timeout=10)
                        except subprocess.TimeoutExpired:display.kill();display.wait()


def main(host):
    """A parent deadline and cleanup record survive worker errors, Ctrl-C and termination."""
    args=options(sys.argv[1:])
    host.LOGS.mkdir(parents=True,exist_ok=True)
    if args.control:
        if not host.OWNED.exists():raise RuntimeError('No owned manual session')
        if json.loads(host.OWNED.read_text()).get('variant')!=args.variant:
            raise ValueError('Control variant does not match the owned manual session')
        control(host.LOGS,args.control);return
    def interrupted(_signal,_frame):
        """Unwind through cleanup instead of abandoning a live simulator."""
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM,interrupted)
    if args.manual_worker:
        session(host,args);return
    if host.OWNED.exists():raise RuntimeError('Owned-device record exists; inspect previous cleanup')
    if host.hub_pids() or host.booted_devices():raise RuntimeError('An owner display/device is active')
    if (host.LOGS/'manual-control').exists():raise RuntimeError('Stale manual control; inspect before launch')
    print('Manual review: 011 and installed 013. Fixed checks are above Profile UI. '
          'Use --control restart to check persistence; --control stop to finish. '
          f'Deadline: {args.duration_seconds}s including setup.',flush=True)
    env=dict(os.environ,PROFILE_UI_REVIEW_DEADLINE=str(time.monotonic()+args.duration_seconds))
    with (host.LOGS/'manual-launch.log').open('w') as log:
        child=subprocess.Popen([sys.executable,str(Path(host.__file__)),*sys.argv[1:],'--manual-worker'],
                               env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
        result=None
        try:
            result=child.wait(timeout=args.duration_seconds)
        except (subprocess.TimeoutExpired,KeyboardInterrupt):
            # Give the worker's finally blocks a chance, then enforce the bound.
            try:os.killpg(child.pid,signal.SIGTERM)
            except ProcessLookupError:pass
        finally:
            try:child.wait(timeout=20)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid,signal.SIGKILL);child.wait()
            finally:recover(host)
    (host.LOGS/'manual-cleanup.json').write_text(json.dumps(dict(variant=args.variant,
        worker_code=child.returncode,owned_record_removed=not host.OWNED.exists(),
        acceptance='unverified; record owner findings separately'),indent=2)+'\n')
    if result not in (None,0):raise RuntimeError('Manual review failed; see manual-launch.log')
    print('Manual session ended; owned device/display cleaned. Owner acceptance remains separate.',flush=True)
