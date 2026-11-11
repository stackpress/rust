// Execute the original email event with an inert transport, never a live provider.
// Only TypeScript type syntax/import/export scaffolding is removed; the handler is intact.
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

async function main() {
  const root = process.argv[2];
  const output = process.argv[3];
  assert(root && output, 'usage: node source-options.cjs <upstream package src> <fixture output>');
  const file = path.join(root, 'events.ts');
  const source = fs.readFileSync(file, 'utf8');
  // Fail if source shape drifts instead of silently running a different handwritten event.
  for (const fragment of ['ctx.config<EmailConfig>', 'req.data<SendMailOptions>', 'new Promise<SentMessageInfo>']) {
    assert.equal(source.split(fragment).length, 2, fragment);
  }
  const runnable = source.replace(/^import .*;\r?\n/gm, '')
    .replace('ctx.config<EmailConfig>', 'ctx.config')
    .replace('req.data<SendMailOptions>', 'req.data')
    .replace('new Promise<SentMessageInfo>', 'new Promise')
    .replace('export default emitter;', 'return emitter;');
  const variants = [null, false, 0, '', {},
    {host:'localhost', port:25, secure:false, tls:{rejectUnauthorized:false}},
    {pool:true, maxConnections:2}, {jsonTransport:true},
    {streamTransport:true, buffer:false}, {sendmail:true, args:['-i']},
    {SES:{extension:'native-callback-fixture'}},
    'smtp://user:pass@localhost:25/?pool=true',
    {customProvider:{field:false}}];
  const options = {from:'sender@example.test', to:['recipient@example.test'],
    text:'fixture', html:'<p>fixture</p>', attachments:[{path:'/fixture', href:'https://example.test/fixture'}],
    headers:{'X-Fixture':'yes'}, extension:{empty:[], flag:false}};
  const info = {messageId:'fixture-id', accepted:['recipient@example.test'],
    rejected:[], response:'fixture provider response', extension:{retained:true}};
  const cases = [];
  for (const config of variants) {
    const calls = [];
    const fake = {createTransport(received) {
      calls.push(['create', received]);
      return {sendMail(receivedOptions, callback) {
        calls.push(['send', receivedOptions]);
        callback(null, info);
      }};
    }};
    const server = () => ({on(name, handler) { assert.equal(name, 'email-send'); this.handler = handler; }});
    const emitter = new Function('nodemailer', 'server', runnable)(fake, server);
    let result = null;
    await emitter.handler({req:{data:()=>options}, res:{results:value=>{result=value;}}, ctx:{config:()=>config}});
    cases.push({config, options, calls, result});
  }
  fs.writeFileSync(output, JSON.stringify({source:'packages/stackpress-email/src/events.ts',
    sha256:crypto.createHash('sha256').update(source).digest('hex'), cases}, null, 2)+'\n');
  process.stdout.write(`Original email event: ${cases.length} config/default cases, fake transports only\n`);
}
main().catch(error => { process.stderr.write(String(error)+'\n'); process.exitCode=1; });
