//! Source API data events bound to one application-generated model family.
//!
//! The lower source runtime owns event semantics. This module selects the original
//! twenty API event names and preserves the direct default-model callback paths.
use stackpress_framework_sql::source::{
    self, ApiModels, SourceEnvironment, SourceEventActions, SourceModel,
};
use stackpress_lib::{Flow, HandlerFuture, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, StartupContext};

pub use source::Event;

/// Select the shipped default family's API data events during Listen, without IO.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    register_with::<stackpress_framework_models::Models>(app)
}

/// Select the application's actual Application/Session actions at the original Listen phase.
/// The inherited Profile type is identical to the session family's Profile.
/// # Errors
/// Returns ordinary registration errors; missing Engine remains invocation-time pass-through.
pub fn register_with<M: ApiModels>(app: &mut ServerBuilder) -> Result<()>
where
    <M::Application as SourceModel>::Actions: SourceEventActions,
    <M::Session as SourceModel>::Actions: SourceEventActions,
{
    app.on_listen(listen::<M>)
}

/// Preserve the source event-major, Application-then-Session registration order.
async fn listen<M: ApiModels>(start: &mut StartupContext) -> Result<Flow>
where
    <M::Application as SourceModel>::Actions: SourceEventActions,
    <M::Session as SourceModel>::Actions: SourceEventActions,
{
    for event in [
        Event::Batch,
        Event::Create,
        Event::Detail,
        Event::Get,
        Event::Purge,
        Event::Remove,
        Event::Restore,
        Event::Search,
        Event::Update,
        Event::Upsert,
    ] {
        // The original API constructors supply empty seed/UTC, independently of
        // an application's ambient source-model codec environment.
        start.on_priority(
            &format!("application-{}", event.name()),
            0,
            source::action_with::<M::Application>(event, SourceEnvironment::default()),
        )?;
        start.on_priority(
            &format!("session-{}", event.name()),
            0,
            source::action_with::<M::Session>(event, SourceEnvironment::default()),
        )?;
    }
    Ok(Flow::Continue)
}

/// Direct callback over the generated default descriptor, composable with other handlers (G42).
pub fn action<M: SourceModel>(
    event: Event,
) -> impl for<'a> Fn(
    &'a mut Request,
    &'a mut Response,
    Invocation<'a>,
) -> HandlerFuture<'a, Result<Flow>>
+ Send
+ Sync
where
    M::Schema: stackpress_framework_schema::source::SourceSchema<
            Value = crate::models::Value,
            Record = crate::models::Record,
        >,
    M::Store: source::SourceStore<Error = source::ActionError>,
    M::Actions: SourceEventActions,
{
    source::action_with::<M>(event, SourceEnvironment::default())
}
