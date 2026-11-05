//! Generated application client; native only. Regenerate, never edit.
use stackpress_framework_sql::source::{SourceModel, SourceStore};
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;
/// Complete native application metadata; never a browser/SSR seed.
pub fn describe() -> Vec<stackpress_framework_schema::Model> {
    vec![
        super::super::source::profile::describe(),
        super::super::source::application::describe(),
        super::super::source::session::describe(),
        super::super::source::auth::describe(),
    ]
}
/// Explicit DDL preparation; installing this module performs no database work.
pub fn definition() -> stackpress_sql::Result<Vec<stackpress_sql::TableDefinition>> {
    Ok(vec![
        table::<super::super::stores::profile::ProfileModel>()?,
        table::<super::super::stores::application::ApplicationModel>()?,
        table::<super::super::stores::session::SessionModel>()?,
        table::<super::super::stores::auth::AuthModel>()?,
    ])
}
/// Prepare a definition from the actual associated generated store, without an Engine.
fn table<M: SourceModel>() -> stackpress_sql::Result<stackpress_sql::TableDefinition> {
    M::Store::new(String::new(), 0).create().definition()
}

/// Queue only this model's generated source event inventory, preserving catalog order.
fn listen_profile(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_sql::source::register_with::<super::super::stores::profile::ProfileModel>(
        app,
        stackpress_framework_sql::source::ResponseValues::Native,
    )
}

#[cfg(feature = "ssr")]
/// Application-owned rendering/route adapter, registered at the aggregate's final stage.
fn admin_profile(app: &mut ServerBuilder) -> Result<()> {
    super::super::admin(app, "profile")
}

/// Queue only this model's generated source event inventory, preserving catalog order.
fn listen_application(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_sql::source::register_with::<
        super::super::stores::application::ApplicationModel,
    >(app, stackpress_framework_sql::source::ResponseValues::Json)
}

#[cfg(feature = "ssr")]
/// Application-owned rendering/route adapter, registered at the aggregate's final stage.
fn admin_application(app: &mut ServerBuilder) -> Result<()> {
    super::super::admin(app, "application")
}

/// Queue only this model's generated source event inventory, preserving catalog order.
fn listen_session(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_sql::source::register_with::<super::super::stores::session::SessionModel>(
        app,
        stackpress_framework_sql::source::ResponseValues::Json,
    )
}

#[cfg(feature = "ssr")]
/// Application-owned rendering/route adapter, registered at the aggregate's final stage.
fn admin_session(app: &mut ServerBuilder) -> Result<()> {
    super::super::admin(app, "session")
}

/// Queue only this model's generated source event inventory, preserving catalog order.
fn listen_auth(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_sql::source::register_with::<super::super::stores::auth::AuthModel>(
        app,
        stackpress_framework_sql::source::ResponseValues::Native,
    )
}

#[cfg(feature = "ssr")]
/// Application-owned rendering/route adapter, registered at the aggregate's final stage.
fn admin_auth(app: &mut ServerBuilder) -> Result<()> {
    super::super::admin(app, "auth")
}

/// Bind handwritten auth orchestration to this exact generated family.
fn session(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_session::composite::register_with::<super::super::stores::family::Models>(
        app,
    )
}

#[cfg(feature = "api")]
/// API processes consume this family's ordinary named model events.
fn api(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_api::register_with::<super::super::stores::family::Models>(app)
}

/// One static source catalog; callbacks run only when explicitly registered by the app.
pub static CLIENT: crate::framework::CompiledClient = crate::framework::CompiledClient {
    build: "g50-app-profile",
    describe,
    bindings: crate::framework::FrameworkBindings {
        session: { Some(session as crate::framework::Register) },
        api: {
            #[cfg(feature = "api")]
            {
                Some(api as crate::framework::Register)
            }
            #[cfg(not(feature = "api"))]
            {
                None
            }
        },
    },
    models: &[
        crate::framework::CompiledModel {
            name: "Profile",
            listen: Some(listen_profile),
            admin: {
                #[cfg(feature = "ssr")]
                {
                    Some(admin_profile as crate::framework::Register)
                }
                #[cfg(not(feature = "ssr"))]
                {
                    None
                }
            },
        },
        crate::framework::CompiledModel {
            name: "Application",
            listen: Some(listen_application),
            admin: {
                #[cfg(feature = "ssr")]
                {
                    Some(admin_application as crate::framework::Register)
                }
                #[cfg(not(feature = "ssr"))]
                {
                    None
                }
            },
        },
        crate::framework::CompiledModel {
            name: "Session",
            listen: Some(listen_session),
            admin: {
                #[cfg(feature = "ssr")]
                {
                    Some(admin_session as crate::framework::Register)
                }
                #[cfg(not(feature = "ssr"))]
                {
                    None
                }
            },
        },
        crate::framework::CompiledModel {
            name: "Auth",
            listen: Some(listen_auth),
            admin: {
                #[cfg(feature = "ssr")]
                {
                    Some(admin_auth as crate::framework::Register)
                }
                #[cfg(not(feature = "ssr"))]
                {
                    None
                }
            },
        },
    ],
};
