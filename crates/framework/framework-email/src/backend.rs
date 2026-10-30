//! Native SMTP/sendmail/SES boundaries; fakes exercise adapters without delivery.

use crate::options::{SmtpOptions, flag, number, string};
use crate::{EmailResult, SentMessageInfo, SmtpError};
use lettre::transport::smtp::{
    authentication::{Credentials, Mechanism},
    client::{
        AsyncSmtpConnection, Certificate, CertificateStore, Identity, Tls, TlsParameters,
        TlsVersion,
    },
    extension::{ClientId, Extension},
};
use serde_json::{Value, json};
use std::{
    future::Future,
    io,
    pin::Pin,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::io::AsyncWriteExt;

/// Owned native provider operation, movable between executor threads.
pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = EmailResult<T>> + Send + 'a>>;

/// Complete SMTP SDK invocation after source option mapping and MIME compilation.
pub struct SmtpRequest {
    /// Full source constructor fields plus effective connection defaults.
    pub options: SmtpOptions,
    /// Original source envelope; extensions remain visible to a bound provider.
    pub envelope: Value,
    /// Compiled message bytes; Bcc is removed only from the message headers.
    pub message: Vec<u8>,
}

/// SDK seam used by the lettre adapter and deterministic no-network tests.
pub trait SmtpBackend: Send + Sync {
    /// Send one native SMTP request, preserving the provider's result or error.
    fn send(&self, request: SmtpRequest) -> ProviderFuture<'_, SentMessageInfo>;
}

/// Built-in approved lettre SMTP adapter with source-shaped connection sequencing.
#[derive(Default)]
pub struct LettreBackend;

/// One native connection mapping, inspected without IO by the protocol/option proofs.
struct ConnectionSettings {
    /// Validated protocol port.
    port: u16,
    /// Source local interface, when representable by the SDK IP binding.
    local_address: Option<std::net::IpAddr>,
    /// Source connection timeout, converted from milliseconds.
    timeout: Duration,
    /// Configured name or source-compatible OS/FQDN fallback.
    hello: ClientId,
    /// Source implicit/STARTTLS/disabled choice.
    tls: Tls,
    /// Optional source auth; an empty object does not activate authentication.
    auth: Option<(Credentials, Vec<Mechanism>)>,
}

impl ConnectionSettings {
    /// Validate only native protocol representations and map the source defaults.
    fn new(options: &SmtpOptions) -> EmailResult<Self> {
        if !options.port.is_finite()
            || options.port.fract() != 0.0
            || !(1.0..=65535.0).contains(&options.port)
        {
            return Err(io::Error::other("Invalid SMTP port").into());
        }
        let fields = &options.fields;
        let local_address = fields
            .get("localAddress")
            .filter(|v| crate::options::truthy(v))
            .map(string)
            .map(|v| v.parse())
            .transpose()?;
        let timeout = fields
            .get("connectionTimeout")
            .and_then(number)
            .filter(|v| *v != 0.0)
            .unwrap_or(120_000.0);
        let timeout = Duration::try_from_secs_f64(timeout / 1000.0)
            .map_err(|_| io::Error::other("native provider required for SMTP timeout coercion"))?;
        let hello = match fields.get("name").filter(|v| crate::options::truthy(v)) {
            Some(name) => ClientId::Domain(string(name)),
            None => match ClientId::default() {
                ClientId::Domain(name) if !name.contains('.') => {
                    ClientId::Ipv4(std::net::Ipv4Addr::LOCALHOST)
                }
                ClientId::Domain(name) => match name.parse() {
                    Ok(ip) => ClientId::Ipv4(ip),
                    Err(_) => ClientId::Domain(name),
                },
                value => value,
            },
        };
        let tls = if options.secure || !flag(fields, "ignoreTLS") {
            let tls = fields.get("tls").and_then(Value::as_object);
            let domain = tls
                .and_then(|v| v.get("servername"))
                .or_else(|| {
                    fields
                        .get("servername")
                        .filter(|v| crate::options::truthy(v))
                })
                .map(string)
                .unwrap_or_else(|| options.host.clone());
            let mut params = TlsParameters::builder(domain);
            if tls
                .and_then(|v| v.get("rejectUnauthorized"))
                .and_then(Value::as_bool)
                == Some(false)
            {
                // Source explicitly allows this developer-selected verification setting.
                params = params
                    .dangerous_accept_invalid_certs(true)
                    .dangerous_accept_invalid_hostnames(true);
            }
            if let Some(ca) = tls.and_then(|v| v.get("ca")) {
                params = params.certificate_store(CertificateStore::None);
                for value in ca.as_array().cloned().unwrap_or_else(|| vec![ca.clone()]) {
                    params =
                        params.add_root_certificate(Certificate::from_pem(&pem_bytes(&value)?)?);
                }
            }
            if let Some(version) = tls
                .and_then(|v| v.get("minVersion"))
                .filter(|v| crate::options::truthy(v))
            {
                params = params.set_min_tls_version(match string(version).as_str() {
                    "TLSv1.2" => TlsVersion::Tlsv12,
                    "TLSv1.3" => TlsVersion::Tlsv13,
                    _ => {
                        return Err(io::Error::other(
                            "native SMTP provider required for TLS minVersion",
                        )
                        .into());
                    }
                });
            }
            if let Some(tls) = tls
                && let (Some(cert), Some(key)) = (
                    tls.get("cert").filter(|v| crate::options::truthy(v)),
                    tls.get("key").filter(|v| crate::options::truthy(v)),
                )
            {
                params = params.identify_with(Identity::from_pem(
                    &identity_certificate_der(cert)?,
                    &pem_bytes(key)?,
                )?);
            }
            let params = params.build()?;
            if options.secure {
                Tls::Wrapper(params)
            } else if flag(fields, "requireTLS") {
                Tls::Required(params)
            } else {
                Tls::Opportunistic(params)
            }
        } else {
            Tls::None
        };
        let auth = match fields
            .get("auth")
            .and_then(Value::as_object)
            .filter(|auth| !auth.is_empty())
        {
            None => None,
            Some(auth) => {
                let user = auth
                    .get("user")
                    .filter(|v| crate::options::truthy(v))
                    .map(string)
                    .unwrap_or_default();
                let oauth = auth
                    .get("type")
                    .map(string)
                    .is_some_and(|v| v.eq_ignore_ascii_case("oauth2"));
                if oauth && user.is_empty() && !flag(auth, "service") {
                    None
                } else {
                    let secret = auth
                        .get(if oauth { "accessToken" } else { "pass" })
                        .map(string)
                        .unwrap_or_default();
                    let method = auth
                        .get("method")
                        .filter(|v| crate::options::truthy(v))
                        .map(string)
                        .filter(|v| !v.trim().is_empty())
                        .or_else(|| {
                            fields
                                .get("authMethod")
                                .filter(|v| crate::options::truthy(v))
                                .map(string)
                        });
                    let mechanisms = if oauth {
                        vec![Mechanism::Xoauth2]
                    } else {
                        match method
                            .as_deref()
                            .map(str::trim)
                            .map(str::to_uppercase)
                            .as_deref()
                        {
                            None | Some("") => vec![Mechanism::Plain, Mechanism::Login],
                            Some("LOGIN") => vec![Mechanism::Login],
                            Some("PLAIN") => vec![Mechanism::Plain],
                            Some("XOAUTH2") => vec![Mechanism::Xoauth2],
                            Some(method) => {
                                return Err(io::Error::other(format!(
                                    "native SMTP provider required for auth method {method}"
                                ))
                                .into());
                            }
                        }
                    };
                    Some((Credentials::new(user, secret), mechanisms))
                }
            }
        };
        Ok(Self {
            port: options.port as u16,
            local_address,
            timeout,
            hello,
            tls,
            auth,
        })
    }
}

impl SmtpBackend for LettreBackend {
    /// Use lettre's connection API to preserve source optional auth and partial RCPT success.
    fn send(&self, request: SmtpRequest) -> ProviderFuture<'_, SentMessageInfo> {
        Box::pin(async move {
            let settings = ConnectionSettings::new(&request.options)?;
            let implicit = match &settings.tls {
                Tls::Wrapper(params) => Some(params.clone()),
                _ => None,
            };
            let mut connection = AsyncSmtpConnection::connect_tokio1(
                (request.options.host.as_str(), settings.port),
                Some(settings.timeout),
                &settings.hello,
                implicit,
                settings.local_address,
            )
            .await?;
            match &settings.tls {
                Tls::Required(params) => {
                    connection.starttls(params.clone(), &settings.hello).await?
                }
                Tls::Opportunistic(params) if connection.can_starttls() => {
                    connection.starttls(params.clone(), &settings.hello).await?
                }
                _ => {}
            }
            authenticate(&mut connection, &settings).await?;
            // Dropping the owned connection closes it after the callback result, as source close().
            send_connection(&mut connection, &request.envelope, &request.message).await
        })
    }
}

/// Preserve source authentication activation after EHLO: missing AUTH remains a no-op.
async fn authenticate(
    connection: &mut AsyncSmtpConnection,
    settings: &ConnectionSettings,
) -> EmailResult<()> {
    if let Some((credentials, mechanisms)) = &settings.auth
        && [Mechanism::Plain, Mechanism::Login, Mechanism::Xoauth2]
            .iter()
            .any(|method| connection.server_info().supports_auth_mechanism(*method))
    {
        connection.auth(mechanisms, credentials).await?;
    }
    Ok(())
}

/// Execute the source MAIL/RCPT/DATA sequence using the SDK's framing/response parser.
async fn send_connection(
    connection: &mut AsyncSmtpConnection,
    envelope: &Value,
    message: &[u8],
) -> EmailResult<SentMessageInfo> {
    use lettre::transport::smtp::{
        commands::{Data, Mail, Rcpt},
        extension::{MailBodyParameter, MailParameter},
    };
    let from: Option<lettre::Address> = envelope
        .get("from")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .map(str::parse)
        .transpose()?;
    let to = envelope_recipients(envelope);
    let recipients = to
        .iter()
        .map(|v| v.parse())
        .collect::<Result<Vec<lettre::Address>, _>>()?;
    let mut parameters = Vec::new();
    // Source adds SMTPUTF8 only when advertised; it adds no new mandatory-extension gate.
    if from
        .iter()
        .chain(&recipients)
        .any(|address| !address.to_string().is_ascii())
        && connection
            .server_info()
            .supports_feature(Extension::SmtpUtfEight)
    {
        parameters.push(MailParameter::SmtpUtfEight);
    }
    if envelope
        .get("use8BitMime")
        .is_some_and(crate::options::truthy)
        && connection
            .server_info()
            .supports_feature(Extension::EightBitMime)
    {
        parameters.push(MailParameter::Body(MailBodyParameter::EightBitMime));
    }
    let envelope_start = Instant::now();
    connection
        .command(Mail::new(from, parameters))
        .await
        .map_err(|error| {
            SmtpError::response(error, "Mail command failed", "EENVELOPE", "MAIL FROM")
        })?;
    if recipients.is_empty() {
        return Err(SmtpError::no_recipients().into());
    }
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut errors = Vec::new();
    let mut last_rejection = None;
    for recipient in &recipients {
        match connection
            .command(Rcpt::new(recipient.clone(), vec![]))
            .await
        {
            Ok(_) => accepted.push(recipient.to_string()),
            Err(error) if error.is_permanent() || error.is_transient() => {
                rejected.push(recipient.to_string());
                let mut rejection =
                    SmtpError::response(error, "Recipient command failed", "EENVELOPE", "RCPT TO");
                rejection.fields["recipient"] = json!(recipient.to_string());
                errors.push(rejection.fields.clone());
                last_rejection = rejection.take_cause();
            }
            Err(error) => return Err(error.into()),
        }
    }
    if accepted.is_empty() {
        // At least one recipient existed; every response produced a retained rejection.
        if let Some(cause) = last_rejection {
            let mut error = SmtpError::response(
                cause,
                "Can't send mail - all recipients were rejected",
                "EENVELOPE",
                "RCPT TO",
            );
            error.fields["rejected"] = json!(rejected);
            error.fields["rejectedErrors"] = json!(errors);
            return Err(error.into());
        }
        return Err(SmtpError::no_recipients().into());
    }
    let envelope_time = envelope_start.elapsed().as_millis();
    let message_start = Instant::now();
    connection
        .command(Data)
        .await
        .map_err(|error| SmtpError::response(error, "Data command failed", "EENVELOPE", "DATA"))?;
    // Source normalizes bare LF and supplies exactly one final line break before the dot.
    // Lettre always supplies that break, so remove only the matching terminal break here.
    let mut normalized = Vec::with_capacity(message.len());
    for (index, byte) in message.iter().enumerate() {
        if *byte == b'\n' && (index == 0 || message[index - 1] != b'\r') {
            normalized.push(b'\r');
        }
        normalized.push(*byte);
    }
    if normalized.ends_with(b"\r\n") {
        normalized.truncate(normalized.len() - 2);
    } else if normalized.ends_with(b"\r") {
        normalized.pop();
    }
    let response = connection
        .message(&normalized)
        .await
        .map_err(|error| SmtpError::response(error, "Message failed", "EMESSAGE", "DATA"))?;
    let mut info = json!({"accepted":accepted,"rejected":rejected,"response":format!("{} {}",response.code(),response.message().collect::<Vec<_>>().join("\n")),"envelopeTime":envelope_time,"messageTime":message_start.elapsed().as_millis(),"messageSize":message.len()});
    if !errors.is_empty() {
        info["rejectedErrors"] = json!(errors);
    }
    Ok(info.into())
}

/// Lettre 0.11.23 Rustls Identity::from_pem stores its cert argument as DER; adapt that SDK mechanic.
fn identity_certificate_der(value: &Value) -> EmailResult<Vec<u8>> {
    use base64::Engine;
    let pem = String::from_utf8(pem_bytes(value)?)?;
    let mut parts = pem.split("-----BEGIN CERTIFICATE-----").skip(1);
    let first = parts
        .next()
        .and_then(|value| value.split_once("-----END CERTIFICATE-----"))
        .map(|(value, _)| value)
        .ok_or_else(|| io::Error::other("invalid TLS identity certificate PEM"))?;
    if parts.next().is_some() {
        return Err(
            io::Error::other("native SMTP provider required for TLS certificate chains").into(),
        );
    }
    let encoded = first
        .chars()
        .filter(|value| !value.is_whitespace())
        .collect::<String>();
    Ok(base64::engine::general_purpose::STANDARD.decode(encoded)?)
}

/// Native PEM strings, source Buffer JSON, or certificate-chain string arrays.
fn pem_bytes(value: &Value) -> EmailResult<Vec<u8>> {
    match value {
        Value::String(value) => Ok(value.as_bytes().to_vec()),
        Value::Array(values) => {
            let mut bytes = Vec::new();
            for value in values {
                bytes.extend(pem_bytes(value)?);
                bytes.push(b'\n');
            }
            Ok(bytes)
        }
        Value::Object(value) if value.get("type").and_then(Value::as_str) == Some("Buffer") => {
            Ok(value
                .get("data")
                .and_then(Value::as_array)
                .ok_or_else(|| io::Error::other("invalid PEM Buffer"))?
                .iter()
                .map(|v| v.as_u64().unwrap_or(0) as u8)
                .collect())
        }
        _ => Err(io::Error::other("native SMTP provider required for PEM representation").into()),
    }
}

/// Exact source process command; no shell interpolation or process-policy restriction.
#[derive(Clone, Debug)]
pub struct SendmailRequest {
    /// Selected executable, default `sendmail`.
    pub program: String,
    /// Source `-i`, explicit/default arguments and recipient sequence.
    pub args: Vec<String>,
    /// Compiled bytes written to stdin; source keeps Bcc in these bytes.
    pub message: Vec<u8>,
}

/// Process provider is replaceable so tests never invoke a real sendmail executable.
pub trait ProcessBackend: Send + Sync {
    /// Return the process exit code; None represents termination by a signal.
    fn run(&self, request: SendmailRequest) -> ProviderFuture<'_, Option<i32>>;
}

/// Tokio process IO with kill-on-drop cancellation and normal-path child reaping.
///
/// Dropping a pending call requests termination; only an awaited completion proves reaping.
#[derive(Default)]
pub struct NativeProcessBackend;

impl ProcessBackend for NativeProcessBackend {
    /// Start the requested executable, write stdin, close it and reap the child.
    fn run(&self, request: SendmailRequest) -> ProviderFuture<'_, Option<i32>> {
        Box::pin(async move {
            let mut child = tokio::process::Command::new(request.program)
                .args(request.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .kill_on_drop(true)
                .spawn()?;
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("sendmail stdin unavailable"))?;
            let status = {
                let write = async {
                    stdin.write_all(&request.message).await?;
                    stdin.shutdown().await
                };
                tokio::pin!(write);
                // Source exit and stdin errors compete; the first observed completion wins.
                tokio::select! {
                    status=child.wait()=>Some(status?),
                    result=&mut write=>{
                        if let Err(error)=result {
                            let _=child.kill().await;
                            let _=child.wait().await;
                            return Err(error.into());
                        }
                        None
                    }
                }
            };
            drop(stdin);
            match status {
                Some(status) => Ok(status.code()),
                None => Ok(child.wait().await?.code()),
            }
        })
    }
}

/// SESv2 command input and native raw data; arbitrary message.ses overrides are retained.
pub struct SesRequest {
    /// Complete source command object, with Buffer represented by the source Buffer JSON shape.
    pub input: Value,
    /// Original MIME bytes; the binding can place these directly into its SDK command.
    pub raw: Vec<u8>,
}

/// Native equivalent of source SES.sesClient and SES.SendEmailCommand callbacks.
pub trait SesClient: Send + Sync {
    /// Resolve configured region. Source treats region errors/missing as us-east-1.
    fn region(&self) -> ProviderFuture<'_, Option<String>> {
        Box::pin(async { Ok(None) })
    }
    /// Construct/send the bound SESv2 command and preserve its provider response.
    fn send(&self, request: SesRequest) -> ProviderFuture<'_, Value>;
}

/// Source sendmail/SMTP recipient expansion from an explicit envelope.
pub(crate) fn envelope_recipients(envelope: &Value) -> Vec<String> {
    ["to"]
        .iter()
        .flat_map(|key| {
            envelope
                .get(*key)
                .map(|value| match value {
                    Value::Array(values) => values.iter().map(string).collect(),
                    Value::Null => vec![],
                    value => vec![string(value)],
                })
                .unwrap_or_default()
        })
        .collect()
}

#[cfg(test)]
#[path = "smtp_tests.rs"]
mod tests;
