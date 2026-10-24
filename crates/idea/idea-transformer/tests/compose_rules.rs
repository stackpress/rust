//! Exact merge-table, order, provenance, cycle and independent snapshot contracts.

use std::error::Error;

use serde_json::json;
use stackpress_idea_parser::{CompileInputs, IdeaError};
use stackpress_idea_transformer::{
    ComposedSchema, Composer, CompositionLimits, LoadError, LoadedSource, MemoryLoader,
    ProjectLoader, SourceId, SourceLoader,
};

/// Build a deterministic project with no host IO or process environment.
fn project(files: &[(&str, &str)]) -> MemoryLoader {
    let mut loader = MemoryLoader::new("/project").unwrap();
    for (path, text) in files {
        loader.insert(path, *text);
    }
    loader
}

/// Compose ordinary fixtures under finite default budgets.
fn compose(files: &[(&str, &str)]) -> ComposedSchema {
    Composer::new(project(files), CompositionLimits::default())
        .compose("root")
        .unwrap()
}

/// Convert config for assertions that apply identically to model and type declarations.
fn config(result: &ComposedSchema) -> serde_json::Value {
    serde_json::to_value(&result.config).unwrap()
}

#[test]
/// Local absent/mutable/immutable × imported mutable/immutable proves the G19 table by kind.
fn full_type_and_model_merge_table() {
    for kind in ["model", "type"] {
        for local in [None, Some(true), Some(false)] {
            for child_mutable in [true, false] {
                let child_text = format!(
                    "{kind} User{} @label(\"child\") @child {{
                    first Number id Number @child second Boolean }}",
                    if child_mutable { "" } else { "!" }
                );
                let local_text = local.map(|mutable| format!(
                    "{kind} User{} @label(\"local\") @local {{ id String @local own String }}",
                    if mutable { "" } else { "!" })).unwrap_or_default();
                let root = format!("use \"./child\" {local_text}");
                let result = compose(&[("root.idea", &root), ("child.idea", &child_text)]);
                let actual = config(&result)[kind]["User"].clone();
                let column_names = actual["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|column| column["name"].as_str().unwrap())
                    .collect::<Vec<_>>();
                if local == Some(true) {
                    // A mutable parent keeps its flag, attributes and the entire colliding column.
                    assert_eq!(actual["mutable"], true);
                    assert_eq!(actual["attributes"]["label"], json!(["local"]));
                    assert_eq!(
                        actual["attributes"]
                            .as_object()
                            .unwrap()
                            .keys()
                            .map(String::as_str)
                            .collect::<Vec<_>>(),
                        ["label", "child", "local"]
                    );
                    assert_eq!(column_names, ["first", "second", "id", "own"]);
                    assert_eq!(actual["columns"][2]["type"], "String");
                    assert_eq!(actual["columns"][2]["attributes"], json!({"local":true}));
                    assert_eq!(
                        result.provenance[&vec![
                            kind.into(),
                            "User".into(),
                            "attributes".into(),
                            "label".into()
                        ]],
                        SourceId::new("/project/root.idea")
                    );
                    assert_eq!(
                        result.provenance
                            [&vec![kind.into(), "User".into(), "columns".into(), "first".into()]],
                        SourceId::new("/project/child.idea")
                    );
                } else {
                    // G19 is deliberate: immutable locals lose all their own attributes/columns.
                    assert_eq!(actual["mutable"], child_mutable);
                    assert_eq!(
                        actual["attributes"],
                        json!({"label":["child"],"child":true})
                    );
                    assert_eq!(column_names, ["first", "id", "second"]);
                    assert_eq!(
                        result.provenance[&vec![kind.into(), "User".into()]],
                        SourceId::new("/project/child.idea")
                    );
                    assert!(!result.provenance.contains_key(&vec![
                        kind.into(),
                        "User".into(),
                        "columns".into(),
                        "own".into()
                    ]));
                }
                assert!(result.config.r#use.is_empty());
            }
        }
    }
}

#[test]
/// Missing child sections retain locals and root plugins are the only active plugin config.
fn absent_sections_and_inert_child_plugins() {
    let result = compose(&[
        (
            "root.idea",
            "use \"child\" plugin \"root-z\" {} plugin \"root-a\" {}
            prop Local { x 1 } enum E { A 1 } type T { id String } model M! { id String }",
        ),
        ("child.idea", "use \"grand\" plugin \"inactive\" {}"),
        ("grand.idea", "plugin \"also-inactive\" {}"),
    ]);
    assert_eq!(
        result
            .config
            .plugin
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["root-z", "root-a"]
    );
    assert_eq!(result.config.model["M"].columns[0].name, "id");
    assert!(!result.config.model["M"].mutable);
    assert!(result.config.r#type.contains_key("T"));
    assert!(result.config.r#enum.contains_key("E"));
    assert!(result.config.prop.contains_key("Local"));
    assert!(
        result
            .provenance
            .keys()
            .filter(|path| path[0] == "plugin")
            .all(|path| path[1].starts_with("root-"))
    );
}

#[test]
/// Whole props/enums retain local values while child-first spread determines declaration order.
fn props_enums_and_multiple_import_order() {
    let result = compose(&[
        (
            "root.idea",
            "use \"first\" use \"second\"
            prop Shared { own 1 } prop Local { x 1 } enum SharedEnum { LOCAL 1 }
            model User @label(\"root\") { own String }",
        ),
        (
            "first.idea",
            "prop First { x 1 } prop Shared { imported 1 }
            enum FirstEnum { FIRST 1 } enum SharedEnum { CHILD 1 }
            model User @first @shared(1) { first String collision String @first }",
        ),
        (
            "second.idea",
            "prop Second { x 2 } prop First { x 99 }
            enum SecondEnum { SECOND 2 } enum FirstEnum { OTHER 99 }
            model User @second @shared(2) { second String collision Number @second }",
        ),
    ]);
    assert_eq!(
        result
            .config
            .prop
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Second", "First", "Shared", "Local"]
    );
    assert_eq!(
        result
            .config
            .r#enum
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["SecondEnum", "FirstEnum", "SharedEnum"]
    );
    let value = config(&result);
    assert_eq!(value["prop"]["Shared"], json!({"own":1}));
    assert_eq!(value["prop"]["First"], json!({"x":1}));
    assert_eq!(value["enum"]["SharedEnum"], json!({"LOCAL":1}));
    assert_eq!(value["enum"]["FirstEnum"], json!({"FIRST":1}));
    let user = &result.config.model["User"];
    assert_eq!(
        user.columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>(),
        ["second", "first", "collision", "own"]
    );
    assert_eq!(value["model"]["User"]["attributes"]["shared"], json!([1]));
    assert_eq!(user.columns[2].column_type, "String");
    assert_eq!(
        result.provenance[&vec!["prop".into(), "First".into()]],
        SourceId::new("/project/first.idea")
    );
}

#[test]
/// An immutable accumulated child can be replaced again; a mutable child keeps prior winners.
fn multiple_imports_recheck_accumulated_mutability() {
    for first_mutable in [false, true] {
        let first = format!(
            "model User{} {{ first String }}",
            if first_mutable { "" } else { "!" }
        );
        let result = compose(&[
            ("root.idea", "use \"first\" use \"second\""),
            ("first.idea", &first),
            ("second.idea", "model User! { second Number }"),
        ]);
        let user = &result.config.model["User"];
        assert_eq!(user.mutable, first_mutable);
        assert_eq!(
            user.columns
                .iter()
                .map(|column| column.name.as_str())
                .collect::<Vec<_>>(),
            if first_mutable {
                vec!["second", "first"]
            } else {
                vec!["second"]
            }
        );
    }
}

#[test]
/// A diamond is not a cycle; child order is stable and each parent's mutations stay private.
fn diamond_snapshots_and_result_clones_are_independent() {
    let loader = project(&[
        ("root.idea", "use \"left\" use \"right\""),
        (
            "left.idea",
            "use \"base\" model User { left String id Number }",
        ),
        ("right.idea", "use \"base\" model User { right String }"),
        (
            "base.idea",
            "model User { id String first String second String }",
        ),
    ]);
    let composer = Composer::new(&loader, CompositionLimits::default());
    let result = composer.compose("root").unwrap();
    assert_eq!(
        result
            .sources
            .iter()
            .map(|source| source.name.as_str())
            .collect::<Vec<_>>(),
        ["root.idea", "left.idea", "base.idea", "right.idea"]
    );
    assert_eq!(result.imports.len(), 4);
    assert_eq!(
        result.imports[3].imported,
        SourceId::new("/project/base.idea")
    );
    let right = composer.compose("right").unwrap();
    assert_eq!(
        right.config.model["User"]
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>(),
        ["id", "first", "second", "right"]
    );
    assert_eq!(right.config.model["User"].columns[0].column_type, "String");
    let mut cloned = result.clone();
    cloned.config.model["User"].columns.clear();
    cloned.sources[0].text.clear();
    cloned.provenance.clear();
    assert!(!result.config.model["User"].columns.is_empty());
    assert!(!result.sources[0].text.is_empty());
    assert!(!result.provenance.is_empty());
    assert_eq!(result.config, composer.compose("root").unwrap().config);
}

/// Collect phase explanations without discarding the original typed cause.
fn chain(error: &(dyn Error + 'static)) -> Vec<String> {
    let mut messages = vec![error.to_string()];
    let mut next = error.source();
    while let Some(error) = next {
        messages.push(error.to_string());
        next = error.source();
    }
    messages
}

#[test]
/// Direct/indirect/alias cycles report a finite complete logical import chain.
fn canonical_cycles_are_active_stack_errors() {
    for files in [
        vec![("root.idea", "use \"root\"")],
        vec![
            ("root.idea", "use \"a\""),
            ("a.idea", "use \"b\""),
            ("b.idea", "use \"a\""),
        ],
    ] {
        let error = Composer::new(project(&files), CompositionLimits::default())
            .compose("root")
            .unwrap_err();
        let messages = chain(&error).join(" | ");
        assert!(messages.contains("root.idea"));
        assert!(messages.contains("import cycle:"));
        if files.len() > 1 {
            assert!(messages.contains("root.idea -> a.idea -> b.idea -> a.idea"));
        }
    }
    let mut loader = project(&[("root.idea", "use \"alias.idea\"")]);
    loader.alias("alias.idea", "root.idea").unwrap();
    assert!(
        chain(
            &Composer::new(loader, CompositionLimits::default())
                .compose("root")
                .unwrap_err()
        )
        .join(" | ")
        .contains("root.idea -> root.idea")
    );
}

#[cfg(unix)]
#[test]
/// Real symlink aliases participate in canonical cycle detection without process cwd changes.
fn native_symlink_cycle() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("root.idea"), "use \"alias.idea\"").unwrap();
    std::os::unix::fs::symlink(
        directory.path().join("root.idea"),
        directory.path().join("alias.idea"),
    )
    .unwrap();
    let loader = ProjectLoader::new(directory.path()).unwrap();
    let error = Composer::new(loader, CompositionLimits::default())
        .compose("root")
        .unwrap_err();
    assert!(chain(&error).join(" | ").contains("root.idea -> root.idea"));
}

#[test]
/// Child parse/IO failures retain their types and each importer without exposing source text.
fn failure_causes_and_importer_chain_survive() {
    let loader = project(&[
        ("root.idea", "use \"nested/child\""),
        ("nested/child.idea", "use \"./bad\""),
        ("nested/bad.idea", "model Missing {"),
    ]);
    let error = Composer::new(loader, CompositionLimits::default())
        .compose("root")
        .unwrap_err();
    let messages = chain(&error).join(" | ");
    assert!(messages.contains("import from root.idea failed"));
    assert!(messages.contains("import from nested/child.idea failed"));
    let mut cause: &(dyn Error + 'static) = &error;
    while let Some(next) = cause.source() {
        cause = next;
    }
    let diagnostic = cause.downcast_ref::<IdeaError>().unwrap();
    assert_eq!(diagnostic.file, "nested/bad.idea");
    assert_eq!(diagnostic.line, 1);
    assert!(!error.to_string().contains("model Missing"));
    let error = Composer::new(
        project(&[("root.idea", "use \"missing\"")]),
        CompositionLimits::default(),
    )
    .compose("root")
    .unwrap_err();
    let mut cause: &(dyn Error + 'static) = &error;
    while let Some(next) = cause.source() {
        cause = next;
    }
    assert_eq!(
        cause.downcast_ref::<std::io::Error>().unwrap().kind(),
        std::io::ErrorKind::NotFound
    );
}

#[test]
/// Explicit environments are isolated and M0 placeholder lookalikes remain untouched.
fn explicit_inputs_and_unresolved_imported_props() {
    let loader = project(&[
        (
            "root.idea",
            "use \"props\"
        model User { id String @actual(Config) @literal(\"${Config}\") }",
        ),
        ("props.idea", "prop Config { url env(\"URL\") }"),
    ]);
    for value in ["first", "second"] {
        let mut inputs = CompileInputs::default();
        inputs.environment.insert("URL".into(), value.into());
        let result = Composer::new(&loader, CompositionLimits::default())
            .with_inputs(inputs)
            .compose("root")
            .unwrap();
        assert_eq!(result.config.prop["Config"]["url"], value);
        // Only actual syntax is a reference; the adjacent placeholder-looking string stays literal.
        assert_eq!(result.references.len(), 1);
        assert_eq!(
            config(&result)["model"]["User"]["columns"][0]["attributes"]["literal"],
            json!(["${Config}"])
        );
    }
}

/// Deliberately nonconforming custom adapter for the composer's defensive acquisition check.
struct Oversized;

impl SourceLoader for Oversized {
    /// Ignore the budget to prove the composer rejects invalid custom-loader results.
    fn load(&self, _: Option<&SourceId>, _: &str, _: usize) -> Result<LoadedSource, LoadError> {
        Ok(LoadedSource {
            id: SourceId::new("/oversized"),
            name: "oversized".into(),
            kind: stackpress_idea_transformer::SourceKind::Idea,
            text: "model User {}".into(),
        })
    }
}

#[test]
/// Per-source, aggregate, file, edge, depth and expansion limits reject finite boundary cases.
fn composition_budgets_have_meaningful_boundaries() {
    let loader = project(&[
        ("root.idea", "use \"a\" use \"b\""),
        ("a.idea", "model A {}"),
        ("b.idea", "model B {}"),
    ]);
    for limits in [
        CompositionLimits {
            max_source_bytes: 1,
            ..CompositionLimits::default()
        },
        CompositionLimits {
            max_total_bytes: 20,
            ..CompositionLimits::default()
        },
        CompositionLimits {
            max_files: 2,
            ..CompositionLimits::default()
        },
        CompositionLimits {
            max_imports: 1,
            ..CompositionLimits::default()
        },
        CompositionLimits {
            max_depth: 1,
            ..CompositionLimits::default()
        },
        CompositionLimits {
            max_expanded_bytes: 10,
            ..CompositionLimits::default()
        },
    ] {
        assert!(Composer::new(&loader, limits).compose("root").is_err());
    }
    let limits = CompositionLimits {
        max_files: 3,
        max_imports: 2,
        max_depth: 2,
        ..CompositionLimits::default()
    };
    assert!(Composer::new(&loader, limits).compose("root").is_ok());
    let limits = CompositionLimits {
        max_source_bytes: 2,
        ..CompositionLimits::default()
    };
    assert!(
        Composer::new(Oversized, limits)
            .compose("root")
            .unwrap_err()
            .to_string()
            .contains("source adapter exceeded")
    );
}

#[test]
/// Even caller-raised import limits cannot bypass the hard recursive ceiling.
fn hard_import_depth_and_bounded_diamond_expansion() {
    let mut loader = project(&[]);
    for index in 0..=CompositionLimits::HARD_DEPTH {
        loader.insert(
            format!("{index}.idea"),
            if index == CompositionLimits::HARD_DEPTH {
                "model Last {}".into()
            } else {
                format!("use \"{}\"", index + 1)
            },
        );
    }
    let limits = CompositionLimits {
        max_depth: usize::MAX,
        ..CompositionLimits::default()
    };
    assert!(
        chain(&Composer::new(&loader, limits).compose("0").unwrap_err())
            .join(" | ")
            .contains("depth budget exceeded")
    );
    assert!(Composer::new(&loader, limits).compose("1").is_ok());
    // Many repeated imports have little source text but must still spend config-copy work.
    let root = "use \"base\" ".repeat(30);
    let base = "prop Large { value \"0123456789012345678901234567890123456789\" }";
    let limits = CompositionLimits {
        max_expanded_bytes: 1500,
        ..CompositionLimits::default()
    };
    let loader = project(&[("root.idea", &root), ("base.idea", base)]);
    assert!(
        chain(&Composer::new(loader, limits).compose("root").unwrap_err())
            .join(" | ")
            .contains("expansion budget exceeded")
    );
}

#[test]
/// The same canonical ID cannot silently denote changed bytes during an import diamond.
fn changed_canonical_snapshots_are_rejected() {
    /// Fault-inject a cooperative edit when the same canonical child is acquired again.
    struct Changing {
        /// Ordinary immutable fixtures plus an isolated per-test call counter.
        memory: MemoryLoader,
        /// Number of base acquisitions; never shared between composer runs or threads.
        reads: std::cell::Cell<usize>,
    }
    impl SourceLoader for Changing {
        /// Preserve IDs but change source bytes on a repeated acquisition.
        fn load(
            &self,
            importer: Option<&SourceId>,
            specifier: &str,
            max_bytes: usize,
        ) -> Result<LoadedSource, LoadError> {
            let mut source = self.memory.load(importer, specifier, max_bytes)?;
            if source.name == "base.idea" {
                let reads = self.reads.get();
                self.reads.set(reads + 1);
                if reads > 0 {
                    source.text = "model Changed {}".into();
                }
            }
            Ok(source)
        }
    }
    let loader = Changing {
        memory: project(&[
            ("root.idea", "use \"base\" use \"base\""),
            ("base.idea", "model Original {}"),
        ]),
        reads: std::cell::Cell::new(0),
    };
    let error = Composer::new(loader, CompositionLimits::default())
        .compose("root")
        .unwrap_err();
    assert!(
        chain(&error)
            .join(" | ")
            .contains("canonical source changed")
    );
}

#[test]
/// Fresh native calls see edits, and concurrent explicit environments never share compiled values.
fn native_refresh_and_concurrent_memory_runs() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("root.idea");
    std::fs::write(&root, "model Before {}").unwrap();
    let composer = Composer::new(
        ProjectLoader::new(directory.path()).unwrap(),
        CompositionLimits::default(),
    );
    assert!(
        composer
            .compose("root")
            .unwrap()
            .config
            .model
            .contains_key("Before")
    );
    std::fs::write(&root, "model After {}").unwrap();
    assert!(
        composer
            .compose("root")
            .unwrap()
            .config
            .model
            .contains_key("After")
    );
    let loader = project(&[("root.idea", "prop Config { value env(\"VALUE\") }")]);
    std::thread::scope(|scope| {
        let handles = ["one", "two"].map(|value| {
            let loader = &loader;
            scope.spawn(move || {
                let mut inputs = CompileInputs::default();
                inputs.environment.insert("VALUE".into(), value.into());
                let result = Composer::new(loader, CompositionLimits::default())
                    .with_inputs(inputs)
                    .compose("root")
                    .unwrap();
                assert_eq!(result.config.prop["Config"]["value"], value);
            })
        });
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
/// Model/type insertion retains local order then the first encounter order of child-only names.
fn declaration_order_is_local_then_imported() {
    let result = compose(&[
        (
            "root.idea",
            "use \"child\" model Z {} model A {} type ZType {}",
        ),
        (
            "child.idea",
            "model B {} model A {} type BType {} type AType {}",
        ),
    ]);
    assert_eq!(
        result
            .config
            .model
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Z", "A", "B"]
    );
    assert_eq!(
        result
            .config
            .r#type
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["ZType", "BType", "AType"]
    );
}
