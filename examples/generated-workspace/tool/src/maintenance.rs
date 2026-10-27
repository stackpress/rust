//! App-owned operator events over existing CLI emit grammar; no stock migration binary.

use serde_json::{Value, json};
use stackpress_framework_sql::migrations::{
    DialectId, Literal, MigrationPlanner, RenameMap, RevisionId, RevisionOutcome, RevisionStore,
    SeedRow,
};
#[cfg(feature = "database-tools")]
use stackpress_framework_sql::migrations::{PlanHash, PlanOperation, RevisionSnapshot};
use stackpress_lib::{Error, ErrorKind, Flow, Invocation, Request, Response};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Await blocking history/compiler work; database cleanup is awaited on the async runtime.
pub(super) async fn event(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> stackpress_lib::Result<Flow> {
    let event = inv.event().to_owned();
    let data = req.data.to_json()?;
    let result = run(&event, data).await?;
    res.results(&result)?;
    Ok(Flow::Continue)
}
/// Execute one compiled app capability. Unknown options are rejected before filesystem/DB work.
pub async fn run(event: &str, data: Value) -> stackpress_lib::Result<Value> {
    let data = data
        .as_object()
        .ok_or_else(|| Error::invalid("operator options must be an object"))?
        .clone();
    let allowed: &[&str] = match event {
        "revision" => &["history", "input", "app", "name", "parent"],
        "history-check" => &["history"],
        "migration-plan" => &[
            "history",
            "from",
            "to",
            "dialect",
            "rename",
            "table-rename",
            "drop-fields",
            "backfills",
            "operation",
            "seed",
            "seed-file",
        ],
        "db-status" => &["history", "database", "to"],
        "db-install" | "db-upgrade" | "db-downgrade" | "db-purge" | "db-uninstall" => {
            &["history", "database", "plan", "target", "allow-data-loss"]
        }
        "db-populate" => &["history", "database", "plan", "target", "seed"],
        _ => {
            return Err(Error::invalid(
                "operator event is not compiled into this tool",
            ));
        }
    };
    if data.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(Error::invalid("unknown operator option"));
    }
    let event = event.to_owned();
    // File composition and immutable publication complete on an awaited owned worker.
    let prepared = tokio::task::spawn_blocking(move || prepare(&event, &data))
        .await
        .map_err(|cause| external("operator preparation worker failed", cause))??;
    // Only fully checked runtime work reaches this phase; private paths never enter artifacts.
    match prepared {
        Prepared::Report(report) => Ok(report),
        #[cfg(feature = "database-tools")]
        Prepared::Apply {
            plan,
            options,
            hash,
            target,
            loss,
        } => {
            let owner = stackpress_framework_sql::migrations::SqliteMaintenance::open(options)
                .await
                .map_err(|cause| external("database maintenance open failed", cause))?;
            let outcome = stackpress_framework_sql::migrations::MigrationRunner::apply_to(
                owner,
                &plan,
                &hash,
                &target,
                loss.as_ref(),
            )
            .await
            .map_err(|cause| external("database maintenance failed", cause))?;
            Ok(json!({"outcome":format!("{outcome:?}"),"plan":hash.as_str()}))
        }
        #[cfg(feature = "database-tools")]
        Prepared::Status { options, snapshot } => {
            let status = stackpress_framework_sql::migrations::MigrationRunner::status(
                options,
                snapshot.as_ref(),
            )
            .await
            .map_err(|cause| external("database status failed", cause))?;
            serde_json::to_value(status)
                .map_err(|cause| external("database status encoding failed", cause))
        }
    }
}
/// Pure reports and fully preflighted runtime work have distinct ownership.
enum Prepared {
    Report(Value),
    #[cfg(feature = "database-tools")]
    Apply {
        plan: Box<stackpress_framework_sql::migrations::MigrationPlan>,
        options: stackpress_framework_sql::migrations::DatabaseOptions,
        hash: PlanHash,
        target: String,
        loss: Option<stackpress_framework_sql::migrations::LossApproval>,
    },
    #[cfg(feature = "database-tools")]
    Status {
        options: stackpress_framework_sql::migrations::DatabaseOptions,
        snapshot: Option<RevisionSnapshot>,
    },
}
/// Complete bounded file/semantic checks before publishing work to a native executor.
fn prepare(event: &str, data: &serde_json::Map<String, Value>) -> stackpress_lib::Result<Prepared> {
    let default = Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema");
    let history = RevisionStore::new(
        optional(data, "history")?
            .map(PathBuf::from)
            .unwrap_or(default),
    );
    match event {
        "history-check" => {
            let revisions = history.check().map_err(data_error)?;
            Ok(Prepared::Report(
                json!({"revisions":revisions.iter().map(|s|s.revision()).collect::<Vec<_>>()}),
            ))
        }
        "revision" => {
            let input = optional(data, "input")?
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/schema.idea")
                });
            let name = required(data, "name")?;
            let app = optional(data, "app")?.unwrap_or("workspace");
            let snapshot = super::operator_schema::snapshot(&input, app, name)?;
            let checked = history.check().map_err(data_error)?;
            let parent = optional(data, "parent")?
                .map(RevisionId::parse)
                .transpose()
                .map_err(data_error)?
                .or_else(|| {
                    checked
                        .last()
                        .and_then(|s| RevisionId::parse(s.revision()).ok())
                });
            let outcome = history
                .record(&snapshot, parent.as_ref())
                .map_err(data_error)?;
            let (state, id) = match outcome {
                RevisionOutcome::Recorded(id) => ("Recorded", id),
                RevisionOutcome::NoChanges(id) => ("NoChanges", id),
            };
            Ok(Prepared::Report(
                json!({"outcome":state,"revision":id.as_str()}),
            ))
        }
        "migration-plan" => {
            let to = history
                .revision(&RevisionId::parse(required(data, "to")?).map_err(data_error)?)
                .map_err(data_error)?;
            let dialect = match optional(data, "dialect")?.unwrap_or("sqlite") {
                "sqlite" => DialectId::Sqlite,
                "postgres" => DialectId::Postgres,
                "mysql" => DialectId::Mysql,
                _ => return Err(Error::invalid("unknown migration dialect")),
            };
            let operation = optional(data, "operation")?.unwrap_or("schema");
            let plan=match operation {
                "purge"=>MigrationPlanner::purge(&to,dialect),
                "uninstall"=>MigrationPlanner::uninstall(&to,dialect),
                "populate"=> {
                    let source=read_regular(Path::new(required(data,"seed-file")?))?;
                    let rows:Vec<SeedRow>=serde_json::from_str(&source).map_err(|cause|external("seed file decoding failed",cause))?;
                    MigrationPlanner::populate(&to,required(data,"seed")?,rows,dialect)
                }
                "schema"=> {
                    let from=optional(data,"from")?.or(to.parent());
                    let renames=mappings(data)?;
                    match from {
                        None=>MigrationPlanner::baseline(&to,dialect),
                        Some(id)=> {
                            let from=history.revision(&RevisionId::parse(id).map_err(data_error)?).map_err(data_error)?;
                            let suggestions=MigrationPlanner::unresolved_suggestions(&from,&to,&renames).map_err(data_error)?;
                            let findings=MigrationPlanner::review(&from,&to,&renames,dialect).map_err(data_error)?;
                            match MigrationPlanner::between(&from,&to,renames,dialect) {
                                Ok(plan)=>Ok(plan),
                                Err(error) if !suggestions.is_empty()=>return Ok(Prepared::Report(json!({"outcome":"NeedsExplicitMapping","suggestions":suggestions,"reason":error.context(),"findings":findings}))),
                                Err(error)=>return Ok(Prepared::Report(json!({"outcome":"Blocked","findings":findings,"reason":error.context()}))),
                            }
                        }
                    }
                }
                _=>return Err(Error::invalid("unknown migration operation")),
            }.map_err(data_error)?;
            let hash = history.write_plan(&plan).map_err(data_error)?;
            Ok(Prepared::Report(
                json!({"plan":hash.as_str(),"findings":plan.findings(),"blocked":plan.blocked_reasons(),"reverse_available":plan.reverse().is_some(),"reverse_plan":plan.reverse().map(|p|p.hash().as_str())}),
            ))
        }
        #[cfg(feature = "database-tools")]
        "db-status" => {
            let checked = history.check().map_err(data_error)?;
            let snapshot = match optional(data, "to")? {
                Some(id) => Some(
                    history
                        .revision(&RevisionId::parse(id).map_err(data_error)?)
                        .map_err(data_error)?,
                ),
                None => checked
                    .last()
                    .map(|s| history.revision(&RevisionId::parse(s.revision())?))
                    .transpose()
                    .map_err(data_error)?,
            };
            Ok(Prepared::Status {
                options: database_options(data, false)?,
                snapshot,
            })
        }
        #[cfg(feature = "database-tools")]
        _ => prepare_apply(event, data, &history),
        #[cfg(not(feature = "database-tools"))]
        _ => Err(Error::invalid("database-tools feature is required")),
    }
}
/// Read/re-render authority, enforce command purpose and bind loss approval to an explicit target.
#[cfg(feature = "database-tools")]
fn prepare_apply(
    event: &str,
    data: &serde_json::Map<String, Value>,
    history: &RevisionStore,
) -> stackpress_lib::Result<Prepared> {
    use stackpress_framework_sql::migrations::LossApproval;
    let hash = PlanHash::parse(required(data, "plan")?).map_err(data_error)?;
    let plan = history.plan(&hash).map_err(data_error)?;
    let kind = plan.operation();
    let desired = match event {
        "db-install" | "db-upgrade" | "db-downgrade" => PlanOperation::Schema,
        "db-purge" => PlanOperation::Purge,
        "db-uninstall" => PlanOperation::Uninstall,
        "db-populate" => PlanOperation::Populate,
        _ => return Err(Error::invalid("unknown database operator")),
    };
    if kind != desired {
        return Err(Error::invalid(
            "plan does not authorize this operator command",
        ));
    }
    if event == "db-install" && !plan.source_snapshot().tables().is_empty() {
        return Err(Error::invalid("install requires a baseline plan"));
    }
    if event == "db-upgrade" && plan.source_snapshot().tables().is_empty() {
        return Err(Error::invalid("upgrade requires an applied predecessor"));
    }
    if (event == "db-downgrade") != plan.is_reverse() {
        return Err(Error::invalid(
            "command requires the exact forward or reverse plan hash",
        ));
    }
    if event == "db-populate" && plan.seed_id() != Some(required(data, "seed")?) {
        return Err(Error::invalid("seed ID does not match approved plan"));
    }
    let target = optional(data, "target")?.unwrap_or("default").to_owned();
    let allow = match data.get("allow-data-loss") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(Value::String(value)) if value == "true" => true,
        Some(Value::String(value)) if value == "false" => false,
        _ => return Err(Error::invalid("allow-data-loss must be a boolean")),
    };
    if allow && !data.contains_key("target") {
        return Err(Error::invalid(
            "data-loss approval requires an explicit target label",
        ));
    }
    let approval_hash = plan.hash().clone();
    let loss = if allow {
        Some(LossApproval::new(approval_hash.clone(), &target).map_err(data_error)?)
    } else {
        None
    };
    // Capability/loss refusal must happen before opening a file or acquiring native policy.
    stackpress_framework_sql::migrations::MigrationRunner::validate(
        &plan,
        &approval_hash,
        &target,
        loss.as_ref(),
    )
    .map_err(data_error)?;
    Ok(Prepared::Apply {
        plan: Box::new(plan),
        options: database_options(data, event == "db-install")?,
        hash: approval_hash,
        target,
        loss,
    })
}
/// Only explicit operator configuration chooses a private database path; history never stores it.
#[cfg(feature = "database-tools")]
fn database_options(
    data: &serde_json::Map<String, Value>,
    create: bool,
) -> stackpress_lib::Result<stackpress_framework_sql::migrations::DatabaseOptions> {
    Ok(
        stackpress_framework_sql::migrations::DatabaseOptions::file(required(data, "database")?)
            .create_if_missing(create),
    )
}
/// Parse command-specific mapping data from the existing emit envelope, never argv directly.
fn mappings(data: &serde_json::Map<String, Value>) -> stackpress_lib::Result<RenameMap> {
    let mut mappings = RenameMap::new();
    if let Some(value) = optional(data, "table-rename")? {
        for pair in value.split(',') {
            let (old, new) = pair
                .split_once('=')
                .ok_or_else(|| Error::invalid("table mapping must be old=new"))?;
            mappings = mappings.table(old, new).map_err(data_error)?;
        }
    }
    if let Some(value) = optional(data, "rename")? {
        for pair in value.split(',') {
            let (old, new) = pair
                .split_once('=')
                .ok_or_else(|| Error::invalid("field mapping must be table.old=new"))?;
            let (table, field) = old
                .split_once('.')
                .ok_or_else(|| Error::invalid("field mapping must name a table"))?;
            mappings = mappings.field(table, field, new).map_err(data_error)?;
        }
    }
    if let Some(value) = optional(data, "drop-fields")? {
        for pair in value.split(',') {
            let (table, field) = pair
                .split_once('.')
                .ok_or_else(|| Error::invalid("deliberate removal must name table.field"))?;
            mappings = mappings.drop_field(table, field).map_err(data_error)?;
        }
    }
    if let Some(value) = optional(data, "backfills")? {
        let values: BTreeMap<String, Literal> = serde_json::from_str(value)
            .map_err(|cause| external("backfill decoding failed", cause))?;
        for (key, value) in values {
            let (table, field) = key
                .split_once('.')
                .ok_or_else(|| Error::invalid("backfill must name table.field"))?;
            mappings = mappings.backfill(table, field, value).map_err(data_error)?;
        }
    }
    Ok(mappings)
}
/// Read only bounded regular seed files; symlinks/FIFOs cannot become blocking inputs.
fn read_regular(path: &Path) -> stackpress_lib::Result<String> {
    use std::io::Read;
    let meta = std::fs::symlink_metadata(path)
        .map_err(|cause| external("seed file inspection failed", cause))?;
    if !meta.is_file() || meta.len() > 1024 * 1024 {
        return Err(Error::invalid("seed must be a bounded regular file"));
    }
    let mut text = String::new();
    std::fs::File::open(path)
        .map_err(|cause| external("seed file open failed", cause))?
        .take(1024 * 1024 + 1)
        .read_to_string(&mut text)
        .map_err(|cause| external("seed file read failed", cause))?;
    if text.len() > 1024 * 1024 {
        return Err(Error::invalid("seed byte budget exceeded"));
    }
    Ok(text)
}
/// Decode only string options, keeping absent distinct from malformed values.
fn optional<'a>(
    data: &'a serde_json::Map<String, Value>,
    key: &str,
) -> stackpress_lib::Result<Option<&'a str>> {
    data.get(key)
        .map(|v| {
            v.as_str()
                .ok_or_else(|| Error::invalid("operator option must be text"))
        })
        .transpose()
}
/// Demand an explicit nonempty operator input without echoing private values.
fn required<'a>(
    data: &'a serde_json::Map<String, Value>,
    key: &str,
) -> stackpress_lib::Result<&'a str> {
    optional(data, key)?
        .filter(|v| !v.is_empty())
        .ok_or_else(|| Error::invalid("required operator option is missing"))
}
/// Preserve data-layer errors as concrete causes beneath distinct operator context.
fn data_error(cause: stackpress_framework_sql::DataError) -> Error {
    external("operator plan or history validation failed", cause)
}
/// Keep causes separate from safe context; never interpolate their Display.
fn external(context: &'static str, cause: impl std::error::Error + Send + Sync + 'static) -> Error {
    Error::new(context)
        .with_kind(ErrorKind::External)
        .with_cause(cause)
}
