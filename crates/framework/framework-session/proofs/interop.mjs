// Verify the actual Rust-produced token using the upstream library, without fetching packages.
import fs from 'node:fs';
const upstream=process.argv[2];
const {jwtVerify}=await import(upstream+'/node_modules/jose/dist/webapi/index.js');
const {token}=JSON.parse(fs.readFileSync(process.argv[3],'utf8'));
const result=await jwtVerify(token,new TextEncoder().encode('abc123'));
if(result.payload.id!=='interop'||result.payload.roles[0]!=='USER'||result.protectedHeader.alg!=='HS256'||'typ' in result.protectedHeader||'exp' in result.payload)throw Error('Rust source contract mismatch');
console.log('Original jose verified the Rust-produced token: alg-only header, short seed, optional expiry, retained object claims.');
