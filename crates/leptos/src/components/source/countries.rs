//! Source country/currency lookup behavior over the unchanged Frui record arrays.
use super::data::{COUNTRIES, CURRENCIES, CountryData, CurrencyData};
use super::{SourceAppearance, SourceChoice, SourceSelect};
use leptos::prelude::*;

/// Source country/currency label and flag display, preserving different lookup keys and filters.
#[component]
pub fn SourceCountry(
    /// Country iso3 or currency code; unknown values are shown verbatim.
    #[prop(into)]
    value: String,
    /// Select currency rather than country lookup.
    #[prop(optional)]
    currency: bool,
    /// Original flag default.
    #[prop(default = true)]
    flag: bool,
    /// Original text default; false only suppresses text while flag=true.
    #[prop(default = true)]
    text: bool,
    /// Native appearance (flag-only source branches intentionally ignore it).
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let found = if currency {
        CURRENCIES
            .iter()
            .filter(|currency| currency.r#type == "fiat")
            .filter_map(|currency| {
                COUNTRIES
                    .iter()
                    .find(|country| country.cur == currency.code)
                    .map(|country| (currency, country.flag))
            })
            .find(|(currency, _)| currency.code == value)
            .map(|(currency, flag)| (currency.name, flag))
    } else {
        COUNTRIES
            .iter()
            .find(|country| country.iso3 == value)
            .map(|country| (country.name, country.flag))
    };
    let Some((name, emoji)) = found else {
        return view! {<span class=appearance.class_name style=appearance.style>{value}</span>}
            .into_any();
    };
    if flag && text {
        view!{<span class=format!("sp-source-country {}",appearance.class_name) style=appearance.style><span>{emoji}</span><span>{name}</span></span>}.into_any()
    } else if flag {
        view! {<span>{emoji}</span>}.into_any()
    } else {
        view! {<span class=appearance.class_name style=appearance.style>{name}</span>}.into_any()
    }
}

/// Exact source country selector; codes are iso2, callbacks return complete country records.
#[component]
pub fn SourceCountrySelect(
    /// Native trigger prefix.
    #[prop(into)]
    id: String,
    /// Hidden successful form key.
    #[prop(into)]
    name: String,
    /// Mount-only original selector value.
    #[prop(optional_no_strip)]
    initial: Option<String>,
    /// Source search toggle.
    #[prop(optional)]
    searchable: bool,
    /// Authored search placeholder (source accepts a searchable string).
    #[prop(default="Search...".into(),into)]
    search_placeholder: String,
    /// Empty display text.
    #[prop(default="Select a country".into(),into)]
    placeholder: String,
    /// Full original record callback.
    #[prop(optional)]
    on_update: Option<Callback<CountryData>>,
    /// Root classes/style.
    #[prop(optional)]
    appearance: SourceAppearance,
    /// Display slot.
    #[prop(optional)]
    display: SourceAppearance,
    /// Dropdown slot.
    #[prop(optional)]
    dropdown: SourceAppearance,
    /// Option slot.
    #[prop(optional)]
    option: SourceAppearance,
) -> impl IntoView {
    let choices = COUNTRIES
        .iter()
        .map(|country| SourceChoice {
            value: country.iso2.into(),
            label: format!("{} {}", country.flag, country.name),
            search: vec![
                country.name,
                country.iso2,
                country.iso3,
                country.cur,
                country.lang,
                country.tel,
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        })
        .collect();
    view! {<SourceSelect id name initial choices searchable search_placeholder placeholder appearance display dropdown option
    on_update=Callback::new(move|code:String|{if let Some(country)=COUNTRIES.iter().find(|country|country.iso2==code)&&let Some(callback)=on_update{callback.run(country.clone());}})/ >}
}

/// Exact source currency selector, including non-fiat/unflagged choices and USD flag override.
#[component]
pub fn SourceCurrencySelect(
    /// Native trigger prefix.
    #[prop(into)]
    id: String,
    /// Hidden successful form key.
    #[prop(into)]
    name: String,
    /// Mount-only original currency code.
    #[prop(optional_no_strip)]
    initial: Option<String>,
    /// Source search toggle.
    #[prop(optional)]
    searchable: bool,
    /// Authored search placeholder.
    #[prop(default="Search...".into(),into)]
    search_placeholder: String,
    /// Empty display text.
    #[prop(default="Select a currency".into(),into)]
    placeholder: String,
    /// Full original currency record callback, without the display flag enrichment.
    #[prop(optional)]
    on_update: Option<Callback<CurrencyData>>,
    /// Root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
    /// Display slot.
    #[prop(optional)]
    display: SourceAppearance,
    /// Dropdown slot.
    #[prop(optional)]
    dropdown: SourceAppearance,
    /// Option slot.
    #[prop(optional)]
    option: SourceAppearance,
) -> impl IntoView {
    let choices = CURRENCIES
        .iter()
        .map(|currency| {
            let flag = COUNTRIES
                .iter()
                .find(|country| {
                    if currency.code == "USD" {
                        country.iso3 == "USA"
                    } else {
                        country.cur == currency.code
                    }
                })
                .map(|country| country.flag)
                .unwrap_or("");
            SourceChoice {
                value: currency.code.into(),
                label: format!("{flag} {}", currency.name),
                search: vec![currency.name, currency.code, currency.plural]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            }
        })
        .collect();
    view! {<SourceSelect id name initial choices searchable search_placeholder placeholder appearance display dropdown option
    on_update=Callback::new(move|code:String|{if let Some(currency)=CURRENCIES.iter().find(|currency|currency.code==code)&&let Some(callback)=on_update{callback.run(currency.clone());}})/ >}
}
