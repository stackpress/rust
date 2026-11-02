//! Observable engine contracts, malformed-input matrices, and exact budget boundaries.

use super::*;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "../../tests/support/watchdog.rs"]
mod watchdog;

/// Construct a valid engine while varying only the requested budget.
fn limited(change: impl FnOnce(&mut TemplateLimits)) -> TemplateEngine {
    let mut options = TemplateOptions::default();
    change(&mut options.limits);
    TemplateEngine::new(options).unwrap()
}

/// Read a failure category without requiring successful templates to expose their source in Debug.
fn kind<T>(result: TemplateResult<T>) -> TemplateErrorKind {
    result.err().unwrap().kind()
}

/// Scalars retain their spelling, while all inserted markers remain ordinary text.
#[test]
fn scalars_and_insertions_are_inert() {
    let data = json!({"s":"{{missing}} [%missing%]", "n":-4, "b":false, "values":[{"x":"ok"}]});
    assert_eq!(
        render("{{s}}|{{n}}|{{b}}|{{values.0.x}}", &data).unwrap(),
        "{{missing}} [%missing%]|-4|false|ok"
    );
    assert_eq!(
        render("{{{{name}} }}}} ", &json!({})).unwrap(),
        "{{name}} }}}} "
    );
    assert_eq!(render(" \r\né\n", &json!({})).unwrap(), " \r\né\n");
    for value in [Value::Null, json!([]), json!({})] {
        assert_eq!(
            kind(render("{{value}}", &json!({"value":value}))),
            TemplateErrorKind::InvalidType
        );
    }
}

/// Repeated block identities bind to their nearest opener and keep else arms separate.
#[test]
fn identical_conditions_and_loops_nest_reliably() {
    let source = "{{#?:active}}A{{#?:active}}B{{|}}bad{{/?:active}}{{|}}bad{{/?:active}}";
    assert_eq!(render(source, &json!({"active":true})).unwrap(), "AB");
    assert_eq!(render(source, &json!({"active":false})).unwrap(), "bad");
    let source = "{{#!:active}}{{#!:active}}N{{|}}bad{{/!:active}}{{|}}bad{{/!:active}}";
    assert_eq!(render(source, &json!({"active":false})).unwrap(), "N");
    let source = "{{#@:list x}}{{x}}{{#@:list x}}{{x}}{{/@:list}}{{x}}{{/@:list}}";
    assert_eq!(render(source, &json!({"list":[1,2]})).unwrap(), "11212122");
}

/// Lookup and helper effects occur only in selected branches or visited loop bodies.
#[test]
fn branches_are_lazy_but_compile_checks_every_arm() {
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    fn tick(_: &[&Value]) -> TemplateResult<String> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        Ok("tick".into())
    }
    let mut engine = TemplateEngine::default();
    engine.register_helper("tick", tick).unwrap();
    let source = "{{#?:ok}}yes{{|}}{{tick:missing}}{{/?:ok}}{{#@:empty x}}{{missing}}{{/@:empty}}";
    assert_eq!(
        engine
            .render(source, &json!({"ok":true,"empty":[]}))
            .unwrap(),
        "yes"
    );
    assert_eq!(CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(
        kind(engine.render(source, &json!({"ok":false,"empty":[]}))),
        TemplateErrorKind::MissingValue
    );
    assert_eq!(
        kind(engine.compile("{{#?:ok}}yes{{|}}{{unknown:x}}{{/?:ok}}")),
        TemplateErrorKind::UnknownHelper
    );
    assert_eq!(
        kind(engine.compile("{{#?:ok}}yes{{|}}{{bad..path}}{{/?:ok}}")),
        TemplateErrorKind::Syntax
    );
}

/// Lexical aliases retain parents and the root without spreading object members.
#[test]
fn loop_scope_and_metadata_are_explicit() {
    let data = json!({"title":"root", "rows":[{"title":"item","children":[2,3]}]});
    let source = "{{#@:rows row ri}}{{title}}/{{row.title}}/{{ri}}:{{#@:row.children child key}}{{@root.title}}{{row.title}}{{child}}{{key}}{{@key}}{{@index}}{{@first}}{{@last}}{{@odd}}{{@even}};{{/@:row.children}}{{/@:rows}}";
    assert_eq!(
        render(source, &data).unwrap(),
        "root/item/0:rootitem2000truefalsefalsetrue;rootitem3111falsetruetruefalse;"
    );
    assert_eq!(
        render("{{#@:rows}}{{@this.title}}/{{@key}}{{/@:rows}}", &data).unwrap(),
        "item/0"
    );
    assert_eq!(
        kind(render("{{#@:rows row}}{{children}}{{/@:rows}}", &data)),
        TemplateErrorKind::MissingValue
    );
}

/// Object entries preserve insertion order rather than sorting their keys.
#[test]
fn object_iteration_retains_entry_order() {
    let mut entries = serde_json::Map::new();
    entries.insert("z".into(), json!(1));
    entries.insert("a".into(), json!(2));
    let source = "{{#@:entries value key}}{{key}}={{value}}@{{@index}};{{/@:entries}}";
    assert_eq!(
        render(source, &json!({"entries":entries})).unwrap(),
        "z=1@0;a=2@1;"
    );
    assert_eq!(
        render(
            "{{#@:entries}}{{missing}}{{/@:entries}}",
            &json!({"entries":{}})
        )
        .unwrap(),
        ""
    );
}

/// Actual external structural templates preserve braces, literals and source whitespace.
#[test]
fn external_template_and_custom_delimiters_preserve_rust() {
    let engine = TemplateEngine::new(TemplateOptions {
        delimiters: ["[%".into(), "%]".into()],
        ..Default::default()
    })
    .unwrap();
    let template = engine
        .compile(include_str!("fixtures/contract.rs.tpl"))
        .unwrap();
    let source = template.render(&json!({"model":"Profile","fields":[{"wire_literal":"\"id\"","member":"id","ty":"String"}]})).unwrap();
    assert_eq!(
        source,
        "//! Owned Profile contract.\npub struct Profile {\n    #[serde(rename = \"id\")]\n    pub id: String,\n}\n// Runtime format syntax remains literal: format!(\"{{{}}}\", value).\n"
    );
    assert_eq!(
        engine
            .render_pairs("[%[%marker%] {{rust}} [%x%]", &[("x", "[%inert%]")])
            .unwrap(),
        "[%marker%] {{rust}} [%inert%]"
    );
    let unicode = TemplateEngine::new(TemplateOptions {
        delimiters: ["«".into(), "»".into()],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        unicode.render_pairs("««x»«x»", &[("x", "é")]).unwrap(),
        "«x»é"
    );
}

/// Bad block structures fail predictably instead of swallowing or emitting fragments.
#[test]
fn malformed_grammar_is_rejected() {
    let engine = TemplateEngine::default();
    for source in [
        "{{",
        "{{}}",
        "{{|}}",
        "{{/?:x}}",
        "{{#?:x}}",
        "{{#?:x}}{{/!:x}}",
        "{{#?:x}}{{|}}{{|}}{{/?:x}}",
        "{{#@:x}}{{|}}{{/@:x}}",
        "{{#?:x arg}}{{/?:x}}",
        "{{#@:x a a}}{{/@:x}}",
        "{{#@:x @this}}{{/@:x}}",
        "{{#@:x a b c}}{{/@:x}}",
        "{{#bad:x}}{{/bad:x}}",
        "{{a..b}}",
        "{{x[0]}}",
        "{{x.01}}",
        "{{x.method()}}",
        "{{A:}}",
        "{{.:x 'x'}}",
        "{{.:x 01}}",
        "{{.:x \"unterminated}}",
        "{{.:x \"a\"b}}",
    ] {
        assert_eq!(
            kind(engine.compile(source)),
            TemplateErrorKind::Syntax,
            "{source}"
        );
    }
}

/// Runtime invalid-type and missing-value branches remain distinct and strict.
#[test]
fn strict_lookups_and_operation_types() {
    for data in [Value::Null, json!(true), json!([])] {
        assert_eq!(kind(render("text", &data)), TemplateErrorKind::InvalidData);
    }
    assert_eq!(
        kind(render("{{x}}", &json!({}))),
        TemplateErrorKind::MissingValue
    );
    assert_eq!(
        kind(render("{{x.a}}", &json!({"x":1}))),
        TemplateErrorKind::InvalidType
    );
    assert_eq!(
        kind(render(
            "{{x.1000000000000000000000000000000}}",
            &json!({"x":[]})
        )),
        TemplateErrorKind::MissingValue
    );
    for value in [Value::Null, json!(0), json!("true"), json!([])] {
        assert_eq!(
            kind(render("{{#?:x}}yes{{/?:x}}", &json!({"x":value}))),
            TemplateErrorKind::InvalidType
        );
    }
    assert_eq!(
        kind(render("{{#@:x}}{{/@:x}}", &json!({"x":false}))),
        TemplateErrorKind::InvalidType
    );
}

/// Built-ins use explicit scalar contracts and Unicode character clipping.
#[test]
fn builtin_helpers_have_typed_arguments() {
    let data = json!({"text":"éßabc", "none":null, "no":false, "zero":0, "empty":""});
    assert_eq!(
        render(
            "{{A:text}}|{{a:text}}|{{.:text 2 \" ...\"}}|{{.:text 0 \"!\"}}",
            &data
        )
        .unwrap(),
        "ÉSSABC|éßabc|éß ...|!"
    );
    assert_eq!(render("{{.:text 20}}", &data).unwrap(), "éßabc");
    assert_eq!(
        render("{{|:none no zero empty \"a \\\"quote\\\"\"}}", &data).unwrap(),
        "a \"quote\""
    );
    assert_eq!(render("{{|:none no zero empty}}", &data).unwrap(), "");
    assert_eq!(
        kind(render("{{|:text missing}}", &data)),
        TemplateErrorKind::MissingValue
    );
    for source in [
        "{{A:zero}}",
        "{{a:text 1}}",
        "{{.:text}}",
        "{{.:text -1}}",
        "{{.:text 1.5}}",
        "{{.:text 1 true}}",
        "{{|:text @root}}",
    ] {
        assert_eq!(kind(render(source, &data)), TemplateErrorKind::InvalidType);
    }
}

/// Custom helpers are explicit, immutable snapshots and insert returned markers once.
#[test]
fn helper_registration_and_failures_are_typed() {
    fn inert(_: &[&Value]) -> TemplateResult<String> {
        Ok("{{missing}}".into())
    }
    fn fail(_: &[&Value]) -> TemplateResult<String> {
        Err(TemplateError::helper("compile-time helper failed"))
    }
    let mut engine = TemplateEngine::default();
    engine.register_helper("inert", inert).unwrap();
    let before = engine.compile("{{inert:x}}").unwrap();
    engine.register_helper("fail", fail).unwrap();
    assert!(!before.helpers.contains_key("fail"));
    assert_eq!(before.render(&json!({"x":1})).unwrap(), "{{missing}}");
    assert_eq!(
        kind(engine.render("{{fail:x}}", &json!({"x":1}))),
        TemplateErrorKind::Helper
    );
    for name in ["A", ".", "?", "inert"] {
        assert_eq!(
            kind(engine.register_helper(name, inert)),
            TemplateErrorKind::DuplicateHelper
        );
    }
    assert_eq!(
        kind(engine.register_helper("bad-name", inert)),
        TemplateErrorKind::Syntax
    );
}

/// Scalar pairs reject duplicates and input limits before string copies are created.
#[test]
fn pairs_are_a_bounded_map_convenience() {
    let engine = TemplateEngine::default();
    assert_eq!(
        engine
            .render_pairs("{{ty}}", &[("ty", "Vec<String>")])
            .unwrap(),
        "Vec<String>"
    );
    assert_eq!(
        kind(engine.render_pairs("", &[("a", "x"), ("a", "y")])),
        TemplateErrorKind::InvalidData
    );
    assert_eq!(
        kind(limited(|limits| limits.max_data_bytes = 3).render_pairs("", &[("a", "large")])),
        TemplateErrorKind::Limit
    );
    assert_eq!(
        kind(limited(|limits| limits.max_data_depth = 1).render_pairs("", &[("a", "x")])),
        TemplateErrorKind::Limit
    );
    assert_eq!(
        limited(|limits| limits.max_data_depth = 1)
            .render_pairs("", &[])
            .unwrap(),
        ""
    );
}

/// Options cannot enable ambiguous delimiters, unlimited sentinels or unsafe recursion.
#[test]
fn invalid_options_fail_without_panics() {
    for pair in [
        ["", "}}"],
        ["x", "x"],
        ["x", "xy"],
        ["{{", " "],
        ["abcdefghijklmnopq", "!"],
    ] {
        let options = TemplateOptions {
            delimiters: pair.map(str::to_owned),
            ..Default::default()
        };
        assert_eq!(
            kind(TemplateEngine::new(options)),
            TemplateErrorKind::InvalidOptions
        );
    }
    for change in [
        |limits: &mut TemplateLimits| limits.max_template_bytes = 0,
        |limits: &mut TemplateLimits| limits.max_data_bytes = 0,
        |limits: &mut TemplateLimits| limits.max_output_bytes = 0,
        |limits: &mut TemplateLimits| limits.max_iterations = 0,
        |limits: &mut TemplateLimits| limits.max_steps = 0,
        |limits: &mut TemplateLimits| limits.max_nesting = 129,
        |limits: &mut TemplateLimits| limits.max_data_depth = 0,
    ] {
        let mut options = TemplateOptions::default();
        change(&mut options.limits);
        assert_eq!(
            kind(TemplateEngine::new(options)),
            TemplateErrorKind::InvalidOptions
        );
    }
}

/// Limits include UTF-8 bytes, aggregate nested work, and exact inclusive boundaries.
#[test]
fn source_output_iteration_and_step_boundaries() {
    assert_eq!(
        limited(|limits| limits.max_template_bytes = 2)
            .render("é", &json!({}))
            .unwrap(),
        "é"
    );
    assert_eq!(
        kind(limited(|limits| limits.max_template_bytes = 1).compile("é")),
        TemplateErrorKind::Limit
    );
    assert_eq!(
        limited(|limits| limits.max_output_bytes = 2)
            .render("{{x}}", &json!({"x":"é"}))
            .unwrap(),
        "é"
    );
    assert_eq!(
        kind(limited(|limits| limits.max_output_bytes = 1).render("{{x}}", &json!({"x":"é"}))),
        TemplateErrorKind::Limit
    );
    let source = "{{#@:x}}{{#@:x}}{{/@:x}}{{/@:x}}";
    assert_eq!(
        limited(|limits| limits.max_iterations = 6)
            .render(source, &json!({"x":[1,2]}))
            .unwrap(),
        ""
    );
    assert_eq!(
        kind(limited(|limits| limits.max_iterations = 5).render(source, &json!({"x":[1,2]}))),
        TemplateErrorKind::Limit
    );
    assert_eq!(
        limited(|limits| limits.max_steps = 2)
            .render("{{x}}", &json!({"x":1}))
            .unwrap(),
        "1"
    );
    assert_eq!(
        kind(limited(|limits| limits.max_steps = 1).render("{{x}}", &json!({"x":1}))),
        TemplateErrorKind::Limit
    );
    assert_eq!(
        limited(|limits| {
            limits.max_steps = usize::MAX;
            limits.max_iterations = usize::MAX;
        })
        .render("ok", &json!({}))
        .unwrap(),
        "ok"
    );
}

/// Borrowed JSON accounting matches serde's compact encoding, including control escapes.
#[test]
fn data_bytes_match_compact_json_at_the_boundary() {
    for value in [
        json!({}),
        json!({"é":"\0\u{0001}\t\r\n\u{0008}\u{000c}\\\"é"}),
        json!({"a":[null,true,false,-2,1.25,{},[]]}),
    ] {
        let length = serde_json::to_vec(&value).unwrap().len();
        assert!(
            limited(|limits| limits.max_data_bytes = length)
                .render("", &value)
                .is_ok()
        );
        assert_eq!(
            kind(limited(|limits| limits.max_data_bytes = length - 1).render("", &value)),
            TemplateErrorKind::Limit
        );
    }
}

/// Codegen may raise input overhead budgets while retaining an independent output cap.
#[test]
fn data_budget_can_be_raised_without_raising_output() {
    let value = json!({"padding":"x".repeat(1024*1024),"source":"ok"});
    assert_eq!(kind(render("{{source}}", &value)), TemplateErrorKind::Limit);
    let engine = limited(|limits| {
        limits.max_data_bytes = 2 * 1024 * 1024;
        limits.max_output_bytes = 2;
    });
    assert_eq!(engine.render("{{source}}", &value).unwrap(), "ok");
    assert_eq!(kind(engine.render("xxx", &value)), TemplateErrorKind::Limit);
}

/// Nested data and template trees stop before descending beyond their chosen ceiling.
#[test]
fn depth_limits_are_checked_before_descent() {
    let engine = limited(|limits| {
        limits.max_nesting = 2;
        limits.max_data_depth = 3;
    });
    let source = "{{#?:x}}{{#?:x}}yes{{/?:x}}{{/?:x}}";
    assert_eq!(engine.render(source, &json!({"x":true})).unwrap(), "yes");
    assert_eq!(
        kind(engine.compile(&format!("{{{{#?:x}}}}{source}{{{{/?:x}}}}"))),
        TemplateErrorKind::Limit
    );
    assert!(engine.render("", &json!({"x":[1]})).is_ok());
    assert_eq!(
        kind(engine.render("", &json!({"x":[[1]]}))),
        TemplateErrorKind::Limit
    );
}

/// A subprocess deadline makes a broken nesting guard fail without hanging the test runner.
#[test]
fn hostile_nesting_has_an_external_deadline() {
    if std::env::var_os("STACKPRESS_TEMPLATE_HOSTILE_CHILD").is_some() {
        let engine = limited(|limits| limits.max_nesting = 128);
        let allowed = format!("{}ok{}", "{{#?:x}}".repeat(128), "{{/?:x}}".repeat(128));
        assert_eq!(engine.render(&allowed, &json!({"x":true})).unwrap(), "ok");
        assert_eq!(
            kind(engine.compile(&"{{#?:x}}".repeat(10_000))),
            TemplateErrorKind::Limit
        );
        return;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "template::tests::hostile_nesting_has_an_external_deadline",
        ])
        .env("STACKPRESS_TEMPLATE_HOSTILE_CHILD", "1");
    let completion = watchdog::run(
        &mut command,
        "template-nesting",
        std::time::Duration::from_secs(10),
    )
    .unwrap();
    assert!(!completion.timed_out, "{}", completion.output);
    assert!(completion.status.success(), "{}", completion.output);
}

/// Failures include byte spans without echoing the rejected source value.
#[test]
fn errors_are_located_and_do_not_capture_data() {
    let error = render("é{{missing}}", &json!({"private":"SECRET"})).unwrap_err();
    assert_eq!(error.span(), Some(2..13));
    assert!(!format!("{error:?} {error}").contains("SECRET"));
    assert_eq!(
        kind(TemplateEngine::default().render_pairs("{{", &[("x", "a"), ("x", "b")])),
        TemplateErrorKind::Syntax
    );
}

/// Reused templates share neither successful output nor exhausted render counters.
#[test]
fn concurrent_and_failed_renders_have_independent_state() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Template>();
    assert_send_sync::<TemplateEngine>();
    let template = limited(|limits| limits.max_output_bytes = 4)
        .compile("{{x}}")
        .unwrap();
    assert_eq!(
        kind(template.render_pairs(&[("x", "too long")])),
        TemplateErrorKind::Limit
    );
    std::thread::scope(|scope| {
        let first = scope.spawn(|| template.render_pairs(&[("x", "one")]).unwrap());
        let second = scope.spawn(|| template.render_pairs(&[("x", "two")]).unwrap());
        assert_eq!(first.join().unwrap(), "one");
        assert_eq!(second.join().unwrap(), "two");
    });
}
