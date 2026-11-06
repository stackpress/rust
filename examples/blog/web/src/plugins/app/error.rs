//! Source terminal/non-GET/body/json/script/style error branches and native snippets.
use serde_json::json;
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result};

/// JavaScript truthiness of response body values, including empty object/array/buffer values.
fn body_present(res: &Response) -> bool {
    match res.body() {
        Body::Empty => false,
        Body::Text(value) => !value.is_empty(),
        Body::Bytes(_) => true,
        Body::Value(value) => match value {
            stackpress_lib::Value::Null => false,
            stackpress_lib::Value::Bool(value) => *value,
            stackpress_lib::Value::Number(value) => value
                .as_f64()
                .is_some_and(|value| value != 0.0 && !value.is_nan()),
            stackpress_lib::Value::String(value) => !value.is_empty(),
            _ => true,
        },
    }
}

/// Retain source guard order and error status while choosing its actual output representation.
pub async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Source terminal errors abort the event; the following ordinary guards only Continue.
    if req.mimetype() == "terminal/arguments" {
        return Ok(Flow::Abort);
    }
    if req.method.as_str() != "GET" || body_present(res) {
        return Ok(Flow::Continue);
    }
    let mut traces = serde_json::to_value(res.stack().unwrap_or_default())?;
    // Preserve trace order and skip the first location before considering owned file IO.
    for (index, trace) in traces.as_array_mut().into_iter().flatten().enumerate() {
        if index == 0 {
            continue;
        }
        let path = std::path::Path::new(trace["file"].as_str().unwrap_or_default());
        if !path.is_absolute() || !path.exists() {
            continue;
        }
        let text = std::fs::read_to_string(path).map_err(stackpress_lib::Error::from_source)?;
        // Native source locations use one-based line/column and retain UTF-16 caret counting.
        let lines = text.split('\n').collect::<Vec<_>>();
        let line = trace["line"].as_u64().unwrap_or_default() as usize;
        let column = trace["column"].as_u64().unwrap_or_default() as usize;
        let main = line
            .checked_sub(1)
            .and_then(|line| lines.get(line))
            .copied()
            .unwrap_or_default();
        trace["snippet"] = json!({"before":line.checked_sub(2).and_then(|line|lines.get(line)),"main":main,
            "after":lines.get(line),"location":(!main.is_empty() && main.encode_utf16().count()>=column).then(||format!("{}^"," ".repeat(column.saturating_sub(1))))});
    }
    res.data_mut()?
        .set("stack", stackpress_lib::Value::try_from(traces)?)?;
    if req.data.has("json") {
        // Snippets have been staged even when the original JSON flag suppresses the view.
        return Ok(Flow::Continue);
    }
    let code = res.status().map_or(500, |status| status.code);
    if req.url.path().ends_with(".js") || req.url.path().ends_with(".css") {
        // Script/style errors retain status while omitting the stack from their wire body.
        let mut value = res.try_to_json()?;
        if let Some(value) = value.as_object_mut() {
            value.remove("stack");
        }
        let js = req.url.path().ends_with(".js");
        let text = if js {
            format!("console.log({value});")
        } else {
            format!("/* {value} */")
        };
        res.set(
            if js {
                "application/javascript"
            } else {
                "text/css"
            },
            Body::Text(text),
        )?;
        res.status_code(code, None)?;
        return Ok(Flow::Continue);
    }
    crate::render::page(blog_shared::Kind::Error, req, res, inv).await
}
