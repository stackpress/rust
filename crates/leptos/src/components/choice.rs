//! Bounded immutable choices and root-owned roving focus for tabs, menus and lists.
use super::{ControlError, SelectOptions};
use leptos::prelude::*;

/// A keyboard movement independent of browser event types.
#[derive(Clone, Copy, Debug)]
pub enum ChoiceMove {
    /// Previous enabled choice, wrapping at the beginning.
    Previous,
    /// Next enabled choice, wrapping at the end.
    Next,
    /// First enabled choice.
    First,
    /// Last enabled choice.
    Last,
}
/// Share arrow/edge key policy across tabs, vertical menus and horizontal menubars.
/// Activation and DOM focus remain the caller's explicit widget policy.
pub(super) fn choice_movement(key: &str, vertical: bool) -> Option<ChoiceMove> {
    match key {
        "Home" => Some(ChoiceMove::First),
        "End" => Some(ChoiceMove::Last),
        "ArrowDown" if vertical => Some(ChoiceMove::Next),
        "ArrowUp" if vertical => Some(ChoiceMove::Previous),
        "ArrowRight" if !vertical => Some(ChoiceMove::Next),
        "ArrowLeft" if !vertical => Some(ChoiceMove::Previous),
        _ => None,
    }
}
/// Root-owned choice state; immutable unique options are admitted before rendering.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_leptos::components::{ChoiceMove, ChoiceState, SelectOption, SelectOptions};
/// let owner = Owner::new();
/// owner.with(|| -> Result<(), stackpress_leptos::components::ControlError> {
///     let options = SelectOptions::new(vec![SelectOption {
///         value: "details".into(), label: "Details".into(), disabled: false,
///     }])?;
///     let tabs = ChoiceState::new(options, None)?;
///     assert_eq!(tabs.navigate(ChoiceMove::First, true), Some(0));
///     assert_eq!(tabs.selected().as_deref(), Some("details"));
///     Ok(())
/// })?;
/// owner.cleanup();
/// # Ok::<(), stackpress_leptos::components::ControlError>(())
/// ```
#[derive(Clone, Copy)]
pub struct ChoiceState {
    options: StoredValue<SelectOptions>,
    selected: RwSignal<Option<String>>,
    focused: RwSignal<Option<usize>>,
}
impl ChoiceState {
    /// Construct inside an Owner; reject an unknown/disabled initial selection.
    /// # Errors
    /// Invalid initial values are not silently replaced with the first option.
    pub fn new(options: SelectOptions, initial: Option<String>) -> Result<Self, ControlError> {
        // Reject forged/disabled initial keys before allocating reactive state.
        if initial.as_ref().is_some_and(|key| {
            !options
                .entries()
                .iter()
                .any(|o| &o.value == key && !o.disabled)
        }) {
            return Err(ControlError);
        }
        // Remember a usable roving stop even when selection is explicitly absent.
        let focused = initial
            .as_ref()
            .and_then(|key| options.entries().iter().position(|o| &o.value == key))
            .or_else(|| options.entries().iter().position(|o| !o.disabled));
        Ok(Self {
            options: StoredValue::new(options),
            selected: RwSignal::new(initial),
            focused: RwSignal::new(focused),
        })
    }
    /// Borrow a clone of the bounded immutable choices for explicit composition.
    pub fn options(self) -> SelectOptions {
        self.options.get_value()
    }
    /// Read the current selection, including None versus an explicit empty value.
    pub fn selected(self) -> Option<String> {
        self.selected.get()
    }
    /// Read the one enabled choice in sequential keyboard order.
    pub fn focused(self) -> Option<usize> {
        self.focused.get()
    }
    /// Clear selection without changing the remembered roving focus.
    pub fn clear(self) {
        self.selected.set(None);
    }
    /// Select an enabled known key; invalid event input never changes state.
    pub fn choose(self, key: &str) -> bool {
        let index = self.options.with_value(|items| {
            items
                .entries()
                .iter()
                .position(|o| o.value == key && !o.disabled)
        });
        if let Some(index) = index {
            self.focused.set(Some(index));
            self.selected.set(Some(key.to_owned()));
            true
        } else {
            false
        }
    }
    /// Focus a known enabled index without selecting in manual-activation widgets.
    pub fn focus(self, index: usize) -> bool {
        if self
            .options
            .with_value(|items| items.entries().get(index).is_some_and(|o| !o.disabled))
        {
            self.focused.set(Some(index));
            true
        } else {
            false
        }
    }
    /// Move among enabled entries. Automatic activation is an explicit caller policy.
    pub fn navigate(self, direction: ChoiceMove, activate: bool) -> Option<usize> {
        // Build one bounded enabled-index projection; disabled choices never enter movement.
        let enabled: Vec<_> = self.options.with_value(|items| {
            items
                .entries()
                .iter()
                .enumerate()
                .filter(|(_, o)| !o.disabled)
                .map(|(i, _)| i)
                .collect()
        });
        if enabled.is_empty() {
            return None;
        }
        let current = self
            .focused
            .get()
            .and_then(|i| enabled.iter().position(|candidate| *candidate == i));
        // Wrap within that projection, then optionally select for automatic activation.
        let next = match direction {
            ChoiceMove::First => 0,
            ChoiceMove::Last => enabled.len() - 1,
            ChoiceMove::Next => current.map_or(0, |i| (i + 1) % enabled.len()),
            ChoiceMove::Previous => current.map_or(enabled.len() - 1, |i| {
                (i + enabled.len() - 1) % enabled.len()
            }),
        };
        let index = enabled[next];
        self.focused.set(Some(index));
        if activate {
            let key = self
                .options
                .with_value(|items| items.entries()[index].value.clone());
            self.choose(&key);
        }
        Some(index)
    }
    /// Find the next enabled case-insensitive label prefix, wrapping once.
    pub fn typeahead(self, prefix: &str) -> Option<usize> {
        if prefix.is_empty() || prefix.len() > 128 {
            return None;
        }
        let items = self.options();
        let count = items.entries().len();
        let start = self
            .focused
            .get()
            .map_or(0, |i| if prefix.chars().count() > 1 { i } else { i + 1 });
        let prefix = prefix.to_lowercase();
        let next = (0..count).map(|offset| (start + offset) % count).find(|i| {
            let option = &items.entries()[*i];
            !option.disabled && option.label.to_lowercase().starts_with(&prefix)
        });
        if let Some(index) = next {
            self.focused.set(Some(index));
        }
        next
    }
}

/// Root-owned disclosure selection, with explicit multiple/collapsible policy.
#[derive(Clone, Copy)]
pub struct DisclosureState {
    choices: ChoiceState,
    opened: RwSignal<Vec<String>>,
    multiple: bool,
    collapsible: bool,
}
impl DisclosureState {
    /// Admit only unique enabled initial keys; a required group starts with a choice.
    /// # Errors
    /// Rejects unknown, disabled, duplicate or policy-inconsistent initial selections.
    pub fn new(
        options: SelectOptions,
        initial: Vec<String>,
        multiple: bool,
        collapsible: bool,
    ) -> Result<Self, ControlError> {
        if (!multiple && initial.len() > 1)
            || (!collapsible && initial.is_empty())
            || options.admit(&initial).len() != initial.len()
        {
            return Err(ControlError);
        }
        Ok(Self {
            choices: ChoiceState::new(options, None)?,
            opened: RwSignal::new(initial),
            multiple,
            collapsible,
        })
    }
    /// Is this known key expanded? Exact empty values remain meaningful.
    pub fn is_open(self, key: &str) -> bool {
        self.opened.get().iter().any(|value| value == key)
    }
    /// Toggle an admitted enabled key; return whether a real transition occurred.
    pub fn toggle(self, key: &str) -> bool {
        if !self
            .choices
            .options()
            .entries()
            .iter()
            .any(|o| o.value == key && !o.disabled)
        {
            return false;
        }
        // Compute first so required-single and multiple transitions publish atomically.
        let mut next = self.opened.get();
        if next.iter().any(|value| value == key) {
            if !self.collapsible && next.len() == 1 {
                return false;
            }
            next.retain(|value| value != key);
        } else {
            if !self.multiple {
                next.clear();
            }
            next.push(key.to_owned());
        }
        self.opened.set(next);
        true
    }
    /// Immutable option list shared by labels and panels.
    pub fn options(self) -> SelectOptions {
        self.choices.options()
    }
}

/// Bounded incremental keyboard typeahead with a 700ms reset and repeated-letter cycling.
#[derive(Clone, Debug, Default)]
pub struct TypeaheadState {
    prefix: String,
    last: f64,
}
impl TypeaheadState {
    /// Accept one printable key; invalid time/length/key input leaves the buffer unchanged.
    pub fn push(&mut self, key: &str, now: f64) -> Option<String> {
        if key.chars().count() != 1
            || key == " "
            || key.chars().any(char::is_control)
            || !now.is_finite()
            || now < 0.0
        {
            return None;
        }
        // Reset on a gap, clock reversal, byte exhaustion or repeated-letter cycling.
        if now < self.last
            || now - self.last > 700.0
            || self.prefix.len() + key.len() > 128
            || self.prefix.chars().all(|c| key.starts_with(c))
        {
            self.prefix.clear();
        }
        self.prefix.push_str(key);
        self.last = now;
        Some(self.prefix.clone())
    }
}
