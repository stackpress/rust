//! The same Profile component source at every render target.

use leptos::prelude::*;
use stackpress_example_profile_ui_contracts::{SearchInput, SearchPage, SearchReply};
use stackpress_framework_ui::leptos_meta::{Meta, Title};

/// A local transport completes with a validated public reply or safe error.
pub type SearchCompletion = std::sync::Arc<dyn Fn(Result<SearchReply, String>) + Send + Sync>;
/// WASM-local adapter; it need not be Send and is never stored by a native host.
pub type SearchTransport = UnsyncCallback<(SearchInput, SearchCompletion)>;

/// Signal handles for one component; owning cleanup remains with the Leptos Owner.
#[derive(Clone, Copy)]
struct SearchController {
    /// Current editable prefix.
    id: RwSignal<String>,
    /// Last useful public rows, retained while a new search runs.
    rows: RwSignal<Vec<(String, String)>>,
    /// Returned-row count, including zero.
    total: RwSignal<u64>,
    /// Safe user-facing failure, never native diagnostics.
    error: RwSignal<String>,
    /// Whether the newest submission is still in flight.
    pending: RwSignal<bool>,
    /// Only this generation may publish a completion.
    generation: RwSignal<u64>,
}

impl SearchController {
    /// Consume a server seed once, without starting a transport request.
    fn new(seed: SearchPage) -> Self {
        let id = RwSignal::new(seed.input.id);
        let rows = RwSignal::new(
            seed.reply
                .rows
                .into_iter()
                .map(|row| (row.id, row.name))
                .collect::<Vec<_>>(),
        );
        let total = RwSignal::new(seed.reply.total);
        let error = RwSignal::new(String::new());
        let pending = RwSignal::new(false);
        let generation = RwSignal::new(0_u64);
        Self {
            id,
            rows,
            total,
            error,
            pending,
            generation,
        }
    }

    /// Start one validated submission and suppress stale or disposed completions.
    fn submit(self, transport: SearchTransport) {
        let input = match SearchInput::new(self.id.get_untracked()) {
            Ok(input) => input,
            Err(message) => {
                self.error.set(message.into());
                return;
            }
        };
        let current = self.generation.get_untracked().saturating_add(1);
        self.generation.set(current);
        self.pending.set(true);
        self.error.set(String::new());
        let done: SearchCompletion =
            std::sync::Arc::new(move |reply: Result<SearchReply, String>| {
                // Disposal or a newer submission makes late completion harmless.
                if self.generation.try_get_untracked() != Some(current) {
                    return;
                }
                self.pending.set(false);
                match reply
                    .and_then(|reply| reply.validate().map(|()| reply).map_err(str::to_owned))
                {
                    Ok(reply) => {
                        self.total.set(reply.total);
                        self.rows.set(
                            reply
                                .rows
                                .into_iter()
                                .map(|row| (row.id, row.name))
                                .collect(),
                        );
                    }
                    Err(message) => self.error.set(message),
                }
            });
        transport.run((input, done));
    }
}

/// Component input is an owned public seed, never a live invocation or host handle.
#[component]
pub fn ProfileSearch(
    /// Public initial data, consumed without an initial fetch.
    seed: SearchPage,
    /// Explicit browser/IPC adapter, absent for the synchronous server render.
    #[prop(optional)]
    transport: Option<SearchTransport>,
) -> impl IntoView {
    let action = format!(
        "{}/ui/profiles",
        stackpress_framework_ui::routing::use_route_context()
            .base
            .as_str()
    );
    let controller = SearchController::new(seed);
    let SearchController {
        id,
        rows,
        total,
        error,
        pending,
        ..
    } = controller;
    let submit = move |event: leptos::ev::SubmitEvent| {
        // Without an interactive adapter the normal GET form remains usable.
        let Some(transport) = transport else {
            return;
        };
        event.prevent_default();
        controller.submit(transport);
    };
    // No Effect/Resource runs an initial search: SSR and hydration consume identical data.
    view! {
        <Title text="Profiles"/>
        <Meta name="description" content="Search public demo profiles"/>
        <main>
            <h1>"Profiles"</h1>
            <form method="get" action=action on:submit=submit>
                <label for="profile-id">"Profile ID prefix"</label>
                <input id="profile-id" name="id" value=move || id.get()
                    on:input=move |event| id.set(event_target_value(&event))
                    aria-describedby="profile-error"/>
                <button type="submit">"Search"</button>
            </form>
            <p role="status" aria-live="polite">{move || if pending.get() { "Searching…".into() } else { format!("{} profiles", total.get()) }}</p>
            <p id="profile-error" role="alert">{move || error.get()}</p>
            <ul>{move || rows.get().into_iter().map(|(id, name)| view! { <li data-profile-id=id>{name}</li> }).collect_view()}</ul>
        </main>
    }
}

#[cfg(test)]
mod tests;
