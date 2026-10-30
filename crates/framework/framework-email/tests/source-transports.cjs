// Run only local transports and inert SDK/process boundaries against original Nodemailer.
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const {EventEmitter} = require('node:events');
const {PassThrough,Readable} = require('node:stream');
const crypto = require('node:crypto');

async function main() {
  const root = process.argv[2], output = process.argv[3];
  const nm = require(root), Connection = require(path.join(root,'lib/smtp-connection'));
  const configs = [{},{secure:true},{port:465},{port:465,secure:false},{port:0,secure:false},
    {service:'google mail',host:'ignored.test',port:25,secure:false},
    {pool:true},{pool:true,maxConnections:0,maxMessages:4,rateDelta:0,rateLimit:'5'},
    {pool:true,sendmail:true,jsonTransport:true},{sendmail:true,streamTransport:true},
    {streamTransport:true,newline:'WINDOWS'},{jsonTransport:true,skipEncoding:true},
    {SES:{}},'smtps://user:p%3Aa%2Bss@localhost:2465/?pool=true&maxConnections=2&tls.rejectUnauthorized=false',
    'smtp://us%3Aer:p@localhost/',
    'smtp://localhost/?port=99&secure=true&x=1&x=2&tls.a=1&tls.a=2&other.nested=skip&empty=',
    {url:'smtp://localhost:25/?jsonTransport=true',pool:true,ignored:'discarded-before-selection'}];
  const kind = name => ({SMTP:'Smtp','SMTP (pool)':'Pool',JSONTransport:'Json',StreamTransport:'Stream',Sendmail:'Sendmail',SESTransport:'SesTransport'})[name];
  const selection=[];
  for (const config of configs) {
    const mailer=nm.createTransport(config), t=mailer.transporter;
    const row={config,kind:kind(t.name),fields:t.options};
    if (t.name.startsWith('SMTP')) {
      const conn=new Connection(t.options);
      row.smtp={host:conn.host,port:conn.port,secure:conn.secureConnection,secured:conn.alreadySecured};
      if (t._rateLimit) row.rate=[t._rateLimit.delta,t._rateLimit.limit];
    }
    selection.push(row); mailer.close();
  }
  const base={from:'Sender <sender@example.test>',to:'recipient@example.test',messageId:'fixture@example.test',date:'Thu, 01 Jan 1970 00:00:00 +0000'};
  const jsonCases=[];
  const messages=[{}, {text:'hello',html:'<p>hello</p>'},
    {from:['First <first@example.test>','second@example.test'],sender:'Agent <agent@example.test>',cc:['cc@example.test'],bcc:'Hidden <hidden@example.test>',replyTo:{name:'Reply',address:'reply@example.test'}},
    {envelope:{from:'return@example.test',to:['envelope@example.test'],extra:{retained:true}},headers:{'X-Array':['one','two']},extension:{retained:true}},
    {attachments:[{filename:false,content:'abc'},{content:Buffer.from([0,1,255]),contentType:'image/png'},{path:'data:text/plain;base64,YWJj'}]},
    {text:{content:'aGVsbG8=',encoding:'base64'},watchHtml:'watch',amp:{content:'amp'},icalEvent:{method:'REQUEST',content:Buffer.from('calendar')}},
    {alternatives:[{content:Buffer.from('other'),contentType:'text/x-fixture'}],raw:'Subject: raw\r\n\r\nbody'},
    {to:['recipient@example.test','recipient@example.test'],cc:'recipient@example.test',envelope:{from:'Bounce <bounce@example.test>',to:['One <one@example.test>'],cc:['one@example.test','two@example.test'],extension:17}},
    {from:false,to:false,messageId:false,headers:[{key:'From',value:'Custom <custom@example.test>'},{key:'To',value:'custom-to@example.test'},{key:'Message-ID',value:'custom@example.test'}]},
  ];
  for (const skipEncoding of [false,true]) for (const message of messages) {
    const config={jsonTransport:true,skipEncoding}; const options={...base,...message};
    const result=await nm.createTransport(config).sendMail(options);
    jsonCases.push({config,options,result});
  }
  const raw='From: source@example.test\r\nSubject: raw\r\n\r\nBody.\r\n';
  const streamCases=[];
  for (const config of [{streamTransport:true,buffer:true},{streamTransport:true,buffer:true,newline:'windows'},{streamTransport:true,buffer:false}]) {
    const options={...base,raw}; const result=await nm.createTransport(config).sendMail(options);
    if (!Buffer.isBuffer(result.message)) { const chunks=[];for await(const chunk of result.message)chunks.push(chunk);result.message=Buffer.concat(chunks); }
    streamCases.push({config,options,result});
  }
  const sendmailCases=[];
  for (const [extra,code] of [[{},0],[{args:[]},0],[{path:'/fixture/sendmail',args:['-v']},127],[{},3]]) {
    const config={sendmail:true,...extra}, options={...base,raw,envelope:{from:'return@example.test',to:['one@example.test','two@example.test']}};
    let invocation;const mailer=nm.createTransport(config);
    mailer.transporter._spawn=(program,args)=>{const child=new EventEmitter();child.stdin=new PassThrough();const chunks=[];child.stdin.on('data',data=>chunks.push(data));child.stdin.on('finish',()=>{invocation={program,args,message:Buffer.concat(chunks)};queueMicrotask(()=>child.emit('exit',code));});child.kill=()=>{};return child;};
    let result=null,error=null;try{result=await mailer.sendMail(options);}catch(e){error=e.message;}
    sendmailCases.push({config,options,invocation,result,error});
  }
  const sesCases=[];
  for (const [region,id,overrides] of [['us-east-1','provider-id',{}],['eu-west-1','existing@example.test',{ConfigurationSetName:'fixture',Destination:{ToAddresses:['override@example.test']}}],[null,'another',{}]]) {
    let input;const sesClient={config:{region:async()=>{if(region===null)throw new Error('region failed');return region;}},send:async command=>{input=command.input;return {MessageId:id};}};
    class SendEmailCommand {constructor(value){this.input=value;}}
    const options={...base,raw,ses:overrides}; const result=await nm.createTransport({SES:{sesClient,SendEmailCommand}}).sendMail(options);
    sesCases.push({region,id,options,input,result});
  }
  // Patch only the SDK connection methods. No native socket is opened by this SMTP proof.
  const saved={};for(const key of ['connect','send','quit','close'])saved[key]=Connection.prototype[key];
  const smtpCases=[];
  let captured;
  Connection.prototype.connect=function(cb){queueMicrotask(cb);};
  Connection.prototype.quit=function(){};Connection.prototype.close=function(){};
  Connection.prototype.send=function(envelope,stream,cb){const chunks=[];stream.on('data',v=>chunks.push(v));stream.on('end',()=>{captured={fields:this.options,envelope,message:Buffer.concat(chunks)};cb(null,{accepted:['recipient@example.test'],rejected:[],response:'250 fixture',envelopeTime:1,messageTime:2,messageSize:3});});};
  for(const [config,extra] of [[{},{}],[{host:'relay.example.test',port:25,ignoreTLS:true},{}],[{service:'Gmail',auth:{user:'fixture',pass:'fixture'}},{}],[{}, {envelope:{from:'Bounce <bounce@example.test>',to:['One <one@example.test>'],cc:['one@example.test','two@example.test'],extension:17}}]]) {
    const options={...base,raw,...extra};const result=await nm.createTransport(config).sendMail(options);smtpCases.push({config,options,captured,result});
  }
  Object.assign(Connection.prototype,saved);
  const mimeCases=[];
  const originalRead=fs.createReadStream;
  fs.createReadStream=(file,...args)=>file==='/fixture/mime-alternative.html'?Readable.from(['fixture content']):originalRead(file,...args);
  for (const extra of [
    {text:'hello'}, {text:'hello',html:'<b>hello</b>'},
    {alternatives:[{path:'/fixture/mime-alternative.html'}]},
    {alternatives:[{path:'/fixture/mime-alternative.html'}],textEncoding:' base64 '},
    {text:'a'.repeat(80),textEncoding:'base64'}, {text:'café',textEncoding:'quoted-printable'},
    {html:'<img src="cid:fixture">',attachments:[{filename:'logo.png',cid:'fixture',content:Buffer.from([1,2,3])},{filename:false,content:'plain'}]},
    {attachments:[{filename:'forward.eml',contentType:'message/rfc822',content:'From: a@example.test\r\n\r\nforward'}]},
    {attachments:[{content:'abc',filename:'test.txt',contentDisposition:'x-fixture'}]},
    {text:'hello',priority:'high',references:['one@example.test','<two@example.test>'],headers:{'X-Extra':['one','two'],'X-Priority':'bad'}},
    {text:'hello',priority:'low',from:false,to:false,messageId:false,headers:{From:'Custom <custom@example.test>',To:'custom-to@example.test','Message-ID':'custom@example.test'}},
    {alternatives:[{content:'other',contentType:'text/x-fixture'}],attachments:[{path:'data:text/plain;base64,YWJj'}]},
  ]) {
    const options={...base,...extra};const result=await nm.createTransport({streamTransport:true,buffer:true}).sendMail(options);
    mimeCases.push({options,envelope:result.envelope,messageId:result.messageId,message:result.message.toString()});
  }
  fs.createReadStream=originalRead;
  const files=['lib/nodemailer.js','lib/shared/index.js','lib/json-transport/index.js','lib/stream-transport/index.js','lib/sendmail-transport/index.js','lib/ses-transport/index.js','lib/smtp-transport/index.js','lib/smtp-pool/index.js','lib/mail-composer/index.js','lib/mime-node/index.js','lib/fetch/index.js'];
  const hashes=Object.fromEntries(files.map(file=>[file,crypto.createHash('sha256').update(fs.readFileSync(path.join(root,file))).digest('hex')]));
  fs.writeFileSync(output,JSON.stringify({version:require(path.join(root,'package.json')).version,hashes,selection,jsonCases,mimeCases,streamCases,sendmailCases,sesCases,smtpCases},null,2)+'\n');
  process.stdout.write(`Original transport oracle: ${selection.length} selection, ${jsonCases.length} JSON, ${streamCases.length} stream, ${sendmailCases.length} fake sendmail, ${sesCases.length} fake SES, ${smtpCases.length} fake SMTP cases\n`);
}
main().catch(error=>{process.stderr.write(String(error.stack||error)+'\n');process.exitCode=1;});
