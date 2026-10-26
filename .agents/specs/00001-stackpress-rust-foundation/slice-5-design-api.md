# Slice 5 API and developer experience sketch

Proposed usage for [checkpoint review](slice-5-design-decisions.md); new framework-view/UI/example names below are **not implemented or compile-verified APIs**. The [assessment](slice-5-design-assessment.md) identifies the small real APIs that were compiled. M0 must compile the selected author experience with actual production types before the signatures are frozen.

## A page in framework-view

Page definitions own data and public projection; rendering remains an adapter. Proposed data types:

```rust
pub enum PageLoad<T> {
    Ready(T),
    Abort,
}

pub struct PageSnapshot<T> {
    pub revision: u32,
    pub build: String,
    pub page: PageId,
    pub location: PublicLocation,
    pub locale: String,
    pub theme: InitialTheme,
    pub data: T,
    pub status: PublicStatus,
}

pub trait PageRenderer<T>: Send + Sync {
    fn render(&self, page: std::sync::Arc<PageSnapshot<T>>)
        -> Result<RenderedPage, ViewError>;
}
```

`PageId` and public location are validated bounded values, not source module paths. `RenderedPage` contains one complete HTML document and trusted asset metadata; it cannot claim native output authority. `ViewError` distinguishes invalid input/snapshot, incompatible revision, render limits and renderer failures, preserving a diagnostic cause without automatically exposing it. PageLoad::Abort preserves current response/stopped state and skips rendering. A Result error enters G12 through the wrapper's normal handler Result.

Illustrative app page loader (names/types from the proposed app-owned operation adapter):

```rust
use stackpress::framework_view::PageLoad;
use stackpress::lib::{Invocation, Request, Response, Result};
use profile_ui_contracts::{SearchInput, SearchPage};
use profile_ui_host::{CallCancellation, SsrProfileClient};

/// Load public search data through the current invocation's capability boundary.
async fn load_profiles(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<PageLoad<SearchPage>> {
    let input: SearchInput = SsrProfileClient::input(req)?;
    // The borrowed adapter delegates within this root and preserves child Abort.
    let outcome = SsrProfileClient::new(req, res, inv)
        .profile_search(input, CallCancellation::for_page())
        .await?;
    Ok(outcome.into_page_load())
}
```

The adapter maps the approved input into the existing operation's `id`, calls `inv.delegate("profile-search", req, res)`, preserves Abort/errors, and decodes only approved result/total/public-error fields. No raw Request/Response cloning or independent business search implementation. The public prefix input is shared with browser/IPC validation; the native operation remains authoritative.

## Register UI with ServerBuilder

Proposed extension methods live in framework-view's optional server module, so server does not depend on view/UI. A simple application still supplies ordinary registration functions:

```rust
use stackpress::framework_ui::render::LeptosRenderer;
use stackpress::framework_view::{PageSpec, ServerBuilderViewExt};
use stackpress::lib::Result;
use stackpress::server::ServerBuilder;

fn pages(app: &mut ServerBuilder) -> Result<()> {
    app.page(
        PageSpec::get("/ui/profiles", "profiles", load_profiles)?,
        LeptosRenderer::new(profiles_view),
    )?;
    app.page(
        PageSpec::get("/ui/profiles/:id", "profile-detail", load_profile_detail)?,
        LeptosRenderer::new(profile_detail_view),
    )?;
    Ok(())
}

async fn application() -> stackpress::server::BuildResult<stackpress::server::Server> {
    let mut app = ServerBuilder::new();
    profile_domain::generated::register(&mut app)?;
    app.plugin("profile-pages", pages)?;
    app.plugin("profile-ui-api", profile_http::register)?;
    app.build().await
}
```

**Returns:** registration errors for invalid IDs/route patterns, duplicate page identities, incompatible renderer/snapshot policy or existing builder conflicts; app.build publishes only a complete immutable host. Route listeners retain normal priority/FIFO/all-matching semantics. Page identity duplicates are view-registry errors; duplicate native listeners remain allowed by server. `page_with` may expose RouteSpec priority/label when the first real caller needs it; do not add a second router.

`PageSpec::get` accepts one named loader with a Send future borrowing req/res/Invocation, and a renderer for its data type. The extension's private Handler adapter bridges that signature. Type erasure stores only owned registration functions/config and Send+Sync render closures; no live reactive Owner is stored in a host service.

## Shared Leptos component

The same component and typed operation/controller source are compiled for SSR, hydrated web and packaged CSR. This first design deliberately preloads SSR data in the page loader; it does not promise a component can await a borrowed host invocation from a `'static` Leptos Resource.

```rust
use leptos::prelude::*;
use leptos_meta::Title;
use stackpress::framework_ui::{Alert, Button, FieldControl, Input, LayoutBlank};
use profile_ui_client::{use_profile_search, SearchSeed};
use profile_ui_contracts::SearchInput;

#[component]
pub fn ProfileSearch(seed: SearchSeed) -> impl IntoView {
    let search = use_profile_search(seed);
    let id = RwSignal::new(search.initial_id());
    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        // dispatch owns cancellation, loading/error state and late-result suppression.
        search.dispatch(SearchInput { id: id.get() });
    };
    view! {
        <Title text="Profiles"/>
        <LayoutBlank>
            <h1>"Profiles"</h1>
            <form method="get" action="/ui/profiles" on:submit=submit>
                <FieldControl label="Profile ID prefix" for_id="profile-id">
                    <Input id="profile-id" name="id" value=id/>
                </FieldControl>
                <Button button_type="submit" disabled=move || search.pending()>
                    "Search"
                </Button>
            </form>
            <p role="status" aria-live="polite">{move || search.status_text()}</p>
            <Alert message=move || search.public_error()/>
            // results_view renders escaped names and stable-ID detail links.
            {move || search.results_view()}
        </LayoutBlank>
    }
}
```

`use_profile_search` is initially app-owned, not a universal generated operation framework. It owns a target-local concrete client from context, immutable shared reply handles, generation/cancel state and cleanup. It validates the initial seed and displays visible ready/empty/invalid/error/loading states. Signal access uses normal Leptos ownership. Components accept explicit HTML semantics/attributes and children; React-style callable-class/style objects are not copied.

## Identical typed call, explicit adapters

The controller's action is the same at both interactive targets:

```rust
let reply = client.profile_search(SearchInput { id }, cancellation).await?;
```

On SSR, the same `profile_search(input, cancellation)` call uses borrowed page/lifecycle values; its typed completed/aborted/cancelled outcome converts to PageLoad and the seed. Hydration restores that seed without calling again. A user submission invokes `profile_search` through HTTP; Tauri CSR invokes the same method name/input/output through IPC. The domain implementation is the existing generated `profile_search` function in both native hosts.

| Build | Client implementation | Lifetime/output contract |
| --- | --- | --- |
| Native page loader | `SsrProfileClient<'a>` wraps current borrowed req/res/Invocation | Send future bounded by invocation lifetime; PageLoad preserves Abort. Own/serialize the result before rendering. |
| Browser hydrate | `BrowserProfileClient` uses fixed same-origin endpoint | Local WASM future, AbortController, bounded decoder and revision check; never includes server or Tauri. |
| Tauri CSR | `IpcProfileClient` invokes permitted fixed native command | Local WASM promise/future, scoped call ID and explicit cancel; native admission/lifecycle errors map into typed PublicError. |

These clients share contracts and observable controller behavior, not one forced trait-object future type. If M0 proves a small common trait improves substitution without unsafe or Send distortion, use it; otherwise concrete target adapters are simpler. Runtime transport selection is explicit app construction, never an environment guess or silent remote fallback.

The `SearchReply` codec must handle status, error/errors, results and zero totals without JS truthiness. Required fields/revision mismatches are errors. A transport failure differs from a valid host error envelope and a cancelled request. HostReply/HandleReport, stack traces, headers and cookies are not blindly exposed to the component.

## Minimum reusable framework surfaces

- View: checked PageId/PageSpec/PageLoad/PageSnapshot/PublicStatus/RenderedPage/ViewError and the optional ServerBuilder extension.
- UI: LayoutBlank, a small responsive LayoutPanel without account assumptions; Input/FieldControl/Button, Alert/Loader, Link/List; scoped notification state; phrase/node translation context; shared route/Head integration. SSR/hydrate/CSR all use the same markup rules.
- App: Profile DTO codec, search state/controller, detail presentation, typed fixed HTTP/IPC adapters and UI route catalog. Promote a generic operation-client abstraction only after two real callers justify its contract.
- Native: a small app-owned Tauri bootstrap/commands/capabilities, not a new mandatory framework-desktop crate or embedded HTTP host. Platform integrations stay in adapters.

Documentation after implementation must compile the real examples and include errors/cancellation/feature limits. The sketch is an implementation target to challenge at M0, not a published API reference.
