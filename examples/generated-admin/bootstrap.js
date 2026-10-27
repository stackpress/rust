import init, { start } from './admin.js';
await init();
let active;
const seed = document.getElementById('admin-seed');
const page = seed ? JSON.parse(seed.textContent) : undefined;
const endpoints = new Set(page ? ['search','get','detail','create','update','remove','restore','preview','confirm'].map(action=>`${page.mount}api/${page.spec.module}/${action}`) : []);
const request = async event => {
  if (active) return;
  const command = JSON.parse(event.detail);
  const endpoint = new URL(command.endpoint, location.href);
  if (endpoint.origin !== location.origin || !endpoints.has(endpoint.pathname)) throw Error('Invalid compiled admin endpoint');
  const controller = new AbortController(); active = controller;
  let reply;
  try {
    const transfer = endpoint.pathname.endsWith('/preview') || endpoint.pathname.endsWith('/confirm');
    const headers = {'content-type':transfer?'text/csv':'application/json','x-admin-csrf':command.csrf};
    if(transfer) {
      headers['x-admin-decode-export-guards']=command.input.decode_export_guards?'1':'0';
      if(command.input.confirm===true)headers['x-admin-confirmed']='true';
    }
    const response = await fetch(endpoint, { method:'POST', credentials:'same-origin', headers, body:transfer?command.input.csv:JSON.stringify(command.input), signal:controller.signal });
    reply = { status:response.status, envelope:await response.json() };
  } catch {
    reply = { status:500, envelope:{code:500,error:command.write?'Write acknowledgement unavailable':'Unable to load records',errors:command.write?{outcome:'unknown'}:{}} };
  } finally { if (active === controller) active = undefined; }
  if (!controller.signal.aborted) window.dispatchEvent(new CustomEvent('stackpress-admin-reply',{detail:JSON.stringify(reply)}));
};
// File admission precedes arrayBuffer allocation; only this selected File is ever read.
let uploadGeneration=0, disposed=false;
const upload = value => {if(!disposed)window.dispatchEvent(new CustomEvent('stackpress-admin-upload',{detail:JSON.stringify(value)}));};
const fileChanged = async event => {
  if(event.target.id!=='admin-import-file'||event.target.disabled)return;
  const generation=++uploadGeneration,file=event.target.files?.[0];
  const empty={name:'',csv:'',error:'',reading:false,generation};
  if(!file){upload(empty);return;}
  // Truncate by Unicode scalar, preserving valid UTF-8 when a name ends at an emoji boundary.
  const name=Array.from(file.name).slice(0,128).join(''),max=page.transfer_limits.bytes;
  if(file.size>max){upload({...empty,name,error:'CSV file exceeds byte limit'});return;}
  upload({...empty,name,reading:true});
  try {
    const bytes=await file.arrayBuffer();
    if(disposed||generation!==uploadGeneration)return;
    if(bytes.byteLength>max){upload({...empty,name,error:'CSV file exceeds byte limit'});return;}
    const csv=new TextDecoder('utf-8',{fatal:true}).decode(bytes);
    upload({...empty,name,csv});
  } catch {if(generation===uploadGeneration)upload({...empty,name,error:'Unable to read UTF-8 CSV file'});}
};
const cancelUpload = event => {if(event.target.id==='admin-import-cancel'&&!event.target.disabled){uploadGeneration++;const input=document.getElementById('admin-import-file');if(input)input.value='';}};
document.addEventListener('change',fileChanged);
document.addEventListener('click',cancelUpload);
window.addEventListener('stackpress-admin-request',request);
const dispose = start();
window.adminDispose = () => { disposed=true;uploadGeneration++;document.removeEventListener('change',fileChanged);document.removeEventListener('click',cancelUpload);window.removeEventListener('stackpress-admin-request',request); active?.abort(); active=undefined; dispose(); };
window.adminReady = true;
