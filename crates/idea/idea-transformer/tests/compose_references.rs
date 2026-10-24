//! End-to-end typed references retain source provenance and survive only winning merge parts.

use serde_json::{Value, json};
use stackpress_idea_parser::{
    CompileInputs, ErrorCode, FinalSchema, ParseLimits, SourceFile, compile_ast_with_references,
    finalize_with_references, parse_ast,
};
use stackpress_idea_transformer::{
    ComposedSchema, Composer, CompositionLimits, MemoryLoader, Reference, SourceId,
};

/// Compile an isolated import closure without process environment or filesystem state.
fn compose(files: &[(&str, &str)]) -> ComposedSchema {
    let mut loader = MemoryLoader::new("/project").unwrap();
    for (name, text) in files {
        loader.insert(name, *text);
    }
    Composer::new(loader, CompositionLimits::default())
        .compose("root")
        .unwrap()
}

/// Use the parser's checked finalization boundary directly; composition adds no parallel API.
fn finalize(composed: &ComposedSchema) -> FinalSchema {
    finalize_with_references(
        &composed.config,
        &composed.references,
        &CompileInputs::default(),
    )
    .expect("all surviving typed sites have imported targets")
}

#[test]
/// Imported literals stay verbatim next to actual references, including nested argument paths.
fn imported_literal_lookalikes_remain_literal() {
    let composed = compose(&[
        ("root.idea", "use \"nested/child\""),
        (
            "nested/child.idea",
            "use \"../values\"
            model Read @value(Name \"${Name}\") {
                id String @value([Name \"${Name}\"] { actual Name literal \"${Name}\" })
            }",
        ),
        ("values.idea", "prop Name { label \"resolved\" }"),
    ]);
    // The re-export is the parser type itself, without a local wrapper or conversion step.
    let references: &[Reference] = &composed.references;
    assert_eq!(references.len(), 3);
    assert!(
        references
            .iter()
            .all(|site| site.span.file == "nested/child.idea")
    );
    assert_eq!(
        composed.imports[1].importer,
        SourceId::new("/project/nested/child.idea")
    );
    let result = serde_json::to_value(finalize(&composed)).unwrap();
    assert_eq!(
        result["model"]["Read"]["attributes"]["value"],
        json!([{"label":"resolved"}, "${Name}"])
    );
    assert_eq!(
        result["model"]["Read"]["columns"][0]["attributes"]["value"],
        json!([[{"label":"resolved"}, "${Name}"],
            {"actual":{"label":"resolved"},"literal":"${Name}"}])
    );
}

#[test]
/// A reference retained from the first import resolves against a later G19 replacement target.
fn reference_target_uses_later_g19_replacement() {
    let composed = compose(&[
        ("root.idea", "use \"first\" use \"second\""),
        (
            "first.idea",
            "type Name! @label(\"first\") { old String }
            model Read @value(Name) { id String }",
        ),
        (
            "second.idea",
            "type Name @label(\"second\") { replacement Number }",
        ),
    ]);
    assert_eq!(composed.references.len(), 1);
    assert_eq!(composed.references[0].span.file, "first.idea");
    assert!(!composed.references[0].forward);
    assert_eq!(
        composed.provenance[&vec!["type".into(), "Name".into()]],
        SourceId::new("/project/second.idea")
    );
    let expected = serde_json::to_value(&composed.config.r#type["Name"]).unwrap();
    let result = serde_json::to_value(finalize(&composed)).unwrap();
    assert_eq!(
        result["model"]["Read"]["attributes"]["value"],
        Value::Array(vec![expected])
    );
    assert_eq!(result["type"]["Name"]["columns"][0]["name"], "replacement");
}

#[test]
/// An unresolved imported reference reports the original referencing file, line and column.
fn unresolved_reference_reports_referencing_coordinates() {
    let child = "// source coordinates\nmodel Read {\n  id String @use(Missing)\n}";
    let composed = compose(&[
        ("root.idea", "use \"nested/child\""),
        ("nested/child.idea", child),
    ]);
    let error = finalize_with_references(
        &composed.config,
        &composed.references,
        &CompileInputs::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(error.file, "nested/child.idea");
    assert_eq!((error.line, error.column), (3, 18));
    assert_eq!(error.start, child.find("Missing"));
    assert!(error.to_string().starts_with("nested/child.idea:3:18:"));
    assert_eq!(
        composed.provenance[&vec!["model".into(), "Read".into(), "columns".into(), "id".into()]],
        composed.imports[0].imported
    );
}

#[test]
/// Mutable merges discard losing sites and retain named column addresses after prepending.
fn only_winning_attributes_and_columns_are_finalized() {
    let composed = compose(&[
        (
            "root.idea",
            "prop Root { x 1 } use \"child\"
            model Read @same(Root) { id String @value(Root) local String }",
        ),
        (
            "child.idea",
            "prop Child { x 2 }
            model Read @same(Missing) @child(Child) {
                id Number @value(Missing) child String @value(Child)
            }",
        ),
    ]);
    assert_eq!(composed.references.len(), 4);
    assert!(
        composed
            .references
            .iter()
            .all(|site| site.target != "Missing")
    );
    let result = serde_json::to_value(finalize(&composed)).unwrap();
    assert_eq!(result["model"]["Read"]["columns"][0]["name"], "child");
    assert_eq!(
        result["model"]["Read"]["columns"][0]["attributes"]["value"],
        json!([{"x":2}])
    );
    assert_eq!(result["model"]["Read"]["columns"][1]["name"], "id");
    assert_eq!(
        result["model"]["Read"]["columns"][1]["attributes"]["value"],
        json!([{"x":1}])
    );
}

#[test]
/// G19 removes stale local reference payloads when its complete declaration loses.
fn g19_replacement_discards_losing_reference_sites() {
    let composed = compose(&[
        (
            "root.idea",
            "use \"child\" model Read! @bad(Missing) { id String @bad(Missing) }",
        ),
        (
            "child.idea",
            "prop Name { x 2 } model Read @value(Name) { child String }",
        ),
    ]);
    assert_eq!(composed.references.len(), 1);
    assert_eq!(composed.references[0].span.file, "child.idea");
    assert!(
        finalize(&composed).model["Read"]
            .attributes
            .contains_key("value")
    );
}

#[test]
/// Source-local forward-reference restrictions are retained after the declaration is imported.
fn imported_forward_reference_keeps_its_restriction() {
    let composed = compose(&[
        ("root.idea", "use \"child\""),
        (
            "child.idea",
            "model Read @value(Name) { id String } prop Name { x 1 }",
        ),
    ]);
    assert!(composed.references[0].forward);
    let error = finalize_with_references(
        &composed.config,
        &composed.references,
        &CompileInputs::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(error.file, "child.idea");
    assert!(error.message.contains("prior declaration"));
}

#[test]
/// Typed JSON envelopes preserve explicit sites; normalized plain JSON never infers sites.
fn typed_json_imports_preserve_actual_reference_metadata() {
    let source = SourceFile::new(
        "original.idea",
        "use \"values\"
        model Read @value(Name \"${Name}\") { id String }",
    );
    let compiled = compile_ast_with_references(
        &parse_ast(&source, ParseLimits::default()).unwrap(),
        &CompileInputs::default(),
    )
    .unwrap();
    let typed = compiled.to_json(ParseLimits::default()).unwrap();
    let plain = compiled.config.to_json(ParseLimits::default()).unwrap();
    for (text, typed) in [(&typed, true), (&plain, false)] {
        let composed = compose(&[
            ("root.idea", "use \"child.json\""),
            ("child.json", text),
            ("values.idea", "prop Name { x 1 }"),
        ]);
        assert_eq!(composed.references.len(), usize::from(typed));
        let result = serde_json::to_value(finalize(&composed)).unwrap();
        assert_eq!(result["model"]["Read"]["attributes"]["value"][1], "${Name}");
        if typed {
            assert_eq!(composed.references[0].span.file, "original.idea");
            assert_eq!(
                result["model"]["Read"]["attributes"]["value"][0],
                json!({"x":1})
            );
        } else {
            assert_eq!(result["model"]["Read"]["attributes"]["value"][0], "${Name}");
        }
    }
}

#[test]
/// Reference payload copying spends the same finite expansion allowance as its owner config.
fn reference_metadata_is_charged_before_snapshot_copy() {
    let source = SourceFile::new(
        "root.idea",
        "prop Name { x 1 } model Read @value(Name) { id String }",
    );
    let compiled = compile_ast_with_references(
        &parse_ast(&source, ParseLimits::default()).unwrap(),
        &CompileInputs::default(),
    )
    .unwrap();
    let config_bytes = serde_json::to_vec(&compiled.config).unwrap().len();
    let reference_bytes = serde_json::to_vec(&compiled.references).unwrap().len();
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("root.idea", source.text());
    let limits = CompositionLimits {
        max_expanded_bytes: config_bytes,
        ..CompositionLimits::default()
    };
    let error = Composer::new(&loader, limits).compose("root").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("reference expansion budget exceeded")
    );
    let limits = CompositionLimits {
        max_expanded_bytes: config_bytes + reference_bytes,
        ..CompositionLimits::default()
    };
    let composed = Composer::new(&loader, limits).compose("root").unwrap();
    let mut clone = composed.clone();
    clone.references[0].target.clear();
    clone.references[0].span.file.clear();
    assert_eq!(composed.references[0].target, "Name");
    assert_eq!(composed.references[0].span.file, "root.idea");
}

#[test]
/// Reference cycles remain a distinct checked finalization failure after acyclic file imports.
fn cross_source_reference_cycles_are_not_import_cycles() {
    let composed = compose(&[
        ("root.idea", "use \"a\" use \"b\""),
        ("a.idea", "type A @value(B) { id String }"),
        ("b.idea", "type B @value(A) { id String }"),
    ]);
    assert_eq!(composed.imports.len(), 2);
    assert!(
        composed
            .references
            .iter()
            .all(|reference| !reference.forward)
    );
    let error = finalize_with_references(
        &composed.config,
        &composed.references,
        &CompileInputs::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(error.file, "b.idea");
    assert!(error.message.contains("Cyclic reference"));
}

#[test]
/// A bounded composed config can still exhaust expansion; the actual reference owns the error.
fn finalization_expansion_limit_retains_imported_site() {
    let value = format!("prop Name {{ value \"{}\" }}", "x".repeat(512));
    let composed = compose(&[
        ("root.idea", "use \"child\""),
        (
            "child.idea",
            "use \"values\" model Read @value(Name) { id String }",
        ),
        ("values.idea", &value),
    ]);
    let before = composed.config.to_json(ParseLimits::default()).unwrap();
    let inputs = CompileInputs {
        limits: ParseLimits {
            max_bytes: before.len(),
            ..ParseLimits::default()
        },
        ..CompileInputs::default()
    };
    let error =
        finalize_with_references(&composed.config, &composed.references, &inputs).unwrap_err();
    assert_eq!(error.code, ErrorCode::Limit);
    assert_eq!(error.file, "child.idea");
    assert_eq!(
        (error.line, error.column),
        (
            composed.references[0].span.line,
            composed.references[0].span.column
        )
    );
    assert_eq!(
        composed.config.to_json(ParseLimits::default()).unwrap(),
        before
    );
}

#[test]
/// Typed envelopes cannot import stale or malformed reference destinations into a composed run.
fn invalid_typed_json_sites_fail_before_composition() {
    let source = SourceFile::new("original.idea", "model Read @value(Name) { id String }");
    let compiled = compile_ast_with_references(
        &parse_ast(&source, ParseLimits::default()).unwrap(),
        &CompileInputs::default(),
    )
    .unwrap();
    let valid = serde_json::to_value(compiled).unwrap();
    for (field, value) in [
        ("column", json!("missing")),
        ("path", json!([])),
        ("target", json!("Other")),
    ] {
        let mut invalid = valid.clone();
        invalid["references"][0][field] = value;
        let mut loader = MemoryLoader::new("/project").unwrap();
        loader.insert("root.idea", "use \"child.json\"");
        loader.insert("child.json", invalid.to_string());
        let error = Composer::new(loader, CompositionLimits::default())
            .compose("root")
            .unwrap_err();
        assert!(error.to_string().contains("import from root.idea failed"));
    }
}
