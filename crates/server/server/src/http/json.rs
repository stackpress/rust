//! HTTP-only Date projection; foundation and in-process values remain strictly typed.

use stackpress_lib::{Error, Response, Result, Timestamp, Value};

/// Select the G57 Date wire form without changing results retained by the lifecycle.
pub(super) fn envelope(response: &Response) -> Result<serde_json::Value> {
    let limits = response.limits();
    let mut envelope = response.to_envelope();
    for value in [&mut envelope.results, &mut envelope.errors]
        .into_iter()
        .flatten()
    {
        // Bound traversal before recursion, including publicly replaced response stores.
        value.validate(&limits)?;
        project(value)?;
    }
    // Recheck the projected string budgets and retain the foundation's qualified errors
    // for every other non-JSON variant. Redirect status does not suppress this body.
    envelope.try_to_json_with(&limits)
}

/// Visit only JSON containers and Dates; other native values keep their explicit refusal.
fn project(value: &mut Value) -> Result<()> {
    match value {
        Value::DateTime(time) => *value = Value::String(iso(*time)?),
        Value::Array(values) => {
            for value in values {
                project(value)?;
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                project(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Format the ECMAScript Date range as UTC ISO text, including signed extended years.
fn iso(time: Timestamp) -> Result<String> {
    // JS Date keeps integral milliseconds, truncating fractional epoch milliseconds
    // toward zero. Use i128 so even the foundation's full seconds range cannot overflow.
    let nanos = i128::from(time.seconds) * 1_000_000_000 + i128::from(time.nanoseconds());
    let millis = nanos / 1_000_000;
    if millis.abs() > 8_640_000_000_000_000 {
        return Err(Error::invalid(
            "HTTP Date exceeds the JavaScript Date range",
        ));
    }
    let millis = millis as i64;
    // Euclidean division keeps the date/time components positive before the epoch.
    let days = millis.div_euclid(86_400_000);
    let within = millis.rem_euclid(86_400_000);
    let (year, month, day) = civil_date(days);
    let year = if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else {
        format!("{year:+07}")
    };
    Ok(format!(
        "{year}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        within / 3_600_000,
        within / 60_000 % 60,
        within / 1000 % 60,
        within % 1000,
    ))
}

/// Convert Unix days using Gregorian 400-year eras and a March-based leap-day cycle.
fn civil_date(days: i64) -> (i64, i64, i64) {
    // Shift the epoch to the March-based calendar, whose leap day is the last day.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = month_from_march + if month_from_march < 10 { 3 } else { -9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}
