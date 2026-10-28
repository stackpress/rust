//! Native-semantic controls, explicit layout slots and root-scoped providers.
//!
//! All strings render as escaped text. Styles are small, neutral and identical
//! across SSR/hydration/CSR; application-specific branding remains with callers.
//! Include CORE_STYLES for ordinary controls/action groups and ADVANCED_STYLES
//! for table/dialog presentation and document scroll locking. Add BREADTH_STYLES
//! for disclosure/navigation/collection surfaces. All are opt-in. System themes
//! additionally require SystemThemeStyles in the initial head before paint.

mod advanced;
/// Opt-in base layout tokens and control styles, also usable in static document shells.
/// This is the same stylesheet LayoutProvider supplies; it adds no theme policy.
pub const BASE_STYLES: &str = include_str!("styles.css");

mod controls;
mod display_values;
mod icons;
mod inputs;
mod presentation;
pub use display_values::{
    DisplayError, DisplayRow, DisplayTable, DisplayValues, EmailAddress, HexColor, ImageSource,
    MetadataValues, PhoneNumber, ProgressAmount, ProgressValue, RatingRound, RatingValue,
    TextLimit, overflow_text,
};
pub use presentation::{
    ActionGroup, Badge, CORE_STYLES, Card, CardBody, CardDescription, CardFoot, CardHead,
    CardTitle, ColorView, EmailLink, Image, MetadataView, PhoneLink, Progress, RatingView, Spread,
    SpreadSeparator, Tabular, Tags, TextCase, TextOverflow, TextTransform, YesNo,
};
#[cfg(target_arch = "wasm32")]
mod input_dom;
pub use inputs::{
    Checkbox, ControlError, PasswordInput, Radio, Select, SelectOption, SelectOptions, SelectValue,
    Switch, Textarea,
};
#[cfg(target_arch = "wasm32")]
mod dialog_dom;
mod layout;

pub use advanced::{ADVANCED_STYLES, Dialog, DialogMode, FieldError, Form, FormState, Table};

pub use controls::{
    Alert, Button, ButtonType, FieldControl, Input, Link, List, ListItem, Loader, LoaderContainer,
    NotifierContainer, safe_href,
};
pub use layout::{
    LayoutBlank, LayoutHead, LayoutLeft, LayoutMain, LayoutMenu, LayoutPanel, LayoutProvider,
    LayoutRight, SYSTEM_THEME_STYLES, SystemThemeStyles,
};

#[cfg(all(test, feature = "ssr"))]
mod tests;

#[cfg(all(test, feature = "ssr"))]
mod core_tests;

mod choice;
mod collection_state;
mod collections;
mod combobox;
mod disclosure;
#[cfg(any(target_arch = "wasm32", all(test, feature = "ssr")))]
mod geometry;
mod navigation;
mod overlay;
#[cfg(target_arch = "wasm32")]
mod overlay_dom;
mod paging;
mod table_family;
mod table_state;
mod tag_list;

pub use choice::{ChoiceMove, ChoiceState, DisclosureState, TypeaheadState};
pub use collection_state::{
    BoundedList, CollectionRow, CollectionValue, FieldValue, MetadataEntry,
};
pub use collections::{CollectionFieldset, MetadataInput, TextList};
#[cfg(feature = "async")]
pub use combobox::SuggestionCompletion;
pub use combobox::{Combobox, ComboboxState, SuggestInput, SuggestionStatus};
pub use disclosure::{Accordion, AccordionItem, Details, TabActivation, TabPanel, Tabs};
#[cfg(any(target_arch = "wasm32", all(test, feature = "ssr")))]
pub(crate) use geometry::{effective_zoom, place_popup};
pub use navigation::{BreadcrumbTrail, Breadcrumbs, Crumb, Menu, MenuEntry, Menubar};
pub use overlay::{
    BREADTH_STYLES, DelayMs, Drawer, Popover, PopupRole, Toast, Tooltip, TooltipActivity,
};
pub use paging::{PageTotal, PageWindow, Pagination};
pub use table_family::{
    DataTable, HeaderScope, TableCell, TableFoot, TableGroup, TableHead, TableHeader, TableRow,
    TableRule,
};
pub use table_state::{SortDirection, TableState};
pub use tag_list::TagList;

mod bounded_inputs;
pub use bounded_inputs::{
    RangeScale, RangeState, RangeValue, RatingInput, RatingState, Slider, SlugAlphabet, SlugInput,
    SlugStyle, slugify,
};

#[cfg(all(test, feature = "ssr"))]
mod breadth_tests;
