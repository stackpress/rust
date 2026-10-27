//! Deployment proofs over supplied duplex IO only; no test calls bind or opens a socket.
use sha2::{Digest, Sha256};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_server_http::{
    AcceptedIo, Acceptor, AssetBundle, AssetLimits, ConnectionReport, ExchangeObservation,
    HttpDelivery, HttpRequestContext, HttpServer, IpCidr, NetworkOptions, ProxyHeaders,
    ProxyPolicy, Shutdown,
};
use std::{
    future::pending,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, DuplexStream},
    sync::{Notify, mpsc},
};

/// Independent observations and lifetime counters; no mutable global test state.
#[derive(Default)]
struct Probe {
    calls: AtomicUsize,
    hooks: AtomicUsize,
    drops: AtomicUsize,
    entered: Notify,
    events: Mutex<Vec<ExchangeObservation>>,
}
/// Resource Drop proves the handler is gone before its final observer callback.
struct Dropped(Arc<Probe>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}
/// Metadata is deliberately echoed only by this test operation, never the library observer.
async fn info(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    probe.calls.fetch_add(1, Ordering::SeqCst);
    probe.entered.notify_one();
    match req.url.path() {
        "/pending" => {
            let _guard = Dropped(Arc::clone(probe));
            pending::<()>().await;
        }
        "/panic" => {
            let _guard = Dropped(Arc::clone(probe));
            panic!("private handler panic");
        }
        "/error" => {
            return Err(Error::new("private handler detail")
                .with_kind(stackpress_lib::ErrorKind::External)
                .with_cause(std::io::Error::other("secret-native-cause")));
        }
        _ => {
            let context = inv.adapter_context::<HttpRequestContext>().unwrap();
            res.html(format!(
                "{}|{}|{:?}|{}",
                req.url.as_str(),
                context.scheme,
                context.forwarded_client,
                context.public_origin
            ))?;
        }
    }
    Ok(Flow::Continue)
}
/// A request guard proves static operations use the same authorization ordering as other routes.
async fn guard(req: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if req.headers.get("x-deny") == Some("yes") {
        return Err(Error::new("access denied")
            .with_kind(stackpress_lib::ErrorKind::Domain)
            .with_code(403)
            .with_public_message("Forbidden"));
    }
    Ok(Flow::Continue)
}
/// Ordinary response hooks still run for static routes and preserve independent Set-Cookie fields.
async fn response(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Arc<Probe>>()?
        .hooks
        .fetch_add(1, Ordering::SeqCst);
    res.session_mut()?
        .set("one", "first", None)?
        .set("two", "second", None)?;
    Ok(Flow::Continue)
}
/// Startup is real; from_options is the same pre-bind preparation used by production bind.
async fn build_host(
    probe: Arc<Probe>,
    mut options: NetworkOptions,
    bundle: Option<&AssetBundle>,
) -> Result<Arc<HttpServer>> {
    options.limits.head_timeout = Duration::from_secs(2);
    options.limits.lifecycle_timeout = Duration::from_secs(2);
    options.limits.shutdown_grace = Duration::from_millis(50);
    let mut builder = ServerBuilder::new();
    builder.on_config(task(move |ctx: &mut StartupContext| {
        let probe = Arc::clone(&probe);
        Box::pin(async move {
            ctx.register(Arc::new(probe))?;
            Ok(Flow::Continue)
        })
    }))?;
    for path in ["/info", "/error", "/pending", "/panic"] {
        builder.get(path, info)?;
    }
    builder.on("request", guard)?;
    builder.on("response", response)?;
    if let Some(bundle) = bundle {
        bundle.register(&mut builder)?;
    }
    let app = Arc::new(builder.build().await.map_err(|error| error.error)?);
    Ok(Arc::new(HttpServer::from_options(app, options)?))
}
/// A deliberate public-origin policy; the supplied peer is independent from untrusted headers.
fn options() -> NetworkOptions {
    NetworkOptions::http("127.0.0.1:1234".parse().unwrap(), "https://example.test/")
}
/// Raw bytes pass through real Hyper and normal Stackpress normalization/lifecycle.
async fn raw(host: Arc<HttpServer>, input: &[u8], peer: &str) -> (Vec<u8>, ConnectionReport) {
    let (io, mut client) = tokio::io::duplex(1024);
    let peer = peer.parse().unwrap();
    let task = tokio::spawn(async move { host.serve_io(io, peer, pending()).await });
    let _ = client.write_all(input).await;
    let mut bytes = Vec::new();
    let _ = client.read_to_end(&mut bytes).await;
    (bytes, task.await.unwrap())
}
/// Exact one-hop policies can be tightened/expanded only at trusted startup.
fn trust(headers: ProxyHeaders, hops: usize) -> ProxyPolicy {
    ProxyPolicy {
        peers: vec![
            "127.0.0.0/8".parse().unwrap(),
            "2001:db8:1::/48".parse().unwrap(),
        ],
        hops,
        headers,
    }
}
/// Inspect actual wire status, without fabricating a native request on the test side.
fn status(wire: &[u8], expected: u16) {
    assert!(
        String::from_utf8_lossy(wire).starts_with(&format!("HTTP/1.1 {expected}")),
        "{:?}",
        String::from_utf8_lossy(wire)
    );
}
/// Native authority and proxy policy never dispatch a malformed/untrusted selection.
async fn proxy() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let baseline = build_host(Arc::clone(&probe), options(), None).await?;
    let spoof=b"GET /info HTTP/1.1\r\nHost: example.test\r\nForwarded: nonsense\r\nX-Forwarded-Proto: file\r\nConnection: close\r\n\r\n";
    let (wire, _) = raw(Arc::clone(&baseline), spoof, "127.0.0.1:90").await;
    status(&wire, 200);
    assert!(
        String::from_utf8_lossy(&wire)
            .contains("https://example.test/info|http|None|https://example.test/")
    );
    for target in [
        "GET /info HTTP/1.1\r\nHost: evil.test",
        "GET https://evil.test/info HTTP/1.1\r\nHost: example.test",
        "GET /info HTTP/1.1\r\nHost: example.test:99999",
        "GET /info HTTP/1.1\r\nHost: user@example.test",
        "GET /info HTTP/1.1\r\nHost: example.test:",
        "GET /info HTTP/1.1",
    ] {
        let before = probe.calls.load(Ordering::SeqCst);
        let input = format!("{target}\r\nConnection: close\r\n\r\n");
        let (wire, _) = raw(Arc::clone(&baseline), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, 400);
        assert_eq!(probe.calls.load(Ordering::SeqCst), before);
    }
    for family in [ProxyHeaders::Forwarded, ProxyHeaders::XForwarded] {
        let mut policy = options().with_proxy(trust(family, 1));
        policy.allowed_authorities = vec!["alt.test:443".into(), "example.test:80".into()];
        let configured = build_host(Arc::clone(&probe), policy, None).await?;
        let fields = match family {
            ProxyHeaders::Forwarded => "Forwarded: for=203.0.113.9;proto=https;host=alt.test",
            ProxyHeaders::XForwarded => {
                "X-Forwarded-For: 203.0.113.9\r\nX-Forwarded-Proto: https\r\nX-Forwarded-Host: alt.test"
            }
        };
        let input = format!(
            "GET /info?q=kept HTTP/1.1\r\nHost: example.test\r\n{fields}\r\nConnection: close\r\n\r\n"
        );
        let (wire, _) = raw(Arc::clone(&configured), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, 200);
        assert!(
            String::from_utf8_lossy(&wire)
                .contains("https://alt.test/info?q=kept|http|Some(203.0.113.9)|https://alt.test/")
        );
        let canonical = match family {
            ProxyHeaders::Forwarded => "Forwarded: for=203.0.113.9;proto=HTTPS;host=ALT.TEST:443",
            ProxyHeaders::XForwarded => {
                "X-Forwarded-For: 203.0.113.9\r\nX-Forwarded-Proto: HTTPS\r\nX-Forwarded-Host: ALT.TEST:443"
            }
        };
        let canonical = format!(
            "GET /info HTTP/1.1\r\nHost: example.test\r\n{canonical}\r\nConnection: close\r\n\r\n"
        );
        let (wire, _) = raw(
            Arc::clone(&configured),
            canonical.as_bytes(),
            "[2001:db8:1::1]:90",
        )
        .await;
        status(&wire, 200);
        assert!(
            String::from_utf8_lossy(&wire)
                .contains("https://alt.test/info|http|Some(203.0.113.9)|https://alt.test/")
        );
        let (wire, _) = raw(Arc::clone(&configured), spoof, "128.0.0.1:90").await;
        status(&wire, 200);
        assert!(String::from_utf8_lossy(&wire).contains("|None|https://example.test/"));
        let (wire, _) = raw(
            Arc::clone(&configured),
            input.as_bytes(),
            "[::ffff:127.0.0.1]:90",
        )
        .await;
        status(&wire, 200);
        assert!(String::from_utf8_lossy(&wire).contains("|None|https://example.test/"));
    }
    let trusted = build_host(
        Arc::clone(&probe),
        options().with_proxy(trust(ProxyHeaders::Forwarded, 1)),
        None,
    )
    .await?;
    let bad = [
        "Forwarded: for=203.0.113.9;proto=https;host=evil.test",
        "Forwarded: for=203.0.113.9;proto=file;host=example.test",
        "Forwarded: for=203.0.113.9;proto=https;host=example.test;host=example.test",
        "Forwarded: for=203.0.113.9;proto=https;host=example.test\r\nForwarded: for=127.0.0.1;proto=https;host=example.test",
        "Forwarded: for=203.0.113.9;proto=https;host=example.test,for=127.0.0.1;proto=https;host=example.test",
        "Forwarded: for=unknown;proto=https;host=example.test",
        "Forwarded: for=_hidden;proto=https;host=example.test",
        "Forwarded: for=203.0.113.9;proto=https;host=example.test\r\nX-Forwarded-For: 127.0.0.1",
        "Forwarded: for=203.0.113.9;host=example.test",
        "Forwarded: for=\"203.0.113.9;proto=https;host=example.test",
        "Forwarded: for=2001:db8::1;proto=https;host=example.test",
    ];
    for fields in bad {
        let before = probe.calls.load(Ordering::SeqCst);
        let input = format!(
            "GET /info HTTP/1.1\r\nHost: example.test\r\n{fields}\r\nConnection: close\r\n\r\n"
        );
        let (wire, _) = raw(Arc::clone(&trusted), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, 400);
        assert_eq!(probe.calls.load(Ordering::SeqCst), before);
    }
    let trusted = build_host(
        Arc::clone(&probe),
        options().with_proxy(trust(ProxyHeaders::XForwarded, 1)),
        None,
    )
    .await?;
    for fields in [
        "X-Forwarded-For: 203.0.113.9\r\nX-Forwarded-Proto: https\r\nX-Forwarded-Host: example.test\r\nX-Forwarded-Host: example.test",
        "X-Forwarded-For: 203.0.113.9,127.0.0.1\r\nX-Forwarded-Proto: https\r\nX-Forwarded-Host: example.test",
        "X-Forwarded-For: 203.0.113.9\r\nX-Forwarded-Proto: https\r\nX-Forwarded-Host: example.test\r\nForwarded: ignored",
    ] {
        let input = format!(
            "GET /info HTTP/1.1\r\nHost: example.test\r\n{fields}\r\nConnection: close\r\n\r\n"
        );
        let before = probe.calls.load(Ordering::SeqCst);
        let (wire, _) = raw(Arc::clone(&trusted), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, 400);
        assert_eq!(probe.calls.load(Ordering::SeqCst), before);
    }
    let trusted = build_host(
        Arc::clone(&probe),
        options().with_proxy(trust(ProxyHeaders::Forwarded, 2)),
        None,
    )
    .await?;
    for (middle, expected) in [("127.255.255.255", 200), ("128.0.0.0", 400)] {
        let input = format!(
            "GET /info HTTP/1.1\r\nHost: example.test\r\nForwarded: for=\"[2001:db8::7]:40\";proto=https;host=example.test,for={middle};proto=https;host=example.test\r\nConnection: close\r\n\r\n"
        );
        let (wire, _) = raw(Arc::clone(&trusted), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, expected);
        if expected == 200 {
            assert!(String::from_utf8_lossy(&wire).contains("Some(2001:db8::7)"));
        }
    }
    // Equivalent IPv6 spellings are one authority; family membership still remains exact.
    let mut policy = options();
    policy.http.base_url = "https://[2001:db8::1]/".into();
    let ipv6 = build_host(Arc::clone(&probe), policy, None).await?;
    let (wire, _) = raw(
        ipv6,
        b"GET /info HTTP/1.1\r\nHost: [2001:0db8:0:0:0:0:0:1]\r\nConnection: close\r\n\r\n",
        "[2001:db8:1::1]:90",
    )
    .await;
    status(&wire, 200);
    for (cidr, inside, outside) in [
        ("10.0.0.0/8", "10.255.255.255", "11.0.0.0"),
        ("2001:db8::/32", "2001:db8:ffff::", "2001:db9::"),
        ("127.0.0.1/32", "127.0.0.1", "127.0.0.2"),
        ("::1/128", "::1", "::2"),
    ] {
        let network: IpCidr = cidr.parse()?;
        assert!(network.contains(inside.parse().unwrap()));
        assert!(!network.contains(outside.parse().unwrap()));
    }
    assert!(
        "0.0.0.0/0"
            .parse::<IpCidr>()?
            .contains("255.255.255.255".parse().unwrap())
    );
    assert!(
        "::/0"
            .parse::<IpCidr>()?
            .contains("ffff::".parse().unwrap())
    );
    for invalid in [
        "127.0.0.1",
        "127.0.0.1/33",
        "::/129",
        "bad/8",
        "127.0.0.1/bad",
    ] {
        assert!(invalid.parse::<IpCidr>().is_err());
    }
    Ok(())
}
/// Owned private startup tree; paths are canonical so platform /var symlinks are not introduced.
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    manifest: PathBuf,
    entry: serde_json::Value,
}
impl Fixture {
    /// Create one known representation and strict JSON manifest.
    fn new(bytes: &[u8]) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::write(root.join("hello.txt"), bytes).unwrap();
        let hash = format!("{:x}", Sha256::digest(bytes));
        let entry = serde_json::json!({"route":"/assets/hello.txt","file":"hello.txt","size":bytes.len(),"sha256":hash,"mime":"text/plain; charset=utf-8","cache":"revalidate"});
        let manifest = root.join("manifest.json");
        let fixture = Self {
            _temp: temp,
            root,
            manifest,
            entry,
        };
        fixture.write(vec![fixture.entry.clone()]);
        fixture
    }
    /// Rewrite only this fixture's declared manifest, never any shared repository file.
    fn write(&self, entries: Vec<serde_json::Value>) {
        std::fs::write(
            &self.manifest,
            serde_json::to_vec(&serde_json::json!({"entries":entries})).unwrap(),
        )
        .unwrap();
    }
    /// The production loader is exercised directly, without a listener.
    fn load(&self, limits: AssetLimits) -> Result<AssetBundle> {
        AssetBundle::load(&self.root, &self.manifest, limits)
    }
}
/// Preloaded bytes keep exact GET/HEAD/cache behavior through ordinary lifecycle hooks.
async fn assets() -> Result<()> {
    let bytes = vec![b'x'; 70 * 1024];
    let fixture = Fixture::new(&bytes);
    let bundle = fixture.load(Default::default())?;
    assert_eq!(bundle.len(), 1);
    let probe = Arc::new(Probe::default());
    let host = build_host(Arc::clone(&probe), options(), Some(&bundle)).await?;
    std::fs::remove_file(fixture.root.join("hello.txt")).unwrap();
    let (wire, report) = raw(
        Arc::clone(&host),
        b"GET /assets/hello.txt HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n",
        "127.0.0.1:90",
    )
    .await;
    status(&wire, 200);
    assert!(wire.ends_with(&bytes));
    assert!(String::from_utf8_lossy(&wire).contains("content-length: 71680"));
    assert_eq!(report.exchanges[0].delivery, HttpDelivery::Completed);
    let text = String::from_utf8_lossy(&wire);
    assert_eq!(text.matches("set-cookie:").count(), 2);
    assert!(text.contains("x-content-type-options: nosniff"));
    assert!(text.contains("max-age=0"));
    let (wire, _) = raw(
        Arc::clone(&host),
        b"HEAD /assets/hello.txt HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n",
        "127.0.0.1:90",
    )
    .await;
    status(&wire, 200);
    assert!(wire.ends_with(b"\r\n\r\n"));
    assert!(String::from_utf8_lossy(&wire).contains("content-length: 71680"));
    for method in ["GET", "HEAD"] {
        let input = format!(
            "{method} /assets/hello.txt HTTP/1.1\r\nHost: example.test\r\nIf-None-Match: W/\"{}\", \"other\"\r\nConnection: close\r\n\r\n",
            fixture.entry["sha256"].as_str().unwrap()
        );
        let (wire, _) = raw(Arc::clone(&host), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, 304);
        assert!(wire.ends_with(b"\r\n\r\n"));
        assert!(!String::from_utf8_lossy(&wire).contains("content-length:"));
    }
    for path in [
        "/assets",
        "/assets/absent.txt",
        "/assets/%68ello.txt",
        "/assets/hello.txt/extra",
    ] {
        let input =
            format!("GET {path} HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n");
        let (wire, _) = raw(Arc::clone(&host), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, 404);
    }
    assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
    assert_eq!(probe.hooks.load(Ordering::SeqCst), 8);
    let(wire,report)=raw(Arc::clone(&host),b"GET /assets/hello.txt HTTP/1.1\r\nHost: example.test\r\nX-Deny: yes\r\nConnection: close\r\n\r\n","127.0.0.1:90").await;
    status(&wire, 403);
    assert!(!wire.ends_with(&bytes));
    assert!(
        report.exchanges[0]
            .host
            .as_ref()
            .unwrap()
            .operation
            .is_none()
    );
    let fixture = Fixture::new(b"small");
    let mut hashed = fixture.entry.clone();
    hashed["route"] = format!("/assets/{}.txt", hashed["sha256"].as_str().unwrap()).into();
    hashed["cache"] = "immutable".into();
    fixture.write(vec![hashed.clone()]);
    let bundle = fixture.load(Default::default())?;
    let host = build_host(Arc::new(Probe::default()), options(), Some(&bundle)).await?;
    let input = format!(
        "GET {} HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n",
        hashed["route"].as_str().unwrap()
    );
    let (wire, _) = raw(host, input.as_bytes(), "127.0.0.1:90").await;
    status(&wire, 200);
    assert!(String::from_utf8_lossy(&wire).contains("max-age=31536000, immutable"));
    let empty = Fixture::new(b"");
    let bundle = empty.load(Default::default())?;
    let host = build_host(Arc::new(Probe::default()), options(), Some(&bundle)).await?;
    for method in ["GET", "HEAD"] {
        let input = format!(
            "{method} /assets/hello.txt HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n"
        );
        let (wire, report) = raw(Arc::clone(&host), input.as_bytes(), "127.0.0.1:90").await;
        status(&wire, 200);
        assert!(wire.ends_with(b"\r\n\r\n"));
        assert!(String::from_utf8_lossy(&wire).contains("content-length: 0"));
        assert_eq!(report.exchanges[0].delivery, HttpDelivery::Completed);
    }
    let(wire,_)=raw(host,b"GET /assets/hello.txt HTTP/1.1\r\nHost: example.test\r\nIf-None-Match: *\r\nConnection: close\r\n\r\n","127.0.0.1:90").await;
    status(&wire, 304);

    Ok(())
}
/// Unsafe/unbounded descriptors fail before opening assets; files never come from request paths.
async fn asset_rejections() -> Result<()> {
    let fixture = Fixture::new(b"abc");
    for (key, value) in [
        ("route", "/../x"),
        ("route", "/%2e/x"),
        ("route", "/λ"),
        ("route", "/:capture"),
        ("route", "/a//b"),
        ("file", "../hello.txt"),
        ("file", "/hello.txt"),
        ("file", "hello\\txt"),
        ("mime", "application/secret"),
        ("sha256", "wrong"),
        ("cache", "immutable"),
    ] {
        let mut entry = fixture.entry.clone();
        entry[key] = value.into();
        fixture.write(vec![entry]);
        assert!(fixture.load(Default::default()).is_err(), "{key}={value}");
    }
    for (key, value) in [
        ("size", serde_json::json!(4)),
        ("unknown", serde_json::json!(true)),
        ("sha256", serde_json::json!("0".repeat(64))),
    ] {
        let mut entry = fixture.entry.clone();
        entry[key] = value;
        fixture.write(vec![entry]);
        assert!(fixture.load(Default::default()).is_err());
    }
    fixture.write(vec![fixture.entry.clone(), fixture.entry.clone()]);
    assert!(fixture.load(Default::default()).is_err());
    std::fs::write(&fixture.manifest, b"{\"entries\":[],\"typo\":true}").unwrap();
    assert!(fixture.load(Default::default()).is_err());
    std::fs::write(&fixture.manifest, b"invalid-json").unwrap();
    let error = fixture.load(Default::default()).err().unwrap();
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<serde_json::Error>()
            .is_some()
    );
    fixture.write(vec![fixture.entry.clone()]);
    let length = std::fs::metadata(&fixture.manifest).unwrap().len() as usize;
    for (limit, valid) in [(length, true), (length - 1, false)] {
        let limits = AssetLimits {
            manifest_bytes: limit,
            ..Default::default()
        };
        assert_eq!(fixture.load(limits).is_ok(), valid);
    }
    for (cap, valid) in [(3, true), (2, false)] {
        assert_eq!(
            fixture
                .load(AssetLimits {
                    file_bytes: cap,
                    total_bytes: cap,
                    ..Default::default()
                })
                .is_ok(),
            valid
        );
    }
    let mut second = fixture.entry.clone();
    second["route"] = "/second.txt".into();
    fixture.write(vec![fixture.entry.clone(), second]);
    assert!(
        fixture
            .load(AssetLimits {
                entries: 1,
                ..Default::default()
            })
            .is_err()
    );
    assert!(
        fixture
            .load(AssetLimits {
                total_bytes: 5,
                ..Default::default()
            })
            .is_err()
    );
    assert!(
        fixture
            .load(AssetLimits {
                total_bytes: 6,
                ..Default::default()
            })
            .is_ok()
    );
    #[cfg(unix)]
    {
        let mut entry = fixture.entry.clone();
        entry["file"] = "link".into();
        fixture.write(vec![entry.clone()]);
        std::os::unix::fs::symlink(fixture.root.join("hello.txt"), fixture.root.join("link"))
            .unwrap();
        assert!(fixture.load(Default::default()).is_err());
        std::fs::create_dir(fixture.root.join("directory")).unwrap();
        std::fs::write(fixture.root.join("directory/hello.txt"), b"abc").unwrap();
        std::os::unix::fs::symlink(fixture.root.join("directory"), fixture.root.join("alias"))
            .unwrap();
        entry["file"] = "alias/hello.txt".into();
        fixture.write(vec![entry.clone()]);
        assert!(fixture.load(Default::default()).is_err());
        entry["file"] = "directory".into();
        fixture.write(vec![entry.clone()]);
        assert!(fixture.load(Default::default()).is_err());
        entry["file"] = "pipe".into();
        fixture.write(vec![entry]);
        assert!(
            Command::new("mkfifo")
                .arg(fixture.root.join("pipe"))
                .status()
                .unwrap()
                .success()
        );
        let start = Instant::now();
        assert!(fixture.load(Default::default()).is_err());
        assert!(start.elapsed() < Duration::from_millis(200));
    }
    fixture.write(vec![]);
    let empty = fixture.load(Default::default())?;
    assert!(empty.is_empty());
    Ok(())
}
/// Observer callbacks see bounded final metadata only, outside ownership/locks and exactly once.
async fn observer() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let seen = Arc::clone(&probe);
    let weak = Arc::new(Mutex::new(std::sync::Weak::<HttpServer>::new()));
    let reenter = Arc::clone(&weak);
    let options = options().with_observer(Arc::new(move |event: ExchangeObservation| {
        let host = reenter.lock().unwrap().upgrade().unwrap();
        assert_eq!(host.active(), (0, 0));
        seen.events.lock().unwrap().push(event);
    }));
    let host = build_host(Arc::clone(&probe), options, None).await?;
    *weak.lock().unwrap() = Arc::downgrade(&host);
    let input=b"GET /info?secret-query HTTP/1.1\r\nHost: example.test\r\nAuthorization: secret-header\r\nCookie: secret-cookie=value\r\n\r\nGET /error HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n";
    let (_, report) = raw(Arc::clone(&host), input, "127.0.0.1:90").await;
    assert_eq!(report.exchanges.len(), 2);
    let events = probe.events.lock().unwrap().clone();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].connection_id, events[1].connection_id);
    assert_eq!(events[0].exchange_id, 1);
    assert_eq!(events[1].exchange_id, 2);
    assert!(events[1].host_report);
    let debug = format!("{events:?}");
    for private in [
        "secret-query",
        "secret-header",
        "secret-cookie",
        "secret-native-cause",
        "private handler detail",
    ] {
        assert!(!debug.contains(private));
    }
    let (io, mut peer) = tokio::io::duplex(256);
    let owner = Arc::clone(&host);
    let panicking = tokio::spawn(async move {
        owner
            .serve_io(io, "127.0.0.1:90".parse().unwrap(), pending())
            .await
    });
    peer.write_all(b"GET /panic HTTP/1.1\r\nHost: example.test\r\n\r\n")
        .await
        .unwrap();
    let mut bytes = Vec::new();
    let _ = peer.read_to_end(&mut bytes).await;
    let error = panicking.await.unwrap_err();
    assert!(error.is_panic());
    assert_eq!(
        *error.into_panic().downcast::<&'static str>().ok().unwrap(),
        "private handler panic"
    );
    let events = probe.events.lock().unwrap().clone();
    assert_eq!(events.len(), 3);
    assert_eq!(events[2].delivery, HttpDelivery::Failed);
    assert_eq!(
        events[2].error_kind,
        Some(stackpress_lib::ErrorKind::External)
    );
    assert!(!format!("{events:?}").contains("private handler panic"));
    let before = events.len();
    let (io, mut peer) = tokio::io::duplex(128);
    let owner = Arc::clone(&host);
    let task = tokio::spawn(async move {
        owner
            .serve_io(io, "127.0.0.1:90".parse().unwrap(), pending())
            .await
    });
    peer.write_all(b"GET /pending HTTP/1.1\r\nHost: example.test\r\n\r\n")
        .await
        .unwrap();
    // Drain the two earlier notifications before waiting for this operation's entry.
    while probe.calls.load(Ordering::SeqCst) < 4 {
        tokio::task::yield_now().await;
    }
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(probe.drops.load(Ordering::SeqCst), 2);
    let events = probe.events.lock().unwrap();
    assert_eq!(events.len(), before + 1);
    assert_eq!(events.last().unwrap().delivery, HttpDelivery::Cancelled);
    assert!(!events.last().unwrap().host_report);
    Ok(())
}
/// The observer's own panic unwinds once after IO/permits are released; there is no retry callback.
async fn observer_panic() -> Result<()> {
    let count = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&count);
    let host = build_host(
        Arc::new(Probe::default()),
        options().with_observer(Arc::new(move |_: ExchangeObservation| {
            seen.fetch_add(1, Ordering::SeqCst);
            panic!("intentional observer panic");
        })),
        None,
    )
    .await?;
    let (io, mut peer) = tokio::io::duplex(512);
    let owner = Arc::clone(&host);
    let task = tokio::spawn(async move {
        owner
            .serve_io(io, "127.0.0.1:90".parse().unwrap(), pending())
            .await
    });
    peer.write_all(b"GET /info HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut bytes = Vec::new();
    peer.read_to_end(&mut bytes).await.unwrap();
    assert!(task.await.unwrap_err().is_panic());
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(host.active(), (0, 0));
    Ok(())
}
/// No listener is involved when invalid deployment policy fails preparation.
async fn configuration() -> Result<()> {
    for case in 0..5 {
        let mut policy = options();
        match case {
            0 => policy.http.base_url = "file:///tmp/".into(),
            1 => policy.limits.max_connections = 0,
            2 => policy.allowed_authorities = vec!["bad@host".into()],
            3 => policy.proxy = Some(trust(ProxyHeaders::Forwarded, 0)),
            _ => {
                policy.proxy = Some(ProxyPolicy {
                    peers: vec![],
                    hops: 1,
                    headers: ProxyHeaders::Forwarded,
                })
            }
        }
        assert!(
            build_host(Arc::new(Probe::default()), policy, None)
                .await
                .is_err()
        );
    }
    Ok(())
}

/// Only the parent watchdog invokes this entry; direct ignored-child execution is unsupported.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run only by M2 external watchdog"]
async fn scenario_child() -> Result<()> {
    match std::env::var("S7_M2_SCENARIO").unwrap().as_str() {
        "proxy" => proxy().await,
        "assets" => assets().await,
        "asset_rejections" => asset_rejections().await,
        "observer" => observer().await,
        "observer_admission" => observer_admission().await,
        "future_stop" => future_stop().await,
        "observer_panic" => observer_panic().await,
        "configuration" => configuration().await,
        #[cfg(feature = "tls")]
        "tls_files" => tls_files().await,
        _ => panic!("unknown M2 scenario"),
    }
}
/// Every hang-class proof has an independent process kill/reap watchdog, including special files.
#[test]
fn m2_watchdog() {
    for scenario in [
        "proxy",
        "assets",
        "asset_rejections",
        "observer",
        "observer_panic",
        "observer_admission",
        "future_stop",
        "configuration",
        #[cfg(feature = "tls")]
        "tls_files",
    ] {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "scenario_child", "--ignored", "--nocapture"])
            .env("S7_M2_SCENARIO", scenario)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "M2 {scenario} failed: {status}");
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                child.wait().unwrap();
                panic!("M2 {scenario} timed out; killed/reaped");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// File preparation validates before bind, and selected HTTPS works on the same duplex IO seam.
#[cfg(feature = "tls")]
async fn tls_files() -> Result<()> {
    use rustls::{
        ClientConfig, RootCertStore,
        pki_types::{CertificateDer, ServerName},
    };
    use stackpress_server_http::{TlsFileFormat, TlsFiles};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    for (cert, key, format) in [
        (
            include_bytes!("fixtures/server.der").as_slice(),
            include_bytes!("fixtures/server-key.der").as_slice(),
            TlsFileFormat::Der,
        ),
        (
            include_bytes!("fixtures/server.pem").as_slice(),
            include_bytes!("fixtures/server-key.pem").as_slice(),
            TlsFileFormat::Pem,
        ),
    ] {
        let cert_path = root.join("cert");
        let key_path = root.join("key");
        std::fs::write(&cert_path, cert).unwrap();
        std::fs::write(&key_path, key).unwrap();
        let files = TlsFiles {
            certificates: cert_path,
            private_key: key_path,
            format,
        };
        let options = NetworkOptions::https(
            "127.0.0.1:1234".parse().unwrap(),
            "https://localhost/",
            files,
        );
        let probe = Arc::new(Probe::default());
        let server = build_host(Arc::clone(&probe), options, None).await?;
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from(
                include_bytes!("fixtures/ca.der").to_vec(),
            ))
            .unwrap();
        let client =
            ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_root_certificates(roots)
                .with_no_client_auth();
        // Startup has consumed the file contents; requests/handshakes do not reopen them.
        std::fs::remove_file(root.join("cert")).unwrap();
        std::fs::remove_file(root.join("key")).unwrap();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client));
        let (io, peer) = tokio::io::duplex(1024);
        let owner = Arc::clone(&server);
        let task = tokio::spawn(async move {
            owner
                .serve_io(io, "127.0.0.1:90".parse().unwrap(), pending())
                .await
        });
        let mut client = connector
            .connect(ServerName::try_from("localhost").unwrap(), peer)
            .await
            .unwrap();
        client
            .write_all(b"GET /info HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut wire = Vec::new();
        client.read_to_end(&mut wire).await.unwrap();
        status(&wire, 200);
        assert!(String::from_utf8_lossy(&wire).contains("|https|None|https://localhost/"));
        assert!(task.await.unwrap().failure.is_none());
        assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
        // Sending plaintext to a selected HTTPS host never reaches the route or falls back to HTTP.
        let (wire, report) = raw(
            Arc::clone(&server),
            b"GET /info HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            "127.0.0.1:90",
        )
        .await;
        assert!(!wire.starts_with(b"HTTP/"));
        assert!(report.failure.is_some());
        assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    }
    let cert = root.join("cert");
    let key = root.join("key");
    for (cert_bytes, key_bytes) in [
        (
            b"bad-cert".as_slice(),
            include_bytes!("fixtures/server-key.pem").as_slice(),
        ),
        (
            include_bytes!("fixtures/server.pem").as_slice(),
            b"bad-key".as_slice(),
        ),
        (
            include_bytes!("fixtures/server.pem").as_slice(),
            b"".as_slice(),
        ),
    ] {
        std::fs::write(&cert, cert_bytes).unwrap();
        std::fs::write(&key, key_bytes).unwrap();
        assert!(
            build_host(
                Arc::new(Probe::default()),
                NetworkOptions::https(
                    "127.0.0.1:90".parse().unwrap(),
                    "https://localhost/",
                    TlsFiles::pem(&cert, &key)
                ),
                None
            )
            .await
            .is_err()
        );
    }
    std::fs::write(&cert, include_bytes!("fixtures/server.pem")).unwrap();
    std::fs::write(
        &key,
        [
            include_bytes!("fixtures/server-key.pem").as_slice(),
            include_bytes!("fixtures/server-key.pem").as_slice(),
        ]
        .concat(),
    )
    .unwrap();
    assert!(stackpress_server_http::TlsConfig::from_files(&TlsFiles::pem(&cert, &key)).is_err());
    std::fs::write(&key, vec![0; 16 * 1024 + 1]).unwrap();
    assert!(stackpress_server_http::TlsConfig::from_files(&TlsFiles::pem(&cert, &key)).is_err());
    std::fs::write(&cert, include_bytes!("fixtures/server.der")).unwrap();
    std::fs::write(&key, include_bytes!("fixtures/mismatched-key.der")).unwrap();
    let error = stackpress_server_http::TlsConfig::from_files(&TlsFiles {
        certificates: cert.clone(),
        private_key: key.clone(),
        format: TlsFileFormat::Der,
    })
    .err()
    .unwrap();
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<rustls::Error>()
            .is_some()
    );
    std::fs::write(&key, b"invalid DER key").unwrap();
    assert!(
        stackpress_server_http::TlsConfig::from_files(&TlsFiles {
            certificates: cert.clone(),
            private_key: key.clone(),
            format: TlsFileFormat::Der
        })
        .is_err()
    );
    std::fs::remove_file(&key).unwrap();
    let error = stackpress_server_http::TlsConfig::from_files(&TlsFiles::pem(&cert, &key))
        .err()
        .unwrap();
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .is_some()
    );
    Ok(())
}

/// Reentrant stop from an admission observer proves callback invocation occurs outside the gate.
async fn observer_admission() -> Result<()> {
    struct Source(mpsc::Receiver<AcceptedIo<DuplexStream>>);
    impl Acceptor for Source {
        type Io = DuplexStream;
        async fn accept(&mut self) -> Result<Option<AcceptedIo<Self::Io>>> {
            Ok(self.0.recv().await)
        }
    }
    let probe = Arc::new(Probe::default());
    let seen = Arc::clone(&probe);
    let stop = Shutdown::new();
    let reenter = stop.clone();
    let mut policy = options().with_observer(Arc::new(move |event: ExchangeObservation| {
        if event.status == Some(503) {
            reenter.stop();
        }
        seen.events.lock().unwrap().push(event);
    }));
    policy.limits.max_connections = 1;
    policy.limits.max_handshakes = 1;
    let host = build_host(Arc::clone(&probe), policy, None).await?;
    let (tx, rx) = mpsc::channel(2);
    let owner = Arc::clone(&host);
    let stopping = stop.clone();
    let task = tokio::spawn(async move { owner.serve_acceptor(Source(rx), stopping).await });
    let (io, mut peer) = tokio::io::duplex(256);
    tx.send(AcceptedIo::plain(io, "127.0.0.1:90".parse().unwrap()))
        .await
        .unwrap();
    peer.write_all(b"GET /pending HTTP/1.1\r\nHost: example.test\r\n\r\n")
        .await
        .unwrap();
    probe.entered.notified().await;
    let (io, mut excess) = tokio::io::duplex(32);
    tx.send(AcceptedIo::plain(io, "127.0.0.1:90".parse().unwrap()))
        .await
        .unwrap();
    let mut byte = [0; 1];
    assert_eq!(excess.read(&mut byte).await.unwrap(), 0);
    let report = task.await.unwrap();
    assert_eq!(report.admitted, 1);
    assert_eq!(report.rejected, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(host.active(), (0, 0));
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    let events = probe.events.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert!(events.iter().any(|event| event.status == Some(503)));
    assert!(
        events
            .iter()
            .any(|event| event.delivery == HttpDelivery::Cancelled)
    );
    Ok(())
}

/// The production stop-future path drops its source before drain and returns after owned work drops.
async fn future_stop() -> Result<()> {
    struct Source {
        rx: mpsc::Receiver<AcceptedIo<DuplexStream>>,
        order: Arc<Mutex<Vec<&'static str>>>,
    }
    impl Acceptor for Source {
        type Io = DuplexStream;
        async fn accept(&mut self) -> Result<Option<AcceptedIo<Self::Io>>> {
            Ok(self.rx.recv().await)
        }
    }
    impl Drop for Source {
        fn drop(&mut self) {
            self.order.lock().unwrap().push("source closed");
        }
    }
    let order = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&order);
    let probe = Arc::new(Probe::default());
    let counters = Arc::clone(&probe);
    let host = build_host(
        Arc::clone(&probe),
        options().with_observer(Arc::new(move |_: ExchangeObservation| {
            assert_eq!(counters.drops.load(Ordering::SeqCst), 1);
            observed.lock().unwrap().push("observed after drop");
        })),
        None,
    )
    .await?;
    let (tx, rx) = mpsc::channel(1);
    let owner = Arc::clone(&host);
    let source_order = Arc::clone(&order);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        owner
            .serve_acceptor_until(
                Source {
                    rx,
                    order: source_order,
                },
                async {
                    let _ = stopped.await;
                },
            )
            .await
    });
    let (io, mut peer) = tokio::io::duplex(256);
    tx.send(AcceptedIo::plain(io, "127.0.0.1:90".parse().unwrap()))
        .await
        .unwrap();
    peer.write_all(b"GET /pending HTTP/1.1\r\nHost: example.test\r\n\r\n")
        .await
        .unwrap();
    probe.entered.notified().await;
    stop.send(()).unwrap();
    let report = task.await.unwrap();
    assert_eq!(report.admitted, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(host.active(), (0, 0));
    assert_eq!(
        *order.lock().unwrap(),
        vec!["source closed", "observed after drop"]
    );
    // Process services may be closed now: the accepted handler and its context have been dropped.
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    Ok(())
}
