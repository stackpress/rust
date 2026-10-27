//! Pure specialized editing and an optional native-text Leptos adapter.
//!
//! Drafts are not values. Typing retains native text/undo; commit validates and
//! normalizes once. Blur may commit while preserving native text/undo. Locale changes
//! never reinterpret an outstanding draft. Clipboard grammar is strict; existing
//! grouping may move during native middle edits and is rebuilt on explicit commit.
//!
//! ```
//! use std::ops::Bound::Unbounded;
//! use stackpress_framework_ui::format::{DisplayLocale, edit::{DecimalPolicy, EditState, EditSelection, EditError}};
//! let policy = DecimalPolicy::new(2, 24, Unbounded, Unbounded)?;
//! let mut edit = EditState::new(policy, None, DisplayLocale::French)?;
//! edit.input("1,", EditSelection::end("1,"))?;
//! assert_eq!(edit.commit(true), Err(EditError::Incomplete));
//! edit.input("1234,135", EditSelection::end("1234,135"))?;
//! assert!(edit.commit(true)?);
//! assert_eq!(edit.canonical().as_deref(), Some("1234.14"));
//! assert_eq!(edit.draft(), "1\u{202f}234,14");
//! # Ok::<(), EditError>(())
//! ```

mod decimal;
#[cfg(target_arch = "wasm32")]
mod dom;
mod input;
mod phone;
mod selection;
mod temporal;
#[cfg(any(target_arch = "wasm32", test))]
mod transport;
#[cfg(target_arch = "wasm32")]
pub use dom::adopt_native;

use super::DisplayLocale;
pub use decimal::DecimalPolicy;
pub use input::FormattedInput;
pub use phone::{PhonePolicy, PhoneRegion};
pub use selection::{EditSelection, SelectionDirection};
pub use temporal::{IsoValue, TemporalKind, TemporalPolicy};

/// A fixed safe field diagnostic; user text is never reflected in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditError {
    /// Draft/clipboard bytes or digits exceed the configured bound.
    Limit,
    /// Text does not follow the explicitly selected editing grammar.
    Grammar,
    /// A valid prefix still lacks complete fields/digits.
    Incomplete,
    /// A complete field is impossible or fails phone metadata validation.
    Invalid,
    /// Empty is not allowed by this controller's explicit nullable policy.
    Required,
    /// Value is outside an inclusive/exclusive typed bound.
    Bounds,
    /// Fractional precision exceeds the configured scale.
    Precision,
    /// A policy/region change requires first committing or cancelling the draft.
    DirtyContext,
    /// Selection splits a UTF-16 surrogate or lies beyond the current draft.
    Selection,
}
impl std::fmt::Display for EditError {
    /// Fixed messages are safe for an associated native field error.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Limit => "Draft exceeds the field limit",
            Self::Grammar => "Text does not match the field grammar",
            Self::Incomplete => "Complete the field before committing",
            Self::Invalid => "Enter a valid complete value",
            Self::Required => "A value is required",
            Self::Bounds => "Value is outside the field bounds",
            Self::Precision => "Too many fractional digits",
            Self::DirtyContext => {
                "Press Enter or cancel the draft before changing region or policy"
            }
            Self::Selection => "Invalid text selection",
        })
    }
}
impl std::error::Error for EditError {}

/// Pure bounded policy used by the three shipped editor families.
/// Implementations perform no I/O and retain exact canonical values.
pub trait EditPolicy: Clone + Send + Sync + 'static {
    /// Complete validated value, independent of localized presentation.
    type Value: Clone + PartialEq + Send + Sync + 'static;
    /// Maximum raw UTF-8 bytes, checked before copying a native draft.
    fn max_bytes(&self) -> usize;
    /// Independent ASCII digit budget for pre-copy browser admission.
    fn max_digits(&self) -> usize {
        self.max_bytes()
    }
    /// Admit byte/digit budgets before the controller allocates a replacement draft.
    fn budget(&self, draft: &str) -> Result<(), EditError> {
        if draft.len() > self.max_bytes() {
            Err(EditError::Limit)
        } else {
            Ok(())
        }
    }
    /// Validate a complete draft with its captured editing locale.
    fn parse(&self, draft: &str, locale: DisplayLocale) -> Result<Self::Value, EditError>;
    /// Check clipboard grammar, permitting only unambiguous incomplete prefixes.
    fn admit(&self, draft: &str, locale: DisplayLocale) -> Result<(), EditError>;
    /// Native edits may temporarily move existing presentation separators.
    fn admit_input(&self, draft: &str, locale: DisplayLocale) -> Result<(), EditError> {
        self.admit(draft, locale)
    }
    /// Recheck a typed value against this policy's limits and inclusive/exclusive bounds.
    fn check(&self, value: &Self::Value) -> Result<(), EditError>;
    /// Render an already validated value at the chosen locale.
    fn display(&self, value: &Self::Value, locale: DisplayLocale) -> String;
    /// Canonical storage text, never a localized display string.
    fn canonical(&self, value: &Self::Value) -> String;
    /// Tokens retained through presentation changes; grouping/phone spacing is ignored.
    fn semantic(&self, ch: char, _locale: DisplayLocale) -> bool {
        ch.is_ascii_digit()
    }
}

/// A composition has exactly one lifetime and one deferred commit request.
#[derive(Clone, Default)]
enum Composition {
    /// No pending input ordering or native timer settlement.
    #[default]
    Idle,
    /// Intermediate IME text must remain untouched.
    Active(CompositionDraft),
    /// End arrived; final input may still arrive before the next task.
    Ending(CompositionDraft),
}
/// Cancellation restores the exact pre-composition context, not a reparsed value.
#[derive(Clone)]
struct CompositionDraft {
    /// Native text before the first intermediate composition input.
    text: String,
    /// Exact cancellation selection, including backward direction.
    selection: EditSelection,
    /// Whether a draft already existed before IME started.
    dirty: bool,
    /// Grammar of that original draft, independent of later locale changes.
    locale: DisplayLocale,
    /// Restore any original diagnostic if composition is cancelled.
    error: Option<EditError>,
    /// A blur belongs only to this session and cannot migrate to another IME.
    commit_after: bool,
}

/// Independent value/draft/selection/composition state; no DOM or reactive borrow guards.
#[derive(Clone)]
pub struct EditState<P: EditPolicy> {
    /// Immutable validation/interpretation policy until an explicit clean reconfiguration.
    policy: P,
    /// Last complete valid committed domain value, independent of native text.
    value: Option<P::Value>,
    /// Bounded native text, including partials and grammar-invalid edits.
    draft: String,
    /// Immutable SSR attribute seed; hydration adopts properties separately.
    initial: String,
    /// Latest native UTF-16 selection, checked before retaining a draft.
    selection: EditSelection,
    /// Requested presentation locale for the next clean/normalized value.
    locale: DisplayLocale,
    /// Grammar captured by the outstanding draft/composition.
    draft_locale: DisplayLocale,
    /// Uncommitted text or outstanding normalization; blocks policy reinterpretation.
    dirty: bool,
    /// One explicit IME lifecycle; deferred commits belong to that session.
    composition: Composition,
    /// Named field diagnostic; never contains the native input.
    error: Option<EditError>,
}
impl<P: EditPolicy> EditState<P> {
    /// Construct a deterministic clean draft from an explicitly checked typed value.
    /// # Errors
    /// Rejects initial values outside the policy, or empty nonnullable values on commit.
    pub fn new(
        policy: P,
        value: Option<P::Value>,
        locale: DisplayLocale,
    ) -> Result<Self, EditError> {
        if let Some(value) = &value {
            policy.check(value)?;
        }
        let draft = value
            .as_ref()
            .map(|v| policy.display(v, locale))
            .unwrap_or_default();
        // Typed values still pass the raw presentation budget before state is retained.
        policy.budget(&draft)?;
        let selection = EditSelection::end(&draft);
        Ok(Self {
            policy,
            value,
            initial: draft.clone(),
            draft,
            selection,
            locale,
            draft_locale: locale,
            dirty: false,
            composition: Composition::Idle,
            error: None,
        })
    }
    /// Borrow the last committed value; failed edits never replace it.
    pub fn value(&self) -> Option<&P::Value> {
        self.value.as_ref()
    }
    /// Borrow the verbatim current draft, including partial/invalid text.
    pub fn draft(&self) -> &str {
        &self.draft
    }
    /// Immutable server seed attribute; early property adoption never rewrites SSR attributes.
    pub fn initial(&self) -> &str {
        &self.initial
    }
    /// Adopt pre-bootstrap native text/selection without emitting commit or early validation.
    /// # Errors
    /// Applies the same byte/selection bounds as subsequent native input.
    pub fn adopt(&mut self, draft: &str, selection: EditSelection) -> Result<(), EditError> {
        if draft == self.draft {
            return self.select(selection);
        }
        self.input(draft, selection)?;
        self.error = None;
        Ok(())
    }
    /// Read UTF-16 selection and its forward/backward direction.
    pub fn selection(&self) -> EditSelection {
        self.selection
    }
    /// Safe associated error, independent of the prior value.
    pub fn error(&self) -> Option<EditError> {
        self.error
    }
    /// Whether native edits remain outstanding, even if the parsed value is unchanged.
    pub fn dirty(&self) -> bool {
        self.dirty
    }
    /// Composition remains active until its bounded next-turn settlement.
    pub fn composing(&self) -> bool {
        !matches!(self.composition, Composition::Idle)
    }
    /// Byte admission bound for the thin adapter.
    pub fn max_bytes(&self) -> usize {
        self.policy.max_bytes()
    }
    /// Copy transport limits before calling any JavaScript APIs, outside reactive guards.
    #[cfg(any(target_arch = "wasm32", test))]
    fn transport_budget(&self) -> (usize, usize) {
        (self.policy.max_bytes(), self.policy.max_digits())
    }
    /// Reject a pre-copy native snapshot/clipboard item without retaining its text.
    #[cfg(any(target_arch = "wasm32", test))]
    fn reject_native(&mut self, error: EditError, selected: EditSelection) {
        let _ = self.select(selected);
        self.error = Some(error);
    }
    /// Canonical successful-field value; None is distinct from zero/signed zero.
    pub fn canonical(&self) -> Option<String> {
        self.value.as_ref().map(|v| self.policy.canonical(v))
    }
    /// Record a selection without changing the text or committing anything.
    /// # Errors
    /// Rejects out-of-range offsets and surrogate splits atomically.
    pub fn select(&mut self, selection: EditSelection) -> Result<(), EditError> {
        selection.bytes(&self.draft)?;
        self.selection = selection;
        Ok(())
    }
    /// Admit native input before copying; typing may retain invalid grammar for correction.
    /// # Errors
    /// Oversized drafts/invalid selections preserve the prior draft and value.
    pub fn input(&mut self, draft: &str, selection: EditSelection) -> Result<(), EditError> {
        // All admission precedes copying/mutation, so rejection retains the last draft.
        if let Err(error) = self.policy.budget(draft) {
            self.error = Some(error);
            return Err(error);
        }
        if let Err(error) = selection.bytes(draft) {
            self.error = Some(error);
            return Err(error);
        }
        // Snapshot/duplicate-final events may update selection without becoming edits.
        // In particular compositionstart must preserve a previously clean cancellation base.
        if draft == self.draft {
            self.selection = selection;
            return Ok(());
        }
        if !self.dirty && !self.composing() {
            self.draft_locale = self.locale;
        }
        self.draft = draft.to_owned();
        self.selection = selection;
        self.dirty = true;
        self.error = if matches!(self.composition, Composition::Active(_)) {
            None
        } else {
            self.policy.admit_input(draft, self.draft_locale).err()
        };
        Ok(())
    }
    /// Validate a proposed native paste without performing it or touching native undo.
    /// # Errors
    /// Rejects line breaks, invalid grammar, excessive bytes/digits or invalid selection.
    pub fn paste(&mut self, text: &str, selection: EditSelection) -> Result<(), EditError> {
        let result = (|| {
            let (start, end) = selection.bytes(&self.draft)?;
            self.selection = selection;
            if text.len() > self.policy.max_bytes()
                || self.draft.len() - (end - start) + text.len() > self.policy.max_bytes()
            {
                return Err(EditError::Limit);
            }
            if text.contains(['\n', '\r']) {
                return Err(EditError::Grammar);
            }
            // Only an admitted, bounded range reaches candidate allocation.
            let candidate = format!("{}{}{}", &self.draft[..start], text, &self.draft[end..]);
            self.policy.admit(
                &candidate,
                if self.dirty {
                    self.draft_locale
                } else {
                    self.locale
                },
            )
        })();
        self.error = result.err();
        result
    }
    /// Keep the current selection as the cancellation baseline; no parsing runs.
    pub fn composition_start(&mut self) {
        if matches!(self.composition, Composition::Active(_)) {
            return;
        }
        // A new session never inherits a prior session's deferred blur.
        self.composition = Composition::Active(CompositionDraft {
            text: self.draft.clone(),
            selection: self.selection,
            dirty: self.dirty,
            locale: self.draft_locale,
            error: self.error,
            commit_after: false,
        });
        if !self.dirty {
            self.draft_locale = self.locale;
        }
        self.error = None;
    }
    /// End composition without parsing; duplicate ends keep the same pending session.
    pub fn composition_end(&mut self) {
        self.composition = match std::mem::take(&mut self.composition) {
            Composition::Active(before) => Composition::Ending(before),
            other => other,
        };
    }
    /// Cancel composition and restore draft, selection, dirty flag and captured locale.
    pub fn composition_cancel(&mut self) {
        if let Composition::Active(before) | Composition::Ending(before) =
            std::mem::take(&mut self.composition)
        {
            self.draft = before.text;
            self.selection = before.selection;
            self.dirty = before.dirty;
            self.draft_locale = before.locale;
            self.error = before.error;
        }
    }
    /// Settle only an ended session; an old timer cannot terminate a fresh active IME.
    pub fn settle_composition(&mut self) -> bool {
        if !matches!(self.composition, Composition::Ending(_)) {
            return false;
        }
        if let Composition::Ending(before) = std::mem::take(&mut self.composition) {
            before.commit_after
        } else {
            false
        }
    }
    /// Publish only a complete valid value; normalize and map semantic selection once.
    /// Returns true only if the committed value changed, avoiding duplicate callbacks.
    /// # Errors
    /// Failed commits retain draft/selection/value and expose a named field diagnostic.
    pub fn commit(&mut self, nullable: bool) -> Result<bool, EditError> {
        self.commit_mode(nullable, true)
    }
    /// Blur commits the typed value while retaining native text/selection/undo history.
    /// Outstanding presentation remains dirty until explicit normalization or cancellation.
    /// # Errors
    /// Uses the same complete validation and failure atomicity as [`Self::commit`].
    pub fn commit_preserving_draft(&mut self, nullable: bool) -> Result<bool, EditError> {
        self.commit_mode(nullable, false)
    }
    /// Validate first, then either normalize explicitly or preserve native edit history.
    fn commit_mode(&mut self, nullable: bool, normalize: bool) -> Result<bool, EditError> {
        if let Composition::Active(before) | Composition::Ending(before) = &mut self.composition {
            before.commit_after = true;
            return Ok(false);
        }
        let result = if self.draft.is_empty() {
            if nullable {
                Ok(None)
            } else {
                Err(EditError::Required)
            }
        } else {
            self.policy
                .parse(&self.draft, self.draft_locale)
                .and_then(|v| {
                    self.policy.check(&v)?;
                    Ok(Some(v))
                })
        };
        // Publishing happens only after the entire parse/bound check succeeds.
        let value = match result {
            Ok(value) => value,
            Err(error) => {
                self.error = Some(error);
                return Err(error);
            }
        };
        let next = value
            .as_ref()
            .map(|v| self.policy.display(v, self.locale))
            .unwrap_or_default();
        let changed = self.value != value;
        self.value = value;
        if normalize {
            self.selection = self.selection.remap(&self.draft, &next, |ch, new| {
                self.policy
                    .semantic(ch, if new { self.locale } else { self.draft_locale })
            });
            self.draft = next;
            self.draft_locale = self.locale;
            self.dirty = false;
        } else {
            // Keeping raw text avoids a programmatic DOM value setter on blur.
            self.dirty = self.draft != next || self.draft_locale != self.locale;
        }
        self.error = None;
        Ok(changed)
    }
    /// Preserve native text if a host cannot perform an undo-preserving replacement.
    #[cfg(any(target_arch = "wasm32", test))]
    fn retain_native(&mut self, text: &str, selected: EditSelection, locale: DisplayLocale) {
        if self.policy.budget(text).is_ok() && selected.bytes(text).is_ok() {
            self.draft = text.to_owned();
            self.selection = selected;
            self.draft_locale = locale;
            self.dirty = true;
        }
    }
    /// Captured grammar belongs to the current native draft.
    #[cfg(any(target_arch = "wasm32", test))]
    fn draft_locale(&self) -> DisplayLocale {
        self.draft_locale
    }
    /// Explicitly abandon outstanding edits; committed values are not reparsed.
    pub fn cancel(&mut self) {
        self.draft = self
            .value
            .as_ref()
            .map(|v| self.policy.display(v, self.locale))
            .unwrap_or_default();
        self.selection = EditSelection::end(&self.draft);
        self.dirty = false;
        self.composition = Composition::Idle;
        self.error = None;
    }
    /// Change presentation locale; a dirty draft keeps its captured grammar until commit.
    pub fn set_locale(&mut self, locale: DisplayLocale) {
        self.locale = locale;
        if !self.dirty && !self.composing() {
            let next = self
                .value
                .as_ref()
                .map(|v| self.policy.display(v, locale))
                .unwrap_or_default();
            self.selection = self.selection.remap(&self.draft, &next, |ch, new| {
                self.policy
                    .semantic(ch, if new { locale } else { self.draft_locale })
            });
            self.draft = next;
            self.draft_locale = locale;
        }
    }
    /// Switch explicit region/bounds only from a clean state; no silent reinterpretation.
    /// # Errors
    /// Dirty/composing state or an incompatible existing value leaves the policy intact.
    pub fn set_policy(&mut self, policy: P) -> Result<(), EditError> {
        // Reconfiguration cannot change the interpretation of an in-progress native edit.
        if self.dirty || self.composing() {
            self.error = Some(EditError::DirtyContext);
            return Err(EditError::DirtyContext);
        }
        if let Some(value) = &self.value {
            policy.check(value)?;
        }
        self.policy = policy;
        self.cancel();
        Ok(())
    }
}

#[cfg(test)]
mod tests;

/// Check inclusive/exclusive bounds without floating-point conversion.
pub(super) fn check_bounds<T>(
    value: &T,
    min: &std::ops::Bound<T>,
    max: &std::ops::Bound<T>,
    compare: impl Fn(&T, &T) -> std::cmp::Ordering,
) -> Result<(), EditError> {
    use std::{cmp::Ordering, ops::Bound};
    let below = match min {
        Bound::Unbounded => false,
        Bound::Included(min) => compare(value, min) == Ordering::Less,
        Bound::Excluded(min) => compare(value, min) != Ordering::Greater,
    };
    let above = match max {
        Bound::Unbounded => false,
        Bound::Included(max) => compare(value, max) == Ordering::Greater,
        Bound::Excluded(max) => compare(value, max) != Ordering::Less,
    };
    if below || above {
        Err(EditError::Bounds)
    } else {
        Ok(())
    }
}
