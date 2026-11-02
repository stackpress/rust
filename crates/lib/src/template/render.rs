//! Render-local lexical scopes, lazy branches, and checked aggregate budgets.

use super::parse::{Argument, Kind, Node, Path};
use super::{Template, TemplateError, TemplateErrorKind, TemplateLimits, TemplateResult, helpers};
use serde_json::Value;

/// One lexical loop scope borrows its item and enclosing scope without cloning root data.
struct Scope<'a> {
    /// The live enclosing entry; synchronous child rendering cannot outlive this borrow.
    parent: Option<&'a Scope<'a>>,
    /// Current caller-owned array/object value, never an implicit property namespace.
    value: &'a Value,
    /// Explicit aliases shadow outer aliases without modifying the root object.
    item_alias: Option<&'a str>,
    key_alias: Option<&'a str>,
    /// Object key or array index, owned only for this lexical frame.
    key: Value,
    /// Zero-based iteration position, independently of an object's key spelling.
    index: Value,
    /// Scalar metadata can be borrowed by the same strict path resolver as ordinary values.
    first: Value,
    last: Value,
    odd: Value,
    even: Value,
}

/// One visited loop's lexical definition is shared by its entries, independently of their keys.
struct LoopBody<'a> {
    /// The exact parsed child list, borrowing the immutable compiled template.
    body: &'a [Node],
    /// Enclosing frame and aliases shared by all entries of this visited loop.
    parent: Option<&'a Scope<'a>>,
    item_alias: Option<&'a str>,
    key_alias: Option<&'a str>,
}

/// Independent work counters are never stored on the reusable compiled template.
struct Budget<'a> {
    /// Limits belong to the immutable template; each render owns its usage counters.
    limits: &'a TemplateLimits,
    steps: usize,
    iterations: usize,
}

impl Budget<'_> {
    /// Charge work before visiting a node, resolving a segment, or invoking a helper.
    fn step(&mut self) -> TemplateResult<()> {
        increment(&mut self.steps, self.limits.max_steps, "render step limit")
    }

    /// Charge each loop entry, including bodies that produce no output.
    fn iteration(&mut self) -> TemplateResult<()> {
        increment(
            &mut self.iterations,
            self.limits.max_iterations,
            "render iteration limit",
        )
    }
}

/// Rendering borrows the immutable template and caller data, owning only its output/counters.
struct Renderer<'a> {
    template: &'a Template,
    root: &'a Value,
    output: String,
    budget: Budget<'a>,
}

/// Enter a fresh render after the public API has validated borrowed input.
pub(super) fn render(template: &Template, data: &Value) -> TemplateResult<String> {
    let mut renderer = Renderer {
        template,
        root: data,
        output: String::new(),
        budget: Budget {
            limits: &template.options.limits,
            steps: 0,
            iterations: 0,
        },
    };
    renderer.nodes(&template.nodes, None)?;
    Ok(renderer.output)
}

impl Renderer<'_> {
    /// Visit siblings in source order and attach a node span to runtime failures.
    fn nodes(&mut self, nodes: &[Node], scope: Option<&Scope<'_>>) -> TemplateResult<()> {
        for node in nodes {
            self.node(node, scope)
                .map_err(|error| error.at(node.span.clone()))?;
        }
        Ok(())
    }

    /// Evaluate only this visited node; unselected child lists perform no lookups.
    fn node(&mut self, node: &Node, scope: Option<&Scope<'_>>) -> TemplateResult<()> {
        self.budget.step()?;
        match &node.kind {
            Kind::Text => self.append(&self.template.source[node.span.clone()]),
            Kind::Value(path) => {
                let value = lookup(self.root, scope, path, &mut self.budget)?;
                // Strings bypass a temporary allocation; other scalars have bounded spelling.
                if let Value::String(value) = value {
                    self.append(value)
                } else {
                    self.append(&helpers::scalar(value)?)
                }
            }
            Kind::Helper { name, arguments } => {
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    values.push(match argument {
                        Argument::Literal(value) => value,
                        Argument::Path(path) => lookup(self.root, scope, path, &mut self.budget)?,
                    });
                }
                self.budget.step()?;
                let helper = self.template.helpers.get(name).ok_or_else(|| {
                    TemplateError::new(TemplateErrorKind::UnknownHelper, "missing compiled helper")
                })?;
                let output = helper(&values)?;
                self.append(&output)
            }
            Kind::If {
                path,
                inverted,
                yes,
                no,
            } => {
                let condition = lookup(self.root, scope, path, &mut self.budget)?
                    .as_bool()
                    .ok_or_else(|| invalid("condition requires a boolean"))?;
                self.nodes(if condition != *inverted { yes } else { no }, scope)
            }
            Kind::Each {
                path,
                item,
                key,
                body,
            } => {
                let value = lookup(self.root, scope, path, &mut self.budget)?;
                // Keep aliases and the enclosing frame attached to this exact parsed body.
                let definition = LoopBody {
                    body,
                    parent: scope,
                    item_alias: item.as_deref(),
                    key_alias: key.as_deref(),
                };
                match value {
                    Value::Array(values) => {
                        for (index, value) in values.iter().enumerate() {
                            self.entry(
                                &definition,
                                value,
                                Value::from(index),
                                index,
                                values.len(),
                            )?;
                        }
                    }
                    Value::Object(values) => {
                        for (index, (name, value)) in values.iter().enumerate() {
                            self.entry(
                                &definition,
                                value,
                                Value::String(name.clone()),
                                index,
                                values.len(),
                            )?;
                        }
                    }
                    _ => return Err(invalid("loop requires an array or object")),
                }
                Ok(())
            }
        }
    }

    /// Create one bounded lexical frame and borrow it only for the child render.
    fn entry(
        &mut self,
        definition: &LoopBody<'_>,
        value: &Value,
        key: Value,
        index: usize,
        length: usize,
    ) -> TemplateResult<()> {
        self.budget.iteration()?;
        // Entries come from enumerate(), so length is positive and index is strictly below it.
        let scope = Scope {
            parent: definition.parent,
            value,
            item_alias: definition.item_alias,
            key_alias: definition.key_alias,
            key,
            index: Value::from(index),
            first: Value::Bool(index == 0),
            last: Value::Bool(index == length - 1),
            odd: Value::Bool(!index.is_multiple_of(2)),
            even: Value::Bool(index.is_multiple_of(2)),
        };
        self.nodes(definition.body, Some(&scope))
    }

    /// Refuse the next append before mutating the result or overflowing its length.
    fn append(&mut self, text: &str) -> TemplateResult<()> {
        let _next = self
            .output
            .len()
            .checked_add(text.len())
            .filter(|length| *length <= self.budget.limits.max_output_bytes)
            .ok_or_else(|| TemplateError::new(TemplateErrorKind::Limit, "output byte limit"))?;
        self.output.push_str(text);
        Ok(())
    }
}

/// Resolve bindings lexically before the root, charging every path segment.
fn lookup<'a>(
    root: &'a Value,
    mut scope: Option<&'a Scope<'_>>,
    path: &Path,
    budget: &mut Budget<'_>,
) -> TemplateResult<&'a Value> {
    let Some(first) = path.first() else {
        return Err(invalid("empty compiled path"));
    };
    budget.step()?;
    let mut value = if first == "@root" { Some(root) } else { None };
    while value.is_none() {
        let Some(frame) = scope else {
            break;
        };
        value = binding(frame, first);
        scope = frame.parent;
    }
    let mut value = value.or_else(|| root.get(first)).ok_or_else(missing)?;
    for segment in path.iter().skip(1) {
        budget.step()?;
        value = match value {
            Value::Object(value) => value.get(segment),
            Value::Array(value) => segment
                .parse::<usize>()
                .ok()
                .and_then(|index| value.get(index)),
            _ => return Err(invalid("path traverses a scalar")),
        }
        .ok_or_else(missing)?;
    }
    Ok(value)
}

/// Borrow one scope's aliases/metadata without spreading arbitrary item properties.
fn binding<'a>(scope: &'a Scope<'_>, name: &str) -> Option<&'a Value> {
    match name {
        "@this" => Some(scope.value),
        "@key" => Some(&scope.key),
        "@index" => Some(&scope.index),
        "@first" => Some(&scope.first),
        "@last" => Some(&scope.last),
        "@odd" => Some(&scope.odd),
        "@even" => Some(&scope.even),
        name if scope.item_alias == Some(name) => Some(scope.value),
        name if scope.key_alias == Some(name) => Some(&scope.key),
        _ => None,
    }
}

/// Checked counters make configured `usize::MAX` budgets finite without overflow.
fn increment(counter: &mut usize, maximum: usize, message: &'static str) -> TemplateResult<()> {
    *counter = counter
        .checked_add(1)
        .filter(|next| *next <= maximum)
        .ok_or_else(|| TemplateError::new(TemplateErrorKind::Limit, message))?;
    Ok(())
}

/// Missing paths never silently become empty strings.
fn missing() -> TemplateError {
    TemplateError::new(TemplateErrorKind::MissingValue, "missing value or binding")
}

/// Type diagnostics intentionally omit the visited value.
fn invalid(message: &'static str) -> TemplateError {
    TemplateError::new(TemplateErrorKind::InvalidType, message)
}
