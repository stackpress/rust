// Join the original page with the actual generated SQL event bodies. No source edits/IO.
import fs from 'node:fs';
import {createRequire} from 'node:module';
const root=process.argv[2],require=createRequire(root+'/package.json'),ts=require('typescript');
const {default:Response}=await import(root+'/node_modules/@stackpress/lib/esm/router/Response.js');
const {nest}=await import(root+'/node_modules/@stackpress/lib/esm/data/Nest.js');
const crypto=require('node:crypto');
const hash=value=>crypto.createHash('shake256',{outputLength:32}).update(value).digest('hex');
const counters={authRemove:0,profileRemove:0};
const profile={id:'p1',name:'Ada'};
const auth=[{id:'a1',profileId:'p1',secret:hash('correct')}];
function generated(kind,Actions){
 const source=fs.readFileSync(root+'/packages/stackpress-sql/src/transform/events/remove.ts','utf8');
 let body=source.match(/REMOVE:\s*`([\s\S]*?)`/)[1];
 body=body.replaceAll('<%#@:ids%>','').replaceAll('<%/@:ids%>','').replaceAll('<%column%>','id').replaceAll('<%actions%>',kind+'Actions');
 const code=ts.transpileModule('async function execute({req,res,ctx}){'+body+'}\nreturn execute;',{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
 return new Function(kind+'Actions','Exception',code)(Actions,{upgrade:error=>{throw error}});
}
const authRemove=generated('Auth',class{async remove(){counters.authRemove++;return auth}});
const profileRemove=generated('Profile',class{async remove(){counters.profileRemove++;return [profile]}});
const module={exports:{}};
const page=fs.readFileSync(root+'/packages/stackpress-session/src/session/pages/remove.ts','utf8');
const code=ts.transpileModule(page,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText;
new Function('require','exports','module',code)(id=>{
 if(id==='@stackpress/ingest/Server')return {action:fn=>fn};
 if(id==='stackpress-schema/helpers')return {hash};
 if(id==='stackpress-view/helpers')return {setViewProps:()=>{}};
 throw Error(id);
},module.exports,module);
const req={method:'POST',data:nest({confirmed:true,secret:'correct'})};
const res=new Response();
const ctx={config:{path:(key,fallback)=>fallback},plugin:key=>key==='session'?{load:()=>({data:async()=>profile,guest:async()=>false})}:{},resolve:async(event,input,parent)=>{
 const response=parent||new Response();const child={data:nest(input)};
 if(event==='profile-detail')response.results(profile);
 if(event==='auth-search')response.results(auth);
 if(event==='auth-remove')await authRemove({req:child,res:response,ctx});
 if(event==='profile-remove')await profileRemove({req:child,res:response,ctx});
 return response.toStatusResponse();
}};
await module.exports.default({req,res,ctx});
if(counters.authRemove!==1||counters.profileRemove!==0||res.code!==302)throw Error('Original joined behavior changed');
const result={counters,code:res.code,location:res.headers.get('Location'),profileStillActive:counters.profileRemove===0,flash:JSON.parse(res.session.get('flash'))};
fs.writeFileSync(process.argv[3],JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
