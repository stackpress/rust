//! One finite proof source compiled for native SSR, real hydrate and real CSR.
//! Optional feature variants also measure exactly the same exported app shape.

#[cfg(feature = "format")]
pub mod editing;

use serde::{Deserialize, Serialize};
use stackpress_leptos::leptos::prelude::*;

/// Reviewable text or checked-rich output, with a deterministic stable identity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofRow {
    /// Corpus identity or typed formatting case.
    pub id: String,
    /// Rich rows use the private checked type again at the insertion boundary.
    pub rich: bool,
    /// Exact native/WASM string; never normalized for parity comparison.
    pub output: String,
}

/// Build the selected proof outputs without native I/O or ambient browser state.
///
/// # Errors
/// Retains library/fixture failures; invalid fixtures fail the proof.
pub fn proof_rows() -> Result<Vec<ProofRow>, Box<dyn std::error::Error>> {
    // The baseline row keeps disabled-feature builds visible and measurable.
    let mut rows = vec![ProofRow {
        id: "baseline".into(),
        rich: false,
        output: "Optional rich content and formatting".into(),
    }];
    #[cfg(feature = "rich")]
    append_rich(&mut rows)?;
    #[cfg(feature = "format")]
    append_formats(&mut rows)?;
    // The mutable binding remains intentional in the no-feature measurement build.
    rows.shrink_to_fit();
    Ok(rows)
}

/// Run the same hostile corpus through both closed-policy entry points.
#[cfg(feature = "rich")]
fn append_rich(rows: &mut Vec<ProofRow>) -> Result<(), Box<dyn std::error::Error>> {
    use stackpress_leptos::rich::SafeHtml;
    let corpus: Vec<serde_json::Value> = serde_json::from_str(include_str!("../corpus.json"))?;
    for entry in corpus {
        // Checked fixture lookup failures retain a real error, never invent empty input.
        let id = entry["id"].as_str().ok_or("missing corpus id")?;
        let input = entry["input"].as_str().ok_or("missing corpus input")?;
        for (mode, safe) in [
            ("html", SafeHtml::html(input)?),
            ("markdown", SafeHtml::markdown(input)?),
        ] {
            if SafeHtml::html(safe.as_str())? != safe {
                return Err("non-idempotent sanitizer output".into());
            }
            rows.push(ProofRow {
                id: format!("{mode}-{id}"),
                rich: true,
                output: safe.as_str().to_owned(),
            });
        }
    }
    Ok(())
}

/// Compare exact Unicode strings and safe invalid outcomes on both execution targets.
#[cfg(feature = "format")]
fn append_formats(rows: &mut Vec<ProofRow>) -> Result<(), Box<dyn std::error::Error>> {
    use stackpress_leptos::format::{
        CalendarDate, DecimalValue, DisplayLocale, E164Phone, NumberFormat, OffsetDateTime,
        TimeOfDay, TimePrecision,
    };
    for (name, locale) in [
        ("en", DisplayLocale::English),
        ("fr", DisplayLocale::French),
    ] {
        let mut add = |id: String, output: String| {
            rows.push(ProofRow {
                id: format!("{name}-{id}"),
                rich: false,
                output,
            })
        };
        for input in [
            "0",
            "-0",
            "-0.004",
            "1234567.125",
            "1234567.135",
            "999.999",
            "9007199254740993",
            "0.000000000000000001",
        ] {
            for precision in [0, 2, 18] {
                add(
                    format!("decimal-{input}-{precision}"),
                    DecimalValue::parse(input)?
                        .display(locale, NumberFormat::new(precision, true)?)?,
                );
            }
        }
        for input in ["0001-01-01", "2000-02-29", "2024-02-29", "9999-12-31"] {
            add(
                format!("date-{input}"),
                CalendarDate::parse(input)?.display(locale)?,
            );
        }
        for (id, precision) in [
            ("minute", TimePrecision::Minutes),
            ("second", TimePrecision::Seconds),
            ("millis", TimePrecision::Milliseconds),
            ("micros", TimePrecision::Microseconds),
            ("nanos", TimePrecision::Nanoseconds),
        ] {
            add(
                format!("time-{id}"),
                TimeOfDay::parse("23:59:59.987654321")?.display(locale, precision)?,
            );
        }
        for input in [
            "2026-03-08T01:59:59-05:00",
            "2026-03-08T03:00:00-04:00",
            "2026-11-01T01:30:00-04:00",
            "2026-11-01T01:30:00-05:00",
            "2026-03-29T01:59:59+01:00",
            "2026-03-29T03:00:00+02:00",
            "2026-10-25T02:30:00+02:00",
            "2026-10-25T02:30:00+01:00",
            "2024-02-29T00:00:00Z",
            "2024-02-29T00:00:00.1+05:45",
            "2024-02-29T00:00:00+14:00",
        ] {
            add(
                format!("offset-{input}"),
                OffsetDateTime::parse(input)?.display(locale, TimePrecision::Seconds)?,
            );
        }
        for input in ["+14155552671", "+33123456789", "+442079460018"] {
            add(format!("phone-{input}"), E164Phone::parse(input)?.display());
        }
        for input in ["", "-", "1.", "NaN", "1e3", "1,2", "١"] {
            add(
                format!("invalid-decimal-{input}"),
                DecimalValue::parse(input)
                    .err()
                    .ok_or("invalid decimal admitted")?
                    .to_string(),
            );
        }
        for input in ["2023-02-29", "2024-13-01"] {
            add(
                format!("invalid-date-{input}"),
                CalendarDate::parse(input)
                    .err()
                    .ok_or("invalid date admitted")?
                    .to_string(),
            );
        }
        add(
            "invalid-time".into(),
            TimeOfDay::parse("24:00")
                .err()
                .ok_or("invalid time admitted")?
                .to_string(),
        );
        add(
            "invalid-offset".into(),
            OffsetDateTime::parse("2024-02-29T12:00-00:00")
                .err()
                .ok_or("unknown offset admitted")?
                .to_string(),
        );
        add(
            "invalid-phone".into(),
            E164Phone::parse("+10000000000")
                .err()
                .ok_or("invalid phone admitted")?
                .to_string(),
        );
    }
    Ok(())
}

/// Shared visual content: sanitizer output is inserted only through `SafeHtml`.
#[component]
pub fn Proof(
    /// Already-computed deterministic proof outputs.
    rows: Vec<ProofRow>,
) -> impl IntoView {
    view! {
        <main id="format-proof">
            <h1>"Rich content and deterministic formatting"</h1>
            <p>"Closed sanitizer policy · explicit precision · explicit UTC offsets"</p>
            {rows.into_iter().map(|row| {
                let id = row.id;
                #[cfg(feature = "rich")]
                if row.rich {
                    // The UI boundary never accepts a raw deserialized HTML string.
                    use stackpress_leptos::rich::{RichContent, SafeHtml};
                    return match SafeHtml::html(&row.output) {
                        Ok(content) => view! { <section data-proof=id><h2>"Inert rich content"</h2><RichContent content/></section> }.into_any(),
                        Err(error) => view! { <section data-proof=id><p>{error.to_string()}</p></section> }.into_any(),
                    };
                }
                view! { <section data-proof=id><p class="formatted">{row.output}</p></section> }.into_any()
            }).collect_view()}
        </main>
    }
}

/// Return exact JSON as a separate payload, never inline executable script content.
///
/// # Errors
/// Retains fixture, formatting and serialization errors.
pub fn snapshot_json() -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string(&proof_rows()?)?)
}

/// Finite native HTML rendering with an isolated, cleaned reactive owner.
///
/// # Errors
/// Retains fixture and formatting failures before rendering.
#[cfg(feature = "ssr")]
pub fn render_html() -> Result<String, Box<dyn std::error::Error>> {
    let rows = proof_rows()?;
    let owner = Owner::new();
    let html = owner.with(|| view! { <Proof rows/> }.to_html());
    owner.cleanup();
    Ok(html)
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::*;
    use wasm_bindgen::{JsCast, prelude::*};

    /// Execute actual WASM policies and return their exact strings to the proof driver.
    #[wasm_bindgen]
    pub fn snapshot() -> Result<String, JsValue> {
        snapshot_json().map_err(|error| JsValue::from_str(&error.to_string()))
    }

    /// Mount/hydrate explicit-locale specialized editors with pre-bootstrap draft adoption.
    #[cfg(feature = "format")]
    #[wasm_bindgen]
    pub fn start_editor(locale: &str) -> Result<js_sys::Function, JsValue> {
        let locale = match locale {
            "en" => stackpress_leptos::format::DisplayLocale::English,
            "fr" => stackpress_leptos::format::DisplayLocale::French,
            _ => return Err(JsValue::from_str("Unsupported editing locale")),
        };
        let app: web_sys::HtmlElement = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("app"))
            .ok_or_else(|| JsValue::from_str("Missing editor root"))?
            .dyn_into()?;
        #[cfg(feature = "hydrate")]
        let handle = stackpress_leptos::leptos::mount::hydrate_from(
            app,
            move || view! {<crate::editing::EditingProof locale/>},
        );
        #[cfg(feature = "csr")]
        let handle = stackpress_leptos::leptos::mount::mount_to(
            app,
            move || view! {<crate::editing::EditingProof locale/>},
        );
        let mut handle = Some(handle);
        let dispose = Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>);
        Ok(dispose.into_js_value().unchecked_into())
    }

    /// Mount/hydrate the same shared component and return a root disposal callback.
    #[wasm_bindgen]
    pub fn start() -> Result<js_sys::Function, JsValue> {
        let rows = proof_rows().map_err(|error| JsValue::from_str(&error.to_string()))?;
        let app: web_sys::HtmlElement = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| JsValue::from_str("Missing proof root"))?
            .dyn_into()?;
        #[cfg(feature = "hydrate")]
        let handle =
            stackpress_leptos::leptos::mount::hydrate_from(app, move || view! { <Proof rows/> });
        #[cfg(feature = "csr")]
        let handle =
            stackpress_leptos::leptos::mount::mount_to(app, move || view! { <Proof rows/> });
        let mut handle = Some(handle);
        let dispose = Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>);
        Ok(dispose.into_js_value().unchecked_into())
    }
}

#[cfg(all(test, feature = "ssr", feature = "rich", feature = "format"))]
mod tests {
    use super::*;

    /// Shared fixture really renders each rich/text case and preserves its identity.
    #[test]
    fn shared_render_and_payload() {
        let rows = proof_rows().unwrap();
        assert!(rows.len() > 150);
        let payload: Vec<ProofRow> = serde_json::from_str(&snapshot_json().unwrap()).unwrap();
        assert_eq!(payload, rows);
        let html = render_html().unwrap();
        for row in rows {
            assert!(html.contains("data-proof=\""));
            if row.rich {
                assert!(html.contains(&row.output));
            }
        }
        assert!(html.contains("UTC-04:00"));
        assert!(html.contains("1\u{202f}234\u{202f}567,12"));
        assert!(!html.contains("<script"));
    }
}
