//! Synthetic typed-reference metadata proofs independent of the frozen M0 parser.

use super::*;
use crate::{
    CompositionLimits, LoadedSource, ReferenceKind, ReferencePath, ReferenceSite, ReferenceSpan,
    SourceKind,
};

/// Make synthetic typed sites so merge policy is proved without guessing M0 placeholders.
fn snapshot(file: &str, text: &str, sites: &[&[&str]]) -> Snapshot {
    let source = LoadedSource {
        id: SourceId::new(file),
        name: file.into(),
        text: text.into(),
        kind: SourceKind::Idea,
    };
    let mut snapshot = super::super::compile_source(
        &source,
        &stackpress_idea_parser::CompileInputs::default(),
        CompositionLimits::default(),
    )
    .expect("valid fixture");
    snapshot.references = sites
        .iter()
        .map(|path| ReferenceSite {
            kind: if path[0] == "type" {
                ReferenceKind::Type
            } else {
                ReferenceKind::Model
            },
            declaration: path[1].into(),
            column: if path[2] == "columns" {
                Some(path[3].into())
            } else {
                None
            },
            attribute: if path[2] == "columns" {
                path[5].into()
            } else {
                path[3].into()
            },
            path: vec![ReferencePath::Index(0), ReferencePath::Key("nested".into())],
            target: "Prop".into(),
            forward: false,
            span: ReferenceSpan {
                file: file.into(),
                start: 0,
                end: 0,
                line: 1,
                column: 1,
            },
        })
        .collect();
    snapshot
}

#[test]
/// Attribute/whole-column collisions keep local sites and missing members adopt child sites.
fn reference_sites_follow_mutable_winners() {
    let root_paths: &[&[&str]] = &[
        &["model", "User", "attributes", "local", "0"],
        &["model", "User", "columns", "id", "attributes", "a", "0"],
    ];
    let child_paths: &[&[&str]] = &[
        &["model", "User", "attributes", "local", "0"],
        &["model", "User", "attributes", "child", "0"],
        &["model", "User", "columns", "id", "attributes", "a", "0"],
        &["model", "User", "columns", "name", "attributes", "a", "0"],
    ];
    let mut local = snapshot(
        "root.idea",
        "model User @local(Prop) { id String @a(Prop) }",
        root_paths,
    );
    let child = snapshot(
        "child.idea",
        "model User @local(Prop) @child(Prop) { id Number @a(Prop) name String @a(Prop) }",
        child_paths,
    );
    merge(&mut local, &child);
    assert_eq!(local.references.len(), 4);
    assert_eq!(
        local
            .references
            .iter()
            .filter(|site| site.span.file == "root.idea")
            .count(),
        2
    );
    assert_eq!(child.references.len(), 4);
}

#[test]
/// G19 discards all local sites while absent types adopt complete child reference metadata.
fn reference_sites_follow_replacement_and_adoption() {
    let mut local = snapshot(
        "root.idea",
        "prop P { x 1 } model User! { id String }",
        &[&["model", "User", "columns", "id", "attributes", "a"]],
    );
    let child = snapshot(
        "child.idea",
        "prop P { x 2 } model User { name String } type Absent { id String }",
        &[
            &["model", "User", "columns", "name", "attributes", "a"],
            &["type", "Absent", "columns", "id", "attributes", "a"],
        ],
    );
    merge(&mut local, &child);
    assert_eq!(local.references.len(), 2);
    assert!(
        local
            .references
            .iter()
            .all(|site| site.span.file == "child.idea")
    );
    assert!(
        local
            .references
            .iter()
            .any(|site| site.kind == ReferenceKind::Type)
    );
}
