//! Public operation assembly and refusal paths with real checked field policy.
#![cfg(all(feature = "server", feature = "codegen"))]
use serde_json::{Value, json};
use stackpress_framework_schema::{
    FieldAccess, RecordSchema, Schema, ValueContext,
    codegen::{DefinitionRegistry, NormalizeOptions},
};
use stackpress_framework_sql::operations::{self, Operation, Registration};
use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};

/// Normalize authored embeddings and explicitly publish only the caller-writable fields.
fn policy() -> RecordSchema {
    let mut loader = MemoryLoader::new("/proof").unwrap();
    loader.insert("schema.idea", "type Detail { label String? secret String? } model Proof { id String @id name String detail Detail? @default({label \"from-default\"}) details Detail[] }");
    let composed = Composer::new(loader, CompositionLimits::default())
        .compose("schema.idea")
        .unwrap();
    let mut registry = DefinitionRegistry::standard();
    for (model, names) in [
        ("Proof", &["id", "name", "detail", "details"][..]),
        ("Detail", &["label"][..]),
    ] {
        for name in names {
            registry = registry
                .with_field_policy(
                    model,
                    name,
                    FieldAccess {
                        readable: true,
                        writable: true,
                        sensitive: false,
                        publish_default: true,
                    },
                )
                .unwrap();
        }
    }
    let schema =
        Schema::normalize(&composed.config, &registry, NormalizeOptions::default()).unwrap();
    RecordSchema {
        model: schema.models()[0].clone(),
        fieldsets: schema.fieldsets().to_vec(),
        enums: schema.enums().to_vec(),
    }
}

/// Host values extend singular defaults and matching public list rows without replacing caller data.
#[test]
fn host_assembly_preserves_defaults_and_public_list_cardinality() {
    let policy = policy();
    let input = json!({"id":"p","name":"caller","details":[{"label":"one"},{"label":"two"}]});
    let host = json!({"detail":{"secret":"host-default"},"details":[{"secret":"first"},{"secret":"second"}]});
    let assembled: Value =
        operations::assemble(&input, &host, &policy, false, &ValueContext::default()).unwrap();
    assert_eq!(
        assembled,
        json!({"id":"p","name":"caller","detail":{"label":"from-default","secret":"host-default"},"details":[{"label":"one","secret":"first"},{"label":"two","secret":"second"}]})
    );
    let patched: Value = operations::assemble(
        &json!({"name":"changed"}),
        &json!({"detail":{"secret":"patch-only"}}),
        &policy,
        true,
        &ValueContext::default(),
    )
    .unwrap();
    assert_eq!(
        patched,
        json!({"name":"changed","detail":{"secret":"patch-only"}})
    );
}

/// Invalid host authority, malformed embedding shapes and cardinality mismatches are typed refusals.
#[test]
fn host_assembly_rejects_replacement_and_malformed_embeddings() {
    let policy = policy();
    for (input, host, patch, message) in [
        (json!({}), json!({}), true, "public patch must be nonempty"),
        (
            json!([]),
            json!({}),
            false,
            "assembled input must be an object",
        ),
        (json!({}), json!([]), false, "host fields must be an object"),
        (
            json!({}),
            json!({"absent":1}),
            false,
            "host field is not writable",
        ),
        (
            json!({"name":"caller"}),
            json!({"name":"replacement"}),
            false,
            "host fields cannot replace approved input",
        ),
        (
            json!({"name":"caller"}),
            json!({"id":"new"}),
            true,
            "host field is not writable",
        ),
        (
            json!({"detail":{"secret":"unexpected"}}),
            json!({"detail":{"secret":"replacement"}}),
            false,
            "private field unexpectedly supplied",
        ),
        (
            json!({"details":{}}),
            json!({"details":[]}),
            false,
            "host list must match supplied input elements",
        ),
        (
            json!({"details":[{}]}),
            json!({"details":[]}),
            false,
            "host list length differs from supplied input",
        ),
    ] {
        let error = operations::assemble::<_, Value>(
            &input,
            &host,
            &policy,
            patch,
            &ValueContext::default(),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains(message),
            "{input} / {host}: {error}"
        );
    }
}

/// Invalid registration never consumes the marker, and host errors retain their private cause.
#[test]
fn invalid_operation_selection_is_atomic_and_host_errors_are_private() {
    let mut registration = Registration::default();
    for prefix in [
        String::new(),
        "a".repeat(129),
        "contains space".into(),
        "unicode-é".into(),
    ] {
        assert_eq!(
            registration
                .claim(&prefix, &[Operation::Get])
                .unwrap_err()
                .kind(),
            stackpress_lib::ErrorKind::InvalidInput
        );
    }
    assert!(
        registration
            .claim("proof", &[Operation::Get, Operation::Get])
            .is_err()
    );
    registration.claim("proof", &[Operation::Get]).unwrap();
    assert!(registration.claim("proof", &[Operation::Get]).is_err());
    let error = operations::host_error(stackpress_framework_sql::DataError::new(
        "private fixture detail",
    ));
    assert_eq!(error.kind(), stackpress_lib::ErrorKind::External);
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .to_string()
            .contains("private fixture detail")
    );
}
