// Reproduce source fixtures outside the repository; supply upstream root and output path.
import fs from 'node:fs';
import crypto from 'node:crypto';
const upstream=process.argv[2];
const {SignJWT,CompactSign,jwtVerify}=await import(upstream+'/node_modules/jose/dist/webapi/index.js');
const key=new TextEncoder().encode('abc123');
const now=1700000000;
const cases=[['empty',{},'HS256'],['no-exp',{id:'u1',name:'Fixture',roles:['USER'],iat:now},'HS256'],['exp-equal',{exp:now},'HS256'],['exp-fraction',{exp:now+0.5},'HS256'],['nbf-equal',{nbf:now},'HS256'],['nbf-future',{nbf:now+0.5},'HS256'],['iat-wrong',{iat:'wrong'},'HS256'],['aud-iss',{aud:'any',iss:'any',iat:now+10000},'HS256'],['roles-empty',{roles:[]},'HS384'],['custom',{extra:{x:true}},'HS512'],['exp-null',{exp:null},'HS256'],['roles-null',{roles:null},'HS256']];
const vectors=[];
for(const [name,claims,alg] of cases){
 const token=await new SignJWT(claims).setProtectedHeader({alg}).sign(key);
 let valid=true;try{await jwtVerify(token,key,{currentDate:new Date(now*1000)});}catch{valid=false;}
 vectors.push({name,claims,alg,token,valid});
}
const malformed=await new CompactSign(new TextEncoder().encode('[]')).setProtectedHeader({alg:'HS256'}).sign(key);
vectors.push({name:'array-payload',token:malformed,valid:false});
for(const [name,header] of [['typ-number',{alg:'HS256',typ:123}],['b64-ignored',{alg:'HS256',b64:false}],['b64-critical-true',{alg:'HS256',b64:true,crit:['b64']}],['crit-empty',{alg:'HS256',crit:[]}],['crit-unknown',{alg:'HS256',crit:['x'],x:true}],['crit-missing',{alg:'HS256',crit:['b64']}]] ) {
 const protectedHeader=Buffer.from(JSON.stringify(header)).toString('base64url');
 const payload=Buffer.from('{}').toString('base64url');
 const input=protectedHeader+'.'+payload;
 const token=input+'.'+crypto.createHmac('sha256',key).update(input).digest('base64url');
 let valid=true;try{await jwtVerify(token,key,{currentDate:new Date(now*1000)});}catch{valid=false;}
 vectors.push({name,claims:{},header,token,valid});
}
{
 const protectedHeader=Buffer.from(JSON.stringify({alg:'HS256'})).toString('base64');
 const payload=Buffer.from(JSON.stringify({id:'padding'})).toString('base64');
 const input=protectedHeader+'.'+payload;
 const token=input+'.'+crypto.createHmac('sha256',key).update(input).digest('base64');
 let valid=true;try{await jwtVerify(token,key,{currentDate:new Date(now*1000)});}catch{valid=false;}
 vectors.push({name:'source-base64-padding',claims:{id:'padding'},token,valid});
}
{
 const protectedHeader=Buffer.from(JSON.stringify({alg:'HS256'})).toString('base64').replaceAll('+','-').replaceAll('/','_');
 const payload=Buffer.from(JSON.stringify({id:'padding-url'})).toString('base64').replaceAll('+','-').replaceAll('/','_');
 const input=protectedHeader+'.'+payload;
 const token=input+'.'+crypto.createHmac('sha256',key).update(input).digest('base64').replaceAll('+','-').replaceAll('/','_');
 let valid=true;try{await jwtVerify(token,key,{currentDate:new Date(now*1000)});}catch{valid=false;}
 vectors.push({name:'source-base64url-padding',claims:{id:'padding-url'},token,valid});
}
fs.writeFileSync(process.argv[3] ,JSON.stringify({now,jose:'6.2.0',vectors},null,2)+'\n');
console.log('Captured '+vectors.length+' original jose signature/claim vectors.');
