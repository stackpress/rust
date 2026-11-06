//! Original-source process traces and real SQLite reusable-event integration.
use serde_json::{Value as Json, json};
use stackpress_framework_csrf::{CsrfConfig, Service as Csrf};
use stackpress_framework_schema::source::SourceSchema;
use stackpress_framework_session::{
    Service,
    auth::{self, AuthConfig, Environment, Hooks, events, pages},
    columns::{self, Record, Value},
    storage::{AuthActions, ProfileActions, StoreQuery},
};
use stackpress_lib::{Flow, Invocation, Request, Response, Result, Timestamp, Value as Lib};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::sync::{Arc, Mutex};

/// Fixture clock follows the existing epoch-seconds Clock contract.
struct FixedClock;
impl stackpress_framework_session::Clock for FixedClock {
    fn now(&self) -> f64 {
        1_700_000_000.0
    }
}
/// A literal source object; rich Dates are imported separately at the event boundary.
fn record(value: Json) -> Record {
    serde_json::from_value(value).unwrap()
}
/// Import oracle Date columns explicitly, never interpreting arbitrary JSON marker objects.
fn native(value: &Json) -> Lib {
    match value {
        Json::Object(values) => Lib::Object(
            values
                .iter()
                .map(|(key, value)| {
                    let native = if ["consumed", "created", "updated"].contains(&key.as_str())
                        && value.is_string()
                    {
                        let source = auth_date(value.as_str().unwrap());
                        Lib::DateTime(Timestamp::new(source / 1000, 0).unwrap())
                    } else {
                        native(value)
                    };
                    (key.clone(), native)
                })
                .collect(),
        ),
        Json::Array(values) => Lib::Array(values.iter().map(native).collect()),
        _ => Lib::try_from(value.clone()).unwrap(),
    }
}
/// Oracle uses one frozen instant; source fixture has second precision.
fn auth_date(value: &str) -> i64 {
    let column = auth::AuthSchema::new(String::new(), 0);
    let date = column
        .column("consumed")
        .unwrap()
        .unserialize(Some(&Value::from(value)), "", false, 0)
        .unwrap()
        .unwrap();
    columns::as_date(&date).unwrap()
}
/// Explicit wire projection in tests uses the production adapter, including nested Dates.
fn json_value(value: Lib) -> Json {
    let mut res = Response::new();
    res.results_value(value).unwrap();
    events::json_result(&res).unwrap()
}

#[path = "auth_flows/events.rs"]
mod events_integration;
/// Raw nested event invocation uses no root/global/request gate.
async fn invoke(host: &Server, event: &str, input: Json) -> Response {
    let mut req = Request::builder().data(input).build().unwrap();
    let mut res = Response::new();
    host.emit(event, &mut req, &mut res).await.unwrap();
    res
}

#[path = "auth_flows/oracle.rs"]
mod oracle;
#[path = "auth_flows/registration.rs"]
mod registration;
pub(crate) use events_integration::concurrency_child;
