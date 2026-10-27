# Generate and execute a model

The repository's [generated workspace](../../../../examples/generated-workspace/README.md) contains a real app, an independent tool, and a pure browser-compatible contract consumer. Generate before compiling an absent app output:

```sh
cargo run -p generated-workspace-tool --locked -- emit generate
cargo test -p generated-workspace-app --test writes --test reads --locked
```

The tool normalizes the finalized Idea schema once per selected transform using the same app-owned definitions. Schema output owns `contracts` and `metadata`; SQL output owns `stores`. Importing output never installs a table, selects a route, authorizes a caller or opens a connection. Enable `framework-sql/codegen` only in the tool and `framework-sql/server` only in a host that selects generated handlers.

This named domain function composes two actual generated stores on one exact transaction. It belongs in the example app, where the generated contracts exist:

```rust
use generated_workspace_app::generated::{
    contracts::{profile::CreateProfile, project::CreateProject},
    stores::{profile::Profiles, project::Projects},
};
use stackpress_framework_sql::{DataResult, DataSession, finish};
use stackpress_sql::Engine;

async fn create_workspace(
    engine: &Engine,
    profile: CreateProfile,
    project: CreateProject,
) -> DataResult<()> {
    let mut transaction = engine.begin().await?;
    let outcome = async {
        let mut session = DataSession::transaction(&mut transaction);
        Profiles::create(&mut session, profile).await?;
        Projects::create(&mut session, project).await?;
        Ok(())
    }.await;
    finish(transaction, outcome).await
}
```

Install the generated checked table definitions explicitly before this call. The caller owns authorization and completion. `finish` commits once on success, awaits rollback on failure, retains a secondary cleanup error, and never retries an uncertain commit. The app's `atomic` integration test proves the equivalent success/failure workflow against a file-backed SQLite database.
