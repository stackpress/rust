//! Standalone M1 contracts: stages, full oracle corpus, lossless references and bounded JSON.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use stackpress_idea_parser::{
    Ast, AttributeValue, CompileInputs, CompiledSchema, DataToken, DeclarationKind, ErrorCode,
    ParseLimits, SchemaConfig, SourceFile, SourceSpan, Statement, TokenKind, TokenValue, ValuePath,
    compile_ast, compile_ast_with_references, finalize, finalize_ast, finalize_with_references,
    parse, parse_ast, tokenize,
};

/// Supply a deterministic fixture environment without changing process-global state.
fn inputs() -> CompileInputs {
    let mut inputs = CompileInputs::default();
    inputs
        .environment
        .insert("DATABASE_URL".into(), "test".into());
    inputs
}

/// Parse checked syntax while keeping test failures focused on the source under examination.
fn ast(source: &str) -> Ast {
    parse_ast(
        &SourceFile::new("named.idea", source),
        ParseLimits::default(),
    )
    .unwrap()
}

/// Borrow an attribute's argument list and fail clearly if normalization changes its kind.
fn arguments(value: &AttributeValue) -> &[Value] {
    let AttributeValue::Array(values) = value else {
        panic!("expected method arguments")
    };
    values
}

/// Run hazardous tests out of process with a monotonic deadline and kill/reap on timeout.
fn watchdog(name: &str) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", name, "--nocapture"])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "child {name}: {status}");
            return;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("watchdog expired for {name}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Compare all ten fixtures to complete current-source compiler/finalizer outputs.
#[test]
fn full_pinned_current_compiled_fixture_matrix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for name in [
        "address", "big", "enum", "model", "mutable", "plugin", "prop", "schema", "type", "use",
    ] {
        let file = format!("{name}.idea");
        let source = std::fs::read_to_string(root.join(&file)).unwrap();
        let golden: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("compiled").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        let syntax = parse_ast(&SourceFile::new(&file, &source), ParseLimits::default()).unwrap();
        let config = compile_ast(&syntax, &inputs()).unwrap();
        assert_eq!(
            serde_json::to_value(&config).unwrap(),
            golden["compiled"],
            "{name}"
        );
        let roundtrip = SchemaConfig::from_json(
            &config.to_json(ParseLimits::default()).unwrap(),
            ParseLimits::default(),
        )
        .unwrap();
        assert_eq!(roundtrip, config, "JSON roundtrip {name}");
        assert!(
            !tokenize(syntax.source(), ParseLimits::default())
                .unwrap()
                .is_empty()
        );
        let typed = compile_ast_with_references(&syntax, &inputs()).unwrap();
        let mut single_source = typed.config;
        // This corpus oracle explicitly uses the legacy discard-imports view; it does not
        // claim that the referenced paths have been loaded or composed.
        single_source.r#use.clear();
        match finalize_with_references(&single_source, &typed.references, &inputs()) {
            Ok(value) => assert_eq!(
                serde_json::to_value(value).unwrap(),
                golden["resolved"]["ok"],
                "typed final {name}"
            ),
            Err(error) => {
                assert_eq!(
                    error.code,
                    ErrorCode::UnknownReference,
                    "typed final {name}"
                );
                assert_eq!(golden["resolved"]["error"], "UnknownReference");
            }
        }
        // Standalone fixtures deliberately omit Text in three files; retain that exact error.
        match finalize(&file, &source, &inputs()) {
            Ok(value) => assert_eq!(
                serde_json::to_value(value).unwrap(),
                golden["resolved"]["ok"],
                "{name}"
            ),
            Err(error) => {
                assert_eq!(error.code, ErrorCode::UnknownReference, "{name}");
                assert_eq!(golden["resolved"]["error"], "UnknownReference", "{name}");
            }
        }
    }
}

/// Syntax keeps raw env nodes and can be compiled concurrently with independent snapshots.
#[test]
fn syntax_environment_and_clone_independence() {
    let syntax = ast(concat!(
        "plugin \"p\" { v env(\"KEY\") } prop P { a [env(\"KEY\")] } ",
        "enum E { A env(\"KEY\") } model X @x(env(\"KEY\")) { id String }",
    ));
    let Statement::Declaration(plugin) = &syntax.syntax().body[0] else {
        panic!("plugin")
    };
    let DataToken::Environment(environment) = &plugin.declarations[0].init.properties[0].value
    else {
        panic!("env must stay typed")
    };
    assert_eq!(environment.value, "KEY");
    assert_eq!(environment.raw, "env(\"KEY\")");
    let results: Vec<_> = std::thread::scope(|scope| {
        ["first", "second"]
            .map(|value| {
                let syntax = &syntax;
                scope.spawn(move || {
                    let mut inputs = CompileInputs::default();
                    inputs.environment.insert("KEY".into(), value.into());
                    compile_ast(syntax, &inputs).unwrap()
                })
            })
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect()
    });
    for (result, expected) in results.iter().zip(["first", "second"]) {
        assert_eq!(result.plugin["p"]["v"], expected);
        assert_eq!(result.prop["P"]["a"], json!([expected]));
        assert_eq!(result.r#enum["E"]["A"], expected);
        assert_eq!(
            arguments(&result.model["X"].attributes["x"]),
            [json!(expected)]
        );
    }
    let mut cloned = results[0].clone();
    cloned.prop["P"].insert("a".into(), Value::Null);
    assert_ne!(cloned, results[0]);
    assert_eq!(
        compile_ast(&syntax, &CompileInputs::default())
            .unwrap_err()
            .code,
        ErrorCode::MissingEnvironment
    );
    let empty = CompileInputs {
        missing_env_is_empty: true,
        ..Default::default()
    };
    assert_eq!(compile_ast(&syntax, &empty).unwrap().r#enum["E"]["A"], "");
    assert_eq!(syntax.syntax().body.len(), 4);
    let source = "prop P { a env(\"ABSENT\") }";
    let error = parse("env.idea", source, &CompileInputs::default()).unwrap_err();
    let start = source.find("env(").unwrap();
    assert_eq!(error.code, ErrorCode::MissingEnvironment);
    assert_eq!(
        (error.start, error.end),
        (Some(start), Some(start + "env(\"ABSENT\")".len()))
    );
    assert!(!format!("{error:?}").contains("first"));
}

/// Typed paths survive serialization, nested arguments and declaration/column addressing.
#[test]
fn typed_references_leave_literal_lookalikes_untouched() {
    let source = concat!(
        "prop Profile { name \"Ada\" } ",
        "model X @meta(Profile \"${Profile}\" [Profile] { nested Profile }) ",
        "{ id String @other([Profile \"${Profile}\"]) }"
    );
    let compiled = compile_ast_with_references(&ast(source), &inputs()).unwrap();
    assert_eq!(compiled.references.len(), 4);
    assert_eq!(compiled.references[0].kind, DeclarationKind::Model);
    assert_eq!(compiled.references[0].declaration, "X");
    assert_eq!(compiled.references[0].path, [ValuePath::Index(0)]);
    assert_eq!(
        compiled.references[1].path,
        [ValuePath::Index(2), ValuePath::Index(0)]
    );
    assert_eq!(
        compiled.references[2].path,
        [ValuePath::Index(3), ValuePath::Key("nested".into())]
    );
    assert_eq!(compiled.references[3].column.as_deref(), Some("id"));
    assert!(tokenize(&SourceFile::new("nested", source), ParseLimits::default()).is_ok());
    let encoded = compiled.to_json(ParseLimits::default()).unwrap();
    let decoded = CompiledSchema::from_json(&encoded, ParseLimits::default()).unwrap();
    assert_eq!(decoded, compiled);
    let final_schema =
        finalize_with_references(&decoded.config, &decoded.references, &inputs()).unwrap();
    let metadata = arguments(&final_schema.model["X"].attributes["meta"]);
    assert_eq!(
        metadata,
        [
            json!({"name":"Ada"}),
            json!("${Profile}"),
            json!([{"name":"Ada"}]),
            json!({"nested":{"name":"Ada"}})
        ]
    );
    assert_eq!(
        arguments(&final_schema.model["X"].columns[0].attributes["other"]),
        [json!([{"name":"Ada"}, "${Profile}"])]
    );
    // The legacy single-source compiler and typed finalization agree on real references.
    assert_eq!(
        final_schema,
        finalize("named.idea", source, &inputs()).unwrap()
    );
    assert_eq!(
        compiled.config.model["X"].attributes["meta"],
        AttributeValue::Array(vec![
            json!("${Profile}"),
            json!("${Profile}"),
            json!(["${Profile}"]),
            json!({"nested":"${Profile}"})
        ])
    );
}

/// Finalization reports missing/forward/cyclic targets at the original reference source.
#[test]
fn reference_errors_and_composed_targets() {
    let compiled =
        compile_ast_with_references(&ast("model X @x(Absent) { id String }"), &inputs()).unwrap();
    let error =
        finalize_with_references(&compiled.config, &compiled.references, &inputs()).unwrap_err();
    let span = &compiled.references[0].span;
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(
        (
            error.file.as_str(),
            error.start,
            error.end,
            error.line,
            error.column
        ),
        (
            "named.idea",
            Some(span.start),
            Some(span.end),
            span.line,
            span.column
        )
    );
    let mut imported = compiled.config.clone();
    imported
        .prop
        .insert("Absent".into(), [("ok".into(), json!(true))].into());
    assert_eq!(
        arguments(
            &finalize_with_references(&imported, &compiled.references, &inputs())
                .unwrap()
                .model["X"]
                .attributes["x"]
        ),
        [json!({"ok":true})]
    );
    let forward = compile_ast_with_references(
        &ast("model X @x(P) { id String } prop P { a 1 }"),
        &inputs(),
    )
    .unwrap();
    assert!(forward.references[0].forward);
    assert_eq!(
        finalize_with_references(&forward.config, &forward.references, &inputs())
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );
    let mut cycle = compile_ast_with_references(
        &ast("model X @x(Y) { id String } model Y @x(X) { id String }"),
        &inputs(),
    )
    .unwrap();
    // Simulate references from independent source files joined by the composition layer.
    cycle.references[0].forward = false;
    assert_eq!(
        finalize_with_references(&cycle.config, &cycle.references, &inputs())
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );
    let mut duplicates = compiled.references.clone();
    duplicates.push(duplicates[0].clone());
    assert_eq!(
        finalize_with_references(&imported, &duplicates, &inputs())
            .unwrap_err()
            .code,
        ErrorCode::ValidationError
    );
    let mut stale = compiled.references.clone();
    stale[0].path = vec![ValuePath::Index(99)];
    assert_eq!(
        finalize_with_references(&imported, &stale, &inputs())
            .unwrap_err()
            .code,
        ErrorCode::ValidationError
    );
    let mut malformed = compiled.references.clone();
    malformed[0].span.start = malformed[0].span.end + 1;
    assert!(finalize_with_references(&imported, &malformed, &inputs()).is_err());
}

/// Strict staged finalization cannot silently discard outstanding imports.
#[test]
fn strict_import_boundary_and_legacy_compatibility() {
    let syntax = ast("use \"missing.idea\" model X { id String }");
    assert!(compile_ast(&syntax, &inputs()).is_ok());
    let error = finalize_ast(&syntax, &inputs()).unwrap_err();
    assert_eq!(error.code, ErrorCode::ValidationError);
    assert_eq!((error.start, error.end), (Some(0), Some(18)));
    let compiled = compile_ast_with_references(&syntax, &inputs()).unwrap();
    assert!(finalize_with_references(&compiled.config, &compiled.references, &inputs()).is_err());
    assert!(finalize("named.idea", syntax.source().text(), &inputs()).is_ok());
    let syntax = ast("prop P { a true } type T @x(P) { id String } model X @x(T) { id String }");
    assert_eq!(
        finalize_ast(&syntax, &inputs()).unwrap(),
        finalize("named.idea", syntax.source().text(), &inputs()).unwrap()
    );
}

/// Every deliberate grammar correction has an asserted stable error code and exact primary span.
#[test]
fn corrected_grammar_codes_and_spans() {
    for (source, needle, code, occurrence) in [
        (
            "model X { a String?? }",
            "String??",
            ErrorCode::InvalidLiteral,
            0,
        ),
        (
            "model X { a String?[] }",
            "String?[]",
            ErrorCode::InvalidLiteral,
            0,
        ),
        (
            "model X { a String[][] }",
            "String[][]",
            ErrorCode::InvalidLiteral,
            0,
        ),
        (
            "model lower { a String }",
            "lower",
            ErrorCode::UnexpectedToken,
            0,
        ),
        ("enum X { lower 1 }", "lower", ErrorCode::UnexpectedToken, 0),
        (
            "model X @9..A { a String }",
            "@9..A",
            ErrorCode::UnexpectedToken,
            0,
        ),
        (
            "model X { a String a Number }",
            "a",
            ErrorCode::DuplicateName,
            1,
        ),
        ("prop X { x 1 x 2 }", "x", ErrorCode::DuplicateName, 1),
        ("enum X { A 1 A 2 }", "A", ErrorCode::DuplicateName, 1),
        (
            "model X @a @a { id String }",
            "@a",
            ErrorCode::DuplicateName,
            1,
        ),
        (
            "model X { id String } type X { id String }",
            "X",
            ErrorCode::DuplicateName,
            1,
        ),
        (
            "prop X { a Elsewhere }",
            "Elsewhere",
            ErrorCode::UnexpectedToken,
            0,
        ),
        (
            "prop X { a 18446744073709551616 }",
            "18446744073709551616",
            ErrorCode::InvalidLiteral,
            0,
        ),
        (
            "prop X { a -9223372036854775809 }",
            "-9223372036854775809",
            ErrorCode::InvalidLiteral,
            0,
        ),
        ("prop X { a 1. }", "1.", ErrorCode::InvalidLiteral, 0),
        (
            "model X @a(Role.ADMIN) { id String }",
            "Role.ADMIN",
            ErrorCode::UnexpectedToken,
            0,
        ),
    ] {
        let error = parse("grammar.idea", source, &inputs()).unwrap_err();
        assert_eq!(error.code, code, "{source}");
        let start = if source.contains("Elsewhere") {
            source.find("Elsewhere").unwrap()
        } else if source.contains("Role.ADMIN") {
            source.find("Role.ADMIN").unwrap()
        } else {
            source.match_indices(needle).nth(occurrence).unwrap().0
        };
        assert_eq!(
            (error.start, error.end),
            (Some(start), Some(start + needle.len())),
            "{source}"
        );
        if code == ErrorCode::DuplicateName {
            assert_eq!(error.related.len(), 1);
            assert!(error.related[0].start < start);
            assert_eq!(error.related[0].file, "grammar.idea");
        }
    }
    // Approved modifiers, unknown metadata, strict casing and raw strings remain usable.
    let schema = parse(
        "s",
        "type T! @custom.ns_2(\"a\\b\") { a Custom b Other? c Custom[] d Other[]? }",
        &inputs(),
    )
    .unwrap();
    assert!(!schema.r#type["T"].mutable);
    assert_eq!(
        schema.r#type["T"]
            .columns
            .iter()
            .map(|c| (c.required, c.multiple))
            .collect::<Vec<_>>(),
        [(true, false), (false, false), (true, true), (false, true)]
    );
    assert_eq!(
        arguments(&schema.r#type["T"].attributes["custom.ns_2"]),
        [json!("a\\b")]
    );
}

/// Required EOF delimiters use zero-width EOF spans, including comments and unterminated strings.
#[test]
fn eof_unicode_and_trivia_boundaries() {
    for source in [
        "model X {",
        "prop P { a [",
        "model X @a(",
        "enum E { A ",
        "/* bad",
        "prop P { a \"bad",
    ] {
        let error = parse("eof.idea", source, &inputs()).unwrap_err();
        assert_eq!(error.code, ErrorCode::UnexpectedEof, "{source}");
        assert_eq!(
            (error.start, error.end),
            (Some(source.len()), Some(source.len())),
            "{source}"
        );
    }
    for (source, line, column) in [
        ("// 😀\r\n\tmodel X { id é }", 2, 15),
        ("// 😀\r\tmodel X { id é }", 2, 15),
        ("// 😀\nmodel X { id é }", 2, 14),
    ] {
        let error = parse("unicode.idea", source, &inputs()).unwrap_err();
        assert_eq!((error.line, error.column), (line, column));
        assert_eq!(&source[error.start.unwrap()..error.end.unwrap()], "é");
    }
    assert!(parse("s", "model/*a*/ X/*b*/{/*c*/id/*d*/String/*e*/}", &inputs()).is_ok());
    for source in [
        "prop P { a [1true] }",
        "prop P { a [\"a\"\"b\"] }",
        "prop P { a 1b 2 }",
        "model X @a(1true) { id String }",
        "use \"x\"model X { id String }",
        "prop P { a [1,2] }",
    ] {
        assert!(
            parse("separators.idea", source, &inputs()).is_err(),
            "accepted {source}"
        );
    }
    assert!(parse("s", "enum E { A 1}", &inputs()).is_ok());
}

/// Tokenizer and parser share raw strings, exact numbers, env names, comments and errors.
#[test]
fn shared_tokenizer_scanner_contract() {
    let source = SourceFile::new(
        "tokens.idea",
        "// 😀\r\nmodel X! @open.ns(1 -2.5 true null env(\"KEY\") \"a\\b\") { id String[]? }",
    );
    let tokens = tokenize(&source, ParseLimits::default()).unwrap();
    assert_eq!(tokens[0].raw, "model");
    assert_eq!((tokens[0].span.line, tokens[0].span.column), (2, 1));
    assert!(
        tokens
            .iter()
            .any(|token| token.kind == TokenKind::Environment
                && token.value == TokenValue::Literal(json!("KEY")))
    );
    assert!(tokens.iter().any(|token| token.raw == "String[]?"));
    for token in tokens {
        assert_eq!(&source.text()[token.span.start..token.span.end], token.raw);
    }
    parse_ast(&source, ParseLimits::default()).unwrap();
    for raw in [
        "\"hello 😀\"",
        "0",
        "-0",
        "-0.5",
        "18446744073709551615",
        "true",
        "false",
        "null",
        "env(\"KEY\")",
    ] {
        let source = SourceFile::new("scalar.idea", raw);
        let token = tokenize(&source, ParseLimits::default()).unwrap();
        assert_eq!(token.len(), 1, "{raw}");
        assert_eq!(token[0].span.end, raw.len());
        let mut inputs = inputs();
        inputs.environment.insert("KEY".into(), "v".into());
        assert!(
            parse("s", &format!("prop P {{ a {raw} }}"), &inputs).is_ok(),
            "{raw}"
        );
    }
    for raw in [
        "\"unterminated",
        "-",
        "1.",
        "18446744073709551616",
        "/* never closed",
    ] {
        let token_error =
            tokenize(&SourceFile::new("bad", raw), ParseLimits::default()).unwrap_err();
        let parser_error = parse("bad", &format!("prop P {{ a {raw}"), &inputs()).unwrap_err();
        assert_eq!(token_error.code, parser_error.code, "{raw}");
    }
    assert!(tokenize(&SourceFile::new("s", "@9..a"), ParseLimits::default()).is_err());
    assert!(tokenize(&SourceFile::new("s", "?"), ParseLimits::default()).is_err());
    assert!(tokenize(&SourceFile::new("s", "String[][]"), ParseLimits::default()).is_err());
    assert!(
        tokenize(&SourceFile::new("s", ""), ParseLimits::default())
            .unwrap()
            .is_empty()
    );
}

/// Normalized JSON validates strict structural fields while retaining ordered open metadata.
#[test]
fn normalized_json_validation_and_order() {
    let original = parse(
        "s",
        concat!(
            "plugin \"z\" { b 1 a { x true } } plugin \"a\" {} prop P { z 2 a [] } ",
            "model X @custom.ns(1 false) { z Other[]? a String }"
        ),
        &inputs(),
    )
    .unwrap();
    let json = original.to_json(ParseLimits::default()).unwrap();
    let decoded =
        SchemaConfig::from_json_named("schema.json", &json, ParseLimits::default()).unwrap();
    assert_eq!(original, decoded);
    assert_eq!(
        decoded
            .plugin
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["z", "a"]
    );
    assert_eq!(
        decoded.model["X"]
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["z", "a"]
    );
    for (source, code) in [
        ("[]", ErrorCode::ValidationError),
        (r#"{"unknown":{}}"#, ErrorCode::ValidationError),
        (r#"{"use":[1]}"#, ErrorCode::ValidationError),
        (r#"{"prop":{"P":{"x":1,"x":2}}}"#, ErrorCode::DuplicateName),
        (r#"{"prop":{},"prop":{}}"#, ErrorCode::DuplicateName),
        (
            r#"{"prop":{"P":{}},"model":{"P":{}}}"#,
            ErrorCode::DuplicateName,
        ),
        (r#"{"prop":{"lower":{}}}"#, ErrorCode::ValidationError),
        (r#"{"enum":{"E":{"lower":1}}}"#, ErrorCode::ValidationError),
        (r#"{"enum":{"E":{"A":[]}}}"#, ErrorCode::ValidationError),
        (
            r#"{"type":{"T":{"name":"Other","mutable":true,"columns":[],"attributes":{}}}}"#,
            ErrorCode::ValidationError,
        ),
        (
            r#"{"model":{"X":{"name":"X"}}}"#,
            ErrorCode::MissingProperty,
        ),
        (
            r#"{"prop":{"P":{"x":18446744073709551616}}}"#,
            ErrorCode::InvalidLiteral,
        ),
        (r#"{"prop":{"P":{"x":1e999}}}"#, ErrorCode::InvalidLiteral),
        ("{} trailing", ErrorCode::ValidationError),
    ] {
        let error =
            SchemaConfig::from_json_named("bad.json", source, ParseLimits::default()).unwrap_err();
        assert_eq!(error.code, code, "{source}");
        assert_eq!(error.file, "bad.json");
        assert!(error.start.unwrap() <= error.end.unwrap() && error.end.unwrap() <= source.len());
    }
    // JSON has its own standard escape syntax, independent from Idea's raw string grammar.
    let schema = SchemaConfig::from_json(
        r#"{"prop":{"P":{"x":"a\"b\\c","n":18446744073709551615}}}"#,
        ParseLimits::default(),
    )
    .unwrap();
    assert_eq!(schema.prop["P"]["x"], "a\"b\\c");
    assert_eq!(schema.prop["P"]["n"].as_u64(), Some(u64::MAX));
}

/// Normalized field/attribute shape cannot silently lose malformed fields during import.
#[test]
fn normalized_field_and_reference_mutation_rejections() {
    let config = parse("s", "model X @x(1) { id String }", &inputs()).unwrap();
    let valid = serde_json::to_value(&config).unwrap();
    for (path, replacement) in [
        (vec!["model", "X", "mutable"], json!("true")),
        (vec!["model", "X", "attributes", "x"], json!(1)),
        (vec!["model", "X", "columns"], json!({})),
    ] {
        let mut value = valid.clone();
        let mut site = &mut value;
        for key in path {
            site = site.get_mut(key).unwrap();
        }
        *site = replacement;
        assert!(SchemaConfig::from_json(&value.to_string(), ParseLimits::default()).is_err());
    }
    for field in ["name", "type", "required", "multiple", "attributes"] {
        let mut value = valid.clone();
        value["model"]["X"]["columns"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert_eq!(
            SchemaConfig::from_json(&value.to_string(), ParseLimits::default())
                .unwrap_err()
                .code,
            ErrorCode::MissingProperty
        );
    }
    for (field, replacement) in [
        ("name", json!("Bad")),
        ("type", json!("String[]")),
        ("required", json!(1)),
        ("multiple", json!("false")),
        ("attributes", json!({"Bad":true})),
    ] {
        let mut value = valid.clone();
        value["model"]["X"]["columns"][0][field] = replacement;
        assert!(SchemaConfig::from_json(&value.to_string(), ParseLimits::default()).is_err());
    }
    let mut value = valid.clone();
    let column = value["model"]["X"]["columns"][0].clone();
    value["model"]["X"]["columns"]
        .as_array_mut()
        .unwrap()
        .push(column);
    assert_eq!(
        SchemaConfig::from_json(&value.to_string(), ParseLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::DuplicateName
    );
    let mut value = valid;
    value["model"]["X"]["extra"] = Value::Null;
    assert!(SchemaConfig::from_json(&value.to_string(), ParseLimits::default()).is_err());
    for source in [
        r#"{"config":{},"references":[],"extra":1}"#,
        r#"{"config":{},"references":[{}]}"#,
        r#"{"config":{}}"#,
    ] {
        assert!(CompiledSchema::from_json(source, ParseLimits::default()).is_err());
    }
}

/// Hazardous paths execute only under the parent watchdog.
#[test]
#[ignore]
fn resource_child() {
    for depth in [63, 64, 20_000] {
        let source = format!("prop P {{ a {}0{} }}", "[".repeat(depth), "]".repeat(depth));
        let result = parse("deep", &source, &inputs());
        if depth == 63 {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err().code, ErrorCode::Limit);
        }
    }
    let syntax = parse_ast(
        &SourceFile::new(
            "deep",
            format!("prop P {{ a {}0{} }}", "[".repeat(63), "]".repeat(63)),
        ),
        ParseLimits::default(),
    )
    .unwrap();
    assert_eq!(syntax.clone(), syntax);
    let deep = SourceFile::new("tokens", "[".repeat(20_000));
    assert_eq!(
        tokenize(&deep, ParseLimits::default()).unwrap_err().code,
        ErrorCode::Limit
    );
    let boundary = format!(
        r#"{{"prop":{{"P":{{"x":{}0{}}}}}}}"#,
        "[".repeat(61),
        "]".repeat(61)
    );
    assert!(SchemaConfig::from_json(&boundary, ParseLimits::default()).is_ok());
    let over = format!(
        r#"{{"prop":{{"P":{{"x":{}0{}}}}}}}"#,
        "[".repeat(62),
        "]".repeat(62)
    );
    assert_eq!(
        SchemaConfig::from_json(&over, ParseLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    let huge = format!("prop P {{ a {}.0 }}", "9".repeat(400));
    assert_eq!(
        parse("number", &huge, &inputs()).unwrap_err().code,
        ErrorCode::InvalidLiteral
    );
    let mut small = inputs();
    small.limits = ParseLimits {
        max_bytes: 1024,
        max_depth: 3,
    };
    assert!(parse("s", "prop P { a [[0]] }", &small).is_ok());
    assert_eq!(
        parse("s", "prop P { a [[[0]]] }", &small).unwrap_err().code,
        ErrorCode::Limit
    );
    let syntax = ast("prop P { a [[[0]]] }");
    assert_eq!(
        compile_ast(&syntax, &small).unwrap_err().code,
        ErrorCode::Limit
    );
    small.limits.max_bytes = 2;
    assert_eq!(
        parse("s", "prop P {}", &small).unwrap_err().code,
        ErrorCode::Limit
    );
    assert_eq!(
        tokenize(&SourceFile::new("s", "model X {}"), small.limits)
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    small.limits.max_bytes = 128;
    small.environment.insert("KEY".into(), "x".repeat(100));
    assert_eq!(
        parse("s", "prop P { a env(\"KEY\") b env(\"KEY\") }", &small)
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    small.environment.insert("KEY".into(), "x".repeat(200));
    assert_eq!(parse("s", "", &small).unwrap_err().code, ErrorCode::Limit);
    for depth in [4, 20_000] {
        let json = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        assert_eq!(
            SchemaConfig::from_json(
                &json,
                ParseLimits {
                    max_bytes: 100_000,
                    max_depth: 3
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::Limit
        );
    }
    let json = r#"{"prop":{"P":{"x":[0]}}}"#;
    assert!(
        SchemaConfig::from_json(
            json,
            ParseLimits {
                max_bytes: 128,
                max_depth: 4
            }
        )
        .is_ok()
    );
    assert_eq!(
        SchemaConfig::from_json(
            json,
            ParseLimits {
                max_bytes: 128,
                max_depth: 3
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Limit
    );
    let mut nested = Value::Null;
    for _ in 0..20_000 {
        nested = Value::Array(vec![nested]);
    }
    let mut schema = SchemaConfig::default();
    schema
        .prop
        .insert("P".into(), [("x".into(), nested)].into());
    assert_eq!(
        schema.validate(ParseLimits::default()).unwrap_err().code,
        ErrorCode::Limit
    );
    assert_eq!(
        schema.to_json(ParseLimits::default()).unwrap_err().code,
        ErrorCode::Limit
    );
    // The intentionally hostile caller-owned Value must also be dismantled iteratively.
    let mut nested = schema.prop.get_mut("P").unwrap().shift_remove("x").unwrap();
    while let Value::Array(mut values) = nested {
        nested = values.pop().unwrap();
    }
    let wide = format!("prop P {{ x [{}] }}", "0 ".repeat(10_001));
    assert_eq!(
        parse("wide", &wide, &inputs()).unwrap_err().code,
        ErrorCode::Limit
    );
    let wide = format!(r#"{{"prop":{{"P":{{"x":[{}0]}}}}}}"#, "0,".repeat(10_000));
    assert_eq!(
        SchemaConfig::from_json(&wide, ParseLimits::default())
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    let mut schema = SchemaConfig::default();
    schema
        .prop
        .insert("P".into(), [("x".into(), json!("z".repeat(1024)))].into());
    assert_eq!(
        schema
            .to_json(ParseLimits {
                max_bytes: 128,
                max_depth: 64
            })
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    let source = "model LongName @x(P P P P P P P P P P) { id String }";
    let small = CompileInputs {
        limits: ParseLimits {
            max_bytes: 512,
            max_depth: 64,
        },
        ..Default::default()
    };
    assert_eq!(
        compile_ast_with_references(&ast(source), &small)
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    let mut chain = String::from("prop P { x 1 }\n");
    for index in 0..30 {
        let previous = if index == 0 {
            "P".into()
        } else {
            format!("M{}", index - 1)
        };
        chain.push_str(&format!(
            "model M{index} @x({previous} {previous}) {{ id String }}\n"
        ));
    }
    let compiled = compile_ast_with_references(&ast(&chain), &inputs()).unwrap();
    assert_eq!(
        finalize_with_references(&compiled.config, &compiled.references, &inputs())
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
}

/// Exercise source/JSON depth, byte/collection limits and reference amplification fail-fast.
#[test]
fn resource_limits_under_watchdog() {
    watchdog("resource_child");
}

/// Source coordinates and serialization preserve the public typed identity without data aliasing.
#[test]
fn source_span_serialization_and_invalid_config() {
    let span = SourceSpan {
        file: "child.idea".into(),
        start: 1,
        end: 3,
        line: 2,
        column: 4,
    };
    assert_eq!(
        serde_json::from_str::<SourceSpan>(&serde_json::to_string(&span).unwrap()).unwrap(),
        span
    );
    let mut invalid = parse("s", "model X { id String }", &inputs()).unwrap();
    invalid.model["X"].name = "Y".into();
    assert_eq!(
        invalid.validate(ParseLimits::default()).unwrap_err().code,
        ErrorCode::ValidationError
    );
    assert!(
        parse_ast(&SourceFile::new("s", ""), ParseLimits::default())
            .unwrap()
            .syntax()
            .body
            .is_empty()
    );
}

/// Every exposed syntax-node span belongs to its source, including synthetic normalized nodes.
#[test]
fn all_syntax_ranges_are_valid_utf8() {
    let source = concat!(
        "// 😀\r\nuse \"a\" plugin \"p\" { x \"é\" } ",
        "prop P { a [1 { x null }] } enum E { A 1 } ",
        "type T @x(P) { id String[]? } model X! { id String }"
    );
    let syntax = ast(source);
    let value = serde_json::to_value(syntax.syntax()).unwrap();
    let mut pending = vec![&value];
    while let Some(value) = pending.pop() {
        if let Value::Object(object) = value {
            if let (Some(start), Some(end)) = (object.get("start"), object.get("end")) {
                let start = start.as_u64().unwrap() as usize;
                let end = end.as_u64().unwrap() as usize;
                assert!(start <= end && end <= source.len());
                assert!(source.is_char_boundary(start) && source.is_char_boundary(end));
            }
            pending.extend(object.values());
        } else if let Value::Array(values) = value {
            pending.extend(values);
        }
    }
}
