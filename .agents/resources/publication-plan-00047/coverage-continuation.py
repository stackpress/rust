import pathlib,subprocess,sys
base=pathlib.Path('/private/tmp/stackpress-task47')
command=['cargo','llvm-cov','nextest','--workspace','--all-targets','--locked','--offline','--features',(base/'features.txt').read_text().strip(),'--no-clean','--json','--output-path',str(base/'coverage.json'),'--test-threads','1','-E','test(tcp_watchdog)']
print('COMMAND',repr(command),flush=True)
code=subprocess.call(command);(base/'coverage-continuation-status.txt').write_text(str(code));sys.exit(code)
