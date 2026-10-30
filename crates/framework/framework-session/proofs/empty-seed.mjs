const {SignJWT,jwtVerify}=await import(process.argv[2]+'/node_modules/jose/dist/webapi/index.js');
import crypto from 'node:crypto';
import fs from 'node:fs';
const key=new Uint8Array();let signError=false;
try{await new SignJWT({}).setProtectedHeader({alg:'HS256'}).sign(key);}catch{signError=true;}
const input=Buffer.from('{"alg":"HS256"}').toString('base64url')+'.'+Buffer.from('{}').toString('base64url');
const token=input+'.'+crypto.createHmac('sha256',key).update(input).digest('base64url');
let verified=true;try{await jwtVerify(token,key);}catch{verified=false;}
const result={signError,verified,token};
fs.writeFileSync(process.argv[3],JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result));
