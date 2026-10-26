# Slice 8 proposed author experience

Sketches for [owner review](slice-8-design-decisions.md). New symbols, metadata definitions and generated names below are **not implemented or compile-verified APIs**. Existing `FieldControl`, `Input`, `Button`, language signals and named registration remain the starting point. M0 must compile the selected signatures before freezing them.

## Compose a table, overlay and form

A plain table remains a native table, not an ARIA grid. Supply explicit column/row slots and stable identity. Sort/paging are controlled state; the table never invents database queries.

```rust
use leptos::prelude::*;
use stackpress::framework_ui::{
    Button, ButtonType, Dialog, FieldControl, Input, Table,
};

#[component]
fn ProfileEditor() -> impl IntoView {
    let open = RwSignal::new(false);
    let name = RwSignal::new("Ada Lovelace".to_owned());
    let error = RwSignal::new(String::new());
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        // An app-owned typed controller validates, calls its fixed adapter,
        // retains the draft on failure, and closes only after confirmed success.
        submit_profile_edit(name.get(), error, open);
    };
    view! {
        <Table caption="Profiles">
            <thead><tr><th scope="col">"Name"</th><th scope="col">"Actions"</th></tr></thead>
            <tbody><tr data-row-id="profile-1">
                <th scope="row">"Ada Lovelace"</th>
                <td><button type="button" on:click=move |_| open.set(true)>
                    "Edit Ada Lovelace"
                </button></td>
            </tr></tbody>
        </Table>
        <Dialog open title="Edit profile" close_on_backdrop=false>
            <form on:submit=save>
                <FieldControl for_id="profile-name" label="Name" error>
                    <Input id="profile-name" name="name" value=name
                        described_by="profile-name-help profile-name-error"
                        invalid=move || !error.get().is_empty()/>
                </FieldControl>
                <Button button_type=ButtonType::Submit>"Save"</Button>
                <button type="button" on:click=move |_| open.set(false)>"Cancel"</button>
            </form>
        </Dialog>
    }
}
```

`submit_profile_edit` is illustrative application code, not a hidden framework operation. The reusable Dialog owns native modal/focus/escape/cleanup mechanics; the app owns dirty-draft confirmation and whether closing is allowed while a write outcome is unknown. Initial dialog state is closed on SSR and hydration. Initially open routes need their own hydrate proof before support is claimed.

Proposed `Table` props are caption, labelled scroll region, explicit children and busy state. `Dialog` props are controlled `RwSignal<bool>`, title/description, initial-focus target and explicit dismissal policy. IDs must be deterministic and unique under repeated component instances. Open state belongs to one root; nested dialogs restore the previous modal's focus before the original trigger.

Use sibling helpers for TableHead/Row/Cell when they improve repetitive composition; do not reproduce React child scanning, callable class/style objects or every source helper as a Rust ABI. New `Select`, Checkbox/Radio/Switch/Textarea and collection fields follow the same label/error/controlled-value pattern. A form helper supplies an error summary and typed dirty/submitting/outcome state, not a second domain validator.

## Generate a schema-driven admin screen

Proposed Idea authoring after Slice 6 registers the metadata below. Parser syntax is illustrative of existing attributes; semantic definitions and exact Rust generated names await the cross-slice handshake.

```text
plugin "data" { }
plugin "ui" { }

model Profile {
  id String @id
  name String @field.input({ placeholder: "Full name" })
    @filter.input @list.text @view.text
  active Boolean @field.switch @filter.checkbox @list.yesno @view.yesno
}
```

The developer registers generation functions in schema/data/UI order in an app-owned tool. UI consumes normalized metadata; it never parses schema source again. Fixed dictionary props win collisions. Missing `span` or any other role is omitted, not guessed from the Rust type. Explicit unsupported roles yield a source diagnostic.

Proposed generation invocation uses the existing command convention:

```sh
cargo run -p sample-codegen -- emit generate
cargo run -p sample-codegen -- emit generate --check true
```

Proposed inspectable outputs, each in an owned partition:

```text
contracts/profile.rs             # frontend-safe public DTOs from Slice 6
server/profile.rs                # stores/events from Slice 6, native only
ui/profile/fields.rs             # checked role-specific Leptos wrappers
ui/profile/pages.rs              # search/detail/forms and layout composition
admin/profile/routes.rs          # fixed routes and typed operation adapters
mod.rs                          # one aggregate owner, deterministic order
```

Application startup still selects ordinary functions:

```rust
fn admin(app: &mut ServerBuilder) -> Result<()> {
    // Missing host policy is a startup error for selected protected routes.
    sample_generated::admin::register(
        app,
        AdminOptions::at("/admin")?.with_policy(sample_admin_policy),
    )
}

async fn sample_admin_policy(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
    action: AdminAction,
) -> Result<AdminAdmission> {
    // Authenticate the real caller, authorize action/row scope, and check CSRF
    // on website mutations. Denial occurs before the generated operation.
    sample_security::admit(req, res, inv, action).await
}
```

`AdminAdmission` is host-side policy output, never accepted from JSON or a client boolean. The browser receives only safe allowed actions and public field data. Private/write-only fields do not appear in table/detail/seed/export by default. Authoritative operations revalidate all mutations and relation scopes.

What the developer gets: `/admin/profiles` has filters, sortable columns and controlled pagination; row links open detail; create/edit forms use registered field components and show field/summary errors. Copy starts a create draft without copying identity. Remove/restore are explicit confirmed operations supported by model capabilities. Related lists use bounded paged relation operations. CSV import previews headers and row errors before explicit submission; export uses the same selected public field policy. No route performs writes on GET.

Every mutation has ready/draft, invalid, pending, confirmed, failed and outcome-unknown behavior. Buttons do not retry writes automatically. A navigation cancellation after submission reports uncertainty when the host cannot prove the write was prevented. Field errors preserve drafts, including false/zero/empty values. No generated endpoint accepts arbitrary event/model/table names.

A custom component uses an explicit registered Rust mapping and supported prop schema; regenerated code calls the mapping and leaves handwritten files untouched. This preserves the existing plugin customization model without string-based code evaluation.

## Switch locale and remember it

Proposed app configuration selects registry, aliases and cookie scope explicitly:

```rust
fn language(app: &mut ServerBuilder) -> Result<()> {
    LocalePlugin::new("en")
        .locale("en", "English", english_phrases())?
        .locale("fr", "Français", french_phrases())?
        .alias("en_US", "en")?
        .alias("fr_FR", "fr")?
        .route_prefixes(true)
        .cookie_path("/demo")?
        .register(app)
}
```

No global `Language::configure`. Registry setup validates locale/default/alias cycles, dictionary and cookie limits before host publication. Applications may register region-specific canonical locales separately; aliases are not guessed by truncating strings.

Inside the shared root, a typed preference controller coordinates persistence:

```rust
let preferences = use_preferences()?;
let change_locale = move |locale: LocaleId| {
    // Fixed HTTP action on the website; fixed native command in installed CSR.
    // Publish the acknowledged dictionary/settings together, preserving drafts.
    preferences.set_locale(locale);
};
```

The website action returns approved public settings and a scoped cookie. Links and Head use the acknowledged locale; a prefixed URL is the explicit locale authority on refresh. A successful picker action updates that prefix while preserving query and return context. Legacy unprefixed locations remain readable using the stored preference. Failed persistence shows “Language could not be saved” with retry as an explicit user action.

Installed startup loads preferences before rendering. The same picker uses `update_preferences`; restart must reproduce locale/theme. No browser cookie, filesystem path or Tauri API leaks into the shared component. Existing `LanguageContext::replace` remains the rendering primitive; persistence is an adapter responsibility.

## Render async only when selected

Proposed explicit registration keeps ordinary pages unchanged:

```rust
app.async_page(
    public_page,
    AsyncLeptosRenderer::buffered(component),
)?;
// Available only after Slice 7's stream contract and streaming M0 pass.
app.stream_page(public_page, AsyncLeptosRenderer::in_order(component))?;
```

Names and bounds are intentionally provisional. The first must prove actual Resource payload replay, Head lifetime and no initial refetch. The second must deliver early bytes through a pull-driven sink and dispose the root on disconnect. Neither is equivalent to changing a synchronous renderer's method name.
