//! Lane A ordinary controls and safe display fixtures, shared by every renderer.
use leptos::prelude::*;
use stackpress_framework_ui::components::*;

/// Security corpus: RCDATA breakout, entities, quotes, Unicode and leading newline.
pub const TEXTAREA_PAYLOAD: &str = "\n</textarea><script>globalThis.pwned=1</script>&\"'\u{2028}";

/// Native edits adopted before hydration; no read of browser preferences occurs.
#[derive(Clone)]
pub struct CoreDraft {
    /// Verbatim text edit.
    pub text: String,
    /// Verbatim multiline/IME edit.
    pub notes: String,
    /// Transient user-entered password; never serialized into a page snapshot.
    pub password: String,
    /// Explicit false is retained.
    pub checked: bool,
    /// Explicit switch state.
    pub enabled: bool,
    /// Nullable radio group selection.
    pub radio: Option<String>,
    /// None and a selected empty string are different.
    pub single: Option<String>,
    /// Exact multiple selection, including zero/empty strings.
    pub multiple: Vec<String>,
}
impl Default for CoreDraft {
    /// Deterministic public seed; the password starts empty on every target.
    fn default() -> Self {
        Self {
            text: "Seed 雪".into(),
            notes: TEXTAREA_PAYLOAD.into(),
            password: String::new(),
            checked: false,
            enabled: true,
            radio: Some("0".into()),
            single: Some(String::new()),
            multiple: vec!["0".into(), "fr".into()],
        }
    }
}
/// Fixed options include a meaningful empty value, zero and a disabled option.
fn options() -> SelectOptions {
    SelectOptions::new(
        [
            ("", "Empty value", false),
            ("0", "Zero", false),
            ("fr", "Français", false),
            ("blocked", "Unavailable", true),
        ]
        .into_iter()
        .map(|(value, label, disabled)| SelectOption {
            value: value.into(),
            label: label.into(),
            disabled,
        })
        .collect(),
    )
    .expect("fixed valid options")
}
/// Actual ordinary input/presentation family fixture; no domain write is implied.
#[component]
pub fn CoreGallery(
    /// Optional early DOM values, captured before hydrate/mount.
    #[prop(optional)]
    initial: Option<CoreDraft>,
    /// Checked mount for the single manifest-owned raster asset.
    #[prop(into)]
    base: String,
) -> impl IntoView {
    let seed = initial.unwrap_or_default();
    let text = RwSignal::new(seed.text);
    let notes = RwSignal::new(seed.notes);
    let password = RwSignal::new(seed.password);
    let checked = RwSignal::new(seed.checked);
    let enabled = RwSignal::new(seed.enabled);
    let radio = RwSignal::new(seed.radio);
    let single = RwSignal::new(seed.single);
    let multiple = RwSignal::new(seed.multiple);
    let disabled = RwSignal::new(false);
    let invalid = RwSignal::new(false);
    let progress = RwSignal::new(ProgressValue::determinate(0.0, 10.0).unwrap());
    let summary = move || {
        serde_json::json!({"text":text.get(),"notes":notes.get(),"passwordLength":password.get().chars().count(),"checked":checked.get(),"enabled":enabled.get(),"radio":radio.get(),"single":single.get(),"multiple":multiple.get()}).to_string()
    };
    // A deliberate inspection publishes live controller state after hydration.
    // Keeping this initially empty preserves the exact server DOM even when native
    // edits were adopted into signals before their first hydration observation.
    let reported = RwSignal::new(String::new());
    // Programmatic root updates exercise controlled properties, independently of user events.
    let reset = move |_| {
        text.set("Programmatic 雪".into());
        notes.set("Updated\n雪".into());
        password.set(String::new());
        checked.set(false);
        enabled.set(false);
        radio.set(None);
        single.set(None);
        multiple.set(Vec::new());
        invalid.set(false);
        reported.set(String::new());
    };
    let table = DisplayTable::new(
        vec!["Key".into(), "Value".into()],
        vec![
            DisplayRow {
                key: "one".into(),
                cells: vec!["Zero".into(), "0".into()],
            },
            DisplayRow {
                key: "two".into(),
                cells: vec!["Markup".into(), "<script>inert</script>".into()],
            },
        ],
    )
    .unwrap();
    view! { <section id="ordinary-core" aria-label="Ordinary component core">
        <h2>"Ordinary controls"</h2>
        <form id="core-form" class="sp-core-form" on:submit=move |event| { event.prevent_default(); invalid.set(text.get().trim().is_empty()); }>
            <FieldControl for_id="core-text" label="Text" help="Verbatim text, including IME" error=move || if invalid.get() { "Enter text" } else { "" }.to_owned()>
                <Input id="core-text" name="text" value=text disabled required=true invalid described_by="core-text-help core-text-error"/>
            </FieldControl>
            <FieldControl for_id="core-notes" label="Notes" help="Multiline draft">
                <Textarea id="core-notes" name="notes" value=notes disabled required=true described_by="core-notes-help core-notes-error"/>
            </FieldControl>
            <FieldControl for_id="core-password" label="Password" help="Transient demo value, never saved">
                <PasswordInput id="core-password" name="password" value=password disabled described_by="core-password-help core-password-error"/>
            </FieldControl>
            <FieldControl for_id="core-check" label="Receive updates">
                <Checkbox id="core-check" name="updates" value="yes".to_owned() checked disabled required=true described_by="core-check-help core-check-error"/>
            </FieldControl>
            <FieldControl for_id="core-switch" label="Enable reminders">
                <Switch id="core-switch" name="enabled" checked=enabled disabled described_by="core-switch-help core-switch-error"/>
            </FieldControl>
            <fieldset><legend>"Delivery mode"</legend>
                <FieldControl for_id="core-radio-zero" label="Zero mode"><Radio id="core-radio-zero" name="mode" value="0" selected=radio disabled/></FieldControl>
                <FieldControl for_id="core-radio-fr" label="French mode"><Radio id="core-radio-fr" name="mode" value="fr" selected=radio disabled/></FieldControl>
            </fieldset>
            <FieldControl for_id="core-select" label="Single choice">
                <Select id="core-select" name="choice" value=SelectValue::Single(single) options=options() disabled required=true described_by="core-select-help core-select-error"/>
            </FieldControl>
            <FieldControl for_id="core-multiple" label="Multiple choices">
                <Select id="core-multiple" name="choices" value=SelectValue::Multiple(multiple) options=options() disabled described_by="core-multiple-help core-multiple-error"/>
            </FieldControl>
            <FieldControl for_id="core-readonly" label="Read only"><Input id="core-readonly" name="readonly" value=RwSignal::new("Kept 0".to_owned()) read_only=true/></FieldControl>
            <ActionGroup label="Input checks"><button type="button" id="core-inspect" on:click=move |_| reported.set(summary())>"Inspect state"</button><button type="button" id="core-reset" on:click=reset>"Reset from state"</button><button type="button" id="core-disabled" on:click=move |_| disabled.update(|value| *value = !*value)>"Toggle disabled"</button><button type="button" id="core-validate" on:click=move |_| invalid.set(text.get().trim().is_empty())>"Validate text"</button></ActionGroup>
        </form>
        <p>"Inspect current values; password text is omitted."</p><output id="core-state" class="sp-core-state" aria-label="Last inspected values">{move || reported.get()}</output>
        <h2>"Safe presentation"</h2>
        <Card label="Presentation example"><CardHead><CardTitle>"A useful card"</CardTitle><CardDescription>"Explicit semantic slots"</CardDescription></CardHead>
            <CardBody><Badge>"0 · <safe>"</Badge><Progress id="core-progress" label="Progress" value=progress/>
                <div id="core-color"><ColorView value=HexColor::new("#777").unwrap()/></div>
                <div id="core-email"><EmailLink value=EmailAddress::new("demo+tag@example.invalid").unwrap()/></div>
                <div id="core-phone"><PhoneLink value=PhoneNumber::new("+639171234567").unwrap()/></div>
                <Image source=ImageSource::new(format!("{base}/assets/sample.png"),68,1,1).unwrap() alt="One pixel demonstration asset"/>
                <div id="core-rating"><RatingView value=RatingValue::new(2.5,5).unwrap()/></div>
                <div id="core-yesno"><YesNo value=enabled yes="Enabled".to_owned() no="Disabled".to_owned()/></div>
                <div id="core-overflow"><TextOverflow value="雪😀abc" limit=TextLimit::Characters(2)/></div>
                <TextTransform value="Mixed Case" format=TextCase::Uppercase/>
                <MetadataView value=MetadataValues::new(vec![("Empty".into(),String::new()),("Count".into(),"0".into())]).unwrap()/>
                <Spread value=DisplayValues::new(vec!["0".into(),"<safe>".into()]).unwrap() separator=SpreadSeparator::Inline(" | ".into())/>
                <Spread value=DisplayValues::new(vec!["Line one".into(),"Line two".into()]).unwrap() separator=SpreadSeparator::Lines/>
                <Tags value=DisplayValues::new(vec!["0".into(),"雪".into(),"<safe>".into()]).unwrap()/>
                <Tabular value=table caption="Read-only values"/>
            </CardBody><CardFoot><ActionGroup label="Progress examples"><button type="button" id="core-progress-known" on:click=move |_| progress.set(ProgressValue::determinate(10.0,10.0).unwrap())>"Complete"</button><button type="button" id="core-progress-unknown" on:click=move |_| progress.set(ProgressValue::Indeterminate)>"Indeterminate"</button></ActionGroup></CardFoot>
        </Card>
    </section> }
}
