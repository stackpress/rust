//! Generated complete core witnesses; overrides retain their declared types until compilation.
use super::{emission::emit_probe, source_schema};
use stackpress_framework_schema::{ValueType, codegen::EmissionSemantics};

/// These generic transfers type-check from family bounds alone; no per-model adapter,
/// JSON round-trip, field projection or parallel error envelope is required.
#[allow(dead_code)]
fn transfer<M:stackpress_framework_sql::source::ApiModels>(
    record:&<<M::Profile as stackpress_framework_sql::source::SourceModel>::Schema as stackpress_framework_schema::source::SourceSchema>::Record,
) -> <<M::Application as stackpress_framework_sql::source::SourceModel>::Schema as stackpress_framework_schema::source::SourceSchema>::Record{
    record.clone()
}

#[test]
fn complete_core_witness_inventory_and_wrong_override_fixtures() {
    let schema = source_schema(false);
    let output = emit_probe(&schema, EmissionSemantics::Source);
    assert_eq!(
        output
            .matches("impl stackpress_framework_schema::source::SourceField<")
            .count(),
        48
    );
    assert_eq!(
        output
            .matches("impl stackpress_framework_schema::source::SourceDeclaration<")
            .count(),
        48
    );
    // Only the recorded authoring step writes fixtures. Normal runs check maintained source.
    let root = std::path::Path::new("tests/source-ui/native");
    let author = std::env::var_os("G50_AUTHOR_FIXTURES").is_some();
    let fixtures = [
        ("generated", None),
        ("extended", None),
        (
            "profile_auth_target",
            Some(("Profile", "auth", ValueType::Model("Application".into()))),
        ),
        (
            "profile_created_kind",
            Some(("Profile", "created", ValueType::Date)),
        ),
        (
            "profile_roles",
            Some(("Profile", "roles", ValueType::Boolean)),
        ),
        (
            "auth_verified",
            Some(("Auth", "verified", ValueType::String)),
        ),
        (
            "application_expires",
            Some(("Application", "expires", ValueType::String)),
        ),
        (
            "session_secret",
            Some(("Session", "secret", ValueType::Integer)),
        ),
    ];
    for (name, wrong) in fixtures {
        let schema = if let Some((model, field, kind)) = wrong {
            super::overridden_source_schema(model, field, &kind)
        } else if name == "extended" {
            source_schema(true)
        } else {
            schema.clone()
        };
        let source = emit_probe(&schema, EmissionSemantics::Source);
        let path = root.join(format!("{name}.rs"));
        if author {
            std::fs::create_dir_all(root).unwrap();
            std::fs::write(&path, &source).unwrap();
        }
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            source,
            "generated witness drift: {name}"
        );
    }
}
