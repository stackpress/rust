//! Accepted upstream merge table, with the same subtree operations for origins and references.

use crate::SourceId;

use super::Snapshot;

/// Record declaration and independently winning member origins before importing children.
pub(super) fn record_origins(snapshot: &mut Snapshot, source: &SourceId) {
    for (kind, names) in [
        ("plugin", snapshot.config.plugin.keys().collect::<Vec<_>>()),
        ("prop", snapshot.config.prop.keys().collect()),
        ("enum", snapshot.config.r#enum.keys().collect()),
        ("type", snapshot.config.r#type.keys().collect()),
        ("model", snapshot.config.model.keys().collect()),
    ] {
        for name in names {
            snapshot
                .provenance
                .insert(vec![kind.into(), name.clone()], source.clone());
        }
    }
    // Attributes merge independently; columns collide as entire declarations, never by field.
    for (kind, types) in [
        ("type", &snapshot.config.r#type),
        ("model", &snapshot.config.model),
    ] {
        for (name, value) in types {
            for attribute in value.attributes.keys() {
                snapshot.provenance.insert(
                    vec![
                        kind.into(),
                        name.clone(),
                        "attributes".into(),
                        attribute.clone(),
                    ],
                    source.clone(),
                );
            }
            for column in &value.columns {
                snapshot.provenance.insert(
                    vec![
                        kind.into(),
                        name.clone(),
                        "columns".into(),
                        column.name.clone(),
                    ],
                    source.clone(),
                );
            }
        }
    }
}

/// Apply one direct import; absent child sections and imported plugins never erase locals.
pub(super) fn merge(local: &mut Snapshot, child: &Snapshot) {
    // Props/enums use child-first order but whole local declaration values take precedence.
    for (kind, child_names) in [
        ("prop", child.config.prop.keys().collect::<Vec<_>>()),
        ("enum", child.config.r#enum.keys().collect()),
    ] {
        for name in child_names {
            let exists = if kind == "prop" {
                local.config.prop.contains_key(name)
            } else {
                local.config.r#enum.contains_key(name)
            };
            if !exists {
                replace_metadata(local, child, &[kind.into(), name.clone()]);
            }
        }
    }
    let mut props = child.config.prop.clone();
    props.extend(std::mem::take(&mut local.config.prop));
    local.config.prop = props;
    let mut enums = child.config.r#enum.clone();
    enums.extend(std::mem::take(&mut local.config.r#enum));
    local.config.r#enum = enums;

    // Models and types have the identical policy, while retaining independent kind namespaces.
    for (kind, child_types) in [
        ("type", &child.config.r#type),
        ("model", &child.config.model),
    ] {
        for (name, imported) in child_types {
            let accumulated = if kind == "type" {
                local.config.r#type.get(name)
            } else {
                local.config.model.get(name)
            };
            let prefix = vec![kind.into(), name.clone()];
            match accumulated {
                // G19: an immutable accumulated local is replaced, not protected from imports.
                None | Some(stackpress_idea_parser::TypeConfig { mutable: false, .. }) => {
                    replace_metadata(local, child, &prefix);
                }
                Some(accumulated) => {
                    // Only absent attributes/columns adopt child origins and reference sites.
                    let mut adopted = Vec::new();
                    for attribute in imported.attributes.keys() {
                        if !accumulated.attributes.contains_key(attribute) {
                            adopted.push(vec![
                                kind.into(),
                                name.clone(),
                                "attributes".into(),
                                attribute.clone(),
                            ]);
                        }
                    }
                    for column in &imported.columns {
                        if !accumulated
                            .columns
                            .iter()
                            .any(|own| own.name == column.name)
                        {
                            adopted.push(vec![
                                kind.into(),
                                name.clone(),
                                "columns".into(),
                                column.name.clone(),
                            ]);
                        }
                    }
                    for prefix in adopted {
                        replace_metadata(local, child, &prefix);
                    }
                }
            }
            let types = if kind == "type" {
                &mut local.config.r#type
            } else {
                &mut local.config.model
            };
            if let Some(accumulated) = types.get_mut(name) {
                // The M0 primitive owns the exact column and attribute precedence policy.
                crate::merge_type(accumulated, imported);
            } else {
                // Insertion preserves local declaration order, then first-seen imported order.
                types.insert(name.clone(), imported.clone());
            }
        }
    }
}

/// Move a winning declaration/member's origins and typed sites as one inseparable subtree.
fn replace_metadata(local: &mut Snapshot, child: &Snapshot, prefix: &[String]) {
    // Remove every losing descendant, including typed sites when G19 replaces a declaration.
    local.provenance.retain(|path, _| !path.starts_with(prefix));
    local.references.retain(|site| !site.is_within(prefix));
    local.provenance.extend(
        child
            .provenance
            .iter()
            .filter(|(path, _)| path.starts_with(prefix))
            .map(|(path, source)| (path.clone(), source.clone())),
    );
    local.references.extend(
        child
            .references
            .iter()
            .filter(|site| site.is_within(prefix))
            .cloned(),
    );
}

#[cfg(test)]
mod tests;
