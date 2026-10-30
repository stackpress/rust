"""Compare existing external package identities and resolved edges without updating a lock."""
import pathlib,sys,tomllib,json,hashlib
before=tomllib.loads(pathlib.Path(sys.argv[1]).read_text())['package']
after=tomllib.loads(pathlib.Path('Cargo.lock').read_text())['package']
def key(p):return p['name'],p['version'],p.get('source','')
old={key(p):p for p in before if 'source' in p}
new={key(p):p for p in after if 'source' in p}
assert old.keys()<=new.keys(),'external package removed/changed'
edge_additions={}
for k,p in old.items():
 assert {f:v for f,v in p.items() if f!='dependencies'}=={f:v for f,v in new[k].items() if f!='dependencies'},k
 def edges(package,packages):
  result=[]
  for d in package.get('dependencies',[]):
   pieces=d.split(' ',2)
   found=[q for q in packages if q['name']==pieces[0] and (len(pieces)<2 or q['version']==pieces[1])]
   assert len(found)==1,(d,found)
   result.append(key(found[0]))
  return sorted(result)
 old_edges=set(edges(p,before));new_edges=set(edges(new[k],after))
 assert old_edges<=new_edges,('original resolved edge changed',k)
 if new_edges-old_edges:edge_additions[str(k[:2])]=sorted(new_edges-old_edges)
identities=json.dumps(sorted((k,p['checksum']) for k,p in old.items()),separators=(',',':')).encode()
print(json.dumps({'unchanged_external_tuples':len(old),'added_external_packages':len(new)-len(old),'original_tuple_digest':hashlib.sha256(identities).hexdigest(),'existing_resolved_edges':'all retained; approved feature unification adds optional edges', 'optional_edge_additions':edge_additions},indent=2))
