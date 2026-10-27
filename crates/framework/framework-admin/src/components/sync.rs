//! Bidirectional adapter state without treating its empty presentation as supplied data.
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input;

/// A local adapter failure keeps the last exact draft and names the rejected value family.
pub(super) type Admission<T> = Result<T, &'static str>;

/// Connect a checked UI controller to the draft, suppressing programmatic echo transitions.
///
/// `last` records the rendered typed value before publishing a controller update. Thus a
/// Missing/Null reset can clear a controller without the controller promoting it to Value.
pub(super) fn connect<T>(
    draft: RwSignal<Input<Value>>,
    read: impl Fn() -> T + Send + Sync + 'static,
    write: impl Fn(T) + Send + Sync + 'static,
    decode: impl Fn(Input<Value>) -> Admission<T> + Send + Sync + 'static,
    encode: impl Fn(T) -> Admission<Input<Value>> + Send + Sync + 'static,
) -> RwSignal<String>
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let last = StoredValue::new(read());
    // Retain the exact input too: metadata object order can differ from editable row order.
    // Recognizing our own publication avoids rebuilding rows or losing their stable keys.
    let published = StoredValue::new(draft.get_untracked());
    let reset = StoredValue::new(None::<T>);
    let error = RwSignal::new(String::new());
    Effect::watch(
        move || draft.get(),
        move |value, _, _| {
            if published.get_value() == *value {
                return;
            }
            published.set_value(value.clone());
            // Programmatic presence changes update presentation before its observer can echo.
            match decode(value.clone()) {
                Ok(next) => {
                    error.set(String::new());
                    if last.get_value() != next {
                        last.set_value(next.clone());
                        reset.set_value(Some(next.clone()));
                        write(next);
                    }
                }
                Err(message) => error.set(message.into()),
            }
        },
        false,
    );
    Effect::watch(
        read,
        move |next, _, _| {
            // Suppress only the known programmatic reset, not repeated native edits.
            // Home at the lower bound still supplies a concrete zero; Clear while Null
            // still selects omission even though the rating presentation remains None.
            let resetting = reset.get_value();
            reset.set_value(None);
            if resetting.as_ref() == Some(next) {
                return;
            }
            last.set_value(next.clone());
            // An invalid editable intermediate stays visible, while the accepted draft survives.
            match encode(next.clone()) {
                Ok(next) => {
                    error.set(String::new());
                    if draft.get_untracked() != next {
                        published.set_value(next.clone());
                        draft.set(next);
                    }
                }
                Err(message) => error.set(message.into()),
            }
        },
        false,
    );
    error
}

/// Fail closed for a direct caller's invalid seed instead of rendering a coerced value.
pub(super) fn rejected() -> AnyView {
    view! { <p role="alert">"Value is incompatible with this control"</p> }.into_any()
}
