import pathlib,subprocess,os,json,re,shutil,tarfile,gzip,io,time
root=pathlib.Path.cwd(); evidence=pathlib.Path('/private/tmp/stackpress-task47'); copy=evidence/'copy';copy.mkdir()
tracked=subprocess.check_output(['git','ls-files','-z']).decode().split('\0')
for rel in filter(None,tracked):
 src=root/rel; dst=copy/rel
 if not src.is_file(): continue
 dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
subprocess.run(['git','init','-q'],cwd=copy,check=True)
subprocess.run(['git','add','--all'],cwd=copy,check=True)
# Add versions to every local dependency in the COPY, including target and dev tables.
for p in copy.rglob('Cargo.toml'):
 text=p.read_text()
 text=re.sub(r'(\{[^\n}]*\bpath\s*=\s*"[^"]+"[^\n}]*)(\})',lambda m:m[0] if re.search(r'\bversion\s*=',m[1]) else m[1]+', version = "0.1.0" '+m[2],text)
 if p==copy/'Cargo.toml': text=text.replace('publish = false','publish = true',1)
 if str(p.relative_to(copy)).startswith('examples/') or '/consumers/' in str(p):
  text=text.replace('publish.workspace = true','publish = false')
 p.write_text(text)
order=json.loads((evidence/'order.json').read_text());inventory=json.loads((evidence/'inventory.json').read_text());packages={p['name']:p for p in inventory};results=[]
for name in order:
 item=packages[name]; row={'name':name,'path':item['path']};folder=evidence/'packages'/name;folder.mkdir(parents=True)
 for label,extra in [('list',['--list']),('package',['--no-verify'])]:
  command=['cargo','package','-p',name,'--offline','--allow-dirty']+extra
  print('COMMAND',name,label,flush=True);start=time.monotonic();run=subprocess.run(command,cwd=copy,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  (folder/(label+'.stdout')).write_text(run.stdout);(folder/(label+'.stderr')).write_text(run.stderr);row[label]={'code':run.returncode,'seconds':round(time.monotonic()-start,3),'command':command,'stderr':run.stderr}
  if label=='list' and run.returncode==0:
   files=run.stdout.splitlines();row['files']=files;row['file_count']=len(files);sizes=[];payload=0;estimated=io.BytesIO()
   with tarfile.open(fileobj=estimated,mode='w:gz',compresslevel=6) as archive:
    for rel in files:
     original=copy/item['path']/rel
     if rel=='Cargo.toml.orig':original=copy/item['path']/'Cargo.toml'
     if rel=='Cargo.lock' and not original.exists():original=copy/'Cargo.lock'
     if original.is_file():
      size=original.stat().st_size;payload+=size;sizes.append((size,rel));archive.add(original,arcname=name+'-0.1.0/'+rel,recursive=False)
   row['source_bytes']=payload;row['estimated_compressed_bytes']=len(estimated.getvalue());row['largest_files']=sorted(sizes,reverse=True)[:8]
 if not any(d['kind']!='dev-dependencies' for d in item['deps']):
  command=['cargo','publish','-p',name,'--dry-run','--offline','--allow-dirty'];print('COMMAND',name,'dry-run',flush=True);start=time.monotonic();run=subprocess.run(command,cwd=copy,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  (folder/'dry-run.stdout').write_text(run.stdout);(folder/'dry-run.stderr').write_text(run.stderr);row['dry_run']={'code':run.returncode,'seconds':round(time.monotonic()-start,3),'command':command,'stderr':run.stderr}
 crate=root/'target/00047/package'/f'{name}-0.1.0.crate'
 if crate.exists():row['archive_bytes']=crate.stat().st_size
 results.append(row);(evidence/'packaging.json').write_text(json.dumps(results,indent=2));print(name,{k:row[k]['code'] for k in ['list','package','dry_run'] if k in row},row.get('archive_bytes'),flush=True)
print('COPY_COMPLETE',copy,flush=True)
