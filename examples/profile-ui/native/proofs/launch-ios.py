#!/usr/bin/env python3
"""Finite installed-app simulator proof; owns and deletes exactly one new device."""
import json
import os
from pathlib import Path
import plistlib
import signal
import socket
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[4]
LOGS = Path(os.environ.get('PROFILE_UI_IOS_LOGS', '/private/tmp/s8-lane-b-ios'))
RUNTIME = 'com.apple.CoreSimulator.SimRuntime.iOS-27-1'
DEVICE_TYPE = 'com.apple.CoreSimulator.SimDeviceType.iPhone-Duo'
APP_ID = 'dev.stackpress.profile-m0'
OWNED = LOGS / 'owned-simulator.json'
def display_bundle():
    """Resolve the selected Xcode's display without installing or changing settings."""
    developer = Path(subprocess.check_output(['xcode-select', '-p'], text=True, timeout=5).strip())
    return Path(os.environ.get('PROFILE_UI_DEVICEHUB', developer.parent / 'Applications/DeviceHub.app'))


def hub_pids():
    """Read executable identities, excluding command text from unrelated shells."""
    rows = subprocess.check_output(['ps', '-axo', 'pid=,comm='], text=True, timeout=5)
    return {int(pid) for row in rows.splitlines() for pid, command in [row.strip().split(None, 1)]
            if Path(command).name in {'DeviceHub', 'DevicesTrampoline'}}


def track_hub():
    """Claim display PIDs only after our own launch; headless work never scans them."""
    record = json.loads(OWNED.read_text())
    if not record.get('display_started', False):
        return record.get('display_pids', [])
    record['display_pids'] = sorted(set(record.get('display_pids', [])) | hub_pids())
    OWNED.write_text(json.dumps(record))
    return record['display_pids']


def close_hub(pids):
    """Graceful then forced exit applies only to recorded, still-matching processes."""
    owned = set(pids) & hub_pids()
    for pid in owned:
        try: os.kill(pid, signal.SIGTERM)
        except ProcessLookupError: pass
    deadline = time.monotonic() + 10
    while owned & hub_pids() and time.monotonic() < deadline:
        time.sleep(.1)
    for pid in owned & hub_pids():
        try: os.kill(pid, signal.SIGKILL)
        except ProcessLookupError: pass
    deadline = time.monotonic() + 5
    while owned & hub_pids() and time.monotonic() < deadline:
        time.sleep(.1)
    assert not hub_pids(), 'DeviceHub remains; never kill an unrecorded PID'


def run(*arguments, timeout=30, env=None, allow_failure=False):
    """Every direct child has a deadline and is reaped, including failing commands."""
    with subprocess.Popen(list(map(str, arguments)), env=env, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, text=True) as child:
        try:
            output, _ = child.communicate(timeout=timeout)
        finally:
            if child.poll() is None:
                child.kill()
            child.wait()
    with (LOGS / 'simctl.log').open('a') as log:
        log.write('$ ' + ' '.join(map(str, arguments)) + '\n' + output + '\n')
    if child.returncode and not allow_failure:
        raise RuntimeError('command failed: ' + ' '.join(map(str, arguments)) + '\n' + output)
    return output.strip()


def devices():
    """Inventory identities only; never operate on a pre-existing device."""
    inventory = json.loads(run('xcrun', 'simctl', 'list', 'devices', '--json'))
    return {item['udid'] for items in inventory['devices'].values() for item in items}


def booted_devices():
    """A display proof may start only when it cannot select an owner's booted device."""
    inventory = json.loads(run('xcrun', 'simctl', 'list', 'devices', 'booted', '--json'))
    return {item['udid'] for items in inventory['devices'].values() for item in items}


def cleanup(device):
    """Terminate the owned app/device, with finite subprocesses even after failure."""
    failures = []
    for arguments, timeout in [(('terminate', device, APP_ID), 30),
                               (('shutdown', device), 45), (('delete', device), 30)]:
        try:
            run('xcrun', 'simctl', *arguments, allow_failure=arguments[0] != 'delete', timeout=timeout)
        except (OSError, subprocess.SubprocessError, RuntimeError) as failure:
            # A failed terminate must not skip shutdown/delete. Report after cleanup.
            failures.append(failure)
    if failures:
        raise failures[0]


def worker():
    """Use a real bundled app; no listener, live-reload command or external device."""
    build = json.loads((LOGS / 'ios-build.json').read_text())
    assert build['exit_code'] == 0 and build['closed_before_proof']
    descriptor = ROOT / 'examples/profile-ui/native/gen/apple/.tauri/cli-options-server.json'
    assert not descriptor.exists()
    for address in build['observed_addresses']:
        host, port = address.rsplit(':', 1)
        try:
            with socket.create_connection((host, int(port)), timeout=1):
                raise RuntimeError('build options service still running')
        except ConnectionRefusedError:
            pass
    archive = ROOT / 'examples/profile-ui/native/gen/apple/build/stackpress-example-profile-ui-native_iOS.xcarchive'
    bundles = list((archive / 'Products/Applications').glob('*.app'))
    assert len(bundles) == 1
    bundle = bundles[0]
    info = plistlib.loads((bundle / 'Info.plist').read_bytes())
    assert info['CFBundleIdentifier'] == APP_ID
    before = devices()
    assert not hub_pids(), "DeviceHub is already running; stop without reusing or quitting it"
    hub = display_bundle()
    assert hub.is_dir()
    name = 'Stackpress-LaneB-iOS-' + uuid.uuid4().hex[:10]
    device = run('xcrun', 'simctl', 'create', name, DEVICE_TYPE, RUNTIME)
    assert device not in before and len(device) == 36
    OWNED.write_text(json.dumps({'device': device, 'name': name, 'display_pids': [], 'display_started': False}))
    console = None
    console_log = None
    simulator = None
    reports = {}
    started = time.monotonic()
    try:
        run('xcrun', 'simctl', 'boot', device)
        run('xcrun', 'simctl', 'bootstatus', device, '-b', timeout=120)
        # Install before attaching DeviceHub: its 27.1 display interaction can stall installation.
        run('xcrun', 'simctl', 'install', device, bundle, timeout=120)
        container = Path(run('xcrun', 'simctl', 'get_app_container', device, APP_ID, 'data'))

        def stop():
            """The app is terminated before reaping its attached launch process."""
            nonlocal console, console_log
            if console is not None:
                run('xcrun', 'simctl', 'terminate', device, APP_ID, allow_failure=True)
                try:
                    console.wait(timeout=10)
                finally:
                    if console.poll() is None:
                        console.kill()
                    console.wait()
                console = None
                console_log.close()
                console_log = None

        def launch(mode):
            """Attach console output so every app launch has a kill/reap owner."""
            nonlocal console, console_log
            stop()
            for old in container.rglob('mobile-proof.json'):
                old.unlink()
            for old in container.rglob('mobile-proof-control'):
                old.unlink()
            console_log = (LOGS / ('simulator-' + mode + '.log')).open('a')
            console = subprocess.Popen(['xcrun', 'simctl', 'launch', '--console', device, APP_ID],
                env=dict(os.environ, SIMCTL_CHILD_PROFILE_UI_MOBILE_PROOF=mode),
                stdout=console_log, stderr=subprocess.STDOUT)

        def wait_report(phase, timeout=45, predicate=lambda value: True):
            """Bound each action and fail immediately on the in-app diagnostic."""
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                track_hub()
                for path in container.rglob('mobile-proof.json'):
                    try:
                        value = json.loads(path.read_text())
                    except (OSError, ValueError):
                        continue
                    state = value.get('javascript') or {}
                    if state.get('phase') == 'failed':
                        raise RuntimeError('in-app proof failed: ' + json.dumps(value))
                    if state.get('phase') == phase and predicate(value):
                        return path.parent, value
                if console.poll() is not None:
                    raise RuntimeError('app launch exited before ' + phase)
                time.sleep(.1)
            raise TimeoutError('simulator action timed out: ' + phase)

        def screenshot(name, display):
            """Capture the owned simulator display after a compositing opportunity."""
            time.sleep(1)
            run('xcrun', 'simctl', 'io', device, 'screenshot', '--display=' + display, LOGS / (name + '.png'))

        launch('first')
        directory, value = wait_report('failure-ready')
        reports['initial'] = value
        destination = directory / 'ui-preferences-v1.json'
        sentinel = container / 'outside-preferences-sentinel'
        sentinel.write_bytes(b'OUTSIDE_SENTINEL')
        destination.symlink_to(sentinel)
        (directory / 'mobile-proof-control').write_text('fail')
        _, reports['failed_persistence'] = wait_report('failed-save-kept-draft')
        assert sentinel.read_bytes() == b'OUTSIDE_SENTINEL' and destination.is_symlink()
        destination.unlink()
        (directory / 'mobile-proof-control').write_text('continue')
        _, background = wait_report('background-ready')
        reports['before_background'] = background
        assert json.loads(destination.read_text()) == {'revision': 1, 'preferences': {'locale': 'fr', 'theme': 'dark'}}
        run('xcrun', 'simctl', 'launch', device, 'com.apple.Preferences')
        wait_report('background-ready', predicate=lambda value: value['suspended'] > background['suspended'])
        # Reopening an already-running app foregrounds the same WebView/root.
        run('xcrun', 'simctl', 'launch', device, APP_ID)
        wait_report('background-ready', predicate=lambda value: value['resumed'] > background['resumed'])
        (directory / 'mobile-proof-control').write_text('finish')
        _, reports['resumed'] = wait_report('complete')
        assert console.poll() is None
        stop()
        launch('restart')
        _, reports['restart'] = wait_report('complete')
        stop()
        # Recovery is also verified in an installed app, not merely native unit tests.
        for label, payload in [('truncated', b'{'), ('oversized', b'x' * 1025), ('corrupt', b'not-json')]:
            destination.write_bytes(payload)
            launch('recovery')
            _, reports[label] = wait_report('complete')
            assert destination.read_bytes() == payload
            stop()
        destination.unlink()
        destination.symlink_to(sentinel)
        launch('recovery')
        _, reports['symlink'] = wait_report('complete')
        assert sentinel.read_bytes() == b'OUTSIDE_SENTINEL' and destination.is_symlink()
        stop()
        # Finish lifecycle/restart checks headlessly before attaching the display.
        destination.unlink()
        destination.write_text(json.dumps({'revision': 1, 'preferences': {'locale': 'fr', 'theme': 'dark'}}))
        launch('restart')
        _, reports['final_visual_state'] = wait_report('complete')
        # Persist successful functional assertions independently of optional displays.
        (LOGS / 'ios-functional.json').write_text(json.dumps({'passed': True, 'device': device, 'reports': reports}, indent=2) + '\n')
        # Xcode 27.1 replaces Simulator.app with DeviceHub. Only this device is booted.
        assert not hub_pids(), 'An owner display appeared; stop without touching it'
        hub_info = plistlib.loads((hub / 'Contents/Info.plist').read_bytes())
        assert hub_info['CFBundleIdentifier'] == 'com.apple.dt.Devices'
        simulator = subprocess.Popen([str(hub / 'Contents/MacOS' / hub_info['CFBundleExecutable'])],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        record = json.loads(OWNED.read_text())
        record.update(display_started=True, display_pids=[simulator.pid])
        OWNED.write_text(json.dumps(record))
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            track_hub()
            time.sleep(1)
        assert track_hub(), 'DeviceHub exited before displaying the device'
        # The Duo's default is its secondary LCD. The installed app uses primary.
        run('xcrun', 'simctl', 'io', device, 'screenConfig', '--display=primary', 'power', 'on')
        # Enumeration/inspection established LCD 1 as the active app display.
        # Inactive Wireless/TVOut capture can stall; they are not app evidence.
        import re
        inventory = run('xcrun', 'simctl', 'io', device, 'enumerate')
        screens = re.findall(r'\((\d+)\) ([^:\n]+):', inventory)
        assert ('1', 'LCD') in screens
        (LOGS / 'ios-displays.json').write_text(json.dumps(screens, indent=2))
        screenshot('ios-app-lcd', '1')
        stop()
    finally:
        if console is not None:
            run('xcrun', 'simctl', 'terminate', device, APP_ID, allow_failure=True)
            try:
                console.wait(timeout=10)
            finally:
                if console.poll() is None:
                    console.kill()
                console.wait()
                console_log.close()
        try:
            cleanup(device)
        finally:
            if simulator is not None:
                close_hub(track_hub())
                # Reap our direct child. Re-executed LaunchServices children are
                # reaped by their OS parent; close_hub verifies their disappearance.
                simulator.wait(timeout=10)
    after = devices()
    assert device not in after and before <= after
    assert not booted_devices(), 'An unowned device is booted; do not shut it down'
    report = {'passed': True, 'device': device, 'name': name, 'runtime': RUNTIME,
              'device_type': DEVICE_TYPE, 'bundle': str(bundle), 'seconds': round(time.monotonic() - started, 2),
              'build_service_closed_before_launch': True, 'owned_device_deleted': True,
              'display': 'DeviceHub 27.1', 'evidence_display': '1 / LCD / primary', 'display_pids': json.loads(OWNED.read_text())['display_pids'],
              'display_exited': not hub_pids(),
              'preexisting_devices_preserved': True, 'reports': reports}
    (LOGS / 'ios-launch.json').write_text(json.dumps(report, indent=2) + '\n')


def main():
    """A parent watchdog survives worker failure and cleans the recorded device."""
    LOGS.mkdir(parents=True, exist_ok=True)
    if OWNED.exists():
        raise RuntimeError('owned-device record exists; inspect previous cleanup first')
    assert not hub_pids(), "DeviceHub is already running; stop without touching it"
    assert not booted_devices(), 'An owner device is booted; stop without touching it'
    with (LOGS / 'ios-launch.log').open('w') as log:
        child = subprocess.Popen([sys.executable, __file__, '--worker'], stdout=log,
                                 stderr=subprocess.STDOUT, start_new_session=True)
        try:
            result = child.wait(timeout=600)
        finally:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
            child.wait()
            if OWNED.exists():
                record = json.loads(OWNED.read_text())
                try:
                    if record['device'] in devices():
                        cleanup(record['device'])
                    assert record['device'] not in devices()
                finally:
                    close_hub(record.get('display_pids', []))
                OWNED.unlink()
    if result:
        raise RuntimeError('iOS proof failed; see ' + str(LOGS / 'ios-launch.log'))
    print('PASS installed iOS simulator app; owned device deleted')


if __name__ == '__main__':
    if '--manual-review' in sys.argv:
        import manual_ios
        manual_ios.main(sys.modules[__name__])
    else:
        worker() if '--worker' in sys.argv else main()
