from pathlib import Path
import json,re,tomllib
root=Path.cwd();rows=json.loads(Path('/private/tmp/stackpress-task47/inventory.json').read_text());output=[]
pattern=re.compile(r'(include_(?:str|bytes)!|include!)\s*\(\s*"([^"]+)"|#\[path\s*=\s*"([^"]+)"\]')
for row in rows:
 if row['path'].startswith('examples/'):continue
 base=(root/row['path']).resolve()
 files=list((base/'src').rglob('*.rs'))+list((base/'tests').rglob('*.rs'))+list((base/'examples').rglob('*.rs'))
 if (base/'build.rs').exists():files+=[base/'build.rs']
 for p in files:
  if base==root and p.parent!=root/'src' and not str(p).startswith(str(root/'tests')):continue
  s=p.read_text()
  for m in pattern.finditer(s):
   literal=m[2] or m[3];dest=(p.parent/literal).resolve()
   if dest.is_relative_to(base):continue
   output.append(dict(crate=row['name'],source=str(p.relative_to(root)),line=s[:m.start()].count('\n')+1,kind=m[1] or '#[path]',literal=literal,destination=str(dest.relative_to(root)) if dest.is_relative_to(root) else str(dest),exists=dest.exists()))
Path('/private/tmp/stackpress-task47/external-reads.json').write_text(json.dumps(output,indent=2))
for row in rows:
 refs=[x for x in output if x['crate']==row['name']]
 if refs:
  print(row['name'],len(refs),'external references')
  for x in refs:print(' ',x['source']+':'+str(x['line']),'=>',x['destination'],'' if x['exists'] else 'MISSING_LITERAL')
