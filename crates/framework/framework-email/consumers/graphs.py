"""Check every declared Lane B feature graph without dev-dependency feature unification."""
import argparse,subprocess,tomllib
from pathlib import Path
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check',action='store_true',help='also compile all four feature combinations')
args=parser.parse_args()
for package in ['stackpress-framework-csrf','stackpress-framework-email']:
 path=Path('crates/framework')/package.removeprefix('stackpress-')/'Cargo.toml'
 assert set(tomllib.loads(path.read_text())['features'])=={'default','server'}
 for enabled in [False,True]:
  command=['cargo','tree','-p',package,'--no-default-features','-e','normal','--depth','1','--locked','--offline']
  if enabled: command+=['--features','server']
  print('COMMAND:',*command,flush=True)
  graph=subprocess.check_output(command,text=True)
  print(graph,flush=True)
  assert ('stackpress-server ' in graph)==enabled
  if package.endswith('email'): assert ('stackpress-lib ' in graph)==enabled
command=['cargo','tree','-p','stackpress','-e','normal','--depth','1','--locked','--offline']
print('COMMAND:',*command,flush=True)
graph=subprocess.check_output(command,text=True);print(graph,flush=True)
assert 'stackpress-lib ' in graph and 'stackpress-server ' not in graph and 'stackpress-framework-email ' not in graph and 'stackpress-framework-csrf ' not in graph
command=['cargo','tree','-p','stackpress-framework-email','-e','features','--depth','1','--locked','--offline']
print('COMMAND:',*command,flush=True)
graph=subprocess.check_output(command,text=True);print(graph,flush=True)
for feature in ['lettre feature "builder"','lettre feature "smtp-transport"','lettre feature "tokio1-rustls-tls"','lettre feature "hostname"','reqwest feature "rustls-no-provider"','reqwest feature "gzip"','reqwest feature "deflate"','reqwest feature "cookies"']: assert feature in graph,feature
assert 'lettre feature "default"' not in graph and 'reqwest feature "default"' not in graph
print('All source transport variants are compiled native adapters, not runtime-loaded or individually feature-gated plugins.',flush=True)

if args.check:
 for features in [None,'stackpress-framework-csrf/server','stackpress-framework-email/server','stackpress-framework-csrf/server,stackpress-framework-email/server']:
  command=['cargo','check','--lib','-p','stackpress-framework-csrf','-p','stackpress-framework-email','--no-default-features','--locked','--offline']
  if features: command+=['--features',features]
  print('COMMAND:',*command,flush=True)
  subprocess.run(command,check=True)
