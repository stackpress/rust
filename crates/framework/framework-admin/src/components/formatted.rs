//! Opt-in bridges to UI editing policies. Only acknowledged canonical commits reach Draft.
use super::{presence, sync};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::{
    Input, RoleBinding, SchemaError, SchemaErrorKind, SchemaResult, UiField, ValueType,
};
use stackpress_framework_ui::format::{
    CalendarDate, DecimalValue, DisplayLocale, E164Phone, NumberFormat, OffsetDateTime, TimeOfDay,
    TimePrecision,
    edit::{
        DecimalPolicy, EditError, EditPolicy, EditState, FormattedInput, IsoValue, PhonePolicy,
        PhoneRegion, TemporalKind, TemporalPolicy,
    },
};
use std::ops::Bound::Unbounded;

/// Retain typed policy/parser failures privately, without reflecting input values.
fn invalid(cause: impl std::error::Error + Send + Sync + 'static) -> SchemaError {
    SchemaError::new(
        SchemaErrorKind::Invalid,
        "invalid formatted field configuration or value",
    )
    .with_cause(cause)
}
fn refuse() -> SchemaError {
    SchemaError::new(
        SchemaErrorKind::Invalid,
        "invalid formatted field configuration or value",
    )
}
fn locale(binding: &RoleBinding) -> SchemaResult<DisplayLocale> {
    // Locale is explicit catalog data; host/browser environment never changes presentation.
    match binding
        .props
        .get("locale")
        .and_then(Value::as_str)
        .unwrap_or("en")
    {
        "en" => Ok(DisplayLocale::English),
        "fr" => Ok(DisplayLocale::French),
        _ => Err(refuse()),
    }
}
fn integer(binding: &RoleBinding, name: &str, default: u8, max: u8) -> SchemaResult<u8> {
    // Schema Number props still require a bounded integral policy parameter.
    match binding.props.get(name) {
        None => Ok(default),
        Some(value) => value
            .as_f64()
            .filter(|n| n.fract() == 0.0 && *n >= 0.0 && *n <= f64::from(max))
            .map(|n| n as u8)
            .ok_or_else(refuse),
    }
}
fn decimal(field: &UiField, binding: &RoleBinding) -> SchemaResult<DecimalPolicy> {
    // Decimal presentation may bridge only these scalar public wire families.
    if field.value_type.multiple
        || !matches!(
            field.value_type.value,
            ValueType::Integer | ValueType::Number | ValueType::String
        )
    {
        return Err(refuse());
    }
    let scale = integer(
        binding,
        "scale",
        if field.value_type.value == ValueType::Integer {
            0
        } else {
            2
        },
        18,
    )?;
    if field.value_type.value == ValueType::Integer && scale != 0 {
        return Err(refuse());
    }
    DecimalPolicy::new(
        scale,
        integer(binding, "digits", 24, 96)?,
        Unbounded,
        Unbounded,
    )
    .map_err(invalid)
}
fn temporal(
    field: &UiField,
    binding: &RoleBinding,
) -> SchemaResult<(TemporalKind, TemporalPresentation)> {
    // The schema type determines the parser; metadata cannot reinterpret strings as instants.
    if field.value_type.multiple {
        return Err(refuse());
    }
    let kind = match field.value_type.value {
        ValueType::Date => TemporalKind::Date,
        ValueType::Time => TemporalKind::Time,
        ValueType::Datetime => TemporalKind::Instant,
        _ => return Err(refuse()),
    };
    let precision = integer(
        binding,
        "precision",
        if kind == TemporalKind::Date { 0 } else { 9 },
        9,
    )?;
    Ok((
        kind,
        TemporalPresentation {
            inner: TemporalPolicy::new(kind, precision, Unbounded, Unbounded).map_err(invalid)?,
            precision: binding.props.contains_key("precision").then_some(precision),
        },
    ))
}

/// Change presentation only; all admission, values and canonical storage remain UI-owned.
#[derive(Clone)]
struct TemporalPresentation {
    inner: TemporalPolicy,
    precision: Option<u8>,
}
impl EditPolicy for TemporalPresentation {
    type Value = IsoValue;
    fn max_bytes(&self) -> usize {
        self.inner.max_bytes()
    }
    fn parse(&self, draft: &str, locale: DisplayLocale) -> Result<IsoValue, EditError> {
        self.inner.parse(draft, locale)
    }
    fn admit(&self, draft: &str, locale: DisplayLocale) -> Result<(), EditError> {
        self.inner.admit(draft, locale)
    }
    fn check(&self, value: &IsoValue) -> Result<(), EditError> {
        self.inner.check(value)
    }
    fn display(&self, value: &IsoValue, locale: DisplayLocale) -> String {
        temporal_text(&self.inner.display(value, locale), self.precision)
    }
    fn canonical(&self, value: &IsoValue) -> String {
        self.inner.canonical(value)
    }
    fn semantic(&self, ch: char, locale: DisplayLocale) -> bool {
        self.inner.semantic(ch, locale)
    }
}

/// ISO/localized fractional text is ASCII after the decimal mark; keep the suffix intact.
fn temporal_text(text: &str, precision: Option<u8>) -> String {
    // A localized calendar date can contain punctuation before the clock's decimal mark.
    let mark = text
        .find(':')
        .and_then(|clock| text[clock..].find(['.', ',']).map(|index| clock + index));
    let Some(mark) = mark else {
        // Explicit fractional precision pads a clock, never a calendar date.
        if let Some(precision) = precision.filter(|p| *p > 0 && text.contains(':')) {
            let clock_start = text.find('T').map_or(0, |index| index + 1);
            let end = text[clock_start..]
                .find(['+', '-', 'Z'])
                .map_or(text.len(), |index| clock_start + index);
            let (clock, suffix) = text.split_at(end);
            return format!("{clock}.{}{suffix}", "0".repeat(usize::from(precision)));
        }
        return text.into();
    };
    let end = mark
        + 1
        + text[mark + 1..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
    if end == mark + 1 {
        return text.into();
    }
    let digits = &text[mark + 1..end];
    let fraction = if let Some(precision) = precision {
        let width = usize::from(precision);
        format!("{:0<width$}", &digits[..digits.len().min(width)])
    } else {
        digits.trim_end_matches('0').into()
    };
    format!(
        "{}{}{}{}",
        &text[..mark],
        if fraction.is_empty() {
            ""
        } else {
            &text[mark..mark + 1]
        },
        fraction,
        &text[end..]
    )
}
fn phone(field: &UiField, binding: &RoleBinding) -> SchemaResult<PhonePolicy> {
    // National drafts require an explicit region; accepted values remain canonical E.164.
    if field.value_type.multiple || field.value_type.value != ValueType::String {
        return Err(refuse());
    }
    let region = binding
        .props
        .get("region")
        .and_then(Value::as_str)
        .ok_or_else(refuse)?;
    Ok(PhonePolicy::new(
        PhoneRegion::parse(region).map_err(invalid)?,
    ))
}
/// The same pure policy checks run during codegen and direct component construction.
pub(crate) fn configuration(field: &UiField, binding: &RoleBinding) -> SchemaResult<()> {
    let input = matches!(
        binding.role,
        stackpress_framework_schema::Role::Field
            | stackpress_framework_schema::Role::Filter
            | stackpress_framework_schema::Role::Span
    );
    let suffix = binding.component.rsplit('.').next().unwrap_or_default();
    if input && matches!(suffix, "date" | "time" | "datetime" | "integer") {
        let expected = match suffix {
            "date" => ValueType::Date,
            "time" => ValueType::Time,
            "datetime" => ValueType::Datetime,
            _ => ValueType::Integer,
        };
        if field.value_type.value != expected {
            return Err(refuse());
        }
    }
    locale(binding)?;
    match binding.component.rsplit('.').next().unwrap_or_default() {
        "number" | "price" | "integer" => {
            decimal(field, binding)?;
        }
        "date" | "time" | "datetime" => {
            temporal(field, binding)?;
        }
        "phone" => {
            if input {
                phone(field, binding)?;
            } else if field.value_type.multiple || field.value_type.value != ValueType::String {
                return Err(refuse());
            }
        }
        _ => return Err(refuse()),
    }
    Ok(())
}
fn binding(context: &RoleContext) -> SchemaResult<RoleBinding> {
    context
        .field
        .roles
        .iter()
        .find(|b| b.role == context.role)
        .cloned()
        .ok_or_else(refuse)
}
fn text(value: &Value) -> SchemaResult<&str> {
    value.as_str().ok_or_else(refuse)
}
/// A checked canonical decimal becomes the schema's exact declared wire family.
fn decimal_wire(kind: &ValueType, value: DecimalValue) -> SchemaResult<Value> {
    let wire = value.canonical();
    let value = match kind {
        ValueType::String => Value::String(wire),
        ValueType::Integer => Value::from(wire.parse::<i64>().map_err(invalid)?),
        ValueType::Number => {
            let value: Value = serde_json::from_str(&wire).map_err(invalid)?;
            stackpress_framework_schema::FiniteNumber::from_json(&value)?;
            value
        }
        _ => return Err(refuse()),
    };
    stackpress_framework_schema::check_browser_json(&value)?;
    Ok(value)
}
fn temporal_wire(kind: TemporalKind, value: IsoValue) -> SchemaResult<Value> {
    let wire = value.as_str();
    let value = match kind {
        TemporalKind::Date => stackpress_framework_schema::DateValue::parse(wire)?
            .as_str()
            .to_owned(),
        TemporalKind::Time => stackpress_framework_schema::TimeValue::parse(wire)?
            .as_str()
            .to_owned(),
        TemporalKind::Instant => stackpress_framework_schema::DatetimeValue::parse(wire)?
            .as_str()
            .to_owned(),
    };
    Ok(Value::String(value))
}
/// A dirty presentation is pending only until its canonical value has been acknowledged.
fn pending_edit<P: EditPolicy>(state: &EditState<P>, nullable: bool) -> bool {
    if state.error().is_some() || state.composing() {
        return true;
    }
    if !state.dirty() {
        return false;
    }
    // Blur may acknowledge without rewriting presentation. Compare canonical values, not text.
    let mut checked = state.clone();
    checked.commit(nullable).is_err() || checked.canonical() != state.canonical()
}

/// One checked controller per root; programmatic presence resets cannot echo as user commits.
fn render<P: EditPolicy>(
    context: RoleContext,
    policy: P,
    locale: DisplayLocale,
    decode: impl Fn(&Value) -> SchemaResult<P::Value> + Clone + Send + Sync + 'static,
    encode: impl Fn(P::Value) -> SchemaResult<Value> + Send + Sync + 'static,
) -> SchemaResult<AnyView> {
    let draft = context.value;
    // Missing and Null have an empty presentation without becoming a supplied domain value.
    let seed = match draft.get_untracked() {
        Input::Value(value) => Some(decode(&value)?),
        _ => None,
    };
    let state = RwSignal::new(EditState::new(policy.clone(), seed, locale).map_err(invalid)?);
    let error = RwSignal::new(String::new());
    let published = StoredValue::new(draft.get_untracked());
    // Recover pre-hydration edits through the UI controller's checked native adoption contract.
    #[cfg(target_arch = "wasm32")]
    stackpress_framework_ui::format::edit::adopt_native(&context.id, state).map_err(invalid)?;
    let decode_reset = decode.clone();
    Effect::watch(
        move || draft.get(),
        move |next, _, _| {
            // A callback's own acknowledged publication must not reset its editing controller.
            if published.get_value() == *next {
                return;
            }
            let value = match next {
                Input::Value(value) => decode_reset(value).map(Some),
                _ => Ok(None),
            };
            match value
                .and_then(|value| EditState::new(policy.clone(), value, locale).map_err(invalid))
            {
                Ok(next_state) => {
                    published.set_value(next.clone());
                    error.set(String::new());
                    state.set(next_state);
                }
                Err(_) => error.set("Enter a value supported by this field".into()),
            }
        },
        false,
    );
    let callback = Callback::new(move |value: Option<P::Value>| {
        // UI commits are typed; the final bridge must also fit the frozen schema wire family.
        let next = value
            .map(&encode)
            .transpose()
            .map(|value| value.map(Input::Value).unwrap_or(Input::Null));
        match next {
            Ok(next) => {
                error.set(String::new());
                published.set_value(next.clone());
                draft.set(next);
            }
            Err(_) => error.set("Value cannot be represented by this field".into()),
        }
    });
    let nullable =
        context.field.nullable || context.role == stackpress_framework_schema::Role::Filter;
    if let Some(pending) = use_context::<crate::pages::PendingEdits>() {
        let name = context.field.wire_name.clone();
        Effect::new(move |_| {
            // The root prevents writes while a visible intermediate value is still uncommitted.
            let blocked = state.with(|s| pending_edit(s, nullable)) || !error.get().is_empty();
            pending.0.update(|fields| {
                fields.insert(name.clone(), blocked);
            });
        });
    }
    let state_selector = presence(&context);
    let public_error = super::field_error(&context);
    Ok(view!{<div class="sp-role sp-formatted" data-role=format!("{:?}",context.role).to_lowercase()>
        <FormattedInput id=context.id.clone() name=context.field.wire_name.clone() label=context.text_prop("label",&context.field.label) state nullable disabled=Signal::stored(context.bool_prop("disabled",false)) on_commit=callback help="Press Enter to apply the value, or Escape to cancel."/>
        <p role="alert">{move ||error.get()}</p><p role="alert">{move ||public_error.get()}</p>{state_selector}</div>}.into_any())
}

/// Exact decimals use String fields; numeric schema fields retain their JSON numeric family.
#[component]
pub fn DecimalField(
    /// Approved field context.
    context: RoleContext,
) -> impl IntoView {
    let result = (|| {
        let binding = binding(&context)?;
        configuration(&context.field, &binding)?;
        let policy = decimal(&context.field, &binding)?;
        let locale = locale(&binding)?;
        let kind = context.field.value_type.value.clone();
        render(
            context,
            policy,
            locale,
            |value| {
                DecimalValue::parse(&match value {
                    Value::String(v) => v.clone(),
                    Value::Number(v) => v.to_string(),
                    _ => return Err(refuse()),
                })
                .map_err(invalid)
            },
            move |value| decimal_wire(&kind, value),
        )
    })();
    result.unwrap_or_else(|_| sync::rejected())
}
/// ISO date/local time or explicit-offset instant, committed to the actual schema wire grammar.
#[component]
pub fn TemporalField(
    /// Approved field context.
    context: RoleContext,
) -> impl IntoView {
    let result = (|| {
        let binding = binding(&context)?;
        configuration(&context.field, &binding)?;
        let (kind, policy) = temporal(&context.field, &binding)?;
        let locale = locale(&binding)?;
        render(
            context,
            policy,
            locale,
            move |value| IsoValue::parse(kind, text(value)?).map_err(invalid),
            move |value| temporal_wire(kind, value),
        )
    })();
    result.unwrap_or_else(|_| sync::rejected())
}
/// Explicit metadata region admits national drafts; only valid canonical E.164 reaches Draft.
#[component]
pub fn PhoneField(
    /// Approved field context.
    context: RoleContext,
) -> impl IntoView {
    let result = (|| {
        let binding = binding(&context)?;
        configuration(&context.field, &binding)?;
        let policy = phone(&context.field, &binding)?;
        let locale = locale(&binding)?;
        render(
            context,
            policy,
            locale,
            |value| E164Phone::parse(text(value)?).map_err(invalid),
            |value| Ok(Value::String(value.as_str().into())),
        )
    })();
    result.unwrap_or_else(|_| sync::rejected())
}
/// Matching escaped decimal, temporal or phone display; null never becomes zero or empty data.
#[component]
pub fn FormattedView(
    /// Approved public display context.
    context: RoleContext,
) -> impl IntoView {
    let output = (|| {
        let binding = binding(&context)?;
        configuration(&context.field, &binding)?;
        let locale = locale(&binding)?;
        let Input::Value(value) = context.value.get_untracked() else {
            return Ok(context.text_prop("null_label", "—"));
        };
        match binding.component.rsplit('.').next().unwrap_or_default() {
            "number" | "price" => DecimalValue::parse(&match value {
                Value::String(v) => v,
                Value::Number(v) => v.to_string(),
                _ => return Err(refuse()),
            })
            .map_err(invalid)?
            .display(
                locale,
                NumberFormat::new(
                    integer(
                        &binding,
                        "scale",
                        if context.field.value_type.value == ValueType::Integer {
                            0
                        } else {
                            2
                        },
                        18,
                    )?,
                    true,
                )
                .map_err(invalid)?,
            )
            .map_err(invalid),
            "phone" => Ok(E164Phone::parse(text(&value)?).map_err(invalid)?.display()),
            "date" => match context.field.value_type.value {
                ValueType::Date => CalendarDate::parse(text(&value)?)
                    .map_err(invalid)?
                    .display(locale)
                    .map_err(invalid),
                ValueType::Time => TimeOfDay::parse(text(&value)?)
                    .map_err(invalid)?
                    .display(locale, TimePrecision::Nanoseconds)
                    .map(|value| temporal_text(&value, None))
                    .map_err(invalid),
                ValueType::Datetime => OffsetDateTime::parse(text(&value)?)
                    .map_err(invalid)?
                    .display(locale, TimePrecision::Nanoseconds)
                    .map(|value| temporal_text(&value, None))
                    .map_err(invalid),
                _ => Err(refuse()),
            },
            _ => Err(refuse()),
        }
    })();
    match output {
        Ok(text) => view! {<span class="sp-formatted-value">{text}</span>}.into_any(),
        Err(_) => sync::rejected(),
    }
}

#[cfg(all(test, feature = "codegen"))]
#[path = "formatted_tests.rs"]
mod tests;
