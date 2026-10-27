//! Finite specialized-editor composition; all policy/context is explicit fixture data.
use stackpress_framework_ui::{
    format::{
        DecimalValue, DisplayLocale, E164Phone,
        edit::{
            DecimalPolicy, EditPolicy, EditState, FormattedInput, IsoValue, PhonePolicy,
            PhoneRegion, TemporalKind, TemporalPolicy,
        },
    },
    leptos::prelude::*,
};
use std::ops::Bound::{Excluded, Included, Unbounded};

/// Construct known fixture state; application code normally propagates constructor errors.
fn state<P: EditPolicy>(
    policy: P,
    value: P::Value,
    locale: DisplayLocale,
) -> RwSignal<EditState<P>> {
    let state = EditState::new(policy, Some(value), locale).expect("fixed admitted editor fixture");
    RwSignal::new(state)
}
/// Admit a pre-JavaScript native draft and exact UTF-16 selection before hydrating nodes.
#[cfg(target_arch = "wasm32")]
fn adopt<P: EditPolicy>(id: &str, state: RwSignal<EditState<P>>) {
    // The library bridge enforces JS-owned byte/digit limits before any Rust text copy.
    let _ = stackpress_framework_ui::format::edit::adopt_native(id, state);
}
/// Shared SSR/hydrate/CSR editor source; neither locale nor region is negotiated.
#[component]
pub fn EditingProof(
    /// Host-seeded explicit editing locale.
    locale: DisplayLocale,
) -> impl IntoView {
    let current = RwSignal::new(locale);
    let modal = RwSignal::new(false);
    let modal_edit = state(
        DecimalPolicy::new(2, 24, Unbounded, Unbounded).unwrap(),
        DecimalValue::parse("1.00").unwrap(),
        locale,
    );
    let number = state(
        DecimalPolicy::new(
            2,
            24,
            Included(DecimalValue::parse("-1000000").unwrap()),
            Excluded(DecimalValue::parse("1000000").unwrap()),
        )
        .unwrap(),
        DecimalValue::parse("1234.50").unwrap(),
        locale,
    );
    let date = state(
        TemporalPolicy::new(
            TemporalKind::Date,
            0,
            Included(IsoValue::parse(TemporalKind::Date, "2020-01-01").unwrap()),
            Excluded(IsoValue::parse(TemporalKind::Date, "2030-01-01").unwrap()),
        )
        .unwrap(),
        IsoValue::parse(TemporalKind::Date, "2024-02-29").unwrap(),
        locale,
    );
    let time = state(
        TemporalPolicy::new(TemporalKind::Time, 3, Unbounded, Unbounded).unwrap(),
        IsoValue::parse(TemporalKind::Time, "12:30:00.000").unwrap(),
        locale,
    );
    let instant = state(
        TemporalPolicy::new(TemporalKind::Instant, 0, Unbounded, Unbounded).unwrap(),
        IsoValue::parse(TemporalKind::Instant, "2026-11-01T01:30:00-04:00").unwrap(),
        locale,
    );
    let phone = state(
        PhonePolicy::new(PhoneRegion::parse("FR").unwrap()),
        E164Phone::parse("+33123456789").unwrap(),
        locale,
    );
    #[cfg(target_arch = "wasm32")]
    {
        adopt("edit-number", number);
        adopt("edit-date", date);
        adopt("edit-time", time);
        adopt("edit-instant", instant);
        adopt("edit-phone", phone);
    }
    let count = RwSignal::new(0usize);
    let region = RwSignal::new("FR".to_owned());
    let number_commit = Callback::new(move |_: Option<DecimalValue>| count.update(|n| *n += 1));
    let temporal_commit = Callback::new(move |_: Option<IsoValue>| count.update(|n| *n += 1));
    let phone_commit = Callback::new(move |_: Option<E164Phone>| count.update(|n| *n += 1));
    let change_locale = Callback::new(move |locale| {
        current.set(locale);
        number.update(|s| s.set_locale(locale));
        date.update(|s| s.set_locale(locale));
        time.update(|s| s.set_locale(locale));
        instant.update(|s| s.set_locale(locale));
        phone.update(|s| s.set_locale(locale));
        modal_edit.update(|s| s.set_locale(locale));
    });
    view! {<main id="editing-proof"><h1>"Specialized text editing"</h1>
        <p>"Enter commits and formats. Blur commits while keeping native text. Escape cancels."</p>
        <div class="edit-actions" role="group" aria-label="Editing language"><button id="edit-en" type="button" on:pointerdown=|event|event.prevent_default() on:click=move |_|change_locale.run(DisplayLocale::English)>"English"</button><button id="edit-fr" type="button" on:pointerdown=|event|event.prevent_default() on:click=move |_|change_locale.run(DisplayLocale::French)>"Français"</button></div>
        <p id="edit-locale">{move ||match current.get(){DisplayLocale::English=>"en",DisplayLocale::French=>"fr"}}</p>
        <form id="editing-form" on:submit=|event|event.prevent_default()>
            <FormattedInput id="edit-number" name="decimal" label="Decimal amount" state=number nullable=true on_commit=number_commit help="English: 1,234.50 · French: 1 234,50. Two fraction digits; half-even commit. Range −1000000 inclusive to 1000000 exclusive."/>
            <FormattedInput id="edit-date" name="date" label="Calendar date" state=date on_commit=temporal_commit help="ISO YYYY-MM-DD. From 2020-01-01 inclusive to 2030-01-01 exclusive."/>
            <FormattedInput id="edit-time" name="time" label="Local time" state=time on_commit=temporal_commit help="ISO HH:MM[:SS[.sss]]. Three fractional digits; no date or timezone."/>
            <FormattedInput id="edit-instant" name="instant" label="Instant with offset" state=instant on_commit=temporal_commit help="ISO date/time and explicit Z or ±HH:MM. Named zones and DST conversion are not inferred."/>
            <label for="edit-region">"National phone region"</label><select id="edit-region" prop:value=move ||region.get() on:change=move |event|{let next=event_target_value(&event);let previous=region.get_untracked();let mut accepted=false;if let Ok(next_region)=PhoneRegion::parse(&next){phone.update(|s|accepted=s.set_policy(PhonePolicy::new(next_region)).is_ok());}region.set(if accepted{next}else{previous});}><option value="FR" selected=true>"France (FR)"</option><option value="US">"United States (US)"</option><option value="GB">"United Kingdom (GB)"</option></select>
            <FormattedInput id="edit-phone" name="phone" label="Phone number" state=phone on_commit=phone_commit help="National draft uses the selected region. + and 00 introduce international drafts. Extensions are excluded. Press Enter or Escape before changing region."/>
        </form><p>"Changed value commits: "<output id="edit-commits">{move ||count.get()}</output></p>
        <div class="edit-actions" role="group" aria-label="Editing actions"><button id="edit-outside" type="button">"Leave field"</button>
        <button id="edit-modal-open" type="button" on:click=move |_|modal.set(true)>"Open edit dialog"</button></div>
        <stackpress_framework_ui::components::Dialog id="edit-modal" open=modal title="Editing dialog" initial_focus_id="edit-modal-number" fallback_focus_id="edit-modal-open"><FormattedInput id="edit-modal-number" name="modal-decimal" label="Modal amount" state=modal_edit/></stackpress_framework_ui::components::Dialog>
        <p>"Chromium composition sequences are synthetic test evidence. Real OS IME and packaged WebView are separate."</p>
    </main>}
}
/// Render one host-seeded finite editor packet without a listener.
#[cfg(feature = "ssr")]
pub fn render(locale: DisplayLocale) -> String {
    let owner = Owner::new();
    let html = owner.with(|| view! {<EditingProof locale/>}.to_html());
    owner.cleanup();
    html
}
