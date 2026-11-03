//! M0 contracts are compiled against real generated probes, not migrated production models.

mod emission;
mod ownership;
mod support;

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

/// Compile-only contracts fail for the intended field/family/future/executor reason.
#[test]
fn source_compile_contracts() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/source-ui/*.rs");
}
