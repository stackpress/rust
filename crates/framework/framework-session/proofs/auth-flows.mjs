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
  if(id.endsWith('/AuthActions.js'))return {__esModule:true,default:{make:()=>actions}};
  if(id.endsWith('/Exception.js'))return {__esModule:true,default:{upgrade:error=>({withErrors(){return this},toResponse(){return {code:error.code||500,error:error.message,errors:error.errors||{}}}})}};
  if(id==='node:crypto')return crypto;
  if(id==='zod')return require(id);
  const path=new URL(id,'file://'+filename).pathname;
  const prefix=root+'/packages/stackpress-session/src/';
  if(path.startsWith(prefix))return load(path.slice(prefix.length).replace(/\.js$/,''));
  throw Error('unhandled import '+id);
 };
 new Function('require','exports','module',code)(localRequire,module.exports,module);
 cache.set(relative,module.exports);return module.exports;
}
const consumed=new Date(1700000000000);
const row={id:'a1',profileId:'p1',type:'email',token:'ada@example.test',secret:'stored-secret',consumed,profile:{id:'p1',name:'Ada',roles:[],image:null}};
const auth={base:'/auth',roles:['USER'],menu:[],password:{},email:{name:'Sender',address:'sender@example.test'}};
const challenge=hash(consumed.toString());
const otpChallenge=hash(consumed.toString()+'000007');
const base={auth,session:{key:'identity'},email:{},host:'https://example.test',brand:{name:'Brand'}};
const cases=[
 {name:'signin-flags',target:'auth/events/signin',data:{type:'email',email:'ada@example.test',password:false,'2fa':false}},
 {name:'signin-password-empty',target:'auth/events/signin',data:{email:'ada@example.test',type:'email',password:'','2fa':false}},
 {name:'signin-2fa-null-is-checked',target:'auth/events/signin',data:{type:'email','2fa':null},search:[{id:'totp'}]},
 {name:'signin-2fa-empty',target:'auth/events/signin',data:{type:'email'}},
 {name:'signin-incomplete',target:'auth/events/signin',data:{},signin:{}},
 {name:'signin-action-error',target:'auth/events/signin',data:{},failure:{code:401,message:'Invalid Parameters',errors:{secret:'Invalid Password'}}},
 {name:'signout-literal',target:'auth/events/signout',data:{}},
 {name:'signup-roles-and-hooks',target:'auth/events/signup',data:{name:'Ada',email:'ada@example.test',phone:'+1 (234) 567',roles:['OVERRIDE']},signup:{id:'p1',auth:{email:{secret:'e'},phone:{secret:'p'},username:{secret:'u'},custom:{secret:'keep'}}}},
 {name:'signup-optional-phone',target:'auth/events/signup',data:{name:'Ada',username:'ada'},signup:{id:'p1',auth:{username:{secret:'u'}}}},
 {name:'username-overwrites-flags',target:'auth/pages/signin/username',method:'POST',data:{password:false,'2fa':false,redirect_uri:'/next'}},
 {name:'phone-normalizes',target:'auth/pages/signin/phone',method:'POST',data:{phone:' +1 (234)-567 ',password:false,'2fa':false}},
 {name:'password-redirect-2fa',target:'auth/pages/signin/username',method:'POST',data:{},search:[{id:'totp'}]},
 {name:'password-invalid-csrf',target:'auth/pages/signin/username',method:'POST',data:{},csrf:false},
 {name:'email-invalid-method',target:'auth/pages/signin/email',method:'POST',data:{auth:'other'}},
 {name:'otp-issue',target:'auth/pages/signin/email',method:'POST',data:{auth:'otp',email:' ada@example.test ',redirect_uri:'/next?a=1'},issue:true},
 {name:'magic-issue',target:'auth/pages/signin/email',method:'POST',data:{auth:'magic',email:'ada@example.test'},issue:true},
 {name:'otp-missing-transport',target:'auth/pages/signin/email',method:'POST',data:{auth:'otp',email:'ada@example.test'},issue:true,noEmail:true},
 {name:'magic-missing-sender',target:'auth/pages/signin/email',method:'POST',data:{auth:'magic',email:'ada@example.test'},issue:true,noSender:true},
 {name:'otp-empty-email',target:'auth/pages/signin/email',method:'POST',data:{auth:'otp',email:' '}},
 {name:'otp-missing-account',target:'auth/pages/signin/email',method:'POST',data:{auth:'otp',email:'ada@example.test'},issue:true,search:[]},
 {name:'otp-get-no-code-check',target:'auth/pages/signin/otp',data:{auth:'a1',challenge:'wrong'},search:[row]},
 {name:'otp-short-code',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:otpChallenge,code:'7'},search:[row]},
 {name:'otp-nondigit-six-chars',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:hash(consumed.toString()+'abcdef'),code:'abcdef'},search:[row]},
 {name:'otp-invalid-code',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:'wrong',code:'7'},search:[row]},
 {name:'otp-long-code',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:'wrong',code:'1234567'},search:[row]},
 {name:'otp-incomplete-link-final-400',target:'auth/pages/signin/otp',data:{}},
 {name:'magic-get-no-csrf',target:'auth/pages/signin/link',data:{auth:'a1',challenge},search:[row]},
 {name:'magic-post-no-csrf',target:'auth/pages/signin/link',method:'POST',data:{auth:'a1',challenge},search:[row],csrf:false},
 {name:'magic-invalid-final-400',target:'auth/pages/signin/link',data:{auth:'a1',challenge:'wrong'},search:[row]},
 {name:'otp-literal-undefined-base',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:otpChallenge,code:'7'},search:[row],noBase:true},
 {name:'2fa-get-without-factor',target:'auth/pages/signin/2fa',data:{profile:'p1',auth:'a1',challenge},search:[row]},
 {name:'2fa-enabled-get-no-code-check',target:'auth/pages/signin/2fa',data:{profile:'p1',auth:'a1',challenge},search:[row,{id:'factor',type:'2fa',token:'JBSWY3DPEHPK3PXP'}]},
 {name:'2fa-enabled-invalid-post',target:'auth/pages/signin/2fa',method:'POST',data:{profile:'p1',auth:'a1',challenge,code:'bad'},search:[row,{id:'factor',type:'2fa',token:'JBSWY3DPEHPK3PXP'}]},
 {name:'2fa-enabled-valid-post',target:'auth/pages/signin/2fa',method:'POST',data:{profile:'p1',auth:'a1',challenge,code:load('session/helpers').generateTOTP('JBSWY3DPEHPK3PXP')},search:[row,{id:'factor',type:'2fa',token:'JBSWY3DPEHPK3PXP'}]},
 {name:'2fa-mismatch-flash',target:'auth/pages/signin/2fa',data:{profile:'p1',auth:'a1',challenge:'wrong'},search:[row]},
 {name:'2fa-missing-auth',target:'auth/pages/signin/2fa',data:{profile:'p1'}},
 {name:'2fa-empty-methods',target:'auth/pages/signin/2fa',data:{profile:'p1',auth:'a1'},search:[]},
 {name:'2fa-invalid-method',target:'auth/pages/signin/2fa',data:{profile:'p1',auth:'missing'},search:[row]},
 {name:'signed-in-link-skips-proof',target:'auth/pages/signin/link',data:{},signedIn:true},
 {name:'signed-in-otp-skips-proof',target:'auth/pages/signin/otp',data:{},signedIn:true},
 {name:'signup-page-post',target:'auth/pages/signup',method:'POST',data:{name:'Ada',username:'ada-user',phone:'',secret:'secret'},signup:{id:'p1',auth:{username:{secret:'u'}}}},
 {name:'signup-page-existing-user',target:'auth/pages/signup',data:{},signedIn:true},
 {name:'otp-search-failure',target:'auth/pages/signin/email',method:'POST',data:{auth:'otp',email:'ada@example.test'},issue:true,failEvent:'auth-search'},
 {name:'otp-rotation-failure',target:'auth/pages/signin/email',method:'POST',data:{auth:'otp',email:'ada@example.test'},issue:true,failEvent:'auth-update'},
 {name:'magic-reload-failure',target:'auth/pages/signin/email',method:'POST',data:{auth:'magic',email:'ada@example.test'},issue:true,failEvent:'reload'},
 {name:'magic-record-disappeared',target:'auth/pages/signin/email',method:'POST',data:{auth:'magic',email:'ada@example.test'},issue:true,disappear:true},
 {name:'otp-mail-failure',target:'auth/pages/signin/email',method:'POST',data:{auth:'otp',email:'ada@example.test'},issue:true,failEvent:'email-send'},
 {name:'otp-guest-shortcut-post',target:'auth/pages/signin/otp',method:'POST',data:{code:'bad'},signedIn:true,csrf:false},
 {name:'otp-object-code',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:'wrong',code:{}},search:[row]},
 {name:'otp-empty-code',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:'wrong',code:''},search:[row]},
 {name:'otp-wrong-csrf',target:'auth/pages/signin/otp',method:'POST',data:{auth:'a1',challenge:'wrong',code:'7'},search:[row],csrf:false},
 {name:'magic-empty-record',target:'auth/pages/signin/link',data:{auth:'a1',challenge},search:[]},
 {name:'two-factor-wrong-csrf',target:'auth/pages/signin/2fa',method:'POST',data:{profile:'p1',auth:'a1',challenge,code:'123456'},search:[row,{id:'factor',type:'2fa',token:'JBSWY3DPEHPK3PXP'}],csrf:false},
 {name:'phone-bom-normalization',target:'auth/pages/signin/phone',method:'POST',data:{phone:'\uFEFF+1 (234)-567\uFEFF','2fa':false}},
 {name:'signout-page-no-csrf',target:'auth/pages/signout',data:{redirect_uri:'/bye'}},
 {name:'chooser-no-csrf-generation',target:'auth/pages/signin',data:{}},
 {name:'earlier-body-skips-page',target:'auth/pages/signin/otp',data:{},body:'earlier'},
];
const NativeDate=Date;globalThis.Date=class extends NativeDate {constructor(...args){super(...(args.length?args:[1700000000000]));} static now(){return 1700000000000;}};const now=Date.now;Date.now=()=>1700000000000;const random=Math.random;Math.random=()=>0.000007;
// Set the valid-code fixture after freezing Date.now too.
cases.find(c=>c.name==='2fa-enabled-valid-post').data.code=load('session/helpers').generateTOTP('JBSWY3DPEHPK3PXP');
const outputs=[];
for(const scenario of cases){
 const trace=[];const nested=[];let searches=0;const input=structuredClone(scenario.data);const data=nest(input);const req={method:scenario.method||'GET',data};const res=new Response();
 if(scenario.body)res.html(scenario.body);
 const config=structuredClone(base);if(scenario.noEmail)delete config.email;if(scenario.noSender)delete config.auth.email;if(scenario.noBase)delete config.auth.base;
 const cfg=key=>key.split('.').reduce((object,key)=>object?.[key],config);const configuration=key=>cfg(key);configuration.path=(key,fallback)=>cfg(key)===undefined?fallback:cfg(key);
 const csrf={generate(){trace.push(['csrf.generate'])},valid(){trace.push(['csrf.valid']);if(scenario.csrf===false){res.setError('csrf',{},[],419,'Page Expired');return false;}return true;},clear(){trace.push(['csrf.clear']);data.delete('csrf')}};
 const session={key:'identity',load:()=>({guest:async()=>{trace.push(['guest']);return !scenario.signedIn;}}),create:async claims=>{trace.push(['session.create',claims]);return 'TOKEN';}};
 actions={signin:async(kind,input,password)=>{trace.push(['actions.signin',kind,password]);if(scenario.failure)throw Object.assign(Error(scenario.failure.message),scenario.failure);return structuredClone(scenario.signin??row)},signup:async input=>{trace.push(['actions.signup',input]);return structuredClone(scenario.signup??{})}};
 const ctx={config:configuration,plugin:name=>({session,csrf}[name]),resolve:async(event,input,parent)=>{
  trace.push(['resolve',event,structuredClone(typeof input?.data==='function'?input.data():input)]);const response=parent||new Response();
  if(event==='auth-search'){searches++;response.results(structuredClone(searches>1&&scenario.disappear?[]:scenario.search??(scenario.issue?[row]:[])));}
  if(event==='auth-update')response.results(row);
  if(event==='email-send')response.results({sent:true});
  if(scenario.failEvent===event || (scenario.failEvent==='reload'&&event==='auth-search'&&searches>1))response.setError('child failure',{},[],503,'Service Unavailable');
  if(event==='auth-signout')await load('auth/events/signout').default({req:input,res:response,ctx});
  return response.toStatusResponse();
 },emit:async(event,req,res)=>{trace.push(['emit',event]);const begin=trace.length;await load('auth/events/'+event.slice(5)).default({req,res,ctx});nested.push({event,trace:trace.slice(begin),code:res.code,error:res.error||null,errors:res.errors(),location:res.headers.get('Location')||null,result:structuredClone(res.body)});}};
 let thrown;
 try{await load(scenario.target).default({req,res,ctx})}catch(error){thrown=error.message}
 outputs.push({scenario,nested,expected:{code:res.code,error:res.error||null,errors:res.errors(),location:res.headers.get('Location')||null,result:res.body,input:data(),trace,flash:res.session.get('flash')||null,thrown:thrown||null}});
}
Date.now=now;Math.random=random;
const templates=[true,false].map(otp=>({otp,input:{brand:'B & <C>',email:'ada@example.test',link:'https://example.test/?a=1&b="two"',name:'  Ada <A>  ',pin:'000007'},expected:(otp?load('auth/templates').makeOTPTemplate:load('auth/templates').makeMagicLinkTemplate)({brand:'B & <C>',email:'ada@example.test',link:'https://example.test/?a=1&b="two"',name:'  Ada <A>  ',pin:'000007'})}));
const helpersSource=load('session/helpers');
const totp=[0,59,1111111109,1700000000].map(seconds=>{const old=Date.now;Date.now=()=>seconds*1000;const code=helpersSource.generateTOTP('JBSWY3DPEHPK3PXP');Date.now=old;return {seconds,code};});
globalThis.Date=NativeDate;
const sqlModule={exports:{}};
const sqlSource=fs.readFileSync(root+'/packages/stackpress-sql/src/helpers.ts','utf8');
new Function('require','exports','module',ts.transpileModule(sqlSource,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText)(()=>({isObject}),sqlModule.exports,sqlModule);
const {toSqlDate}=sqlModule.exports;
const timezone=[];
for(const zone of ['UTC','Asia/Manila','America/New_York']){
 process.env.TZ=zone;
 const date=new NativeDate(1700000000000);
 const Column=load('auth/columns/ConsumedColumn').default;
 const serialized=new Column().serialize(date);
 timezone.push({zone,offset:-date.getTimezoneOffset(),display:date.toString(),serialized,sql:toSqlDate(serialized,true).toISOString()});
}
process.env.TZ='UTC';
fs.writeFileSync(process.argv[3],JSON.stringify({cases:outputs,templates,totp,timezone},null,2)+'\n');
console.log('Original source oracle: '+outputs.length+' invocation traces, 2 templates, 4 TOTP vectors.');
