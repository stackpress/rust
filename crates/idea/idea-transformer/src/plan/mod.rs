//! In-memory ownership claims and ordered declarations for the single aggregate writer.

use std::collections::BTreeMap;

mod path;
use crate::{GeneratedFile, Result, TransformError};
pub(crate) use path::{paths_collide, valid_path};

/// One plugin's declaration for a public module in its exclusive partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleContribution {
    /// Compiled plugin that supplied the declaration.
    pub owner: String,
    /// First output-path component containing the module.
    pub partition: String,
    /// Portable Rust module identifier, without the `.rs` extension.
    pub module: String,
}

/// Fully checked fresh output, independent of prior files on disk.
#[derive(Debug, Default)]
pub struct OutputPlan {
    /// Stable path order controls writing, independently of callback execution order.
    pub(crate) files: BTreeMap<String, (String, String)>,
    /// Executed IDs preserve registration order.
    pub(crate) executed: Vec<String>,
    /// Logical auxiliary inputs actually consumed by callbacks, sorted for reporting.
    pub(crate) inputs: Vec<String>,
    /// Declarations retain callback and contribution order, never alphabetic order.
    contributions: Vec<ModuleContribution>,
    /// Only the registry can grant a callback its write authority.
    active: Option<(String, Vec<String>)>,
    /// Incremental byte accounting avoids rescanning all text on every write.
    bytes: usize,
}

impl OutputPlan {
    /// Return selected IDs in their actual execution order.
    pub fn executed(&self) -> &[String] {
        &self.executed
    }

    /// Return sorted logical names of consumed auxiliary inputs, without their values.
    pub fn inputs(&self) -> &[String] {
        &self.inputs
    }

    /// Inspect completed file claims as `(relative_path, owner, contents)` in path order.
    pub fn files(&self) -> impl Iterator<Item = (&str, &str, &str)> {
        self.files
            .iter()
            .map(|(path, (owner, text))| (path.as_str(), owner.as_str(), text.as_str()))
    }

    /// Inspect prior module declarations without changing another plugin's contribution.
    pub fn contributions(&self) -> &[ModuleContribution] {
        &self.contributions
    }

    /// Claim one fresh file under the currently executing plugin's partition.
    ///
    /// This is available inside a [`crate::ContextTransform`], where the registry supplies
    /// ownership. Duplicate claims fail even when their contents are identical.
    ///
    /// # Errors
    /// Rejects writes outside callback scope, invalid paths, ownership conflicts and budgets.
    ///
    /// ```
    /// use stackpress_idea_transformer::{OutputPlan, Result, TransformContext};
    /// fn emit(_: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    ///     plan.write("contracts/profile.rs", "pub struct Profile;\n")?;
    ///     plan.contribute_module("contracts", "profile")
    /// }
    /// ```
    pub fn write(&mut self, path: impl Into<String>, contents: impl Into<String>) -> Result<()> {
        let path = path.into();
        // Copy only the owner ID so insertion can borrow the plan mutably after authorization.
        let owner = self.authorize(&path)?.to_owned();
        self.insert(&owner, GeneratedFile::new(path, contents))
    }

    /// Declare a public module for the aggregate owner to render after all callbacks succeed.
    ///
    /// The plugin must also write `partition/module.rs`. The aggregate owner creates the
    /// partition's `mod.rs` and the root `mod.rs`, preserving declaration order.
    ///
    /// # Errors
    /// Rejects invalid identifiers, duplicate declarations and claims outside owned partitions.
    pub fn contribute_module(&mut self, partition: &str, module: &str) -> Result<()> {
        // Validate identifiers before using them as Rust source, not merely as file paths.
        if !rust_identifier(partition) || !rust_identifier(module) || module == "mod" {
            return Err(TransformError::new("invalid generated module identifier"));
        }
        let path = format!("{partition}/{module}.rs");
        let owner = self.authorize(&path)?.to_owned();
        if self.contributions.len() >= 1024
            || self.contributions.iter().any(|item| {
                item.partition.eq_ignore_ascii_case(partition)
                    && item.module.eq_ignore_ascii_case(module)
            })
        {
            return Err(TransformError::new(
                "duplicate module contribution or contribution budget",
            ));
        }
        self.contributions.push(ModuleContribution {
            owner,
            partition: partition.into(),
            module: module.into(),
        });
        Ok(())
    }

    /// Grant authority for exactly one callback; callers cannot create this state themselves.
    pub(crate) fn begin(&mut self, id: &str, partitions: &[String]) {
        self.active = Some((id.into(), partitions.to_vec()));
    }

    /// End callback authority before invoking aggregate code.
    pub(crate) fn end(&mut self) {
        self.active = None;
    }

    /// Validate exact partition spelling; module aggregation checks its own final file claims.
    fn authorize(&self, path: &str) -> Result<&str> {
        valid_path(path)?;
        let Some((owner, partitions)) = &self.active else {
            return Err(TransformError::new(
                "output write requires an active transform",
            ));
        };
        let Some((partition, _)) = path.split_once('/') else {
            return Err(TransformError::new("transform wrote outside its partition"));
        };
        if !partitions.iter().any(|item| item == partition) {
            return Err(TransformError::new("transform wrote outside its partition"));
        }
        Ok(owner)
    }

    /// Add a validated file claim; both legacy and contextual APIs share collision policy.
    pub(crate) fn insert(&mut self, owner: &str, file: GeneratedFile) -> Result<()> {
        valid_path(&file.path)?;
        let bytes = self.bytes.checked_add(file.contents.len());
        if self.files.len() >= 1024 || bytes.is_none_or(|total| total > 1024 * 1024) {
            return Err(TransformError::new("generated output budget exceeded"));
        }
        // Prefix collisions matter on case-insensitive hosts and for file/directory conflicts.
        if self
            .files
            .keys()
            .any(|path| paths_collide(path, &file.path))
        {
            return Err(TransformError::new(
                "duplicate generated output ownership or path collision",
            ));
        }
        // Failed claims leave both ownership and aggregate byte accounting unchanged.
        self.bytes = bytes.unwrap_or_default();
        self.files.insert(file.path, (owner.into(), file.contents));
        Ok(())
    }

    /// Render all module declarations once, using only already validated plugin-owned files.
    pub(crate) fn finish_modules(&mut self) -> Result<()> {
        let mut partitions: Vec<(String, String)> = Vec::new();
        for item in &self.contributions {
            // A declaration alone cannot create a dangling module or borrow another owner.
            let path = format!("{}/{}.rs", item.partition, item.module);
            if self
                .files
                .get(&path)
                .is_none_or(|(owner, _)| owner != &item.owner)
            {
                return Err(TransformError::new(
                    "module contribution has no owned generated file",
                ));
            }
            let index = partitions
                .iter()
                .position(|(part, _)| part == &item.partition);
            let index = index.unwrap_or_else(|| {
                partitions.push((item.partition.clone(), String::new()));
                partitions.len() - 1
            });
            partitions[index]
                .1
                .push_str(&format!("pub mod {};\n", item.module));
        }
        // Empty selections leave app-supplied aggregates intact instead of inventing a module.
        if !partitions.is_empty() {
            let mut root = String::new();
            for (partition, contents) in partitions {
                root.push_str(&format!("pub mod {partition};\n"));
                self.insert(
                    "aggregate",
                    GeneratedFile::new(format!("{partition}/mod.rs"), contents),
                )?;
            }
            self.insert("aggregate", GeneratedFile::new("mod.rs", root))?;
        }
        Ok(())
    }
}

/// Accept conservative Rust module identifiers without injecting reserved keywords.
fn rust_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value != "_"
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && !matches!(
            value,
            "as" | "async"
                | "await"
                | "break"
                | "const"
                | "continue"
                | "crate"
                | "dyn"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "abstract"
                | "become"
                | "box"
                | "do"
                | "final"
                | "gen"
                | "macro"
                | "override"
                | "priv"
                | "try"
                | "typeof"
                | "unsized"
                | "virtual"
                | "yield"
        )
}

#[cfg(test)]
mod tests;
