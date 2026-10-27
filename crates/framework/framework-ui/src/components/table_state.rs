//! Controlled local table projection; no SQL, permissions, fetching or URL mutation.
use super::{ControlError, DisplayRow, DisplayTable, PageTotal, PageWindow};
use leptos::prelude::*;
use std::collections::BTreeSet;
/// Explicit stable lexical sorting; locale-aware numeric formatting belongs to M3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending Unicode string order.
    Ascending,
    /// Descending Unicode string order.
    Descending,
}
/// Root-owned table state over a bounded validated snapshot.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_ui::components::{DisplayRow, DisplayTable, TableState};
/// let owner = Owner::new();
/// owner.with(|| -> Result<(), Box<dyn std::error::Error>> {
///     let data = DisplayTable::new(vec!["Name".into()], vec![DisplayRow {
///         key: "ada".into(), cells: vec!["Ada Lovelace".into()],
///     }])?;
///     let table = TableState::new(data, 10)?;
///     table.set_filter("ada".into())?;
///     table.select_page(true);
///     assert!(table.selected().contains("ada"));
///     Ok(())
/// })?;
/// owner.cleanup();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy)]
pub struct TableState {
    data: RwSignal<DisplayTable>,
    filter: RwSignal<String>,
    filter_draft: RwSignal<String>,
    sort: RwSignal<Option<(usize, SortDirection)>>,
    page: RwSignal<PageWindow>,
    selected: RwSignal<BTreeSet<String>>,
}
impl TableState {
    /// Construct from checked data and a bounded page size.
    /// # Errors
    /// Rejects sizes outside 1..=100 before creating state.
    pub fn new(data: DisplayTable, limit: u16) -> Result<Self, ControlError> {
        let page = PageWindow::new(0, limit, PageTotal::Known(data.rows.len() as u64))?;
        Ok(Self {
            data: RwSignal::new(data),
            filter: RwSignal::new(String::new()),
            filter_draft: RwSignal::new(String::new()),
            sort: RwSignal::new(None),
            page: RwSignal::new(page),
            selected: RwSignal::new(BTreeSet::new()),
        })
    }
    /// Reactive clone of the bounded current dataset.
    pub fn data(self) -> DisplayTable {
        self.data.get()
    }
    /// Reactive column count without copying the bounded row snapshot.
    pub fn column_count(self) -> usize {
        self.data.with(|data| data.columns.len())
    }
    /// Reactive column label for a retained header; obsolete indices return empty text.
    pub fn column(self, index: usize) -> String {
        self.data
            .with(|data| data.columns.get(index).cloned().unwrap_or_default())
    }
    /// Expose the application-owned paging signal to an explicit Pagination control.
    pub fn page(self) -> RwSignal<PageWindow> {
        self.page
    }
    /// Current literal case-insensitive filter; no regex/query language executes.
    pub fn filter(self) -> String {
        self.filter_draft.get()
    }
    /// Adopt an early native draft without changing the initial SSR row projection.
    /// The next explicit set_filter/input event applies it to the dataset.
    /// # Errors
    /// Rejects text beyond 1024 bytes before mutation.
    pub fn adopt_filter(self, value: String) -> Result<(), ControlError> {
        if value.len() > 1024 {
            return Err(ControlError);
        }
        self.filter_draft.set(value);
        Ok(())
    }
    /// Bound filter text and reset to the first result page on a real change.
    /// # Errors
    /// Rejects excessive text without modifying state.
    pub fn set_filter(self, value: String) -> Result<(), ControlError> {
        if value.len() > 1024 {
            return Err(ControlError);
        }
        self.filter_draft.set(value.clone());
        self.filter.set(value);
        self.repage(true);
        Ok(())
    }
    /// Current controlled sort column/direction.
    pub fn sort(self) -> Option<(usize, SortDirection)> {
        self.sort.get()
    }
    /// Cycle none -> ascending -> descending -> none on an admitted column.
    pub fn toggle_sort(self, column: usize) -> bool {
        if column >= self.column_count() {
            return false;
        }
        self.sort.update(|sort| {
            *sort = match *sort {
                Some((index, SortDirection::Ascending)) if index == column => {
                    Some((column, SortDirection::Descending))
                }
                Some((index, SortDirection::Descending)) if index == column => None,
                _ => Some((column, SortDirection::Ascending)),
            }
        });
        self.repage(true);
        true
    }
    /// Read one reactive cell by stable identity; replacement updates retained row nodes.
    pub fn cell(self, key: &str, column: usize) -> String {
        self.data.with(|data| {
            data.rows
                .iter()
                .find(|row| row.key == key)
                .and_then(|row| row.cells.get(column))
                .cloned()
                .unwrap_or_default()
        })
    }
    /// Filter and sort at most 100 rows; ties retain original snapshot order.
    pub fn matching(self) -> Vec<DisplayRow> {
        let filter = self.filter.get().to_lowercase();
        // Borrow the bounded snapshot; only matching rows need an owned projection.
        let mut rows: Vec<_> = self.data.with(|data| {
            data.rows
                .iter()
                .filter(|row| {
                    row.cells
                        .iter()
                        .any(|cell| cell.to_lowercase().contains(&filter))
                })
                .cloned()
                .collect()
        });
        if let Some((column, direction)) = self.sort.get() {
            rows.sort_by(|a, b| {
                let order = a.cells[column].cmp(&b.cells[column]);
                match direction {
                    SortDirection::Ascending => order,
                    SortDirection::Descending => order.reverse(),
                }
            });
        }
        rows
    }
    /// Project only the controlled current page after deterministic local filtering.
    pub fn rows(self) -> Vec<DisplayRow> {
        let page = self.page.get();
        self.matching()
            .into_iter()
            .skip(usize::try_from(page.skip()).unwrap_or(usize::MAX))
            .take(usize::from(page.limit()))
            .collect()
    }
    /// Read selected stable identities across pages, including filtered-out rows.
    pub fn selected(self) -> BTreeSet<String> {
        self.selected.get()
    }
    /// Select/deselect a known dataset row. Forged keys cannot grow the selection.
    pub fn select(self, key: &str, selected: bool) -> bool {
        // Membership needs no cloned cells, especially when selecting an entire page.
        if !self
            .data
            .with(|data| data.rows.iter().any(|row| row.key == key))
        {
            return false;
        }
        self.selected.update(|keys| {
            if selected {
                keys.insert(key.to_owned());
            } else {
                keys.remove(key);
            }
        });
        true
    }
    /// Apply selection only to the currently visible page; other pages remain explicit.
    pub fn select_page(self, selected: bool) {
        for row in self.rows() {
            self.select(&row.key, selected);
        }
    }
    /// Replace with checked data, prune removed identities and clamp obsolete paging.
    pub fn replace(self, data: DisplayTable) {
        self.selected
            .update(|keys| keys.retain(|key| data.rows.iter().any(|row| &row.key == key)));
        if self
            .sort
            .get()
            .is_some_and(|(column, _)| column >= data.columns.len())
        {
            self.sort.set(None);
        }
        self.data.set(data);
        self.repage(false);
    }
    /// Keep paging totals synchronized at the same controlled mutation boundary.
    fn repage(self, reset: bool) {
        let count = self.matching().len() as u64;
        self.page.update(|page| {
            *page = PageWindow::new(
                if reset { 0 } else { page.skip() },
                page.limit(),
                PageTotal::Known(count),
            )
            .expect("existing bounded page size")
        });
    }
}
