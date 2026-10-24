//! Upstream compiled fixtures plus corrected grammar, boundedness and named diagnostics.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use stackpress_idea_parser::{CompileInputs, ErrorCode, ParseLimits, finalize, parse};

/// Explicit fixture environment; no test mutates process-global environment state.
fn inputs() -> CompileInputs {
    let mut inputs = CompileInputs::default();
    inputs
        .environment
        .insert("DATABASE_URL".into(), "test".into());
    inputs
}

/// Compile every pinned input and compare both complete config goldens exactly.
#[test]
fn upstream_compiled_fixtures_and_final_references() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for entry in std::fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_some_and(|extension| extension == "idea")
        {
            let source = std::fs::read_to_string(&path).unwrap();
            parse(path.to_str().unwrap(), &source, &inputs()).unwrap();
        }
    }
    let schema = std::fs::read_to_string(root.join("schema.idea")).unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("references.json")).unwrap())
            .unwrap();
    assert_eq!(
        serde_json::to_value(parse("schema.idea", &schema, &inputs()).unwrap()).unwrap(),
        expected
    );
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("final.json")).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(finalize("schema.idea", &schema, &inputs()).unwrap()).unwrap(),
        expected
    );
}

/// Compiled plugin iteration must retain declaration order rather than sort identifiers.
#[test]
fn plugins_and_members_keep_source_order() {
    let schema = parse(
        "s.idea",
        "plugin \"z\" { b 1 a 2 } plugin \"a\" { }",
        &inputs(),
    )
    .unwrap();
    assert_eq!(
        schema.plugin.keys().map(String::as_str).collect::<Vec<_>>(),
        ["z", "a"]
    );
    assert_eq!(
        schema.plugin["z"]
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["b", "a"]
    );
}

/// Test each approved modifier and the deliberate corrections to accidental TS behavior.
#[test]
fn corrected_grammar_table() {
    let schema = parse(
        "s",
        "model X! { a String b String? c String[] d String[]? }",
        &inputs(),
    )
    .unwrap();
    assert!(!schema.model["X"].mutable);
    let flags: Vec<_> = schema.model["X"]
        .columns
        .iter()
        .map(|column| (column.required, column.multiple))
        .collect();
    assert_eq!(
        flags,
        [(true, false), (false, false), (true, true), (false, true)]
    );
    for source in [
        "model X { a String?? }",
        "model X { a String?[] }",
        "model X { a String[][] }",
        "model lower { a String }",
        "enum X { lower 1 }",
        "model X @9..A { a String }",
        "model X { a String a Number }",
        "prop X { a 1 a 2 }",
        "enum X { A 1 A 2 }",
        "model X @a @a { a String }",
        "model X { a String } type X { a String }",
        "prop X { a Elsewhere }",
        "prop X { a 18446744073709551616 }",
        "model X {",
        "model X @a(Role.ADMIN) { a String }",
        "use \"x\" trailing",
        "prop X { a [1,2] }",
    ] {
        assert!(parse("s", source, &inputs()).is_err(), "accepted {source}");
    }
    let source = format!("prop X {{ a {}.0 }}", "9".repeat(400));
    assert_eq!(
        parse("s", &source, &inputs()).unwrap_err().code,
        ErrorCode::InvalidLiteral
    );
    let values = parse(
        "s",
        "prop X { a 18446744073709551615 b -9223372036854775808 }",
        &inputs(),
    )
    .unwrap();
    assert_eq!(values.prop["X"]["a"].as_u64(), Some(u64::MAX));
}

/// Environment is explicit in all scalar positions and never taken from the process.
#[test]
fn environment_and_reference_resolution() {
    let schema = parse("s", "enum X { A env(\"DATABASE_URL\") }", &inputs()).unwrap();
    assert_eq!(schema.r#enum["X"]["A"], "test");
    let error = parse("s", "prop X { a env(\"MISSING\") }", &inputs()).unwrap_err();
    assert_eq!(error.code, ErrorCode::MissingEnvironment);
    let mut empty = inputs();
    empty.missing_env_is_empty = true;
    assert_eq!(
        parse("s", "prop X { a env(\"MISSING\") }", &empty)
            .unwrap()
            .prop["X"]["a"],
        ""
    );
    let source = "prop A { x true } model X @thing(A) { id String @x() }";
    assert!(finalize("s", source, &inputs()).is_ok());
    assert_eq!(
        finalize("s", "model X @thing(A) { id String }", &inputs())
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );
    let source = "// head\n/* note */ model X { id String // line\n }";
    assert!(parse("s", source, &inputs()).is_ok());
}

/// All public diagnostic endpoints are valid byte boundaries, including EOF and multibyte errors.
#[test]
fn source_coordinates_are_valid_and_actionable() {
    for source in [
        "/* bad",
        "model X {",
        "// 😀\nmodel X { id é }",
        "prop X { a \"bad",
    ] {
        let error = parse("child.idea", source, &inputs()).unwrap_err();
        for offset in [error.start.unwrap(), error.end.unwrap()] {
            assert!(offset <= source.len() && source.is_char_boundary(offset));
        }
        assert!(error.to_string().starts_with("child.idea:"));
    }
    let error = parse("s", "// 😀\r\nmodel X { id é }", &inputs()).unwrap_err();
    assert_eq!((error.line, error.column), (2, 14));
    assert_eq!(error.end.unwrap() - error.start.unwrap(), 2);
}

/// Lowerable depth/byte limits hold at the exact accepted boundary and at one beyond.
#[test]
fn finite_limits_and_reference_amplification() {
    let mut inputs = inputs();
    inputs.limits = ParseLimits {
        max_bytes: 1024,
        max_depth: 3,
    };
    assert!(parse("s", "prop X { a [[1]] }", &inputs).is_ok());
    assert_eq!(
        parse("s", "prop X { a [[[1]]] }", &inputs)
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    inputs.limits.max_bytes = 2;
    assert_eq!(
        parse("s", "prop X { }", &inputs).unwrap_err().code,
        ErrorCode::Limit
    );
    // Each prior prop doubles the last; source stays tiny while expansion becomes large.
    let mut source = String::from("prop A { x 1 }\n");
    // References in attributes include prior model values, exercising copied nested metadata.
    for index in 0..25 {
        let prior = if index == 0 {
            "A".into()
        } else {
            format!("M{}", index - 1)
        };
        source.push_str(&format!(
            "model M{index} @x({prior} {prior}) {{ id String }}\n"
        ));
    }
    assert_eq!(
        finalize("s", &source, &CompileInputs::default())
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
}

/// Child helper is invoked only by its deadline-enforcing parent test.
#[test]
#[ignore]
fn deep_child() {
    let source = format!(
        "prop X {{ x {}1{} }}",
        "[".repeat(20_000),
        "]".repeat(20_000)
    );
    assert_eq!(
        parse("deep.idea", &source, &inputs()).unwrap_err().code,
        ErrorCode::Limit
    );
}

/// The formerly aborting input must return normally, without hanging or overflowing a test thread.
#[test]
fn deep_input_returns_a_diagnostic_under_process_watchdog() {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "deep_child"])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("deep-input child timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
