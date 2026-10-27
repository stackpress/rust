//! Build-host DDL conversion uses the same explicit intent as generated table builders.
#![cfg(all(feature = "codegen", feature = "migrations"))]
use stackpress_framework_schema::{
    Schema,
    codegen::{DefinitionRegistry, NormalizeOptions},
};
use stackpress_framework_sql::{codegen, ddl_policy, migrations::*};
use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};
use stackpress_sql::{DialectId, Length, SqlType};

/// Compose through the existing Idea owner without adding another parser or metadata convention.
fn normalize(source: &str) -> stackpress_framework_schema::SchemaResult<Schema> {
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("schema.idea", source);
    let composed = Composer::new(loader, CompositionLimits::default())
        .compose("schema.idea")
        .unwrap();
    Schema::normalize(
        &composed.config,
        &DefinitionRegistry::standard(),
        NormalizeOptions::default(),
    )
}
fn schema(source: &str) -> Schema {
    normalize(source).unwrap()
}

/// @max alone remains TEXT; annotated lengths survive both conversion and immutable signatures.
#[test]
fn explicit_storage_and_target_signatures() {
    let plain = schema("model X { id String @id @max(191) }");
    let bounded = schema("model X { id String @id @store.length(191) }");
    let plain = codegen::storage_definitions(&plain).unwrap();
    assert_eq!(plain[0].fields[0].data_type, SqlType::Text);
    assert!(plain[0].create().render(DialectId::Mysql).is_err());
    let definitions = codegen::storage_definitions(&bounded).unwrap();
    assert_eq!(definitions[0].fields[0].data_type, SqlType::Varchar);
    assert_eq!(definitions[0].fields[0].length, Some(Length::Size(191)));
    assert!(definitions[0].create().render(DialectId::Mysql).is_err());
    let bin = codegen::mysql_storage_definitions(&bounded, "utf8mb4_bin", 16384).unwrap();
    let ci = codegen::mysql_storage_definitions(&bounded, "utf8mb4_unicode_ci", 16384).unwrap();
    assert_ne!(
        RevisionSnapshot::new("test", "one", bin.clone())
            .unwrap()
            .hash(),
        RevisionSnapshot::new("test", "one", ci.clone())
            .unwrap()
            .hash()
    );
    assert!(ddl_policy::mysql_definitions(bin, "utf8mb4_unicode_ci", 16384).is_err());
    let left = RevisionSnapshot::new("test", "one", definitions).unwrap();
    let right = RevisionSnapshot::new("test", "two", ci).unwrap();
    assert!(MigrationPlanner::between(&left, &right, RenameMap::new(), DialectId::Mysql).is_err());
}

/// Missing/different FK declarations refuse; generated widths are never copied from the target.
#[test]
fn complete_catalog_fk_widths() {
    for width in ["", "@store.length(192)"] {
        let invalid = normalize(&format!(
            "model Parent {{ id String @id @store.length(191) children Child[] }} model Child {{ id Integer @id parentId String {width} parent Parent @relation({{local \"parentId\" foreign \"id\" name \"children\"}}) }}"
        ));
        assert!(invalid.is_err());
    }
    let valid = schema(
        "model Parent { id String @id @store.length(191) children Child[] } model Child { id Integer @id parentId String @store.length(191) parent Parent @relation({local \"parentId\" foreign \"id\" name \"children\"}) }",
    );
    let mut tables = codegen::mysql_storage_definitions(&valid, "utf8mb4_bin", 16384).unwrap();
    tables[1] = tables[1]
        .create()
        .mysql_utf8mb4_dynamic("utf8mb4_unicode_ci", 16384)
        .unwrap()
        .definition()
        .unwrap();
    let snapshot = RevisionSnapshot::new("test", "one", tables).unwrap();
    assert!(MigrationPlanner::baseline(&snapshot, DialectId::Mysql).is_err());
}

/// Validation-only changes remain TEXT and yield no DDL; a resize needs stable complete paging.
#[test]
fn no_inference_or_unstable_scan() {
    let left = schema("model X { id String @id @max(191) }");
    let right = schema("model X { id String @id @max(100) }");
    let from = RevisionSnapshot::new("test", "one", codegen::storage_definitions(&left).unwrap())
        .unwrap()
        .with_provenance(&left.fingerprint().unwrap(), "test")
        .unwrap();
    let to = RevisionSnapshot::new("test", "two", codegen::storage_definitions(&right).unwrap())
        .unwrap()
        .with_provenance(&right.fingerprint().unwrap(), "test")
        .unwrap();
    assert!(
        MigrationPlanner::between(&from, &to, RenameMap::new(), DialectId::Sqlite)
            .unwrap()
            .queries()
            .is_empty()
    );
    let source = stackpress_sql::Create::new("note")
        .add_field(stackpress_sql::Field::new("text", SqlType::Text))
        .definition()
        .unwrap();
    let mut target = source.clone();
    target.fields[0].data_type = SqlType::Varchar;
    target.fields[0].length = Some(Length::Size(3));
    assert!(
        MigrationPlanner::between(
            &RevisionSnapshot::new("test", "one", vec![source]).unwrap(),
            &RevisionSnapshot::new("test", "two", vec![target]).unwrap(),
            RenameMap::new(),
            DialectId::Sqlite
        )
        .is_err()
    );
    for parts in [4, 5] {
        let fields = (0..parts)
            .map(|index| format!("part{index} String @id @store.length(191)"))
            .collect::<Vec<_>>()
            .join(" ");
        let source = schema(&format!("model X {{{fields}}}"));
        assert_eq!(
            codegen::mysql_storage_definitions(&source, "utf8mb4_bin", 16384).is_ok(),
            parts == 4
        );
    }
}
