"""Listener-free tests of packaging isolation and manual simulator ownership policy."""
import contextlib
import io
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import MagicMock, Mock, patch
import manual_ios
import review_assets

spec=importlib.util.spec_from_file_location('build_ios',Path(__file__).with_name('build-ios.py'))
build_ios=importlib.util.module_from_spec(spec);spec.loader.exec_module(build_ios)


class Packaging(unittest.TestCase):
    """Use real temporary trees to verify shipped bytes and stale staging removal."""
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.source=self.root/'source';self.source.mkdir()
        for name,value in {'index.html':'<body><div id="app"></div></body>',
                           'start.js':'ordinary','profile_ui.js':'module','profile_ui_bg.wasm':'wasm'}.items():
            (self.source/name).write_text(value)
        self.admin=self.root/'admin';self.admin.mkdir()
        (self.admin/'admin.js').write_text('compiled exact CSR module')
        (self.admin/'admin_bg.wasm').write_bytes(b'exact compiled WASM')
        (self.admin/'start.js').write_text('MUTATING WEB BOOTSTRAP MUST NOT SHIP')
        self.destination=self.root/'bundle'

    def test_variants_exact_assets_no_transport_and_normal_rebuild(self):
        for variant in ('plugin','disabled'):
            review_assets.prepare(self.source,self.admin,self.destination,variant)
            report=review_assets.verify(self.destination,variant)
            self.assertEqual(report['variant'],variant)
            self.assertEqual((self.admin/'admin_bg.wasm').read_bytes(),(self.destination/'admin-review/admin_bg.wasm').read_bytes())
            self.assertNotIn('MUTATING', (self.destination/'admin-review/start.js').read_text())
            self.assertNotIn('admin-seed', (self.destination/'admin-review.html').read_text())
            self.assertIn('not installed admin CRUD',(self.destination/'admin-review.html').read_text())
            with self.assertRaises(ValueError):review_assets.assert_clean(self.destination)
            with self.assertRaises(ValueError):review_assets.verify(self.destination,'disabled' if variant=='plugin' else 'plugin')
            (self.destination/'admin-review/admin_bg.wasm').write_bytes(b'changed')
            with self.assertRaises(ValueError):review_assets.verify(self.destination,variant)
        review_assets.ordinary(self.source,self.destination)
        review_assets.assert_clean(self.destination)
        self.assertFalse((self.destination/'admin-review.html').exists())

    def test_refuse_release_unknown_variant_and_ordinary_contamination(self):
        for variant,profile in [('plugin','release'),('unknown','debug')]:
            with self.assertRaises(ValueError):review_assets.prepare(self.source,self.admin,self.destination,variant,profile)
        self.assertFalse(self.destination.exists())
        (self.source/'profile_ui_bg.wasm').write_bytes(review_assets.MARKER)
        with self.assertRaises(ValueError):review_assets.ordinary(self.source,self.destination)
        (self.source/'profile_ui_bg.wasm').write_bytes(b'normal')
        (self.source/'bad').symlink_to(self.admin/'admin.js')
        with self.assertRaises(ValueError):review_assets.assert_clean(self.source)
        (self.source/'bad').unlink()
        self.destination.symlink_to(self.admin,target_is_directory=True)
        with self.assertRaises(ValueError):review_assets.ordinary(self.source,self.destination)

    def test_build_selection_pairs_feature_and_frontend_without_a_dev_url(self):
        with patch.object(build_ios,'LOGS',self.root):
            ordinary=build_ios.build_command()
            self.assertEqual(ordinary[ordinary.index('--features')+1],'app')
            self.assertNotIn('--config',ordinary)
            for variant in ('plugin','disabled'):
                command=build_ios.build_command(variant,self.destination)
                self.assertEqual(command[command.index('--features')+1],'app,review-preview')
                config=json.loads((self.root/'review-config.json').read_text())
                self.assertEqual(config['build'],{'frontendDist':str(self.destination),'devUrl':None})
                self.assertIn('--debug',command);self.assertNotIn('--release',command)
            with self.assertRaises(ValueError):build_ios.build_command('other',self.destination)

    def test_build_budget_retries_and_reaps_a_child_on_failed_measurement(self):
        failed=types.SimpleNamespace(returncode=1,stdout='',stderr='transient removal')
        passed=types.SimpleNamespace(returncode=0,stdout='12 target',stderr='')
        oversized=types.SimpleNamespace(returncode=0,stdout=str(8*1024*1024+1)+' target',stderr='')
        with patch.object(build_ios,'TARGET',self.root),patch.object(build_ios.subprocess,'run',side_effect=[failed,passed]) as measure:
            build_ios.budget();self.assertEqual(measure.call_count,2)
        with patch.object(build_ios,'TARGET',self.root),patch.object(build_ios.subprocess,'run',return_value=oversized):
            with self.assertRaisesRegex(RuntimeError,'exceeded'):build_ios.budget()
        child=MagicMock();child.__enter__.return_value=child;child.poll.return_value=None
        with patch.object(build_ios,'LOGS',self.root),patch.object(build_ios.subprocess,'Popen',return_value=child), \
             patch.object(build_ios,'budget',side_effect=RuntimeError('measurement failed')), \
             patch.object(build_ios.os,'killpg') as kill:
            with self.assertRaisesRegex(RuntimeError,'measurement failed'):
                build_ios.run('bounded',['never-executed'],{})
        kill.assert_called_once_with(child.pid,build_ios.signal.SIGKILL);child.wait.assert_called_once()


class Manual(unittest.TestCase):
    """Replace only platform effects, retaining real manifests, control files and cleanup order."""
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        root=Path(self.temp.name);bundle=root/'app';bundle.mkdir()
        (bundle/'Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'test.app'}))
        hub=root/'DeviceHub.app';(hub/'Contents').mkdir(parents=True)
        (hub/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleIdentifier':'com.apple.dt.Devices','CFBundleExecutable':'DeviceHub'}))
        self.host=types.SimpleNamespace(ROOT=root,LOGS=root,OWNED=root/'owned.json',APP_ID='test.app',
            RUNTIME='installed-runtime',DEVICE_TYPE='installed-device',socket=types.SimpleNamespace(create_connection=Mock(side_effect=ConnectionRefusedError)),
            devices=Mock(return_value=set()),booted_devices=Mock(return_value=set()),hub_pids=Mock(return_value=set()),
            display_bundle=Mock(return_value=hub),run=Mock(return_value='00000000-0000-0000-0000-000000000001'),
            track_hub=Mock(return_value=[42]),close_hub=Mock(),cleanup=Mock(),__file__='launch-ios.py')
        (root/'ios-build.json').write_text(json.dumps(dict(exit_code=0,closed_before_proof=True,
            review_variant='disabled',bundle=str(bundle),observed_addresses=['127.0.0.1:12345'])))
        self.args=manual_ios.options(['--manual-review','--variant','disabled','--duration-seconds','30'])
        self.child=Mock(pid=42);self.child.poll.return_value=None;self.child.wait.return_value=0
        self.child.returncode=0

    def test_options_and_one_shot_control(self):
        for duration in ('0','29','7201'):
            with contextlib.redirect_stderr(io.StringIO()),self.assertRaises(SystemExit):
                manual_ios.options(['--manual-review','--variant','disabled','--duration-seconds',duration])
        self.assertIsNone(manual_ios.control(self.host.LOGS))
        manual_ios.control(self.host.LOGS,'restart')
        with self.assertRaises(FileExistsError):manual_ios.control(self.host.LOGS,'stop')
        self.assertEqual(manual_ios.control(self.host.LOGS),'restart')
        with self.assertRaises(ValueError):manual_ios.control(self.host.LOGS,'arbitrary')
        path=self.host.LOGS/'manual-control';path.write_text('arbitrary')
        with self.assertRaises(ValueError):manual_ios.control(self.host.LOGS)
        path.symlink_to(self.host.LOGS/'ios-build.json')
        with self.assertRaises(ValueError):manual_ios.control(self.host.LOGS)

    def test_display_waits_for_running_app(self):
        # The display attaches only after launchctl lists the app; a dead console stops first.
        listings=iter(['', '123\t0\tUIKitApplication:test.app[ab12]'])
        self.host.run=Mock(side_effect=lambda *arguments,**_:next(listings))
        with patch.object(manual_ios.time,'sleep'):
            manual_ios.wait_until_running(self.host,'device',self.child)
        self.assertEqual(self.host.run.call_count,2)
        self.child.poll.return_value=1
        with self.assertRaisesRegex(RuntimeError,'console exited'):
            manual_ios.wait_until_running(self.host,'device',self.child)
        self.child.poll.return_value=None
        self.host.run=Mock(return_value='')
        with patch.object(manual_ios.time,'sleep'),patch.object(manual_ios.time,'monotonic',side_effect=[0,0,61]):
            with self.assertRaisesRegex(RuntimeError,'did not start'):
                manual_ios.wait_until_running(self.host,'device',self.child)

    def test_interactive_restart_stop_and_cleanup(self):
        uuid='00000000-0000-0000-0000-000000000001'
        # launchctl reports the running app; every other simctl call returns the fake device.
        self.host.run=Mock(side_effect=lambda *arguments,**_:
            'UIKitApplication:test.app' if 'launchctl' in arguments else uuid)
        with patch.dict(os.environ,PROFILE_UI_REVIEW_DEADLINE='999999999999'), \
             patch.object(manual_ios.subprocess,'Popen',return_value=self.child) as spawn, \
             patch.object(manual_ios,'track_display',return_value=[42]), \
             patch.object(manual_ios,'control',side_effect=['restart','stop']), \
             patch.object(manual_ios.time,'sleep'):
            manual_ios.session(self.host,self.args)
        self.assertEqual(spawn.call_count,3) # first app, owned display, restarted app
        self.host.cleanup.assert_called_once()
        self.host.close_hub.assert_called_once_with([42])
        self.assertEqual(json.loads((self.host.LOGS/'manual-session.json').read_text())['variant'],'disabled')
        for call in spawn.call_args_list:
            if 'env' in call.kwargs:self.assertEqual(call.kwargs['env']['SIMCTL_CHILD_PROFILE_UI_MANUAL_REVIEW'],'1')

    def test_failed_boot_still_deletes_owned_device(self):
        self.host.run.side_effect=['00000000-0000-0000-0000-000000000001',RuntimeError('boot failed')]
        with self.assertRaisesRegex(RuntimeError,'boot failed'):manual_ios.session(self.host,self.args)
        self.host.cleanup.assert_called_once()
        self.host.close_hub.assert_not_called()

    def test_mismatch_and_existing_owner_never_create_device(self):
        self.args.variant='plugin'
        with self.assertRaises(ValueError):manual_ios.session(self.host,self.args)
        self.args.variant='disabled';self.host.hub_pids.return_value={7}
        with self.assertRaises(RuntimeError):manual_ios.session(self.host,self.args)
        self.host.run.assert_not_called();self.host.cleanup.assert_not_called()

    def test_fallback_and_console_failure_do_not_skip_cleanup(self):
        self.host.OWNED.write_text(json.dumps(dict(device='owned',display_pids=[42])))
        self.host.devices.return_value={'owned'};self.host.cleanup.side_effect=RuntimeError('delete failed')
        with self.assertRaisesRegex(RuntimeError,'delete failed'):manual_ios.recover(self.host)
        self.host.close_hub.assert_called_once_with([42]);self.assertTrue(self.host.OWNED.exists())
        self.host.cleanup.side_effect=None;self.host.devices.side_effect=[{'owned'},set()]
        manual_ios.recover(self.host);self.assertFalse(self.host.OWNED.exists())
        manual_ios.recover(self.host)
        self.host.run.side_effect=RuntimeError('terminate failed')
        self.child.wait.side_effect=[subprocess.TimeoutExpired('console',10),0]
        with self.assertRaisesRegex(RuntimeError,'terminate failed'):manual_ios.stop_console(self.host,'owned',self.child)
        self.child.kill.assert_called_once()

    def test_parent_watchdog_reaps_worker_and_recovers(self):
        self.child.wait.side_effect=[subprocess.TimeoutExpired('worker',30),0]
        arguments=['launch-ios.py','--manual-review','--variant','disabled','--duration-seconds','30']
        with patch.object(sys,'argv',arguments),patch.object(manual_ios.subprocess,'Popen',return_value=self.child), \
             patch.object(manual_ios.os,'killpg') as kill,patch.object(manual_ios,'recover') as recover, \
             patch.object(manual_ios.signal,'signal'):
            manual_ios.main(self.host)
        kill.assert_called_once_with(42,manual_ios.signal.SIGTERM);recover.assert_called_once_with(self.host)
        self.assertTrue((self.host.LOGS/'manual-cleanup.json').is_file())

    def test_display_tracking_does_not_claim_an_unrelated_display(self):
        self.host.OWNED.write_text(json.dumps(dict(device='owned',display_started=True,display_pids=[42])))
        self.host.hub_pids.return_value={42,43,99}
        with patch.object(manual_ios.subprocess,'check_output',return_value='42 1\n43 42\n99 1\n'):
            self.assertEqual(manual_ios.track_display(self.host),[42,43])
        self.assertNotIn(99,json.loads(self.host.OWNED.read_text())['display_pids'])

    def test_control_applies_only_to_the_matching_owned_manual_session(self):
        arguments=['launch-ios.py','--manual-review','--variant','disabled','--control','restart']
        with patch.object(sys,'argv',arguments):
            with self.assertRaises(RuntimeError):manual_ios.main(self.host)
            self.host.OWNED.write_text(json.dumps(dict(device='owned',variant='plugin')))
            with self.assertRaises(ValueError):manual_ios.main(self.host)
            self.host.OWNED.write_text(json.dumps(dict(device='owned',variant='disabled')))
            manual_ios.main(self.host)
        self.assertEqual(manual_ios.control(self.host.LOGS),'restart')
        self.host.run.assert_not_called()

    def test_parent_recovers_interrupted_creation_by_unique_name_only(self):
        record=dict(device=None,name='unique-owned-name',before=['owner-device'],display_pids=[])
        self.host.OWNED.write_text(json.dumps(record))
        self.host.run.return_value=json.dumps({'devices':{'runtime':[
            {'udid':'owner-device','name':'unique-owned-name'},
            {'udid':'our-device','name':'unique-owned-name'},
            {'udid':'unrelated','name':'unrelated-name'}]}})
        self.host.devices.side_effect=[{'owner-device','our-device','unrelated'},{'owner-device','unrelated'}]
        manual_ios.recover(self.host)
        self.host.cleanup.assert_called_once_with('our-device')
        self.assertFalse(self.host.OWNED.exists())


if __name__=='__main__':unittest.main()
