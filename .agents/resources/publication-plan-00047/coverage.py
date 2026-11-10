import os,subprocess,sys,pathlib,time,threading
root=pathlib.Path.cwd(); output=pathlib.Path('/private/tmp/stackpress-task47'); features=(output/'features.txt').read_text().strip()
# One accumulated native workspace measurement. Isolate the timing watchdog, as in recent gates.
base=['cargo','llvm-cov','nextest','--workspace','--all-targets','--locked','--offline','--features',features,'--no-report']
status=0
for extra in [['--test-threads','2','--no-fail-fast','-E','not test(tcp_watchdog)'],['--no-clean','--test-threads','1','-E','test(tcp_watchdog)']]:
 cmd=base+extra; print('COMMAND',repr(cmd),flush=True); rc=subprocess.call(cmd); print('EXIT',rc,flush=True); status=status or rc
 if rc: break
cmd=['cargo','llvm-cov','report','--json','--output-path',str(output/'coverage.json')]; print('COMMAND',repr(cmd),flush=True); rc=subprocess.call(cmd); status=status or rc
(output/'coverage-status.txt').write_text(str(status));sys.exit(status)
