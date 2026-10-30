// Execute original handlers with inert host/provider seams; never open a socket or process.
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

/** Remove only checked TypeScript scaffolding, keeping original event control flow. */
function eventBody(source) {
  for (const fragment of ['ctx.config<EmailConfig>', 'req.data<SendMailOptions>', 'new Promise<SentMessageInfo>']) {
    assert.equal(source.split(fragment).length, 2, fragment);
  }
  return source.replace(/^import .*;\r?\n/gm, '')
    .replace('ctx.config<EmailConfig>', 'ctx.config')
    .replace('req.data<SendMailOptions>', 'req.data')
    .replace('new Promise<SentMessageInfo>', 'new Promise')
    .replace('export default emitter;', 'return emitter;');
}

/** Record values at the source invocation boundary, before later source mutations. */
function snapshot(value) { return JSON.parse(JSON.stringify(value)); }

/** Run body/status/config/error/result branches against the original event. */
async function main() {
  const root = process.argv[2];
  const output = process.argv[3];
  assert(root && output, 'usage: node source-events.cjs <upstream packages> <fixture>');
  const files = ['stackpress-email/src/events.ts', 'stackpress-email/src/plugin.ts',
    'stackpress-session/src/plugin.ts', 'stackpress-session/src/auth/events/signup.ts'];
  const sources = Object.fromEntries(files.map(file => [file, fs.readFileSync(path.join(root, file), 'utf8')]));
  const options = {from:'sender@example.test', to:['recipient@example.test'], text:'fixture',
    extension:{empty:[], flag:false}, headers:{'X-Fixture':'yes'}};
  const info = {messageId:'provider-id', accepted:['recipient@example.test'], rejected:[], extension:{raw:true}};
  const cases = [];
  const base = {body:null, code:null, config:{jsonTransport:true}, info};
  const inputs = [
    ...[null, false, 0, '', {}, [], 'body', true, 1].map(body => ({name:`body ${JSON.stringify(body)}`, body})),
    ...[0, 200, 201, 302, 419, 500].map(code => ({name:`code ${code}`, code})),
    ...[null, false, 0, '', {}, {pool:true}, {streamTransport:true}, {sendmail:true}, {SES:{}},
      'smtp://fixture', {customProvider:{extension:false}}].map(config => ({name:`config ${JSON.stringify(config)}`, config})),
    {name:'missing config', missing:true}, {name:'create throw', failure:'create'},
    {name:'send callback error', failure:'send'},
    ...[null, false, 0, '', [], {}].map(info => ({name:`info ${JSON.stringify(info)}`, info})),
    {name:'skip before invalid config', body:'already answered', config:true},
    {name:'skip before provider failure', code:419, failure:'create'},
  ];
  for (const input of inputs) {
    const item = {...base, ...input};
    const trace = [];
    const failure = Object.assign(new Error(`${item.failure} failed`), {code:'EPROVIDER', extension:{raw:true}});
    const fake = {createTransport(config) {
      trace.push(['create', config]);
      if (item.failure === 'create') throw failure;
      return {sendMail(data, callback) {
        trace.push(['send', data]);
        if (item.failure === 'send') callback(failure);
        else callback(null, item.info);
      }};
    }};
    const emitter = new Function('nodemailer', 'server', eventBody(sources[files[0]]))(fake,
      () => ({on(name, handler) { assert.equal(name, 'email-send'); this.handler = handler; }}));
    let result = null;
    let error = null;
    try {
      await emitter.handler({req:{data() { trace.push(['data']); return options; }},
        res:{body:item.body, code:item.code, results(value) { trace.push(['results', value]); result=value; }},
        ctx:{config(key) { trace.push(['config', key]); return item.missing ? undefined : item.config; }}});
    } catch (cause) {
      error = {message:cause.message, code:cause.code, extension:cause.extension};
    }
    cases.push({...item, options, trace, result, error});
  }
  // Keep the original selection has and Listen truthiness checks distinct.
  const registrations = [];
  for (const config of [undefined, null, false, 0, '', {}, {jsonTransport:true}, 'smtp://fixture']) {
    const trace = [];
    let listen;
    const current = config;
    const ctx = {config:{has(key) { trace.push(['has', key]); return config !== undefined; },
      get(key) { trace.push(['get', key]); return current; }},
      on(name, handler) { trace.push(['on', name]); listen=handler; },
      use() { trace.push(['use', 'email-send']); }};
    // The imported emitter is inert; the original plugin retains all phase gates.
    const withEmitter = new Function('emitter', sources[files[1]].replace(/^import .*;\r?\n/gm, '')
      .replace('export default function plugin(ctx: Server)', 'function plugin(ctx)')+'\nreturn plugin;')({});
    withEmitter(ctx);
    if (listen) listen({ctx});
    registrations.push({missing:config === undefined, config:config ?? null, trace});
  }
  // Execute original signup. Pending resolve promises establish that it never awaits either hook.
  const signupSource = sources[files[3]];
  const inputDeclaration = 'const input: Record<string, any> & { roles: string[] } = {';
  assert(signupSource.includes(inputDeclaration));
  const runnable = signupSource.replace(/^import .*;\r?\n/gm, '')
    .replace('export default action', 'return action')
    .replace(inputDeclaration, 'const input = {')
    .replace('let results: Partial<ProfileAuth>;', 'let results;')
    .replace('ctx.config.path<string[]>', 'ctx.config.path')
    .replaceAll('(e as Error)', '(e)').replaceAll('(e as Exception)', '(e)')
    .replace('(req.data() as Record<string, any>)', '(req.data())');
  const signup = [];
  for (const input of [{email:'user@example.test', phone:'+15555550123'}, {email:'user@example.test'}, {},
    {email:'', phone:'', roles:['OVERRIDE']}]) {
    const trace = [];
    const result = {id:'profile', auth:{email:{secret:'email-secret'}, phone:{secret:'phone-secret'}, username:{secret:'name-secret'}}};
    const fn = new Function('action', 'AuthActions', 'normalizePhone', runnable)(handler=>handler,
      {make() { trace.push(['actions']); return {async signup(data) { trace.push(['signup', snapshot(data)]); return result; }}; }},
      phone=>phone || undefined);
    await fn({req:{data() { trace.push(['data']); return input; }},
      res:{results(data) { trace.push(['results', snapshot(data)]); }},
      ctx:{config:{path(name, fallback) { trace.push(['roles', name]); return ['USER']; }},
        resolve(name, data) { trace.push(['resolve', name, snapshot(data)]); return new Promise(()=>{}); }}});
    signup.push({input, trace});
  }
  // Composite source is evaluated with function seams to pin auth -> email -> session.
  const composite = [];
  const compose = new Function('auth', 'email', 'session', sources[files[2]].replace(/^import .*;?\r?\n/gm, '')
    .replace('export default function plugin(ctx: Server)', 'function plugin(ctx)')+'\nreturn plugin;')(
      ()=>composite.push('auth'), ()=>composite.push('email'), ()=>composite.push('session'));
  compose({});
  fs.writeFileSync(output, JSON.stringify({sources:files.map(file=>({file,
    sha256:crypto.createHash('sha256').update(sources[file]).digest('hex')})), cases, registrations, signup, composite}, null, 2)+'\n');
  console.log(`Original events: ${cases.length} email traces, ${registrations.length} registration traces, ${signup.length} signup traces; all providers inert`);
}
main().catch(error => { console.error(error); process.exitCode=1; });
