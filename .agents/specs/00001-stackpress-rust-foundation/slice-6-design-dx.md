# Slice 6 developer walkthrough

Companion to [the decision router](slice-6-design-decisions.md). Everything new below is a proposed API/output shape, not implemented or compile-verified Rust. M0 must compile this experience using real generated source and existing ServerBuilder/SQL contracts.

## What the developer writes

Use a new `examples/generated-workspace/` so the existing generated-profile, sql-profile and profile-ui proofs remain intact. The app owns its Cargo manifests, tool, database plugin, policy and handwritten extensions.

```idea
plugin "framework-schema" { }
plugin "framework-sql" { }

enum ProjectState {
  OPEN "open"
  CLOSED "closed"
}

type Address {
  city String @is.ne @field.input @view.text
  country String @field.input @view.text
}

model Profile @labels("Profile" "Profiles") @query("id" "name") {
  id String @id
  name String @is.ne @searchable @sortable @field.input @list.text @view.text
  active Boolean @active @default(true)
  address Address?
  projects Project[]
}

model Project @labels("Project" "Projects") @query("id" "title" "ownerId") {
  id String @id
  title String @is.ne @searchable @sortable @field.input @list.text @view.text
  ownerId String
  owner Profile @relation({ local "ownerId" foreign "id" name "projects" })
  state ProjectState @default("open") @filter.select @list.text
  tags String[] @default([])
  tasks Task[]
}

model Task @query("id" "title" "projectId" "done") {
  id String @id
  projectId String
  project Project @relation({ local "projectId" foreign "id" name "tasks" })
  title String @is.ne @field.input @list.text
  done Boolean @default(false) @filter.checkbox @list.yesno
}
```

The reverse collections are virtual relations, not array-valued SQL columns. `address` and `tags` are owned JSON storage; `ownerId` and `projectId` are real columns with foreign keys. IDs are supplied explicitly in this example. No enum-member reference syntax or new Idea grammar is needed.

## Generate, inspect, compile

```sh
cargo run -p generated-workspace-tool -- emit generate
cargo run -p generated-workspace-tool -- emit generate --check
cargo check -p generated-workspace-app --locked
```

The independent tool registers schema and SQL function transforms. It never imports the final app. Generated output is in one dedicated marked root, with schema-owned `contracts/` and `metadata/`, SQL-owned `stores/` and `operations/`, and one aggregate owner. Inspectable contracts can be consumed by a separate browser-safe package using a source module path; a separately published contracts crate is not necessary for this repository proof.

```text
app/src/generated/
  mod.rs                     # transformer aggregate: module declarations only
  contracts/{profile,project,task,address,project_state}.rs
  metadata/schema.rs         # safe descriptor and schema/generator revisions
  stores/{profile,project,task,schema}.rs
  operations/{profile,project,task,register}.rs
```

Each partition has the aggregate-generated `mod.rs`. Models do not emit identically owned shared files. SQL transforms do not patch schema files. Handwritten adapters/projections live outside this root. Rename/removal regenerates every aggregate and prunes only promised owned outputs; historical revisions live elsewhere.

Representative generated Rust, with imports shortened for illustration:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub title: String,
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    pub state: ProjectState,
    pub tags: Vec<String>,
}

pub struct ProjectStore;
impl ProjectStore {
    pub fn table() -> stackpress_sql::TableDefinition { /* generated definition */ }
    pub fn select(query: &ProjectQuery) -> DataResult<ProjectReadPlan> { /* typed AST */ }
    pub fn decode(row: stackpress_sql::Row) -> DataResult<Project> { /* checked fields */ }
}

pub struct ProjectWithOwner {
    pub project: Project,
    pub owner: Profile,
}
```

The real generated source includes Rustdoc and explanatory block/branch comments. Full-row DTOs contain stored values; relation result types are finite and generated for declared include shapes, so reciprocal relations do not form infinitely sized Rust structs. Create and patch inputs are separate types with explicit missing/null states; omitted defaulted fields are not fabricated by a DTO's `Default` implementation.

## Call stores and operations directly

```rust
use stackpress_framework_sql::DataSession;
use generated::{contracts::project::CreateProject, stores::project::Projects};

async fn create_project(engine: &stackpress_sql::Engine) -> DataResult<Project> {
    let mut db = DataSession::pooled(engine);
    Projects::create(&mut db, CreateProject::new("p1", "Launch", "u1"), &values()).await
}

async fn create_project_and_task(engine: &stackpress_sql::Engine) -> DataResult<()> {
    let mut tx = engine.begin().await?;
    {
        let mut db = DataSession::transaction(&mut tx);
        Projects::create(&mut db, CreateProject::new("p1", "Launch", "u1"), &values()).await?;
        tokio::task::yield_now().await;
        Tasks::create(&mut db, CreateTask::new("t1", "p1", "Draft"), &values()).await?;
    }
    tx.commit().await?;
    Ok(())
}
```

`values()` supplies the app's explicit default/clock provider. `DataResult` retains typed validation/SQL causes. The caller owns commit/rollback; the illustrated `?` on failure drops the guard and initiates rollback. Production examples also show awaited rollback with a secondary cleanup error retained. No event call inherits `db` or `tx` automatically.

```rust
let query = ProjectQuery::new()
    .eq(ProjectField::State, ProjectState::Open)
    .sort(ProjectSort::Title, Order::Asc)
    .page(0, 20);
let rows: Page<Project> = Projects::search(&mut db, &query).await?;
let rows: Page<ProjectWithOwner> = Projects::with_owner(&mut db, &query).await?;
// Inspect before executing; the plan includes base/count/relation statements.
let plan = ProjectStore::select(&query)?;
let inspection = plan.render(engine.dialect())?;
```

## Register selected generated operations

```rust
fn data(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()> {
    // The app registers its Engine/default provider separately during startup.
    generated::operations::register::projects(
        app,
        ProjectExposure::new(project_policy, project_public)
            .allow(&[ProjectOperation::Search, ProjectOperation::Get]),
    )?;
    Ok(())
}
```

Generated named `project_search`/`project_get` handlers validate request data, use `Invocation::service`, call stores, apply the explicit app projection, and set the existing response shape. `project_policy` checks the caller at this boundary; there is no built-in auth system in Slice 6. A handwritten HTTP/page adapter delegates `project-search` inside its existing invocation. The same handler can be called through CLI, in-process, buffered HTTP or a permitted native entry. No routes, maintenance events or raw SQL endpoint are installed by the model alone.

## Change the schema deliberately

Add `summary String?` to Project, regenerate and run `--check`. Rust now exposes `Option<String>`; the database has not changed. Use the independent tool to record/review/apply history:

```sh
cargo run -p generated-workspace-tool -- emit revision --name add-project-summary
cargo run -p generated-workspace-tool -- emit migration-plan --dialect sqlite --to 0002
cargo run -p generated-workspace-tool --features database-tools -- emit db-upgrade --plan <hash>
```

The first revision/baseline and explicit `db-install` are created during initial setup. A nullable addition plans an ADD COLUMN where supported. Rename `title` to `heading` later: the plan reports remove/add and likely matches; the developer supplies `--rename Project.title=heading`, reviews the resulting data-preserving mapping and new plan hash, then applies it. A changed field type/nullability may require SQLite reconstruction. A required field without a valid backfill is blocked even when data loss is allowed.

An unrelated database schema, edited applied artifact, stale plan hash or missing predecessor causes refusal. Going back uses the stored approved reverse plan only when its data-preservation conditions hold. Purge, uninstall and populate are explicit tool commands with their own reviewed plans; application startup does none of these.

The generated `stores/schema.rs` also exposes all three checked table definitions for applications that call migration libraries directly:

```rust
use stackpress_framework_sql::migrations::{MigrationPlanner, MigrationRunner};

async fn install(maintenance: &mut SqliteMaintenance, approved: PlanHash) -> DataResult<()> {
    let target = generated::stores::schema::definition()?;
    let plan = MigrationPlanner::baseline(&target, DialectId::Sqlite)?;
    MigrationRunner::apply(maintenance, &plan, approved).await?;
    Ok(())
}
```

The app-owned tool uses the same planner from normalized snapshots, so it does not import this generated module or depend on the app to bootstrap. `approved` must match the separately reviewed plan hash. Direct library calls and tool commands apply the same ledger/drift/destructive checks; exposing a table definition never applies it automatically.
