//! G36 parser/format/state proofs over actual normalized public metadata.
#![cfg(feature = "codegen")]
use serde_json::{Value, json};
use stackpress_framework_admin::{
    codegen::register_components,
    transfer::{self, Limits, Phase, Upload},
};
use stackpress_framework_schema::{FieldAccess, Schema, UiModel, codegen::DefinitionRegistry};
fn model() -> UiModel {
    let config=stackpress_idea_parser::parse("transfer.idea","model Example { id String @id text String? count Integer @default(0) flag Boolean @default(false) secret String? }",&Default::default()).unwrap();
    let mut registry = DefinitionRegistry::standard();
    for name in ["id", "text", "count", "flag"] {
        registry = registry
            .with_field_policy(
                "Example",
                name,
                FieldAccess {
                    readable: true,
                    writable: true,
                    publish_default: true,
                    sensitive: false,
                },
            )
            .unwrap();
    }
    registry = registry
        .with_field_policy(
            "Example",
            "secret",
            FieldAccess {
                sensitive: true,
                ..Default::default()
            },
        )
        .unwrap();
    Schema::normalize(
        &config,
        &register_components(registry, vec![]).unwrap(),
        Default::default(),
    )
    .unwrap()
    .project_ui()
    .unwrap()
    .models
    .remove(0)
}
fn upload(csv: &str, guards: bool) -> Upload {
    Upload {
        csv: csv.into(),
        decode_export_guards: guards,
        confirm: false,
    }
}
#[test]
fn strict_parser_and_lowerable_budgets() {
    let limits = Limits::default();
    for bytes in [
        b"".as_slice(),
        b"a",
        b"a\n",
        b"a\n\"open",
        b"a\nx\"y",
        b"a\n\"x\"x",
        b"a\r1",
        b"a\n\xff",
    ] {
        assert!(transfer::parse(bytes, limits).is_err(), "{bytes:?}");
    }
    assert_eq!(
        transfer::parse(b"a,b\r\n\"line\nquote\"\"\",\r\n", limits).unwrap(),
        vec![vec!["a", "b"], vec!["line\nquote\"", ""]]
    );
    assert!(
        transfer::parse(
            b"a,b\nx,y\n",
            Limits {
                columns: 1,
                ..limits
            }
        )
        .is_err()
    );
    assert!(transfer::parse(b"h\na\nb\n", Limits { rows: 1, ..limits }).is_err());
    assert!(transfer::parse(b"h\nlong", Limits { cell: 3, ..limits }).is_err());
    assert!(transfer::parse(b"h\na", Limits { bytes: 2, ..limits }).is_err());
    for invalid in [
        Limits { bytes: 0, ..limits },
        Limits {
            rows: 101,
            ..limits
        },
        Limits {
            columns: 101,
            ..limits
        },
        Limits {
            cell: 65_537,
            ..limits
        },
        Limits {
            bytes: 1_048_577,
            ..limits
        },
    ] {
        assert!(invalid.validate().is_err());
    }
    // The header does not consume a data slot, and a trailing record newline is not another row.
    let exact = format!("header\n{}\n", vec!["value"; 100].join("\n"));
    assert_eq!(
        transfer::parse(exact.as_bytes(), limits).unwrap().len(),
        101
    );
    assert!(transfer::parse(format!("{exact}extra").as_bytes(), limits).is_err());
    let corpus = [
        "", "🦀", "a,b", "x\"y", "a\nb", "a\rb", "a\r\nb", "\\N", "'=@",
    ];
    for text in corpus {
        let records = vec![vec!["header".into()], vec![text.into()]];
        let csv = transfer::write(&records, limits).unwrap();
        assert_eq!(transfer::parse(csv.as_bytes(), limits).unwrap(), records);
    }
    assert!(transfer::write(&[], limits).is_err());
    assert!(
        transfer::write(
            &[vec!["h".into()], vec!["abcdef".into()]],
            Limits { bytes: 5, ..limits }
        )
        .is_err()
    );
}
#[test]
fn typed_preview_headers_presence_and_literal_escapes() {
    let model = model();
    let limits = Limits::default();
    for csv in ["id,id\na,a", "id,secret\na,x", "unknown\nx"] {
        assert!(transfer::preview(&model, &upload(csv, false), limits).is_err());
    }
    let preview=transfer::preview(&model,&upload("id,text,count,flag\na,\\N,0,false\nb,\\\\N,-1,true\nc,\\\\O,\\O,false\nd,\\\\path,2,true\ne,,3,false",false),limits).unwrap();
    assert_eq!(
        preview.rows[0].values,
        json!({"id":"a","text":null,"count":0,"flag":false})
    );
    assert_eq!(preview.rows[1].values["text"], "\\N");
    assert_eq!(preview.rows[2].values["text"], "\\O");
    assert!(preview.rows[2].values.get("count").is_none());
    assert_eq!(preview.rows[3].values["text"], "\\path");
    assert_eq!(preview.rows[4].values["text"], "");
    for csv in [
        "id,text\n\\O,x",
        "id,count\na,not-json",
        "id,text\na,\\invalid",
        "id,text\na,x,extra",
    ] {
        assert!(
            !transfer::preview(&model, &upload(csv, false), limits)
                .unwrap()
                .valid()
        );
    }
}
#[test]
fn export_guard_round_trips_are_explicit_and_no_field_is_invented() {
    let model = model();
    let limits = Limits::default();
    for text in [
        "=1+1",
        "+cmd",
        "-cmd",
        "@SUM(A1)",
        "\ttab",
        "\rcarriage",
        "'=literal",
        "'hello",
        "\\N",
        "\\O",
        "\\root",
        "comma, \"quote\"\n🦀",
        "",
    ] {
        let row = json!({"id":"a","text":text,"count":-1,"flag":false,"secret":"PRIVATE"});
        let csv = transfer::export(&model, std::slice::from_ref(&row), limits).unwrap();
        assert!(!csv.contains("PRIVATE"));
        assert!(!csv.contains("secret"));
        assert!(!csv.contains("\\O\"\r\n"));
        let decoded = transfer::preview(&model, &upload(&csv, true), limits).unwrap();
        assert!(decoded.valid());
        assert_eq!(
            decoded.rows[0].values,
            json!({"id":"a","text":text,"count":-1,"flag":false})
        );
        assert_eq!(decoded.rows[0].values["count"], -1);
        let ordinary = transfer::preview(&model, &upload(&csv, false), limits).unwrap();
        if text.starts_with(['=', '+', '-', '@', '\t', '\r', '\'']) {
            assert_eq!(ordinary.rows[0].values["text"], format!("'{text}"));
        }
        assert_eq!(csv, transfer::export(&model, &[row], limits).unwrap());
    }
    let plain = transfer::preview(&model, &upload("id,text\na,'=literal", false), limits).unwrap();
    assert_eq!(plain.rows[0].values["text"], "'=literal");
    assert!(transfer::export(&model, &[json!({"id":"missing"})], limits).is_err());
    assert!(transfer::export(&model, &vec![json!({}); 101], limits).is_err());
    let row = json!({"id":"a","text":null,"count":0,"flag":false});
    let csv = transfer::export(&model, &[row], limits).unwrap();
    assert!(csv.contains("\\N"));
    assert!(!csv.contains("\\O"));
}
/// Export has only the row/byte ceilings; import-only cell and column caps still apply on reading.
#[test]
fn export_does_not_inherit_import_only_cell_or_column_limits() {
    let model = model();
    let row = json!({"id":"a","text":"x".repeat(65_537),"count":0,"flag":false});
    let limits = Limits {
        columns: 1,
        cell: 1,
        ..Default::default()
    };
    let csv = transfer::export(&model, &[row], limits).unwrap();
    assert!(csv.len() > 65_536);
    assert!(transfer::parse(csv.as_bytes(), Limits::default()).is_err());
    assert!(transfer::export(&model, &vec![json!({}); 101], limits).is_err());
}

#[test]
fn uncertain_or_malformed_acknowledgements_never_claim_imported() {
    assert!(Phase::Unknown.locked());
    assert!(!Phase::Unknown.message().starts_with("Imported"));
    assert_eq!(
        Phase::reply(true, 500, &json!({"errors":{"outcome":"unknown"}})),
        Phase::Unknown
    );
    assert_eq!(Phase::reply(true, 200, &Value::Null), Phase::Unknown);
    assert_eq!(
        Phase::reply(true, 200, &json!({"code":200,"results":{"imported":2}})),
        Phase::Imported(2)
    );
    assert_eq!(
        Phase::reply(false, 200, &json!({"code":200})),
        Phase::Previewed
    );
    for envelope in [
        json!({"code":200,"errors":{"outcome":"unknown"},"results":{"imported":2}}),
        json!({"code":400,"results":{"imported":2}}),
        json!({"results":{"imported":2}}),
        json!({"code":200,"results":{"imported":0}}),
        json!({"code":200,"results":{"imported":101}}),
    ] {
        assert_eq!(Phase::reply(true, 200, &envelope), Phase::Unknown);
    }
    assert!(matches!(
        Phase::reply(false, 200, &json!({})),
        Phase::Failed(_)
    ));
    assert!(matches!(
        Phase::reply(false, 403, &json!({"error":"Denied"})),
        Phase::Failed(_)
    ));
    for phase in [
        Phase::Idle,
        Phase::Previewing,
        Phase::Previewed,
        Phase::Applying,
        Phase::Imported(2),
        Phase::Failed("safe".into()),
    ] {
        assert!(!phase.message().is_empty());
    }
}
