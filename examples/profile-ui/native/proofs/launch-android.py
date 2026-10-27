#!/usr/bin/env python3
"""Finite real ARM64 emulator proof; own/reap one AVD, adb server and app."""
import json
import hashlib
import os
from pathlib import Path
import shlex
import signal
import socket
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[4]
NATIVE = ROOT / 'examples/profile-ui/native'
LOGS = Path(os.environ.get('PROFILE_UI_ANDROID_LOGS', '/private/tmp/s8-lane-b-android'))
APP = 'dev.stackpress.profile_m0'
IMAGE = 'system-images;android-37.0;google_apis;arm64-v8a'


def run(command, env, timeout=20, payload=None, allow_failure=False):
    """Bound/reap every direct child, including remote app-control commands."""
    with subprocess.Popen(list(map(str, command)), env=env, stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          start_new_session=True) as child:
        try:
            output, error = child.communicate(input=payload, timeout=timeout)
        finally:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
            child.wait()
    with (LOGS / 'android-actions.log').open('a') as log:
        log.write('$ ' + shlex.join(list(map(str, command))) + '\nexit=' + str(child.returncode) + '\n')
        if child.returncode:
            log.write(error.decode(errors='replace') + output.decode(errors='replace') + '\n')
    if child.returncode and not allow_failure:
        raise RuntimeError('command failed; inspect android-actions.log: ' + str(command))
    return output


def reap(child, timeout=15):
    """Graceful then forced shutdown applies only to our recorded direct child."""
    if child is None:
        return
    try:
        child.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()


def free_port():
    """Reserve an ephemeral loopback candidate for our isolated adb server."""
    with socket.socket() as candidate:
        candidate.bind(('127.0.0.1', 0))
        return candidate.getsockname()[1]


def verify_acceleration(emulator, env):
    """Require usable Hypervisor.framework before creating a server or device."""
    timed_out = False
    output_path = LOGS / 'android-acceleration.log'
    # Capture both streams even on failure; this preflight has its own deadline
    # and child group, independent of the later emulator lifecycle.
    with output_path.open('w') as log:
        with subprocess.Popen([str(emulator), '-accel-check'], env=env,
                              stdout=log, stderr=subprocess.STDOUT,
                              start_new_session=True) as child:
            try:
                child.wait(timeout=30)
            except subprocess.TimeoutExpired:
                timed_out = True
            finally:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                child.wait()
    output = output_path.read_text()
    usable = not timed_out and child.returncode == 0 and 'Hypervisor.Framework OS X Version' in output
    (LOGS / 'android-acceleration.json').write_text(json.dumps(dict(
        exit_code=child.returncode, timed_out=timed_out,
        usable=usable, output=output), indent=2) + '\n')
    # A successful process alone is insufficient: it must identify the approved
    # host hypervisor. Never retry here or silently permit software emulation.
    if not usable:
        raise RuntimeError('Hypervisor.Framework preflight failed; inspect android-acceleration.log')


def worker():
    """Install bundled APK, exercise real commands, capture pixels, then clean up."""
    LOGS.mkdir(parents=True, exist_ok=True)
    (LOGS / 'android-launch.json').unlink(missing_ok=True)
    (LOGS / 'android-functional.json').unlink(missing_ok=True)
    build = json.loads((LOGS / 'android-build.json').read_text())
    assert build['exit_code'] == 0 and build['closed_before_proof']
    assert build['owned_java_gradle_kotlin_processes_remaining'] == []
    for address in build['observed_addresses']:
        host, port = address.rsplit(':', 1)
        try:
            with socket.create_connection((host, int(port)), timeout=1):
                raise RuntimeError('build options socket remains open')
        except ConnectionRefusedError:
            pass
    assert not (NATIVE / 'gen/android/.tauri/cli-options-server.json').exists()
    sdk = Path(os.environ.get('ANDROID_HOME', Path.home() / 'Library/Android/sdk'))
    java = Path(os.environ.get('JAVA_HOME', Path.home() / 'Library/Java/JavaVirtualMachines/jdk-17.0.21+9/Contents/Home'))
    adb = sdk / 'platform-tools/adb'
    avdmanager = sdk / 'cmdline-tools/latest/bin/avdmanager'
    avd_home = NATIVE / 'gen/android-avds'
    name = 'Stackpress-LaneB-Android-' + uuid.uuid4().hex[:10]
    port = free_port()
    while port == 5037 or 3000 <= port <= 3021:
        port = free_port()
    env = dict(os.environ, JAVA_HOME=str(java), ANDROID_HOME=str(sdk),
               ANDROID_AVD_HOME=str(avd_home), ANDROID_USER_HOME=str(NATIVE / 'gen/android-user'),
               ANDROID_EMULATOR_HOME=str(NATIVE / 'gen/android-emulator'),
               ADB_SERVER_SOCKET='tcp:127.0.0.1:' + str(port), ANDROID_ADB_SERVER_PORT=str(port),
               ADB_USB='0', ADB_MDNS='0', ADB_MDNS_AUTO_CONNECT='0')
    verify_acceleration(sdk / 'emulator/emulator', env)
    apks = list((NATIVE / 'gen/android/app/build/outputs/apk').glob('**/*debug.apk'))
    assert len(apks) == 1, 'expected one ARM64 debug APK'
    server = emulator = None
    server_log = emulator_log = None
    created = False
    serial = None
    reports = {}
    started = time.monotonic()
    cleanup_errors = []
    bound_ports = set()
    owner_avds = avd_definitions()
    (LOGS / 'owner-avds-before.json').write_text(json.dumps(owner_avds, indent=2) + '\n')

    def remote(*arguments, **options):
        """All device commands select our own serial and private adb server."""
        assert serial is not None
        if server is None or server.poll() is not None:
            raise RuntimeError('owned adb server exited; do not auto-start another server')
        return run([adb, '-P', str(port), '-s', serial, *arguments], env, **options)

    def app_shell(script, **options):
        """Fixed proof paths stay in the installed app's private data directory."""
        return remote('exec-out', 'run-as', APP, 'sh', '-c', shlex.quote(script), **options)

    def stop():
        """Force-stop only our package on our disposable device, then verify exit."""
        remote('shell', 'am', 'force-stop', APP)
        assert not remote('shell', 'pidof', APP, allow_failure=True).strip()

    def launch(mode):
        """Select a debug-only closed fixture before the app process is launched."""
        stop()
        app_shell('rm -f mobile-proof.json mobile-proof-control')
        app_shell('cat > mobile-proof-mode', payload=mode.encode())
        remote('shell', 'am', 'start', '-W', '-n', APP + '/.MainActivity', timeout=30)

    def wait_report(phase, timeout=40, predicate=lambda value: True):
        """The in-app diagnostic is authoritative; never infer success from pixels."""
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            raw = remote('exec-out', 'run-as', APP, 'cat', 'mobile-proof.json', timeout=5, allow_failure=True)
            try:
                value = json.loads(raw)
            except ValueError:
                value = {}
            state = value.get('javascript') or {}
            if state.get('phase') == 'failed':
                raise RuntimeError('in-app Android assertion failed: ' + json.dumps(value))
            if state.get('phase') == phase and predicate(value):
                return value
            if emulator.poll() is not None:
                raise RuntimeError('owned emulator exited before ' + phase)
            time.sleep(.2)
        raise TimeoutError('Android phase timed out: ' + phase)

    def screenshot(label):
        """Capture the emulator framebuffer directly; inspect this file separately."""
        image = remote('exec-out', 'screencap', '-p', timeout=20)
        assert image.startswith(b'\x89PNG\r\n\x1a\n')
        (LOGS / (label + '.png')).write_bytes(image)

    try:
        # A foreground, private server avoids starting/reusing the owner's server.
        # A bare tcp:<port> listens locally in adb; this installed version rejects
        # an explicit listen hostname. Actual bindings are checked before any AVD.
        server_log = (LOGS / 'adb-server.log').open('w')
        server = subprocess.Popen([str(adb), '-L', 'tcp:' + str(port), 'server', 'nodaemon'],
                                  env=dict(env, ADB_TRACE='sockets'), stdout=server_log,
                                  stderr=subprocess.STDOUT)
        record = dict(name=name, adb_port=port, server_pid=server.pid)
        (LOGS / 'owned-android.json').write_text(json.dumps(record))
        # Arm cleanup of our unique path, including partially failed creation.
        assert not (avd_home / (name + '.ini')).exists() and not (avd_home / name).exists()
        created = True
        bound_ports = create_avd_after_adb_audit(server, port, avdmanager, env, name, avd_home)
        record['adb_verified_before_avd'] = True
        (LOGS / 'owned-android.json').write_text(json.dumps(record))
        # Console+transport ports are explicit; never connect to an owner's AVD.
        console_port = next(candidate for candidate in range(5600, 5682, 2)
                            if ports_available(candidate))
        serial = 'emulator-' + str(console_port)
        emulator_log = (LOGS / 'emulator.log').open('w')
        emulator = subprocess.Popen([str(sdk / 'emulator/emulator'), '-avd', name,
            '-port', str(console_port), '-no-window', '-no-audio', '-no-snapshot',
            '-no-metrics', '-no-boot-anim', '-gpu', 'swiftshader', '-memory', '2048', '-cores', '2',
            '-accel', 'on'],
            env=env, stdout=emulator_log, stderr=subprocess.STDOUT)
        record.update(emulator_pid=emulator.pid, emulator_ports=[console_port, console_port + 1])
        (LOGS / 'owned-android.json').write_text(json.dumps(record))
        deadline = time.monotonic() + 300
        while remote('shell', 'getprop', 'sys.boot_completed', timeout=5, allow_failure=True).strip() != b'1':
            if emulator.poll() is not None or time.monotonic() > deadline:
                raise TimeoutError('ARM64 emulator boot watchdog')
            time.sleep(1)
        bound_ports |= audit_listeners({server.pid, emulator.pid}, LOGS / 'android-proof-listeners.log')
        assert {console_port, console_port + 1} <= bound_ports
        remote('shell', 'input', 'keyevent', '82')
        remote('install', '-t', str(apks[0]), timeout=60)
        launch('first')
        reports['initial'] = wait_report('failure-ready')
        app_shell('printf OUTSIDE_SENTINEL > outside-preferences-sentinel; ln -s outside-preferences-sentinel ui-preferences-v1.json')
        app_shell('printf fail > mobile-proof-control')
        reports['failed_persistence'] = wait_report('failed-save-kept-draft')
        assert app_shell('cat outside-preferences-sentinel') == b'OUTSIDE_SENTINEL'
        app_shell('test -L ui-preferences-v1.json')
        screenshot('android-failed-persistence')
        app_shell('rm ui-preferences-v1.json; printf continue > mobile-proof-control')
        background = wait_report('background-ready')
        reports['before_background'] = background
        assert json.loads(app_shell('cat ui-preferences-v1.json')) == {'revision': 1, 'preferences': {'locale': 'fr', 'theme': 'dark'}}
        remote('shell', 'input', 'keyevent', 'KEYCODE_HOME')
        wait_report('background-ready', predicate=lambda value: value['suspended'] > background['suspended'])
        remote('shell', 'am', 'start', '-W', '-n', APP + '/.MainActivity')
        wait_report('background-ready', predicate=lambda value: value['resumed'] > background['resumed'])
        app_shell('printf finish > mobile-proof-control')
        reports['resumed'] = wait_report('complete')
        screenshot('android-resumed')
        launch('restart')
        reports['restart'] = wait_report('complete')
        screenshot('android-restart')
        stop()
        # Damaged inputs must remain untouched; recovery is an explicit UI fallback.
        for label, payload in [('truncated', b'{'), ('oversized', b'x' * 1025), ('corrupt', b'not-json')]:
            app_shell('cat > ui-preferences-v1.json', payload=payload)
            launch('recovery')
            reports[label] = wait_report('complete')
            assert app_shell('cat ui-preferences-v1.json') == payload
            stop()
        app_shell('rm ui-preferences-v1.json; ln -s outside-preferences-sentinel ui-preferences-v1.json')
        launch('recovery')
        reports['symlink'] = wait_report('complete')
        assert app_shell('cat outside-preferences-sentinel') == b'OUTSIDE_SENTINEL'
        app_shell('test -L ui-preferences-v1.json')
        screenshot('android-recovery')
        (LOGS / 'android-functional.json').write_text(json.dumps(dict(passed=True, reports=reports), indent=2) + '\n')
        print('PASS real Android functional assertions and framebuffer captures', flush=True)
    except BaseException as error:
        # Preserve the one attempt's failure before cleanup removes its device.
        # Diagnostic reads neither restart the app nor attempt a different setup.
        (LOGS / 'android-failure.json').write_text(json.dumps(dict(
            error_type=type(error).__name__, error=str(error), name=name,
            adb_port=port, emulator_launched=emulator is not None), indent=2) + '\n')
        if emulator is not None and server is not None and server.poll() is None:
            try:
                output = remote('logcat', '-d', '-t', '200', timeout=5, allow_failure=True)
                (LOGS / 'android-failure-logcat.log').write_bytes(output)
            except (OSError, RuntimeError, subprocess.SubprocessError):
                pass
        raise
    finally:
        # Every cleanup phase runs even when an earlier phase fails. Never select
        # a generic emulator/Java PID or call the owner's default adb server.
        if emulator is not None and serial is not None:
            for command in [('shell', 'am', 'force-stop', APP), ('emu', 'kill')]:
                try:
                    remote(*command, timeout=10, allow_failure=True)
                except (OSError, RuntimeError, subprocess.SubprocessError) as error:
                    cleanup_errors.append(str(error))
            try:
                reap(emulator)
            except (OSError, subprocess.SubprocessError) as error:
                cleanup_errors.append(str(error))
        if server is not None:
            try:
                run([adb, '-P', str(port), 'kill-server'], env, timeout=10, allow_failure=True)
            except (OSError, subprocess.SubprocessError) as error:
                cleanup_errors.append(str(error))
            finally:
                try:
                    reap(server)
                except (OSError, subprocess.SubprocessError) as error:
                    cleanup_errors.append(str(error))
        for log in [emulator_log, server_log]:
            if log is not None:
                log.close()
        avd_kib = None
        if created and (avd_home / (name + '.ini')).exists():
            try:
                avd_kib = int(run(['du', '-sk', str(avd_home / name)], env, timeout=5).split()[0])
            except (OSError, RuntimeError, ValueError, subprocess.SubprocessError) as error:
                cleanup_errors.append(str(error))
            try:
                run([avdmanager, 'delete', 'avd', '--name', name], env, timeout=30)
            except (OSError, RuntimeError, subprocess.SubprocessError) as error:
                cleanup_errors.append(str(error))
        (LOGS / 'android-cleanup.json').write_text(json.dumps(dict(
            name=name, adb_port=port, adb_reaped=server is None or server.poll() is not None,
            emulator_launched=emulator is not None,
            emulator_reaped=emulator is None or emulator.poll() is not None,
            avd_kib_before_delete=avd_kib,
            avd_deleted=not (avd_home / (name + '.ini')).exists() and not (avd_home / name).exists(),
            owner_avd_definitions_unchanged=avd_definitions() == owner_avds,
            cleanup_errors=cleanup_errors), indent=2) + '\n')
        if cleanup_errors:
            raise RuntimeError('cleanup actions failed: ' + repr(cleanup_errors))
    assert not (avd_home / (name + '.ini')).exists()
    assert not (avd_home / name).exists()
    try:
        with socket.create_connection(('127.0.0.1', port), timeout=1):
            raise RuntimeError('owned adb server socket remains')
    except ConnectionRefusedError:
        pass
    for candidate in bound_ports | {console_port, console_port + 1}:
        try:
            with socket.create_connection(('127.0.0.1', candidate), timeout=1):
                raise RuntimeError('owned emulator port remains: ' + str(candidate))
        except ConnectionRefusedError:
            pass
    assert avd_definitions() == owner_avds, 'owner AVD definitions changed'
    (LOGS / 'android-launch.json').write_text(json.dumps(dict(passed=True, name=name, image=IMAGE,
        serial=serial, emulator_ports=[console_port, console_port + 1], adb_server_started=True, private_adb_port=port, adb_server_reaped=True,
        observed_tcp_listener_ports=sorted(bound_ports), owner_avd_definitions_unchanged=True,
        emulator_reaped=True, avd_deleted=True, seconds=round(time.monotonic() - started, 2), reports=reports), indent=2) + '\n')


def avd_definitions():
    """Hash owner definitions read-only; all proof mutations use another home."""
    return {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in (Path.home() / '.android/avd').glob('*.ini')}


def create_avd_after_adb_audit(server, port, avdmanager, env, name, avd_home):
    """A verified private listener is required before creating even a disposable AVD."""
    deadline = time.monotonic() + 30
    try:
        while True:
            if server.poll() is not None:
                raise RuntimeError('private adb server exited before binding audit')
            # An empty early inventory can settle; an unsafe address never retries.
            ports = audit_listeners({server.pid}, LOGS / 'adb-preflight-listeners.log', allow_empty=True)
            if ports:
                if ports != {port}:
                    raise RuntimeError('unexpected private adb listener ports: ' + repr(ports))
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('private adb listener readiness deadline')
            time.sleep(.1)
    except BaseException:
        # Stop an unsafe/failed server immediately, without waiting for device
        # cleanup or trying another listen syntax. Only this child is signaled.
        if server.poll() is None:
            server.terminate()
        reap(server, timeout=2)
        raise
    avd_home.mkdir(parents=True, exist_ok=True)
    run([avdmanager, 'create', 'avd', '--name', name, '--package', IMAGE,
         '--path', str(avd_home / name), '--device', 'pixel_9'], env, timeout=60, payload=b'no\n')
    return ports


def audit_listeners(pids, destination, allow_empty=False):
    """Reject non-loopback TCP listeners; only readiness polling permits an empty set."""
    rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid='], text=True, timeout=5)
    pairs = [tuple(map(int, row.split())) for row in rows.splitlines() if row.strip()]
    for _ in range(12):
        pids.update(pid for pid, parent in pairs if parent in pids)
    result = subprocess.run(['lsof', '-nP', '-a', '-p', ','.join(map(str, sorted(pids))),
                             '-iTCP', '-sTCP:LISTEN', '-Fpcn'], capture_output=True, text=True, timeout=5)
    destination.write_text(result.stdout)
    ports = set()
    for line in result.stdout.splitlines():
        if line.startswith('n'):
            address = line[1:]
            assert address.startswith(('127.0.0.1:', '[::1]:')), 'non-loopback proof listener: ' + address
            port = int(address.rsplit(':', 1)[1])
            assert port != 5037 and not 3000 <= port <= 3021
            ports.add(port)
    assert ports or allow_empty, 'no owned emulator/adb listeners observed'
    return ports


def ports_available(port):
    """Select an unused emulator console/transport pair without touching others."""
    sockets = []
    try:
        for candidate in [port, port + 1]:
            probe = socket.socket()
            sockets.append(probe)
            probe.bind(('127.0.0.1', candidate))
        return True
    except OSError:
        return False
    finally:
        for probe in sockets:
            probe.close()


if __name__ == '__main__':
    if '--worker' in sys.argv:
        def cancel(signum, frame):
            """Convert the parent deadline into stack unwinding and owned cleanup."""
            raise TimeoutError('parent Android watchdog expired')
        signal.signal(signal.SIGTERM, cancel)
        worker()
    else:
        # The parent remains alive to kill/reap the whole matrix on a hard hang.
        LOGS.mkdir(parents=True, exist_ok=True)
        with subprocess.Popen([sys.executable, __file__, '--worker'], start_new_session=True) as child:
            try:
                code = child.wait(timeout=600)
            finally:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGTERM)
                    try:
                        child.wait(timeout=150)
                    except subprocess.TimeoutExpired:
                        os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            raise SystemExit(code)
