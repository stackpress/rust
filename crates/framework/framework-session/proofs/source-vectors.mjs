// Reproduce source fixtures outside the repository; supply upstream root and output path.
import fs from 'node:fs';
const upstream=process.argv[2];
const {encrypt,decrypt,hash}=await import(upstream+'/packages/stackpress-schema/src/helpers.ts');
const {init}=await import(upstream+'/node_modules/@paralleldrive/cuid2/src/index.js');
const cryptoVectors = [['',''],['hello','abc123'],['user@example.test','database-seed'],['é🔑\u0000line\n','κλειδί'],['1234567890123456','abc123']].map(([value,seed])=>({value,seed,ciphertext:encrypt(value,seed),hash:hash(value)}));
for(const v of cryptoVectors)if(decrypt(v.ciphertext,v.seed)!==v.value)throw Error('upstream crypto mismatch');
const time=1700000000123;
Date.now=()=>time;
const cuidVectors=[0,0.5,0.999999].flatMap(random=>[2,24,32].map(length=>({time,random,length,salt:Math.floor(random*36).toString(36).repeat(length),counter:42,fingerprint:'fixture-host',first:String.fromCharCode(97+Math.floor(random*26)),id:init({random:()=>random,counter:()=>42,length,fingerprint:'fixture-host'})()})));
const dateVectors=[0,1700000000123,1741064767000].map(ms=>({ms,date:new Date(ms).toString(),hash:hash(new Date(ms).toString()),otpHash:hash(new Date(ms).toString()+'000123')}));
const result={provenance:{schema:'packages/stackpress-schema/src/helpers.ts (direct import)',cuid2:'@paralleldrive/cuid2 3.3.0 (direct init)',timezone:process.env.TZ},cryptoVectors,cuidVectors,dateVectors};
fs.writeFileSync(process.argv[3],JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result,null,2));
