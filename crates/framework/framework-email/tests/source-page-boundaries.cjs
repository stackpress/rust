// Original source page/CSRF orchestration with finite auth, template and email boundaries.
'use strict';
const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');
const assert=require('node:assert/strict');

/** Keep only checked TS scaffolding changes; original branch/await order stays intact. */
function pageSource(source) {
  for(const text of ['ctx.plugin<SessionPlugin>', 'ctx.plugin<CsrfPlugin>', 'const mail: SendMailOptions']) assert(source.includes(text));
  return source.replace(/^import[\s\S]*?;\r?\n/gm,'')
    .replace('export default action','const page = action').replaceAll('export async function','async function')
    .replaceAll('req: Request','req').replaceAll('res: Response','res').replaceAll('ctx: Server','ctx')
    .replaceAll('): Promise<void>',')').replaceAll('const mail: SendMailOptions','const mail')
    .replaceAll('req.data.path<string>','req.data.path')
    .replace(/\.(plugin|resolve|config)(?:\.path)?<[^>]+>/g,match=>match.replace(/<[^>]+>/,''))
    +'\nreturn page;';
}
/** Callable source data bag preserves literal keys and its separate .path view. */
function bag(initial={}) {
  const values={...initial};
  const fn=key=>key===undefined?values:values[key];
  fn.set=(key,value)=>{values[key]=value;};fn.get=key=>values[key];fn.delete=key=>{delete values[key];};
  fn.path=(key,fallback)=>key.split('.').reduce((value,part)=>value?.[part],values)??fallback;
  fn.values=values;return fn;
}
/** Evaluate both original modules; no server, transport, store or template provider executes. */
async function main() {
  const root=process.argv[2], output=process.argv[3];assert(root&&output);
  const files=['stackpress-session/src/auth/pages/signin/email.ts','stackpress-csrf/src/plugin.ts'];
  const [pageText,csrfText]=files.map(file=>fs.readFileSync(path.join(root,file),'utf8'));
  const csrfRunnable=csrfText.replace(/^import .*;\r?\n/gm,'').replace('export default function plugin(ctx: Server)','return function plugin(ctx)')
    .replaceAll('req: Request','req').replaceAll('res: Response','res').replaceAll('ctx: Server','ctx')
    .replaceAll('.path<string>','.path');
  const cases=[];
  const inputs=[{name:'guest-get',method:'GET'},{name:'signed-in-get',method:'GET',guest:false},
    {name:'invalid-csrf',valid:false},{name:'otp-success'},{name:'magic-success',mode:'magic'},
    {name:'otp-child-status',sendStatus:419},{name:'magic-throw',mode:'magic',throws:true},
    {name:'otp-no-email-config',configured:false},{name:'magic-no-sender',mode:'magic',sender:false},
    {name:'password-success',mode:'pass'},{name:'password-status',mode:'pass',signinStatus:401}];
  for(const input of inputs) {
    const item={method:'POST',mode:'otp',valid:true,guest:true,configured:true,sender:true,sendStatus:200,signinStatus:200,...input};
    const trace=[];let csrf;let mail=null;let error=null;
    const settings=bag({csrf:{},auth:{base:'/auth',email:item.sender?{name:'Sender',address:'sender@example.test'}:{}},
      email:item.configured?{jsonTransport:true}:null,host:'https://fixture.example.test',brand:{name:'Fixture'}});
    const res={code:200,body:null,data:bag(),session:bag(),headers:{},
      setError(message){this.error=message;},statusCode(code){this.code=code;},
      redirect(location){trace.push('redirect');this.headers.Location=location;this.code=302;},
      fromStatusResponse(value){trace.push('propagate');this.code=value.code;this.error=value.error;}};
    const req={method:item.method,data:bag({auth:item.mode,email:'recipient@example.test',csrf:item.valid?'incoming':'wrong',redirect_uri:'/after'}),session:bag({csrf:'incoming'})};
    const ctx={config:settings,on(name,handler){assert.equal(name,'config');handler({ctx:this});},
      register(name,plugin){assert.equal(name,'csrf');csrf=plugin;},
      plugin(name){return name==='csrf'?csrf:{load(){return {guest:async()=>item.guest};}};},
      async emit(name){assert.equal(name,'auth-signin');trace.push(name);res.code=item.signinStatus;},
      async resolve(name,data){
        if(name==='email-send') {trace.push(name);mail=JSON.parse(JSON.stringify(data));if(item.throws)throw Error('provider fixture failure');return {code:item.sendStatus,error:item.sendStatus===200?undefined:'provider public failure'};}
        if(name==='auth-update')return {code:200};
        assert.equal(name,'auth-search');return {code:200,results:[{id:'auth-id',consumed:{toString(){return 'fixed consumed';}},profile:{name:'Recipient'}}]};
      }};
    const entropy={randomBytes:()=>Buffer.alloc(32,0x42),timingSafeEqual:crypto.timingSafeEqual};
    new Function('crypto',csrfRunnable)(entropy)(ctx);
    for(const name of ['generate','valid','clear']) {const original=csrf[name];csrf[name]=(...args)=>{trace.push(`csrf.${name}`);return original(...args);};}
    const template=values=>({subject:'fixture sign-in',text:`${values.link} ${values.pin??''}`,html:`<p>${values.link}</p>`});
    const page=new Function('action','hash','makeOTPTemplate','makeMagicLinkTemplate','setViewProps','Math',pageSource(pageText))(
      value=>value,value=>crypto.createHash('shake256',{outputLength:32}).update(value).digest('hex'),template,template,()=>{}, {...Math, floor:Math.floor,random:()=>0.123456});
    try {await page({req,res,ctx});} catch(cause) {error=cause.message;}
    assert.equal(error,item.throws?'provider fixture failure':null,item.name);
    cases.push({...item,trace,mail,error,code:res.code,publicError:res.error??null,redirect:res.headers.Location??null,
      retainedCsrf:!!res.data.get('csrf'),incomingCsrf:req.session.get('csrf'),password:req.data.get('password')??null,type:req.data.get('type')??null});
  }
  fs.writeFileSync(output,JSON.stringify({sources:files.map((file,index)=>({file,sha256:crypto.createHash('sha256').update([pageText,csrfText][index]).digest('hex')})),cases},null,2)+'\n');
  console.log(`Original page/CSRF boundary oracle: ${cases.length} finite cases; no live delivery`);
}
main().catch(error=>{console.error(error);process.exitCode=1;});
