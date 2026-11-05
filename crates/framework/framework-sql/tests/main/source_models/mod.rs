//! M0 contracts are compiled against real generated probes, not migrated production models.

mod emission;
mod ownership;
mod runtime;
mod support;
mod witnesses;

use stackpress_framework_schema::{
    Schema,
    codegen::{DefinitionRegistry, NormalizeOptions},
};
use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader, PackageLoader};

/// Small scalar/Date corpus isolates M0 contract mechanics from the source normalizer work in M1.
const DEFAULT: &str = "model Profile { id String @id name String active Boolean @active created Datetime @timestamp }\nmodel Auth { id String @id }\nmodel Application { id String @id }\nmodel Session { id String @id }";

/// Real import/merge and normalization give the structural probe complete metadata.
fn schema(extended: bool) -> Schema {
    let mut app = MemoryLoader::new("/application").unwrap();
    app.insert(
        "schema.idea",
        if extended {
            "use \"package/default.idea\"\nmodel Profile { nickname String? }"
        } else {
            "use \"package/default.idea\""
        },
    );
    let composed = Composer::new(
        PackageLoader::new(app)
            .with_source("package/default.idea", DEFAULT)
            .unwrap(),
        CompositionLimits::default(),
    )
    .compose("schema.idea")
    .unwrap();
    Schema::normalize(
        &composed.config,
        &DefinitionRegistry::standard(),
        NormalizeOptions::default(),
    )
    .unwrap()
}

/// Unmodified shipped source graph, with optional real application extension merge.
fn source_schema(extended: bool) -> stackpress_framework_schema::codegen::SourceGraph {
    compose_source(if extended {
        "\nmodel Profile { nickname String? }"
    } else {
        ""
    })
}

fn overridden_source_schema(
    model: &str,
    field: &str,
    kind: &stackpress_framework_schema::ValueType,
) -> stackpress_framework_schema::codegen::SourceGraph {
    let kind = match kind {
        stackpress_framework_schema::ValueType::Boolean => "Boolean".to_owned(),
        stackpress_framework_schema::ValueType::String => "String".to_owned(),
        stackpress_framework_schema::ValueType::Integer => "Integer".to_owned(),
        stackpress_framework_schema::ValueType::Date => "Date".to_owned(),
        stackpress_framework_schema::ValueType::Model(name) => format!("{name}[]"),
        _ => unreachable!(),
    };
    compose_source(&format!("\nmodel {model} {{ {field} {kind} }}"))
}

fn compose_source(extension: &str) -> stackpress_framework_schema::codegen::SourceGraph {
    let mut app = MemoryLoader::new("/app").unwrap();
    app.insert(
        "schema.idea",
        format!("use \"stackpress-api/schema.idea\"{extension}"),
    );
    let loader = PackageLoader::new(app)
        .with_source(
            "stackpress-session/schema.idea",
            include_str!("../../../../framework-session/schema.idea"),
        )
        .unwrap()
        .with_source(
            "stackpress-api/schema.idea",
            include_str!("../../../../framework-api/schema.idea"),
        )
        .unwrap();
    let config = Composer::new(loader, CompositionLimits::default())
        .compose("schema.idea")
        .unwrap()
        .config;
    Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap()
}

/// Compile-only contracts fail for the intended field/family/future/executor reason.
#[test]
fn source_compile_contracts() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/source-pass/*.rs");
    cases.compile_fail("tests/source-ui/*.rs");
}
