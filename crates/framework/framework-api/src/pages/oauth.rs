//! Source consent process: browser JWT lookup, shared app result, persisted session-as-code.
use super::{completed, encode, environment, property, text, truthy};
use crate::{ApiConfig, helpers};
use serde_json::json;
use stackpress_lib::{Error, Flow, Invocation, Nest, Request, Response, Result, Timestamp};
use stackpress_server::ServerInvocationExt;

/// Run the original OAuth process; successful GET leaves application data for a later view.
///
/// Only POST creates a session. Request fields are spread first, then applicationId/profileId/
/// expires override them. No PKCE, redirect registry, scope intersection or new guard is added.
/// # Errors
/// Propagates nested event/input/response errors. Typed Date/URL mechanics fail explicitly;
/// cancellation drops the awaited child, without undoing an already completed SQL insert.
pub async fn oauth(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if completed(res) {
        return Ok(Flow::Continue);
    }
    let env = environment(&inv);
    let config = inv
        .config::<Option<ApiConfig>>()?
        .as_ref()
        .ok_or_else(|| Error::new("api config is missing"))?;
    // Publish exactly the source-selected config fields, before checking caller parameters.
    res.data_mut()?.set(
        "api",
        Nest::from_object(json!({"scopes":config.scopes,"endpoints":config.endpoints}))?
            .into_value(),
    )?;
    for (key, source, defaults) in [
        (
            "view",
            &env.view,
            json!({"base":"/","props":{},"notify":{}}),
        ),
        (
            "brand",
            &env.brand,
            json!({"name":"Stackpress","logo":"/logo.png","icon":"/icon.png","favicon":"/favicon.ico"}),
        ),
    ] {
        let mut selected = defaults.as_object().expect("object defaults").clone();
        for (field, fallback) in &mut selected {
            if let Some(value) = source.get(field) {
                let native = stackpress_lib::Value::try_from(value.clone())?;
                if truthy(&native) {
                    *fallback = value.clone();
                }
            }
        }
        res.data_mut()?
            .set(key, stackpress_lib::Value::try_from(json!(selected))?)?;
    }
    let id = req.data.get("client_id").cloned();
    let redirect = req.data.get("redirect_uri").cloned();
    let state = req.data.get("state").cloned();
    let (Some(id), Some(redirect)) = (id.filter(truthy), redirect.filter(truthy)) else {
        helpers::unauthorized(res)?;
        return Ok(Flow::Continue);
    };
    // Same Request preserves the configured JWT cookie, headers and data for the real me event.
    let mut session = Response::new();
    inv.emit("me", req, &mut session).await?;
    let profile = session
        .result()
        .and_then(|value| property(value, "id"))
        .filter(|value| truthy(value))
        .cloned();
    let Some(profile) = profile else {
        let search = req
            .url
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default();
        res.redirect(&format!(
            "/auth/signin?redirect_uri={}",
            encode(&format!("{}{search}", req.url.path()))
        ))?;
        return Ok(Flow::Continue);
    };
    let mut data = Nest::new();
    data.set("id", id.clone())?;
    helpers::resolve_into(&inv, "application-detail", data, res).await?;
    if res.status().map(|status| status.code) != Some(200) {
        helpers::unauthorized(res)?;
        return Ok(Flow::Continue);
    }
    if req.method.as_str() == "POST" {
        // Clone aggregate input verbatim; model-owned preparation filters protocol fields later.
        let mut data = req.data.clone();
        data.set("applicationId", id)?;
        data.set("profileId", profile)?;
        let millis = config.expires_at((env.now)()?).ok_or_else(|| {
            Error::invalid("OAuth expiry is Invalid Date (native Timestamp cannot represent it)")
        })?;
        data.set(
            "expires",
            Timestamp::new(
                millis.div_euclid(1000),
                (millis.rem_euclid(1000) as u32) * 1_000_000,
            )?,
        )?;
        let created = helpers::resolve(&inv, "session-create", data).await?;
        // Deliberately test the shared application status, NOT the creation response status.
        if res.status().map(|status| status.code) != Some(200)
            || !created.result().is_some_and(truthy)
        {
            return Ok(Flow::Continue);
        }
        // Source split() happens only after persistence; a malformed truthy redirect fails here.
        let redirect = redirect
            .as_str()
            .ok_or_else(|| Error::invalid("redirect.split is not a function"))?;
        let code = created
            .result()
            .and_then(|value| property(value, "id"))
            .map(text)
            .transpose()?
            .unwrap_or_else(|| "undefined".into());
        let state = state
            .as_ref()
            .filter(|value| truthy(value))
            .map(text)
            .transpose()?;
        res.redirect(&destination(redirect, &code, state.as_deref())?)?;
    }
    Ok(Flow::Continue)
}

/// Source split('?') keeps only the first URI/query, then URLSearchParams.set preserves order.
fn destination(redirect: &str, code: &str, state: Option<&str>) -> Result<String> {
    let mut split = redirect.split('?');
    let uri = split.next().unwrap_or_default();
    let query = split.next().unwrap_or_default();
    // Reuse the existing URL library through reqwest's public Url. Escape only controls the URL
    // setter would discard; decoded pairs still follow URLSearchParams replacement/UTF8 rules.
    let mut carrier = reqwest::Url::parse("http://query.invalid/").map_err(Error::from_source)?;
    carrier.set_query(Some(
        &query
            .replace('\t', "%09")
            .replace('\r', "%0D")
            .replace('\n', "%0A"),
    ));
    let mut pairs: Vec<(String, String)> = carrier.query_pairs().into_owned().collect();
    for (key, value) in [("code", Some(code)), ("state", state)] {
        let Some(value) = value else {
            continue;
        };
        if let Some(index) = pairs.iter().position(|(name, _)| name == key) {
            pairs[index].1 = value.into();
            let mut seen = false;
            pairs.retain(|(name, _)| {
                if name != key {
                    return true;
                }
                let keep = !seen;
                seen = true;
                keep
            });
        } else {
            pairs.push((key.into(), value.into()));
        }
    }
    carrier.set_query(None);
    carrier.query_pairs_mut().extend_pairs(pairs);
    Ok(format!("{uri}?{}", carrier.query().unwrap_or_default()))
}
