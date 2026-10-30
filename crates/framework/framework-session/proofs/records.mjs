// Golden values execute original source classes; no copied implementation is the oracle.
import fs from 'node:fs';
import { register } from 'node:module';
register('./source-loader.mjs',import.meta.url);
const root=process.argv[2];
const {default:ProfileSchema}=await import(root+'/packages/stackpress-session/src/profile/ProfileSchema.ts');
const {default:AuthSchema}=await import(root+'/packages/stackpress-session/src/auth/AuthSchema.ts');
const {default:ProfileStore}=await import(root+'/packages/stackpress-session/src/profile/ProfileStore.ts');
const {default:AuthStore}=await import(root+'/packages/stackpress-session/src/auth/AuthStore.ts');
const {default:AuthActions,isEmail}=await import(root+'/packages/stackpress-session/src/auth/AuthActions.ts');
const {default:ProfileActions}=await import(root+'/packages/stackpress-session/src/profile/ProfileActions.ts');
const {default:Sqlite}=await import(root+'/node_modules/@stackpress/inquire/esm/dialect/Sqlite.js');
const sample=[undefined,new Date(1700000000123),[new Date(0),null,{}],null,false,true,0,1,'','false','true','0','1',[],[null,3,{}],{},'{"a":1}','[1]','bad'];
const schemas={Profile:new ProfileSchema('database-seed'),Auth:new AuthSchema('database-seed')};
const columns=[];
const safe=fn=>{try {const value=fn();return value===undefined?{undefined:true}:{value}}catch(error){return {error:true}}};
for(const [model,schema] of Object.entries(schemas))for(const [name,column] of Object.entries(schema.columns))for(const input of sample){
  columns.push({model,name,input,nativeDate:input instanceof Date,nestedDate:Array.isArray(input)&&input[0] instanceof Date,serialize:safe(()=>column.serialize(input)),unserialize:safe(()=>column.unserialize(input)),assert:column.assert(input)});
}
const emailInputs=['a@b.co','a@[١.١.١.١]','a@b.co\n','ab@foo.com','a..b@example.com','a.b@example.com','"a@b"@example.com','"a\\"b"@example.com','é@example.com','中@example.com','a@[255.0.1.3]','a@'+ 'b'.repeat(57)+'.test','a@'+ 'b'.repeat(59)+'.test','a@xn--foo.test','a@foo.technology','a@-foo.test','a\\@b@example.com'];
const emails=emailInputs.map(input=>({input,valid:isEmail.test(input)}));
const stores={Profile:new ProfileStore('database-seed'),Auth:new AuthStore('database-seed')};
const scalarInput={id:'record-1',name:'Ada',profileId:'profile-1',token:'user@example.test',secret:'hello',type:'email',roles:['USER'],tags:['tag'],references:{fbid:'abc'},verified:true,active:true,created:new Date(1700000000123),updated:new Date(1700000000123),consumed:new Date(1700000000123)};
const scalar=Object.fromEntries(Object.entries(stores).map(([name,store])=>[name,store.scalarize(scalarInput)]));
// Execute create's returned-row branch and update's returned-map branch with actual builders.
const dialect=Sqlite;
const calls=[];
let nextRows=[];
const engine={dialect,connection:{lastId:0},query:async(sql,values)=>{calls.push({sql,values});return nextRows;}};
const auth=new AuthActions(engine,{seed:'database-seed'});
const profile=new ProfileActions(engine,'database-seed');
// Preflight lookup is overridable in the source; this isolates exact create row decoding.
auth.find=async()=>null;
nextRows=[{id:'auth-1',profile_id:'profile-1',type:'email',token:scalar.Auth.token,secret:scalar.Auth.secret,verified:0,consumed:'2023-11-14T22:13:20.000Z',active:1,created:'2023-11-14T22:13:20.000Z',updated:'2023-11-14T22:13:20.000Z'}];
const authReturned=await auth.create({id:'auth-1',profileId:'profile-1',type:'email',token:'user@example.test',secret:'hello'});
const source={columns,emails,scalar,authReturned,sourceQueries:{search:dialect.select(stores.Profile.select({q:'ad'})),has:dialect.select(stores.Profile.select({has:{tags:['red','blue']}})),hasnt:dialect.select(stores.Profile.select({hasnt:{tags:['red','blue']}}))}};
fs.writeFileSync(process.argv[3],JSON.stringify(source,null,2)+'\n');
console.log('Recorded '+columns.length+' original column cases, '+emails.length+' email cases, scalar and returned-row values.');
