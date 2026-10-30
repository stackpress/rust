// Execute only the original pure checklist/evaluation declarations; no React runtime or DOM.
import fs from 'node:fs';
import {createRequire} from 'node:module';
const root=process.argv[2],require=createRequire(root+'/package.json'),ts=require('typescript');
const source=fs.readFileSync(root+'/packages/stackpress-session/src/auth/views/signup.tsx','utf8');
const ast=ts.createSourceFile('signup.tsx',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TSX);
const declarations=ast.statements.filter(node=>ts.isFunctionDeclaration(node)&&['passwordChecklist','evaluatePassword'].includes(node.name?.text)).map(node=>node.getText(ast)).join('\n');
const module={exports:{}};
new Function('exports','module',ts.transpileModule(declarations,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText)(module.exports,module);
const rules=[{}, {min:8},{max:12},{min:8,max:12},{min:12,max:8},{min:0,max:0},{upper:true,lower:true,number:true,special:true},{min:8,max:8},{min:8,max:12,upper:true,lower:true,number:true,special:true}];
const cases=[];
for(const rule of rules)for(const secret of ['', 'abc', 'Ab1!', 'Abcdef12!', '            ', 'é😀', 'A1_\uFEFF', '12345678901234567890']){
 const checks=module.exports.passwordChecklist(rule),result=module.exports.evaluatePassword(secret,checks);
 cases.push({rule,secret,checks:checks.map(({id,label,weight,score,pass})=>({id,label,weight,score:score(secret),pass:pass(secret)})),result});
}
fs.writeFileSync(process.argv[3],JSON.stringify(cases,null,2)+'\n');console.log('Original signup strength oracle: '+cases.length+' cases');
