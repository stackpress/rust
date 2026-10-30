// Execute original handlers with explicit finite services; production source is read-only.
import fs from 'node:fs';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
const root=process.argv[2];
const require=createRequire(root+'/package.json');
const ts=require('typescript');
const {default:Response}=await import(root+'/node_modules/@stackpress/lib/esm/router/Response.js');
const {TemplateEngine,helpers}=await import(root+'/node_modules/@stackpress/lib/esm/Template.js');
const {nest,isObject}=await import(root+'/node_modules/@stackpress/lib/esm/data/Nest.js');
const hash=value=>crypto.createHash('shake256',{outputLength:32}).update(value).digest('hex');
let actions;
const cache=new Map();
function load(relative){
 if(cache.has(relative))return cache.get(relative);
 const filename=root+'/packages/stackpress-session/src/'+relative+'.ts';
 const module={exports:{}};
 const code=ts.transpileModule(fs.readFileSync(filename,'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,esModuleInterop:true}}).outputText;
 const localRequire=id=>{
  if(id==='@stackpress/ingest/Server')return {action:fn=>fn};
  if(id==='stackpress-view/helpers')return {setViewProps:()=>{}};
  if(id==='stackpress-schema/helpers')return {hash};
  if(id==='@stackpress/lib/Template')return {TemplateEngine,helpers};
  if(id.endsWith('/AuthActions.js'))return {__esModule:true,default:{make:()=>actions},isSpecialChars:/[!@#$%^&*(),.?":{}|<>\s]/};
  if(id.endsWith('/Exception.js'))return {__esModule:true,default:{for:message=>({withCode(code){this.code=code;return this},toResponse(){return {code:this.code,error:message,errors:{}}}}),upgrade:error=>({withErrors(){return this},toResponse(){return {code:error.code||500,error:error.message,errors:error.errors||{}}}})}};
  if(id==='node:crypto')return crypto;
  if(id==='qrcode')return require(id);
  if(id==='zod')return require(id);
  const path=new URL(id,'file://'+filename).pathname;
  const prefix=root+'/packages/stackpress-session/src/';
  if(path.startsWith(prefix))return load(path.slice(prefix.length).replace(/\.js$/,''));
  throw Error('unhandled import '+id);
 };
 new Function('require','exports','module',code)(localRequire,module.exports,module);
 cache.set(relative,module.exports);return module.exports;
}

const NativeDate=Date;
globalThis.Date=class extends NativeDate{constructor(...args){super(...(args.length?args:[1700000000000]))}static now(){return 1700000000000}};
const me={id:'p1',name:'Ada Lovelace'};
const profile={id:'p1',name:'Ada Lovelace',image:null,roles:['USER'],tags:[],active:true};
const auths=[{id:'a1',profileId:'p1',type:'username',token:'ada',secret:hash('correct'),verified:false},{id:'a2',profileId:'p1',type:'email',token:'ada@example.test',secret:hash('correct'),verified:true}];
const secret='JBSWY3DPEHPK3PXP';
const cases=[];
const add=(name,target,extra={})=>cases.push({name,target,...extra});
for(const target of ['account','update','password','remove','export','2fa/detail','2fa/remove']){
 add(target+'-get',target,{data:target==='2fa/detail'?{secret}:{}});
 add(target+'-guest',target,{guest:true});
 add(target+'-missing-profile',target,{missingProfile:true});
 add(target+'-earlier-body',target,{body:'earlier'});
 add(target+'-earlier-error',target,{earlyError:true});
}
for(const entry of [
 ['existing-identifier-no-password',{name:'Changed',image:'',username:'ada2',phone:''}],
 ['add-email-needs-password',{name:'Changed',username:'ada',email:'new@example.test',phone:''}, {auths:[auths[0]]}],
 ['add-email-wrong-password',{name:'Changed',username:'ada',email:'new@example.test',current:'wrong',phone:''},{auths:[auths[0]]}],
 ['add-email-correct',{name:'Changed',username:'ada',email:'new@example.test',current:'correct',phone:''},{auths:[auths[0]]}],
 ['empty-tokens-are-not-deletes',{name:'Changed',image:'',username:' ',email:'',phone:''}],
 ['profile-update-failure',{name:'Changed',phone:''},{failEvent:'profile-update'}],
 ['identifier-first-failure',{name:'Changed',username:'new',email:'changed@example.test',phone:''},{failEvent:'auth-update'}],
 ['source-missing-phone',{name:'Changed'}],
 ['normalized-phone',{name:'Changed',phone:'+63 (917) 123-4567',current:'correct'}],
])add('update-'+entry[0],'update',{method:'POST',data:entry[1],...entry[2]});
for(const entry of [
 ['required',{}],['same',{secret:'correct',current:'correct'}],['wrong-current',{secret:'new',current:'wrong'}],
 ['success-no-csrf',{secret:'new',current:'correct'}],['ignores-update-failure',{secret:'new',current:'correct'},{failEvent:'auth-update'}],
 ['all-auth-types',{secret:'new',current:'factor'},{auths:[{id:'factor',type:'2fa',secret:hash('factor')}]}],
 ['no-auth',{secret:'new',current:'correct'},{auths:[]}],['search-failure',{secret:'new',current:'correct'},{failEvent:'auth-search'}],
 ['rules',{secret:'new',current:'correct'},{password:{min:8,upper:true}}]
])add('password-'+entry[0],'password',{method:'POST',data:entry[1],...entry[2]});
for(const entry of [
 ['without-confirmation',{}],['missing-password',{confirmed:true}],['wrong-password',{confirmed:true,secret:'wrong'}],
 ['success-no-csrf',{confirmed:true,secret:'correct'}],['first-remove-fails',{confirmed:true,secret:'correct'},{failEvent:'auth-remove'}],
 ['profile-remove-fails',{confirmed:true,secret:'correct'},{failEvent:'profile-remove'}],['search-fails',{confirmed:true,secret:'correct'},{failEvent:'auth-search'}],
 ['no-auth',{confirmed:true,secret:'correct'},{auths:[]}]
])add('remove-'+entry[0],'remove',{method:'POST',data:entry[1],...entry[2]});
add('export-download','export',{data:{download:true}});
add('export-formula-cells','export',{data:{download:true},profile:{name:'=1+1',image:null,tags:['+thing','@other'],references:{note:'<tag>'}}});
for(const entry of [
 ['request-secret',{secret}],['invalid-code',{secret,code:'000000'},{method:'POST'}],
 ['valid-create',{secret,code:load('session/helpers').generateTOTP(secret)},{method:'POST'}],
 ['invalid-csrf',{secret,code:'123456'},{method:'POST',csrf:false}],
 ['stored-secret',{}, {auths:[{id:'factor',type:'2fa',token:secret}]}],
 ['valid-update',{secret,code:load('session/helpers').generateTOTP(secret)},{method:'POST',auths:[{id:'factor',type:'2fa',token:secret}]}],
 ['write-error-still-redirects',{secret,code:load('session/helpers').generateTOTP(secret)},{method:'POST',failEvent:'auth-create'}],
 ['search-fails',{secret},{failEvent:'auth-search'}]
])add('enroll-'+entry[0],'2fa/detail',{data:entry[1],...entry[2]});
for(const entry of [
 ['confirmed-get-no-csrf',{confirmed:true,authId:'factor'}],['confirmed-post',{confirmed:true,authId:'factor'},{method:'POST',csrf:false}],
 ['empty-auth-id',{confirmed:true,authId:''}],['missing-auth-id-string-undefined',{confirmed:true}],
 ['search-fails',{confirmed:true,authId:'factor'},{failEvent:'auth-search'}],
 ['remove-fails',{confirmed:true,authId:'factor'},{failEvent:'auth-remove'}],
])add('remove-factor-'+entry[0],'2fa/remove',{data:entry[1],...entry[2]});
const outputs=[];
for(const scenario of cases){
 const trace=[];let search=0;const req={method:scenario.method||'GET',data:nest(structuredClone(scenario.data||{}))};
 const res=new Response();if(scenario.body)res.html(scenario.body);if(scenario.earlyError)res.setError('early',{},[],409);
 const cfg={auth:{base:'/auth',password:scenario.password||{},'2fa':{issuer:'Brand & Co'}},brand:{name:'Brand'}};
 const get=key=>key.split('.').reduce((o,k)=>o?.[k],cfg);const config={path:(key,fallback)=>get(key)===undefined?fallback:get(key)};
 const session={load:()=>({data:async()=>scenario.guest?null:me,guest:async()=>!!scenario.guest})};
 const csrf={generate(){trace.push(['csrf.generate'])},valid(){trace.push(['csrf.valid']);if(scenario.csrf===false){res.setError('csrf',{},[],419,'Page Expired');return false}return true}};
 const ctx={config,plugin:key=>({session,csrf}[key]),resolve:async(event,data,parent)=>{
  trace.push(['resolve',event,structuredClone(data)]);const response=parent||new Response();
  if(event==='profile-detail'){if(scenario.missingProfile)response.setError('missing',{},[],404);else response.results(structuredClone(scenario.profile||profile));}
  else if(event==='auth-search')response.results(structuredClone(scenario.auths??(data.eq?.type==='2fa'?[]:auths)));
  else response.results({id:data.id||'created',...data});
  if(scenario.failEvent===event)response.setError('child failure',{},[],503,'Service Unavailable');
  return response.toStatusResponse();
 }};
 let thrown=null;try{await load('session/pages/'+scenario.target).default({req,res,ctx})}catch(error){thrown=error.message}
 const result=structuredClone(res.body);let qr=null;
 if(result?.url){
  const png=Buffer.from(result.url.split(',')[1],'base64');qr={width:png.readUInt32BE(16),height:png.readUInt32BE(20)};
  result.url='PNG';
 }
 outputs.push({scenario,expected:{code:res.code,error:res.error||null,errors:res.errors(),location:res.headers.get('Location')||null,attachment:res.headers.get('Content-Disposition')||null,result,qr,trace,flash:res.session.get('flash')?JSON.parse(res.session.get('flash')):null,thrown}});
}
fs.writeFileSync(process.argv[3],JSON.stringify({me,profile,auths,cases:outputs},null,2)+'\n');console.log('Original account oracle: '+outputs.length+' invocation traces');
