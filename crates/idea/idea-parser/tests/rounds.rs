//! Frozen public-shape and branch-risk contracts added during behavior-preserving rounds.

use serde_json::{Value, json};
use stackpress_idea_parser::{
    AttributeValue, CompileInputs, CompiledSchema, DeclarationKind, ErrorCode, ParseLimits,
    Reference, SourceFile, ValuePath, compile_ast_with_references, finalize_with_references, parse,
    parse_ast, tokenize,
};

/// Build source-ordered type references, including nested arrays/objects and column metadata.
fn compiled() -> CompiledSchema {
    let source = SourceFile::new(
        "refs.idea",
        "prop P { x 1 } type T @a(P [P] { x P }) { id String @b(P) }",
    );
    let syntax = parse_ast(&source, ParseLimits::default()).unwrap();
    compile_ast_with_references(&syntax, &CompileInputs::default()).unwrap()
}

/// Freeze the interoperable reference/token/error shapes rather than their private module homes.
#[test]
fn exported_json_shapes_and_reference_order_remain_frozen() {
    let compiled = compiled();
    assert_eq!(
        compiled
            .references
            .iter()
            .map(|reference| reference.path.clone())
            .collect::<Vec<_>>(),
        [
            vec![ValuePath::Index(0)],
            vec![ValuePath::Index(1), ValuePath::Index(0)],
            vec![ValuePath::Index(2), ValuePath::Key("x".into())],
            vec![ValuePath::Index(0)]
        ]
    );
    assert_eq!(
        serde_json::to_value(&compiled.references[0]).unwrap(),
        json!({
            "kind":"type", "declaration":"T", "column":null, "attribute":"a",
            "path":[{"Index":0}], "target":"P", "forward":false,
            "span":{"file":"refs.idea","start":25,"end":26,"line":1,"column":26}
        })
    );
    assert_eq!(compiled.references[3].column.as_deref(), Some("id"));
    let tokens = tokenize(
        &SourceFile::new("env.idea", "env(\"KEY\")"),
        ParseLimits::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(tokens).unwrap(),
        json!([{
            "kind":"Environment", "value":{"Literal":"KEY"}, "raw":"env(\"KEY\")",
            "span":{"file":"env.idea","start":0,"end":10,"line":1,"column":1}
        }])
    );
    let error = parse(
        "env.idea",
        "prop P { a env(\"KEY\") }",
        &CompileInputs::default(),
    )
    .unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        json!({
            "code":"MissingEnvironment", "message":"Missing explicit environment input",
            "start":11,"end":21,"file":"env.idea","line":1,"column":12,"related":[]
        })
    );
}

/// Reject malformed structural sites before resolving any references or mutating caller data.
#[test]
fn invalid_reference_site_decision_table() {
    let compiled = compiled();
    let original = serde_json::to_value(&compiled).unwrap();
    let changes: Vec<(&str, Reference)> = [
        ("reversed range", 0),
        ("oversized range", 1),
        ("zero line", 2),
        ("zero column", 3),
        ("empty path", 4),
        ("deep path", 5),
        ("bad declaration", 6),
        ("bad target", 7),
        ("bad attribute", 8),
        ("bad column", 9),
        ("wrong kind", 10),
        ("missing owner", 11),
        ("missing column", 12),
        ("missing attribute", 13),
        ("key at root", 14),
        ("missing argument", 15),
        ("wrong nested kind", 16),
        ("missing object key", 17),
        ("stale target", 18),
    ]
    .into_iter()
    .map(|(name, case)| {
        let mut reference = compiled.references[0].clone();
        match case {
            0 => reference.span.start = reference.span.end + 1,
            1 => reference.span.end = ParseLimits::default().max_bytes + 1,
            2 => reference.span.line = 0,
            3 => reference.span.column = 0,
            4 => reference.path.clear(),
            5 => reference.path = vec![ValuePath::Index(0); ParseLimits::HARD_DEPTH + 1],
            6 => reference.declaration = "lower".into(),
            7 => reference.target = "lower".into(),
            8 => reference.attribute = "Bad".into(),
            9 => reference.column = Some("Bad".into()),
            10 => reference.kind = DeclarationKind::Model,
            11 => reference.declaration = "Absent".into(),
            12 => reference.column = Some("missing".into()),
            13 => reference.attribute = "missing".into(),
            14 => reference.path = vec![ValuePath::Key("x".into())],
            15 => reference.path = vec![ValuePath::Index(99)],
            16 => reference.path = vec![ValuePath::Index(1), ValuePath::Key("x".into())],
            17 => reference.path = vec![ValuePath::Index(2), ValuePath::Key("missing".into())],
            18 => reference.target = "Absent".into(),
            _ => unreachable!(),
        }
        (name, reference)
    })
    .collect();
    for (name, reference) in changes {
        let error = finalize_with_references(
            &compiled.config,
            std::slice::from_ref(&reference),
            &CompileInputs::default(),
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::ValidationError, "{name}");
        assert_eq!(error.message, "Invalid typed reference site", "{name}");
        assert_eq!(error.file, reference.span.file, "{name}");
        assert_eq!(
            (error.start, error.end),
            (Some(reference.span.start), Some(reference.span.end)),
            "{name}"
        );
        assert_eq!(serde_json::to_value(&compiled).unwrap(), original);
    }
    // A flag is valid metadata but cannot hold an argument-addressed reference.
    let mut flag = compiled.config.clone();
    flag.r#type["T"]
        .attributes
        .insert("a".into(), AttributeValue::Boolean(true));
    assert_eq!(
        finalize_with_references(&flag, &compiled.references[..1], &CompileInputs::default())
            .unwrap_err()
            .code,
        ErrorCode::ValidationError
    );
}

/// Lookahead and consuming scanners agree at comment, delimiter and non-ASCII boundaries.
#[test]
fn scanner_lookahead_and_error_precedence() {
    for source in [
        "model/*note*/ X { id String }",
        "prop P { x [true/*note*/false] }",
        "type T @x(P/*note*/[P]) { id String }",
        "prop P { x { _a 1 } }",
    ] {
        let file = SourceFile::new("trivia.idea", source);
        assert!(parse_ast(&file, ParseLimits::default()).is_ok(), "{source}");
        assert!(tokenize(&file, ParseLimits::default()).is_ok(), "{source}");
    }
    for (source, start, end) in [
        ("model lower { id String }", 6, 11),
        ("prop P { x é }", 11, 13),
        ("model X { id é }", 13, 15),
        ("model X { id", 12, 12),
    ] {
        let error = parse("bad.idea", source, &CompileInputs::default()).unwrap_err();
        assert_eq!(
            (error.start, error.end),
            (Some(start), Some(end)),
            "{source}"
        );
    }
    let compiled = compiled();
    let mut stale = compiled.references[0].clone();
    stale.attribute = "absent".into();
    let mut uses = compiled.config.clone();
    uses.r#use.push("child.idea".into());
    // Site validation precedes the outstanding-import guard; keep that visible error order.
    assert_eq!(
        finalize_with_references(&uses, &[stale], &CompileInputs::default())
            .unwrap_err()
            .message,
        "Invalid typed reference site"
    );
    let final_schema = finalize_with_references(
        &compiled.config,
        &compiled.references,
        &CompileInputs::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&final_schema.r#type["T"].attributes["a"]).unwrap(),
        json!([{"x":1},[{"x":1}],{"x":{"x":1}}])
    );
    assert_eq!(
        final_schema.r#type["T"].columns[0].attributes["b"],
        AttributeValue::Array(vec![json!({"x":1})])
    );
    assert!(matches!(
        serde_json::to_value(final_schema).unwrap(),
        Value::Object(_)
    ));
}

/// Run resource-boundary regressions only inside the parent's finite-deadline child.
#[test]
#[ignore]
fn round_limits_child() {
    // Idea keeps raw tab bytes, while JSON escapes each tab into two output bytes.
    let source = format!(
        "prop P {{ text \"{}\" }} model X @a(P) {{ id String }}",
        "\t".repeat(80)
    );
    let inputs = CompileInputs {
        limits: ParseLimits {
            max_bytes: source.len(),
            max_depth: 64,
        },
        ..Default::default()
    };
    assert!(parse("limit.idea", &source, &inputs).is_ok());
    let error = stackpress_idea_parser::finalize("limit.idea", &source, &inputs).unwrap_err();
    let start = source.find("@a(P)").unwrap() + 3;
    assert_eq!(error.code, ErrorCode::Limit);
    assert_eq!(error.message, "Reference expansion budget exceeded");
    assert_eq!((error.start, error.end), (Some(start), Some(start + 1)));
    let limits = ParseLimits {
        max_bytes: 1024,
        max_depth: 0,
    };
    let error = tokenize(&SourceFile::new("limit.idea", "("), limits).unwrap_err();
    assert_eq!(error.code, ErrorCode::Limit);
    assert_eq!((error.start, error.end), (Some(0), Some(0)));
    assert!(tokenize(&SourceFile::new("limit.idea", ")"), limits).is_ok());
    let json = r#"{"prop":{"P":{"x":"😀","y":18446744073709551616}}}"#;
    let error = stackpress_idea_parser::SchemaConfig::from_json_named(
        "unicode.json",
        json,
        ParseLimits::default(),
    )
    .unwrap_err();
    let start = json.find("18446744073709551616").unwrap();
    assert_eq!(error.code, ErrorCode::InvalidLiteral);
    assert_eq!((error.start, error.end), (Some(start), Some(start + 20)));
    assert_eq!(error.column, json[..start].chars().count() + 1);
}

/// Kill and reap a hazardous child if its resource checks fail to return promptly.
#[test]
fn reference_copy_and_span_limits_under_watchdog() {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "round_limits_child"])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "resource child failed: {status}");
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("resource child exceeded its watchdog deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
