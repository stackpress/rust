//! Pre-template fingerprints protect formatted artifacts and retain raw-cap inventory.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use stackpress_framework_schema::{FieldAccess, Schema, ValueType, codegen::*};
use stackpress_idea_parser::{AttributeValue, CompileInputs, SchemaConfig, parse};
use stackpress_idea_transformer::{
    OutputPlan, OutputWorkspace, Registry, Result, TransformContext, TransformError,
    TransformErrorKind,
};

/// Exclusively created test output parent; cleanup never reaches a generated application root.
struct Fixture(std::path::PathBuf);

impl Drop for Fixture {
    /// Remove only the test parent created by this fixture.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Allocate a unique canonical temporary directory so the managed writer rejects no symlink alias.
fn fixture(name: &str) -> Fixture {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "schema-template-{}-{name}-{id}",
        std::process::id()
    ));
    std::fs::create_dir(&path).unwrap();
    Fixture(path)
}

/// Deliberately small source shapes exercise empty, public, nested, enum and host-default paths.
const CASES: [(&str, &str); 4] = [
    ("empty", ""),
    (
        "small",
        "model Profile { id String @id title String @searchable @sortable }",
    ),
    (
        "matrix",
        "enum Choice { FIRST \"a\" SECOND \"b\" } enum Count { ZERO 0 ONE 1 } enum Flag { NO false YES true } type Address { city String country String @default(\"PH\") } model Matrix { id String @id title String @searchable @sortable @default(\"literal {{inert}} [%inert%]\") count Integer @default(0) ratio Number @default(0.25) flag Boolean @default(false) day Date @default(\"2024-02-29\") time Time @default(\"12:00:00\") stamp Datetime @default(\"2024-01-01T00:00:00Z\") choice Choice number Count truth Flag object Object? tags String[] address Address? addresses Address[] optionalAddresses Address[]? }",
    ),
    (
        "host",
        "type Address { city String nonce String @generated @default(\"nanoid(8)\") } model Host { id String @id secret String @default(\"private\") address Address? token String @generated @default(\"nanoid(8)\") created Datetime @generated @default(\"now()\") }",
    ),
];

/// Supply explicit app policy while keeping private host defaults private in the host case.
fn definitions(
    config: &SchemaConfig,
) -> stackpress_framework_schema::SchemaResult<DefinitionRegistry> {
    let mut definitions = DefinitionRegistry::standard().with_provider(ProviderDefinition {
        name: "nanoid".into(),
        revision: "fixture-1".into(),
        output: ValueType::String,
    })?;
    for (name, model) in config.r#type.iter().chain(&config.model) {
        for field in &model.columns {
            let public = name != "Host" || field.name != "secret";
            definitions = definitions.with_field_policy(
                name,
                &field.name,
                FieldAccess {
                    readable: public,
                    writable: public,
                    sensitive: !public,
                    publish_default: public,
                },
            )?;
        }
    }
    Ok(definitions)
}

/// Use the public registry/normalizer/emitter boundary without installing generated files.
fn emit(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    let schema = Schema::normalize(
        context.schema(),
        &definitions(context.schema()).map_err(transform_error)?,
        NormalizeOptions::default(),
    )
    .map_err(transform_error)?;
    emit_contracts(&schema, plan)
}

/// Preserve formatted bytes/path ownership while retaining old raw sizes for cap comparisons.
#[test]
fn emitted_source_matches_pre_template_baseline() {
    let mut snapshots = serde_json::Map::new();
    for (name, source) in CASES {
        let config = parse("template-parity.idea", source, &CompileInputs::default()).unwrap();
        let mut registry = Registry::new();
        register(&mut registry, emit).unwrap();
        let plan = registry.plan(&config, vec![]).unwrap();
        let mut files: Vec<_> = plan
            .files()
            .map(|(path, owner, source)| {
                json!({
                    "path":path, "owner":owner, "raw_bytes":source.len(),
                    "raw_sha256":format!("{:x}", Sha256::digest(source.as_bytes())),
                })
            })
            .collect();
        let parent = fixture(name);
        let root = parent.0.join("generated");
        OutputWorkspace::new(&root).unwrap().install(plan).unwrap();
        for file in &mut files {
            let source = std::fs::read(root.join(file["path"].as_str().unwrap())).unwrap();
            file["bytes"] = json!(source.len());
            file["sha256"] = json!(format!("{:x}", Sha256::digest(&source)));
        }
        snapshots.insert(name.into(), Value::Array(files));
    }
    let actual = Value::Object(snapshots);
    let expected: Value = serde_json::from_str(include_str!("baseline.json")).unwrap();
    assert_eq!(artifacts(actual), artifacts(expected));
}

/// Raw whitespace may change for readable templates; only final artifacts are frozen byte-for-byte.
fn artifacts(mut snapshots: Value) -> Value {
    for files in snapshots.as_object_mut().unwrap().values_mut() {
        for file in files.as_array_mut().unwrap() {
            file.as_object_mut().unwrap().remove("raw_bytes");
            file.as_object_mut().unwrap().remove("raw_sha256");
        }
    }
    snapshots
}

/// Large real JSON defaults exercise context encoding overhead and both unchanged writer caps.
#[test]
fn normalized_json_defaults_retain_managed_output_boundary_and_limit_cause() {
    use std::error::Error;
    // Mutate finalized parser-owned config rather than inventing an unchecked runtime descriptor.
    let config = |bytes| {
        let mut config = parse(
            "template-boundary.idea",
            "model Payload { id String @id data Object }",
            &CompileInputs::default(),
        )
        .unwrap();
        config.model.get_mut("Payload").unwrap().columns[1]
            .attributes
            .insert(
                "default".into(),
                AttributeValue::Array(vec![json!({"payload":"x".repeat(bytes)})]),
            );
        config
    };
    let mut registry = Registry::new();
    register(&mut registry, emit).unwrap();
    // Locate the actual aggregate boundary; no assumption ties JSON input length to Rust size.
    let (mut accepted, mut rejected) = (0, 256 * 1024);
    while rejected - accepted > 1 {
        let middle = accepted + (rejected - accepted) / 2;
        if registry.plan(&config(middle), vec![]).is_ok() {
            accepted = middle;
        } else {
            rejected = middle;
        }
    }
    let plan = registry.plan(&config(accepted), vec![]).unwrap();
    let raw_bytes: usize = plan.files().map(|(_, _, source)| source.len()).sum();
    assert!(
        raw_bytes <= 1024 * 1024 && raw_bytes > 1024 * 1024 - 128,
        "raw bytes: {raw_bytes}"
    );
    let parent = fixture("boundary");
    let report = OutputWorkspace::new(parent.0.join("generated"))
        .unwrap()
        .install(plan);
    // At this raw maximum, rustfmt expansion independently exceeds the unchanged formatted cap.
    assert_eq!(report.unwrap_err().kind(), TransformErrorKind::Limit);
    assert!(!parent.0.join("generated/contracts/payload.rs").exists());
    // Leave real formatting headroom, while keeping the installed aggregate above 95% of cap.
    let plan = registry.plan(&config(accepted - 4096), vec![]).unwrap();
    let raw_bytes: usize = plan.files().map(|(_, _, source)| source.len()).sum();
    assert!(raw_bytes > 1024 * 1024 * 95 / 100);
    let paths: Vec<_> = plan.files().map(|(path, _, _)| path.to_owned()).collect();
    let parent = fixture("accepted-boundary");
    OutputWorkspace::new(parent.0.join("generated"))
        .unwrap()
        .install(plan)
        .unwrap();
    let formatted_bytes: usize = paths
        .iter()
        .map(|path| {
            std::fs::metadata(parent.0.join("generated").join(path))
                .unwrap()
                .len() as usize
        })
        .sum();
    assert!(formatted_bytes <= 1024 * 1024 && formatted_bytes > 1024 * 1024 * 95 / 100);
    eprintln!(
        "boundary payload={} raw={raw_bytes} formatted={formatted_bytes}; raw maximum payload={accepted}",
        accepted - 4096
    );
    let error = registry.plan(&config(rejected), vec![]).unwrap_err();
    // The final aggregate is inserted after callbacks, so its raw overflow is a direct Limit.
    assert_eq!(error.kind(), TransformErrorKind::Limit);
    assert!(!error.to_string().contains("payload"));
    // A single oversized asset exercises the local concrete TemplateError adapter as well.
    let error = registry.plan(&config(512 * 1024), vec![]).unwrap_err();
    assert_eq!(error.kind(), TransformErrorKind::Callback);
    let emitter = error
        .source()
        .unwrap()
        .downcast_ref::<TransformError>()
        .unwrap();
    assert_eq!(emitter.kind(), TransformErrorKind::Limit);
    assert!(emitter.to_string().contains("templates/public.rs.tpl"));
    let cause = emitter
        .source()
        .unwrap()
        .downcast_ref::<stackpress_lib::TemplateError>()
        .unwrap();
    assert_eq!(cause.kind(), stackpress_lib::TemplateErrorKind::Limit);
}
