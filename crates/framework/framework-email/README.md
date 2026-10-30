# Direct email transports

`Service::new(config, Arc::new(NativeFactory::default()))` creates one owned
transport for each `send` or `send_native` call. `EmailConfig` retains the complete
source object/string union; `SendMailOptions` retains arbitrary message fields.
`NativeSendMailOptions` additionally owns Buffer/Readable equivalents at named
body, raw or indexed attachment/alternative slots. Provider results retain all
JSON fields, plus native stream/Buffer outputs. A live output reader must be
consumed explicitly; serialization returns an error without draining it.

The original Stackpress email event uses Nodemailer 7.0.13. Its constructor
selection order is URL resolution, pool, sendmail, stream, JSON, SES, then SMTP.
Object `url` replaces the sibling options before selection. SMTP service catalog
values override supplied settings. These rules and coercions are preserved by
`options::TransportOptions::resolve`; the original config still reaches any
application provider binding unchanged.

| Variant | Built-in direct mapping |
| --- | --- |
| SMTP | host/port, omitted secure at port 465, implicit TLS, optional/required/ignored STARTTLS, name, localAddress, connectionTimeout, LOGIN/PLAIN/static XOAUTH2, TLS servername/rejectUnauthorized/ca/minVersion/client PEM identity, optional advertised AUTH, partial RCPT acceptance, envelope/result fields. Bcc is omitted from generated MIME. |
| Pool | Complete source options and defaults (`maxConnections || 5`, `maxMessages || 100`, rateDelta/rateLimit) resolve now. The **complete pool goes to `with_provider`** because lettre lacks source maxMessages recycling, including the default 100. No provider means a named error before delivery. |
| JSON | Source address/metadata normalization, body strings, attachment/alternative Buffer base64, extension fields, source property order, envelope/normalizedHeaders removal, skipEncoding object versus JSON string. No delivery. |
| Stream | buffer returns bytes; otherwise an owned reader defers content IO/errors until read. Raw bytes remain exact. Source transport newline aliases are resolved; source stores its winbreak flag without rewriting raw bytes. |
| Sendmail | path/default sendmail, `-i`, explicit args (including an empty array) or default `-f from`, then envelope.to. Source leading-dash rejection and 0/127/other exit results are retained. Process IO/exit compete for first observed completion; child exit is awaited. Tests use a fake process and `/bin/cat` only. |
| SES | `with_ses` binds the native SESv2 client/command handles. Full Content.Raw.Data, FromEmailAddress and Destination.ToAddresses, then arbitrary mail.ses overrides. Region failure/missing falls back to us-east-1; source MessageId/response/raw projection is retained. No AWS SDK or live request is imposed. |
| String | smtp/smtps/direct schemes, decoded auth split at first colon, numeric/bool query coercion, first existing property, tls nesting, service resolution. Legacy URL syntax outside `url` crate representation goes to the complete native provider or a named error. |

`with_smtp`, `with_process`, `with_ses` and `with_content` replace the SDK/IO seams
without replacing option/result mapping. `with_provider` instead implements the
complete source transport contract when the built-in library cannot represent it.
The binding receives **the original complete config and message options**, including
owned native content, before any such content is consumed. Unsupported source
options are never silently discarded. Unknown extension fields remain in the bags;
source-ignored fields remain unused just as in the original.

Rust-mechanics deviations (library capability) requiring that full binding:

- LMTP, supplied/already-secured sockets, getSocket/customAuth callbacks, unrepresentable local
  binding/DNS controls, separate DNS/greeting/socket timeouts, opportunistic TLS
  failure recovery, proxy/logging callbacks, forceAuth, unknown auth mechanisms,
  OAuth2 refresh/provisioning, unsupported TLS options and native coercions.
- SMTP DSN, REQUIRETLS and SIZE extension handling (lettre's server capability
  parser does not expose all source extensions). SMTPUTF8 and use8BitMime are
  emitted when advertised. Unknown-only AUTH advertisements are not exposed by
  lettre; live negotiation beyond the in-memory known mechanisms is unverified.
- DKIM/compiler plugins, custom header normalization, attachDataUrls, watchHtml,
  AMP, iCalendar, List headers, custom boundaries, per-message auth, raw/custom
  attachment MIME headers, alternative filename parameters, root MIME overrides,
  newline compiler options, explicit word-encoding preferences for non-ASCII headers,
  group/permissive address syntax outside lettre.
- Native JSON raw readers cannot be represented as ordinary JSON. Source native
  callbacks/resources live in provider traits rather than serialized values.

Generated MIME uses lettre folding, RFC2231 formatting and multipart boundaries;
it is semantically tested, **not claimed byte-identical** to Nodemailer. Native
MIME preparation buffers inputs; stream results defer that preparation until read.
This is a Rust/library mechanics adaptation of the source streaming compiler.
Raw output bytes are tested exactly. Native SMTP command failures expose `SmtpError.fields` (source code/command,
responseCode/response and aggregate rejection details), retaining the Rust SDK
cause without claiming JavaScript Error prototype identity. Timings use a native
monotonic clock. Lettre does not expose the original EHLO response lines, so its
built-in result cannot include the source `ehlo` array. Applications needing that
SDK-specific data use their complete `TransportFactory` directly.

The pinned lettre Rustls identity helper stores its certificate argument as DER,
despite its `from_pem` name. The adapter explicitly converts one PEM certificate
to DER before that call; certificate-chain identities go to the complete native
provider or a named error. This is a library-mechanics adaptation, proven with
public test-only certificate material and no handshake. The SDK controls default
DNS/greeting/socket phase timing; explicit source phase timeout options select
the native binding. Live timing/negotiation equivalence is not claimed.

File/data URI/HTTP(S) content is supported through `ContentResolver`. Native fetch
uses source UA, gzip/deflate, a fresh cookie jar, five redirects, source TLS
verification setting and no added size/origin/timeout policy. Source MIME access
flags and transport-over-message precedence apply; source JSON normalization
bypasses those flags and does not use httpHeaders. HTTP behavior is exercised via
fake resolvers only; real HTTP/TLS, SMTP, SES and sendmail acceptance are unverified.
Native process cancellation uses Tokio kill-on-drop; completion is explicitly reaped.
No stronger cancellation-reaping claim is made for runtime shutdown.

`tests/source-transports.cjs` runs the original local package with inert SMTP,
process and SES boundaries; its checked-in fixture includes source file hashes.
`tests/source-options.cjs` retains the Stackpress event option/default oracle from
M0. Event/plugin registration and signup-hook integration belong to M2 and later
milestones; the direct factory and task-supervisor contracts are already available.
