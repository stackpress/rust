//! Native-semantic controls, explicit layout slots and root-scoped providers.
//!
//! All strings render as escaped text. Styles are small, neutral and identical
//! across SSR/hydration/CSR; application-specific branding remains with callers.
//! Include CORE_STYLES for ordinary controls/action groups and ADVANCED_STYLES
//! for table/dialog presentation and document scroll locking. Both are opt-in.

mod advanced;
mod controls;
mod display_values;
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

pub use advanced::{ADVANCED_STYLES, Dialog, FieldError, Form, FormState, Table};

pub use controls::{
    Alert, Button, ButtonType, FieldControl, Input, Link, List, ListItem, Loader, LoaderContainer,
    NotifierContainer, safe_href,
};
pub use layout::{
    LayoutBlank, LayoutHead, LayoutLeft, LayoutMain, LayoutMenu, LayoutPanel, LayoutProvider,
    LayoutRight,
};

#[cfg(all(test, feature = "ssr"))]
mod tests;

#[cfg(all(test, feature = "ssr"))]
mod core_tests;
