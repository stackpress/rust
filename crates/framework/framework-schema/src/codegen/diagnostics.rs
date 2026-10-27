//! Tool-only provenance mapped through existing checked AST nodes and effective origins.

/// Tool-only winning-source map; never emitted into public generated metadata.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticMap {
    /// Finite structural identities and their original source coordinates.
    pub entries: Vec<DiagnosticLocation>,
}

/// One effective declaration/field/attribute origin, independent of normalized JSON values.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticLocation {
    /// Config path, including exact source spelling.
    pub path: Vec<String>,
    /// Logical source name retained by the composer.
    pub source: String,
    /// Original UTF-8 byte range for Idea; normalized JSON keeps structural paths only.
    pub bytes: Option<(usize, usize)>,
}

impl DiagnosticMap {
    /// Locate the closest winning structural identity, never inventing JSON coordinates.
    pub fn resolve(&self, path: &[String]) -> Option<&DiagnosticLocation> {
        self.entries
            .iter()
            .filter(|entry| path.starts_with(&entry.path))
            .max_by_key(|entry| entry.path.len())
    }

    /// Map effective provenance through the existing checked AST, without another parser.
    ///
    /// Field attributes inherit whole-column provenance because composition replaces columns
    /// as a whole. Declaration attributes retain their independently winning origins.
    pub fn from_composed(
        composed: &stackpress_idea_transformer::ComposedSchema,
    ) -> stackpress_idea_transformer::Result<Self> {
        use stackpress_idea_parser::{DataToken, ParseLimits, SourceFile, Statement, parse_ast};
        use stackpress_idea_transformer::SourceKind;
        let mut entries = Vec::new();
        for source in &composed.sources {
            if source.kind == SourceKind::Json {
                for (path, origin) in &composed.provenance {
                    if origin == &source.id {
                        push(
                            &mut entries,
                            DiagnosticLocation {
                                path: path.clone(),
                                source: source.name.clone(),
                                bytes: None,
                            },
                        )?;
                    }
                }
                continue;
            }
            let ast = parse_ast(
                &SourceFile::new(&source.name, &source.text),
                ParseLimits::default(),
            )?;
            for statement in &ast.syntax().body {
                let Statement::Declaration(declaration) = statement else {
                    continue;
                };
                if !matches!(declaration.kind, "model" | "type" | "enum") {
                    continue;
                }
                for item in &declaration.declarations {
                    let root = vec![declaration.kind.into(), item.id.name.clone()];
                    if composed.provenance.get(&root) == Some(&source.id) {
                        push(
                            &mut entries,
                            DiagnosticLocation {
                                path: root.clone(),
                                source: source.name.clone(),
                                bytes: Some((item.start, item.end)),
                            },
                        )?;
                    }
                    if declaration.kind == "enum"
                        && composed.provenance.get(&root) == Some(&source.id)
                    {
                        for variant in &item.init.properties {
                            let mut path = root.clone();
                            path.push(variant.key.name.clone());
                            push(
                                &mut entries,
                                DiagnosticLocation {
                                    path,
                                    source: source.name.clone(),
                                    bytes: Some((variant.start, variant.end)),
                                },
                            )?;
                        }
                    }
                    for property in &item.init.properties {
                        let DataToken::Object(object) = &property.value else {
                            continue;
                        };
                        if property.key.name == "attributes" {
                            for attribute in &object.properties {
                                let mut path = root.clone();
                                path.extend(["attributes".into(), attribute.key.name.clone()]);
                                if composed.provenance.get(&path) == Some(&source.id) {
                                    push(
                                        &mut entries,
                                        DiagnosticLocation {
                                            path,
                                            source: source.name.clone(),
                                            bytes: Some((attribute.start, attribute.end)),
                                        },
                                    )?;
                                }
                            }
                        } else if property.key.name == "columns" {
                            for column in &object.properties {
                                let mut path = root.clone();
                                path.extend(["columns".into(), column.key.name.clone()]);
                                if composed.provenance.get(&path) != Some(&source.id) {
                                    continue;
                                }
                                push(
                                    &mut entries,
                                    DiagnosticLocation {
                                        path: path.clone(),
                                        source: source.name.clone(),
                                        bytes: Some((column.start, column.end)),
                                    },
                                )?;
                                let DataToken::Object(value) = &column.value else {
                                    continue;
                                };
                                for property in &value.properties {
                                    if property.key.name != "attributes" {
                                        continue;
                                    }
                                    let DataToken::Object(attributes) = &property.value else {
                                        continue;
                                    };
                                    for attribute in &attributes.properties {
                                        let mut path = path.clone();
                                        path.extend([
                                            "attributes".into(),
                                            attribute.key.name.clone(),
                                        ]);
                                        push(
                                            &mut entries,
                                            DiagnosticLocation {
                                                path,
                                                source: source.name.clone(),
                                                bytes: Some((attribute.start, attribute.end)),
                                            },
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if entries.len() > 10_000 {
            return Err(stackpress_idea_transformer::TransformError::new(
                "diagnostic map entry budget exceeded",
            ));
        }
        Ok(Self { entries })
    }
}

/// Charge before retaining the next coordinate; wide imports cannot overshoot the map bound.
fn push(
    entries: &mut Vec<DiagnosticLocation>,
    entry: DiagnosticLocation,
) -> stackpress_idea_transformer::Result<()> {
    if entries.len() >= 10_000 {
        return Err(stackpress_idea_transformer::TransformError::new(
            "diagnostic map entry budget exceeded",
        ));
    }
    entries.push(entry);
    Ok(())
}
