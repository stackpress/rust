//! Actual parser reference metadata follows the same winners as composed configuration.

use super::{Snapshot, merge};
use crate::{CompositionLimits, LoadedSource, SourceId, SourceKind};
use stackpress_idea_parser::DeclarationKind;

/// Compile actual source reference sites before exercising the metadata merge policy.
fn snapshot(file: &str, text: &str) -> Snapshot {
    let source = LoadedSource {
        id: SourceId::new(file),
        name: file.into(),
        text: text.into(),
        kind: SourceKind::Idea,
    };
    super::super::compile_source(
        &source,
        &stackpress_idea_parser::CompileInputs::default(),
        CompositionLimits::default(),
    )
    .expect("valid fixture")
}

#[test]
/// Attribute/whole-column collisions keep local sites and missing members adopt child sites.
fn reference_sites_follow_mutable_winners() {
    let mut local = snapshot(
        "root.idea",
        "model User @local(Prop) { id String @a(Prop) }",
    );
    let child = snapshot(
        "child.idea",
        "model User @local(Prop) @child(Prop) { id Number @a(Prop) name String @a(Prop) }",
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
        "prop P { x 1 } model User! { id String @a(P) }",
    );
    let child = snapshot(
        "child.idea",
        "prop P { x 2 } model User { name String @a(P) } type Absent { id String @a(P) }",
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
            .any(|site| site.kind == DeclarationKind::Type)
    );
}
