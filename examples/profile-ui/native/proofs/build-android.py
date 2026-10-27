#!/usr/bin/env python3
"""Bounded Android debug build using installed SDK components and bundled assets."""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import time

ROOT = Path(__file__).resolve().parents[4]
NATIVE = ROOT / 'examples/profile-ui/native'
PROJECT = NATIVE / 'gen/android'
LOGS = Path(os.environ.get('PROFILE_UI_ANDROID_LOGS', '/private/tmp/s8-lane-b-android'))
TARGET = ROOT / 'target/s8-android-native'
WASM = ROOT / 'target/s8-android-wasm'


def run(name, command, env, cwd=ROOT, timeout=900):
    """Own/reap a child group; a timeout or installer request fails the build."""
    with (LOGS / (name + '.log')).open('w') as log:
        with subprocess.Popen(command, cwd=cwd, env=env, stdout=log,
                              stderr=subprocess.STDOUT, start_new_session=True) as child:
            try:
                code = child.wait(timeout=timeout)
            finally:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                child.wait()
        if code or (LOGS / 'INSTALL-ATTEMPT.txt').exists():
            raise RuntimeError(name + ' failed; inspect ' + str(LOGS / (name + '.log')))
    print('PASS ' + name, flush=True)


def environment():
    """Keep SDK selection, caches, debug keys and installer guards task-local."""
    LOGS.mkdir(parents=True, exist_ok=True)
    guard = LOGS / 'no-install-bin'
    guard.mkdir(exist_ok=True)
    for name in ['rustup', 'sdkmanager', 'brew']:
        real = shutil.which(name)
        script = '''#!/usr/bin/env python3
import os,sys
from pathlib import Path
if %r == 'sdkmanager' or any(arg in {'install','reinstall','upgrade','update','--install','--licenses','add','remove','default','override','self'} for arg in sys.argv[1:]):
 Path(%r).write_text(%r+' '+repr(sys.argv[1:]))
 sys.exit(97)
if not %r: sys.exit(98)
os.execv(%r,[%r]+sys.argv[1:])
''' % (name, str(LOGS / 'INSTALL-ATTEMPT.txt'), name, real, real, real)
        (guard / name).write_text(script)
        (guard / name).chmod(0o755)
    # No global Java/SDK/Gradle/Android setting is modified by these overrides.
    sdk = Path(os.environ.get('ANDROID_HOME', Path.home() / 'Library/Android/sdk'))
    java = Path(os.environ.get('JAVA_HOME', Path.home() / 'Library/Java/JavaVirtualMachines/jdk-17.0.21+9/Contents/Home'))
    ndk = Path(os.environ.get('NDK_HOME', sdk / 'ndk/29.0.13846066'))
    for path in [java / 'bin/java', ndk / 'source.properties',
                 sdk / 'platforms/android-37.0/android.jar', sdk / 'build-tools/36.0.0/aapt2']:
        if not path.is_file():
            raise RuntimeError('required installed tool is missing: ' + str(path))
    return dict(os.environ, PATH=str(guard) + os.pathsep + str(java / 'bin') + os.pathsep + os.environ['PATH'],
                JAVA_HOME=str(java), ANDROID_HOME=str(sdk), NDK_HOME=str(ndk),
                ANDROID_USER_HOME=str(NATIVE / 'gen/android-user'),
                GRADLE_USER_HOME=str(PROJECT / '.gradle-user-home'),
                CARGO_TARGET_DIR=str(TARGET), PROFILE_UI_TARGET=str(WASM),
                CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')


def configure(env):
    """Regenerate ignored scaffolding, then pin installed tools and local signing."""
    if not (PROJECT / 'app/build.gradle.kts').is_file():
        run('android-init-help', ['cargo', 'tauri', 'android', 'init', '--help'], env, NATIVE, 30)
        if '--skip-targets-install' not in (LOGS / 'android-init-help.log').read_text():
            raise RuntimeError('CLI lacks required no-target-installs flag')
        run('android-init', ['cargo', 'tauri', 'android', 'init', '--ci', '--skip-targets-install'], env, NATIVE, 120)
    properties = PROJECT / 'gradle.properties'
    text = properties.read_text()
    marker = '# Stackpress bounded local proof build'
    text = text.split(marker)[0]
    # SDK auto-download is disabled inside AGP itself, beyond the PATH guard.
    text += '\n' + marker + '''
android.builder.sdkDownload=false
org.gradle.daemon=false
org.gradle.vfs.watch=false
org.gradle.workers.max=2
kotlin.compiler.execution.strategy=in-process
'''
    properties.write_text(text)
    # buildSrc is a distinct included build; root project properties do not
    # reliably select its Kotlin compilation strategy.
    (PROJECT / 'buildSrc/gradle.properties').write_text('''org.gradle.daemon=false
kotlin.compiler.execution.strategy=in-process
org.gradle.workers.max=2
''')
    gradle = PROJECT / 'app/build.gradle.kts'
    text = gradle.read_text()
    marker = '// Stackpress installed SDK and disposable debug key'
    if marker not in text:
        text = text.replace('android {', '''android {
    // Stackpress installed SDK and disposable debug key
    buildToolsVersion = "36.0.0"
    ndkVersion = "29.0.13846066"
    signingConfigs.getByName("debug") {
        storeFile = rootProject.file("stackpress-debug.keystore")
    }''', 1)
        gradle.write_text(text)
    marker = '// Stackpress generates only a disposable local debug key'
    if marker not in text:
        # AGP auto-creates only its global default key. A custom path instead
        # needs an explicit debug-only Gradle task; it never reads owner keys.
        text += '\n' + marker + '''
val prepareStackpressDebugKey = tasks.register<Exec>("prepareStackpressDebugKey") {
    val debugKey = rootProject.file("stackpress-debug.keystore")
    onlyIf { !debugKey.exists() }
    commandLine(
        file(System.getenv("JAVA_HOME") + "/bin/keytool"),
        "-genkeypair", "-keystore", debugKey.absolutePath, "-storetype", "JKS",
        "-storepass", "android", "-keypass", "android", "-alias", "androiddebugkey",
        "-dname", "CN=Android Debug,O=Android,C=US", "-keyalg", "RSA",
        "-keysize", "2048", "-validity", "10000", "-noprompt"
    )
}
tasks.matching { it.name.startsWith("validateSigning") && it.name.endsWith("Debug") }
    .configureEach { dependsOn(prepareStackpressDebugKey) }
'''
        gradle.write_text(text)
    assert 'android.builder.sdkDownload=false' in properties.read_text()
    assert 'storeFile = rootProject.file("stackpress-debug.keystore")' in text
    root_gradle = PROJECT / 'build.gradle.kts'
    text = root_gradle.read_text()
    marker = '// Stackpress uses the installed API 37 SDK for the Tauri library too'
    if marker not in text:
        # Tauri's app template targets 37, while its library script pins 36.
        # Set the library's DSL after script evaluation, before AGP finalizes it.
        text += '\n' + marker + '''
subprojects {
    afterEvaluate {
        if (name == "tauri-android") {
            extensions.configure<com.android.build.gradle.LibraryExtension> {
                compileSdk = 37
                buildToolsVersion = "36.0.0"
            }
        }
    }
}
'''
        root_gradle.write_text(text)


def processes():
    """Read identities privately; process arguments never enter the public receipt."""
    rows = subprocess.check_output(['ps', '-ww', '-axo', 'pid=,ppid=,comm=,args='], text=True, timeout=5)
    return [row.strip().split(None, 3) for row in rows.splitlines() if row.strip()]


def listeners(pids):
    """Observe owned TCP/UDP endpoints; never connect to or stop another build."""
    if not pids:
        return ''
    result = subprocess.run(['lsof', '-nP', '-a', '-p', ','.join(map(str, sorted(pids))),
                             '-i', '-Fpcn'], capture_output=True, text=True, timeout=5)
    return result.stdout


def build(env):
    """Observe the build-only options service and require closure before proof."""
    run('android-csr-build', ['python3', 'examples/profile-ui/build-client.py', 'csr'],
        dict(env, CARGO_TARGET_DIR=str(WASM)))
    bundle = NATIVE / 'gen/csr-bundle'
    if bundle.exists():
        shutil.rmtree(bundle)
    shutil.copytree(ROOT / 'examples/profile-ui/dist/csr', bundle)
    shutil.copyfile(NATIVE / 'native-bootstrap.js', bundle / 'start.js')
    config = json.loads((NATIVE / 'tauri.android.conf.json').read_text())
    assert config['build']['devUrl'] is None and config['build']['frontendDist'] == 'gen/csr-bundle'
    options = PROJECT / '.tauri/cli-options-server.json'
    if options.exists():
        raise RuntimeError('stale build service descriptor; inspect before proceeding')
    observed = set()
    owned = set()
    endpoints = set()
    last_scan = 0
    start = time.monotonic()
    command = ['cargo', 'tauri', 'android', 'build', '--ci', '--debug', '--target', 'aarch64', '--apk', '--features', 'app', '--', '--locked']
    with (LOGS / 'android-build.log').open('w') as log:
        with subprocess.Popen(command, cwd=NATIVE, env=env, stdout=log,
                              stderr=subprocess.STDOUT, start_new_session=True) as child:
            try:
                while child.poll() is None:
                    if (LOGS / 'INSTALL-ATTEMPT.txt').exists():
                        raise RuntimeError('installer blocked; stop for review')
                    if time.monotonic() - start > 1200:
                        raise TimeoutError('Android build watchdog')
                    if options.exists():
                        try:
                            observed.add(json.loads(options.read_text())['addr'])
                        except (OSError, ValueError, KeyError):
                            pass
                    # Descendant ownership is established from our parent PID,
                    # not from a generic java/Gradle process-name match.
                    if time.monotonic() - last_scan >= 2:
                        rows = processes()
                        owned.add(child.pid)
                        for _ in range(12):
                            owned.update(int(row[0]) for row in rows if int(row[1]) in owned)
                        # Gradle may detach/reparent its single-use daemon before
                        # a scan. Its private distribution/cache path also proves
                        # ownership; no global Java name match grants ownership.
                        owned.update(int(row[0]) for row in rows
                                     if str(PROJECT / '.gradle-user-home') in row[3]
                                     and Path(row[2]).name == 'java')
                        endpoints.add(listeners(owned))
                        last_scan = time.monotonic()
                    time.sleep(.1)
                code = child.returncode
            finally:
                if child.poll() is None:
                    os.killpg(child.pid, signal.SIGKILL)
                child.wait()
                # This also runs after a watchdog/observer failure. Tool-directed
                # shutdown is scoped by our private Gradle home, never global.
                run('android-gradle-stop', [str(PROJECT / 'gradlew'), '--stop'], env, PROJECT, 45)
    remaining = [row for row in processes()
                 if (int(row[0]) in owned or str(PROJECT) in row[3]) and
                 any(name in Path(row[2]).name.lower() for name in ['java', 'gradle', 'kotlin'])]
    (LOGS / 'android-build-sockets.log').write_text('\n'.join(sorted(endpoints)))
    if remaining:
        raise RuntimeError('owned Java/Gradle/Kotlin process remains after tooling stop')
    # Store dependency coordinates, never service tokens or signing material.
    cache = PROJECT / '.gradle-user-home/caches/modules-2/files-2.1'
    coordinates = sorted('/'.join(path.relative_to(cache).parts) for path in cache.glob('*/*/*') if path.is_dir())
    (LOGS / 'android-dependencies.json').write_text(json.dumps(coordinates, indent=2) + '\n')
    for address in observed:
        host, port = address.rsplit(':', 1)
        try:
            with socket.create_connection((host, int(port)), timeout=1):
                raise RuntimeError('build options service still active')
        except ConnectionRefusedError:
            pass
    report = dict(exit_code=code, build_options_service_ran=bool(observed),
                  observed_addresses=sorted(observed), closed_before_proof=not options.exists(),
                  seconds=round(time.monotonic() - start, 2), debug_only=True,
                  gradle_cache=str(PROJECT / '.gradle-user-home'), dependencies=len(coordinates))
    report['owned_java_gradle_kotlin_processes_remaining'] = []
    report['socket_observation'] = 'android-build-sockets.log'
    (LOGS / 'android-build.json').write_text(json.dumps(report, indent=2) + '\n')
    if code or not observed or options.exists():
        raise RuntimeError('Android build failed; inspect android-build.log')
    print('PASS Android build; options service closed', flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--init-only', action='store_true')
    arguments = parser.parse_args()
    env = environment()
    configure(env)
    if not arguments.init_only:
        build(env)
