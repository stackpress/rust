//! Explicit fresh SQLite fixture setup and exact manifest attachment; never called by serving.
use generated_admin_web::{database, host};
use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Result};
use stackpress_server::http::{HttpAdapter, HttpOutput};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

/// Exact owned fixture identity; no arbitrary path can arrive from HTTP.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    /// One setup-created basename; no HTTP field may replace it.
    database: String,
}
/// Refuse symlink/nonregular or oversized configuration files before decoding.
/// # Errors
/// Returns missing/invalid manifest, identity or regular-file errors without creating anything.
pub fn attach(directory: &Path) -> Result<PathBuf> {
    regular_tree(directory)?;
    let manifest = directory.join("state.json");
    let metadata = fs::symlink_metadata(&manifest).map_err(Error::from_source)?;
    if !metadata.is_file() || metadata.len() > 1024 {
        return Err(Error::invalid("invalid review state manifest"));
    }
    let state: Manifest = serde_json::from_slice(&fs::read(manifest).map_err(Error::from_source)?)
        .map_err(Error::from_source)?;
    // Only a basename created by setup may be attached; no parent or absolute path.
    if !state.database.starts_with("review-")
        || !state.database.ends_with(".sqlite")
        || state.database.contains(['/', '\\'])
    {
        return Err(Error::invalid("invalid review database identity"));
    }
    let path = directory.join(state.database);
    if !fs::symlink_metadata(&path)
        .map_err(Error::from_source)?
        .is_file()
    {
        return Err(Error::invalid("review database must be regular"));
    }
    Ok(path)
}
/// Create a previously absent fixture under the explicit directory and seed real generated writes.
/// # Errors
/// Refuses existing setup, symlinks and all generated/database failures; never deletes caller data.
pub async fn create(directory: &Path) -> Result<PathBuf> {
    regular_tree(directory)?;
    fs::create_dir_all(directory).map_err(Error::from_source)?;
    if !fs::symlink_metadata(directory)
        .map_err(Error::from_source)?
        .is_dir()
    {
        return Err(Error::invalid("setup directory must be real"));
    }
    // Claim the manifest first: concurrent setups cannot replace the selected fixture.
    let mut manifest = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("state.json"))
        .map_err(Error::from_source)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(Error::from_source)?
        .as_nanos();
    let name = format!("review-{}-{stamp}.sqlite", std::process::id());
    let path = directory.join(&name);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(Error::from_source)?;
    let engine = database::open(&path, false)
        .await
        .map_err(Error::from_source)?;
    // Close the pool even when a generated seed fails; keep partial owned files inspectable.
    let result = seed(engine.clone()).await;
    let closed = engine.close().await.map_err(Error::from_source);
    result?;
    closed?;
    manifest
        .write_all(&serde_json::to_vec(&Manifest { database: name }).map_err(Error::from_source)?)
        .map_err(Error::from_source)?;
    manifest.sync_all().map_err(Error::from_source)?;
    Ok(path)
}
/// Install actual tables and create parent/child rows through their compiled generated handlers.
async fn seed(engine: Arc<stackpress_sql::Engine>) -> Result<()> {
    database::initialize(&engine).await?;
    let app = host::build(
        engine,
        host::Config {
            mount: "/".into(),
            csrf: "review-setup".into(),
            csr: false,
            transfer_limits: Default::default(),
        },
        Arc::new(host::FixturePolicy),
    )
    .await?;
    for index in 0..25 {
        let name = if index == 0 {
            "=1+1".to_owned()
        } else {
            format!("Review profile {index:02}")
        };
        insert(
            &app,
            "profile",
            serde_json::json!({"id":format!("p{index}"), "name":name}),
        )
        .await?;
    }
    insert(
        &app,
        "project",
        serde_json::json!({"id":"j1", "ownerId":"p1", "title":"Review project"}),
    )
    .await?;
    for index in 0..25 {
        insert(&app, "task", serde_json::json!({"id":format!("t{index}"), "projectId":"j1", "title":format!("Review task {index:02}"), "done":false})).await?;
    }
    Ok(())
}
/// Exercise the same public operation admission used by the existing finite admin proof.
async fn insert(
    app: &stackpress_server::Server,
    module: &str,
    input: serde_json::Value,
) -> Result<()> {
    let request = http::Request::builder()
        .method("POST")
        .uri(format!("/api/{module}/create"))
        .header("content-type", "application/json")
        .header("origin", "https://admin.invalid")
        .header("x-admin-actor", "editor")
        .header("x-admin-csrf", "review-setup")
        .body(serde_json::json!({"input":input}).to_string().into_bytes())
        .map_err(Error::from_source)?;
    let HttpOutput::Response(reply) = HttpAdapter::new(Default::default())?
        .handle(app, request)
        .await?
        .output
    else {
        return Err(Error::invalid("setup output unavailable"));
    };
    if !reply.status().is_success() {
        return Err(Error::invalid(format!(
            "setup generated create failed: {}",
            String::from_utf8_lossy(reply.body())
        )));
    }
    Ok(())
}

/// Refuse traversal and existing symlink components before any directory/file creation.
/// The caller owns this tree exclusively during setup/startup; portable metadata/open is
/// not an atomic defense against hostile concurrent filesystem mutation.
fn regular_tree(path: &Path) -> Result<()> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(Error::invalid("parent paths are unsupported"));
        }
        prefix.push(component);
        match fs::symlink_metadata(&prefix) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(Error::invalid("symlink setup paths are unsupported"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(Error::from_source(error)),
        }
    }
    Ok(())
}
