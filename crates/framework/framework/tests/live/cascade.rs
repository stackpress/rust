//! G48 purge on a real parent/child catalog, retaining constraints and native refusal evidence.

use super::support;
use stackpress_framework::commands::sql::{self, Scripts};
use stackpress_sql::{
    Create, DialectId, Engine, ErrorKind, Field, ForeignKey, Render, Select, SqlType, TableCommand,
};
use std::error::Error as _;

/// Source create/purge order starts with the parent; uninstall reverses that catalog.
fn tables() -> Vec<stackpress_sql::TableDefinition> {
    vec![
        Create::new("g48_parent")
            .add_field(Field::new("id", SqlType::Integer))
            .primary_key(["id"])
            .definition()
            .unwrap(),
        Create::new("g48_child")
            .add_field(Field::new("id", SqlType::Integer))
            .add_field(Field::new("parent_id", SqlType::Integer))
            .primary_key(["id"])
            .add_foreign_key(ForeignKey::new(
                "g48_child_parent",
                ["parent_id"],
                "g48_parent",
                ["id"],
            ))
            .definition()
            .unwrap(),
    ]
}

/// Run real install/purge/uninstall, with parent-first TRUNCATE's native MySQL 1701 as a control.
pub(super) async fn prove(engine: &Engine) -> support::TestResult<()> {
    let scripts = Scripts::from_tables(&tables(), engine.dialect(), serde_json::json!({}))?;
    sql::install(engine, &scripts, None).await?;
    engine
        .execute(&engine.insert("g48_parent").columns(["id"]).values([1]))
        .await?;
    engine
        .execute(
            &engine
                .insert("g48_child")
                .columns(["id", "parent_id"])
                .values([2, 1]),
        )
        .await?;
    if engine.dialect() == DialectId::Mysql {
        // The general renderer still refuses CASCADE; the aggregate now stops requesting it.
        assert_eq!(
            TableCommand::Truncate {
                table: "g48_parent".into(),
                cascade: true
            }
            .render(engine.dialect())
            .unwrap_err()
            .to_string(),
            "dialect truncate cascade is unsupported"
        );
        // Removing CASCADE alone is insufficient: InnoDB retains the FK even after child deletion.
        let error = engine
            .execute(&TableCommand::Truncate {
                table: "g48_parent".into(),
                cascade: false,
            })
            .await
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Driver);
        let diagnostic = error.source().unwrap().to_string();
        assert!(
            diagnostic.contains("1701") && diagnostic.contains("foreign key"),
            "{diagnostic}"
        );
        println!("MySQL referenced-table TRUNCATE result (verbatim):\n{diagnostic}");
        assert_eq!(
            scripts.purge.iter().map(|q| q.sql()).collect::<Vec<_>>(),
            ["DELETE FROM `g48_child`", "DELETE FROM `g48_parent`",]
        );
    } else {
        assert_eq!(
            scripts.purge.iter().map(|q| q.sql()).collect::<Vec<_>>(),
            [
                "TRUNCATE TABLE \"g48_parent\" CASCADE",
                "TRUNCATE TABLE \"g48_child\" CASCADE",
            ]
        );
    }
    // Exercise the actual public purge, rather than executing a handwritten replacement script.
    sql::purge(engine, &scripts).await?;
    for name in ["g48_parent", "g48_child"] {
        assert!(
            engine
                .fetch(&Select::new(["id"]).from(name))
                .await?
                .is_empty()
        );
    }
    // Purge keeps real constraints active; no session FOREIGN_KEY_CHECKS or drop/recreate is used.
    assert!(
        engine
            .execute(
                &engine
                    .insert("g48_child")
                    .columns(["id", "parent_id"])
                    .values([3, 99])
            )
            .await
            .is_err()
    );
    sql::purge(engine, &scripts).await?;
    sql::uninstall(engine, &scripts).await?;
    println!("related catalog purge passed with constraints retained");
    Ok(())
}
