//! Compact labelled caller/fault controls. All fault state is memory-only and dies with the host.
use crate::options::Options;
mod hooks;
use stackpress_example_ui_stream_web::Probe;
use stackpress_framework_preferences::{CookieScope, FlashCookie, FlashNotice};
use stackpress_framework_view::BasePath;
use stackpress_lib::{Body, Error, Flow, Request, Response, Result, handler};
use stackpress_server::ServerBuilder;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// One host's bounded review state; no operation DTO can arm these fixtures.
pub struct Panel {
    /// The existing SQLite wrapper consumes this only after a committed write.
    pub acknowledgement: Arc<AtomicBool>,
    /// One exact pending read scenario, consumed by the next admin data read.
    read: Mutex<Option<&'static str>>,
    /// One rejected preference save, never an acknowledgement of success.
    preferences: AtomicBool,
    /// Explicit hydration pause is presentation state, never authority.
    pub paused: AtomicBool,
    /// A fixed public status line; never stores native diagnostics or caller input.
    status: Mutex<&'static str>,
    /// Existing stream ownership counters exposed only as review observations.
    pub probe: Arc<Probe>,
}
impl Panel {
    /// Start with no pending faults; caller selection is independently cookie-scoped.
    pub fn new(acknowledgement: Arc<AtomicBool>) -> Self {
        Self {
            acknowledgement,
            read: Mutex::new(None),
            preferences: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            status: Mutex::new("Ready; no fault armed"),
            probe: Arc::new(Probe::default()),
        }
    }
    /// Read bounded public state without keeping a mutex guard across callbacks/await.
    pub fn status(&self) -> &'static str {
        *self.status.lock().expect("panel status lock")
    }
    /// Replace the public status with a compiled message only.
    fn set_status(&self, value: &'static str) {
        *self.status.lock().expect("panel status lock") = value;
    }
    /// Apply a named scenario; invalid text cannot become executable configuration.
    fn arm(&self, action: &str) -> Result<()> {
        match action {
            "slow" => {
                *self.read.lock().expect("read lock") = Some("slow");
                self.set_status("003: next admin read delayed 800 ms");
            }
            "failed" => {
                *self.read.lock().expect("read lock") = Some("failed");
                self.set_status("003: next admin read will fail safely");
            }
            "ack" => {
                self.acknowledgement.store(true, Ordering::SeqCst);
                self.set_status("004/006: next committed write acknowledgement will be uncertain");
            }
            "preferences" => {
                self.preferences.store(true, Ordering::SeqCst);
                self.set_status("007: next preference save will fail");
            }
            "pause" => {
                self.paused.store(true, Ordering::SeqCst);
                self.set_status("001: browser controls paused until Start browser controls");
            }
            "automatic" => {
                self.paused.store(false, Ordering::SeqCst);
                self.set_status("001: automatic hydration restored");
            }
            _ => return Err(Error::invalid("unknown review scenario")),
        }
        Ok(())
    }
}
/// Read exactly one fixture cookie; duplicates/invalid tokens deny rather than guess.
pub fn actor(cookie: &str) -> &'static str {
    let mut values = cookie.split(';').filter_map(|part| {
        let part = part.trim();
        let (name, value) = part.split_once('=').unwrap_or((part, ""));
        (name.trim() == "slice8-review-role").then_some(value)
    });
    let first = values.next();
    if values.next().is_some() {
        return "denied";
    }
    match first {
        Some("editor") => "editor",
        Some("reader") => "reader",
        _ => "denied",
    }
}
/// Register review pages and the request/response seams without adding production policy.
/// # Errors
/// Returns route/listener registration failures before the caller publishes the host.
pub fn configure(builder: &mut ServerBuilder, options: &Options, panel: Arc<Panel>) -> Result<()> {
    let base = options.base.as_str().to_owned();
    let origin = options.origin();
    hooks::configure(builder, options, panel.clone())?;
    let view = panel.clone();
    let config = options.clone();
    builder.get(
        &options.path("/review"),
        handler(move |req, res, _| {
            let panel = view.clone();
            let options = config.clone();
            Box::pin(async move {
                res.html(document(
                    &options,
                    &panel,
                    actor(req.headers.get("cookie").unwrap_or_default()),
                ))?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    let state = panel.clone();
    builder.get(&options.path("/review/state"), handler(move |_, res, _| {
        let state = state.clone(); Box::pin(async move {
            res.json(&serde_json::json!({"status":state.status(),"cleaned_stream_roots":state.probe.cleanups.load(Ordering::SeqCst),"captures_released":state.probe.released()}))?; Ok(Flow::Continue)
        })
    }))?;
    for action in [
        "editor",
        "reader",
        "denied",
        "slow",
        "failed",
        "ack",
        "preferences",
        "pause",
        "automatic",
        "flash",
    ] {
        let panel = panel.clone();
        let origin = origin.clone();
        let base = base.clone();
        builder.post(
            &options.path(&format!("/review/action/{action}")),
            handler(move |req, res, _| {
                let panel = panel.clone();
                let origin = origin.clone();
                let base = base.clone();
                Box::pin(async move { apply(req, res, &panel, action, &origin, &base) })
            }),
        )?;
    }
    for (name, csv) in [
        ("valid", "id,name\nimport-review,Imported review\n"),
        ("invalid", "id,name\n,\n"),
    ] {
        builder.get(
            &options.path(&format!("/review/{name}.csv")),
            handler(move |_, res, _| {
                Box::pin(async move {
                    res.set("text/csv; charset=utf-8", Body::Text(csv.into()))?;
                    res.headers_mut()?.set(
                        "content-disposition",
                        &format!("attachment; filename=\"{name}.csv\""),
                    )?;
                    Ok(Flow::Continue)
                })
            }),
        )?;
    }
    Ok(())
}
/// Exact same-origin POST action; selection persists, but all fault state remains process-local.
fn apply(
    req: &Request,
    res: &mut Response,
    panel: &Panel,
    action: &str,
    origin: &str,
    base: &str,
) -> Result<Flow> {
    if req.headers.get("origin") != Some(origin) {
        return Err(Error::new("review origin refused")
            .with_code(403)
            .with_public_message("Review action forbidden"));
    }
    if ["editor", "reader", "denied"].contains(&action) {
        let path = if base.is_empty() { "/" } else { base };
        res.headers_mut()?.append(
            "set-cookie",
            &format!(
                "slice8-review-role={action}; Path={path}; Max-Age=7200; HttpOnly; SameSite=Lax"
            ),
        )?;
        panel.set_status("004/005: caller selection updated for following requests");
    } else if action == "flash" {
        let scope = CookieScope::new(
            origin.into(),
            BasePath::new(format!("{base}/ui-preferences")).map_err(Error::from_source)?,
        )
        .map_err(Error::from_source)?;
        let notice = FlashNotice::new("owner-review".into(), "Public review notice".into())
            .map_err(Error::from_source)?;
        res.headers_mut()?.append(
            "set-cookie",
            &FlashCookie::new(scope)
                .issue(&notice)
                .map_err(Error::from_source)?,
        )?;
        panel.set_status("008: public notice ready on next preferences page");
    } else {
        panel.arm(action)?;
    }
    res.redirect(&format!("{base}/review"))?;
    Ok(Flow::Continue)
}
/// Compact single-page walkthrough launcher; every fixture names its owner control.
fn document(options: &Options, panel: &Panel, actor: &str) -> String {
    let base = options.base.as_str();
    let actions = [
        ("editor", "004/005/006: Editor"),
        ("reader", "004/005/006: Reader"),
        ("denied", "005: Denied"),
        ("slow", "003/006: Slow next read"),
        ("failed", "003: Fail next read"),
        ("ack", "004/006: Lose next committed acknowledgement"),
        ("preferences", "007: Fail next preference save"),
        ("flash", "008: Issue public notice"),
        ("pause", "001: Pause hydration"),
        ("automatic", "001: Automatic hydration"),
    ];
    let buttons = actions.iter().map(|(action,label)|format!("<form method=\"post\" action=\"{base}/review/action/{action}\"><button>{label}</button></form>")).collect::<String>();
    let links = [
        ("/ui-gallery/gallery", "Gallery"),
        ("/generated-admin/profile/search", "Profiles"),
        ("/generated-admin/project/search", "Projects"),
        ("/generated-admin/task/search", "Tasks"),
        ("/generated-admin/roles", "Generated roles / plugin label"),
        ("/ui-preferences/preferences", "Preferences"),
        ("/ui-format/", "Rich content and formatting"),
        ("/ui-format/edit/en", "English editor"),
        ("/ui-format/edit/fr", "French editor"),
        ("/ui-stream/stream", "012: Delayed stream"),
        ("/ui-stream/late", "012: Late failure"),
        ("/ui-stream/pending", "012: Pending stream"),
    ]
    .iter()
    .map(|(path, label)| format!("<li><a href=\"{base}{path}\">{label}</a></li>"))
    .collect::<String>();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Slice 8 review</title><style>body{{font:16px system-ui;max-width:60rem;margin:auto;padding:1rem}}button{{padding:.5rem;margin:.25rem}}form{{display:inline}}a{{overflow-wrap:anywhere}}</style></head><body><h1>Review fixtures — not authentication</h1><!--STACKPRESS_REVIEW_PREVIEW--><p>Compiled variant {} · {} · caller {actor}</p><p role=\"status\">{}; cleaned stream roots: {}</p><p>Faults are one-shot and reset on restart. Lost acknowledgements are uncertain, never success. Keep this panel in a second tab and refresh it for status.</p><ul>{links}</ul><h2>Review scenarios</h2>{buttons}<p>006: <a href=\"{base}/review/valid.csv\">Valid sample CSV</a> · <a href=\"{base}/review/invalid.csv\">Invalid sample CSV</a></p></body></html>",
        options.variant,
        if options.csr { "CSR" } else { "SSR + hydrate" },
        panel.status(),
        panel.probe.cleanups.load(Ordering::SeqCst)
    )
}
