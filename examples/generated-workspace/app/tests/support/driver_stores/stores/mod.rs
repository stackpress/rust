//! Actual generated three-model sequences over the same SQLite/PostgreSQL/MySQL contract.
use super::super::TestResult;
use generated_workspace_app::generated::{
    contracts::{
        profile::CreateProfile,
        project::CreateProject,
        public::{PatchProfile, PatchProject, PatchTask, ProfileKey, ProjectKey, TaskKey},
        task::CreateTask,
    },
    stores::{
        profile::{self, ProfileStore, Profiles},
        project::{self, ProjectStore, Projects},
        task::{self, TaskStore, Tasks},
    },
};
use stackpress_framework_schema::{Predicate, ValueContext};
use stackpress_framework_sql::{DataErrorKind, DataResult, DataSession};
use stackpress_sql::{Database, DialectId, Engine};
use std::sync::Arc;

mod fixture;
mod observe;
mod preflight;
mod reads;
mod writes;

/// Execute generated stores without replacing the real backend or its capability policy.
pub(super) async fn check(database: Arc<dyn Database>) -> TestResult<()> {
    preflight::check().await?;
    let (engine, calls) = observe::engine(database.clone());
    println!("generated stores {:?}: starting", engine.dialect());
    fixture::install(&engine, &calls).await?;
    fixture::seed(&engine).await?;
    println!("generated stores {:?}: seeded", engine.dialect());
    reads::check(database, &engine, &calls).await?;
    println!("generated stores {:?}: reads complete", engine.dialect());
    writes::check(&engine, &calls).await?;
    // Report refused MySQL mutations separately from the successful RETURNING sequences.
    let outcome = if engine.dialect() == DialectId::Mysql {
        "mutation/conflict/batch/active refusals passed"
    } else {
        "CRUD/conflicts/batches/active passed"
    };
    println!("generated stores {:?}: {outcome}", engine.dialect());
    // The enclosing common suite owns this pool's eventual close; this Engine shares that owner.
    Ok(())
}
