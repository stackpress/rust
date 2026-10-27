//! Explicit bounded offset paging; no location or data-operation side effects.
use super::ControlError;
use leptos::prelude::*;

/// Known totals and unknown totals are different host assertions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageTotal {
    /// Exact number of available rows.
    Known(u64),
    /// Host reports whether the next page exists without inventing a total.
    Unknown {
        /// Whether advancing is currently useful.
        has_more: bool,
    },
}
/// Validated offset/limit/total state; page size is always 1..=100.
///
/// ```
/// use stackpress_framework_ui::components::{PageWindow, PageTotal};
/// let page = PageWindow::from_query("skip=20&limit=10", PageTotal::Known(23))?;
/// assert_eq!(page.query(), "skip=20&limit=10");
/// assert!(!page.has_next());
/// assert_eq!(page.previous().skip(), 10);
/// # Ok::<(), stackpress_framework_ui::components::ControlError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageWindow {
    skip: u64,
    limit: u16,
    total: PageTotal,
}
impl PageWindow {
    /// Validate size and clamp an obsolete offset to the final known page.
    /// # Errors
    /// Rejects zero/excessive sizes or offsets that cannot advance without overflow.
    pub fn new(skip: u64, limit: u16, total: PageTotal) -> Result<Self, ControlError> {
        if !(1..=100).contains(&limit) || skip.checked_add(u64::from(limit)).is_none() {
            return Err(ControlError);
        }
        // Known snapshots clamp stale offsets; an unknown total preserves the host offset.
        let skip = match total {
            PageTotal::Known(0) => 0,
            PageTotal::Known(total) => {
                skip.min(((total - 1) / u64::from(limit)) * u64::from(limit))
            }
            PageTotal::Unknown { .. } => skip,
        };
        Ok(Self { skip, limit, total })
    }
    /// Offset in rows, not a fabricated page number.
    pub fn skip(self) -> u64 {
        self.skip
    }
    /// Maximum rows requested by this page.
    pub fn limit(self) -> u16 {
        self.limit
    }
    /// Exact or unknown host total.
    pub fn total(self) -> PageTotal {
        self.total
    }
    /// Whether a preceding page exists.
    pub fn has_previous(self) -> bool {
        self.skip > 0
    }
    /// Whether the host permits advancing.
    pub fn has_next(self) -> bool {
        if self.skip.checked_add(u64::from(self.limit) * 2).is_none() {
            return false;
        }
        match self.total {
            PageTotal::Known(total) => self.skip + u64::from(self.limit) < total,
            PageTotal::Unknown { has_more } => has_more,
        }
    }
    /// Previous page, saturating at the beginning.
    pub fn previous(self) -> Self {
        Self {
            skip: self.skip.saturating_sub(u64::from(self.limit)),
            ..self
        }
    }
    /// Next page only when allowed; return None at the end or integer boundary.
    pub fn next_page(self) -> Option<Self> {
        if !self.has_next() {
            return None;
        }
        Self::new(
            self.skip.checked_add(u64::from(self.limit))?,
            self.limit,
            self.total,
        )
        .ok()
    }
    /// Canonical query fragment; the host decides whether/where to navigate.
    pub fn query(self) -> String {
        format!("skip={}&limit={}", self.skip, self.limit)
    }
    /// Decode only canonical unsigned skip/limit fields, once, in either order.
    /// # Errors
    /// Rejects extra/duplicate fields, signs, escapes and ambiguous number spellings.
    pub fn from_query(query: &str, total: PageTotal) -> Result<Self, ControlError> {
        // Parse exactly two canonical unsigned fields without implicit percent decoding.
        let mut skip = None;
        let mut limit = None;
        for field in query.split('&') {
            let (key, value) = field.split_once('=').ok_or(ControlError)?;
            if value.is_empty()
                || !value.bytes().all(|b| b.is_ascii_digit())
                || (value.len() > 1 && value.starts_with('0'))
            {
                return Err(ControlError);
            }
            match key {
                "skip" if skip.is_none() => {
                    skip = Some(value.parse::<u64>().map_err(|_| ControlError)?)
                }
                "limit" if limit.is_none() => {
                    limit = Some(value.parse::<u16>().map_err(|_| ControlError)?)
                }
                _ => return Err(ControlError),
            }
        }
        Self::new(skip.ok_or(ControlError)?, limit.ok_or(ControlError)?, total)
    }
}
/// Labelled native paging actions. The application owns the signal and URL update.
#[component]
pub fn Pagination(
    /// Accessible navigation name.
    #[prop(into)]
    label: String,
    /// Controlled immutable page snapshot.
    page: RwSignal<PageWindow>,
) -> impl IntoView {
    view! {<nav class="sp-actions" aria-label=label>
        <button type="button" disabled=move || !page.get().has_previous() on:click=move |_|page.update(|p|*p=p.previous())>"Previous"</button>
        <span role="status">{move || match page.get().total(){PageTotal::Known(total)=>format!("Offset {} of {total}",page.get().skip()),PageTotal::Unknown{..}=>format!("Offset {} · total unknown",page.get().skip())}}</span>
        <button type="button" disabled=move || !page.get().has_next() on:click=move |_|page.update(|p|{if let Some(next)=p.next_page(){*p=next;}})>"Next"</button>
    </nav>}
}
