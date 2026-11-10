//! Conservative markup-only diffs; any uncertain Rust, prop, location or parser change rebuilds.

use super::change::Snapshot;
use super::change::Work;
use leptos_hot_reload::{ViewMacros, diff::Patches};
use quote::ToTokens;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use syn::visit_mut::VisitMut;

/// Parsed macros correspond to the last complete build plus accepted patch history.
pub(super) struct Markup {
    /// Blog root gives the same relative marker IDs emitted by Cargo's `view!` expansions.
    root: PathBuf,
    /// Last accepted source bytes; failed builds never advance this baseline.
    sources: BTreeMap<PathBuf, Vec<u8>>,
    /// Leptos's actual parser and diff engine, not a parallel markup implementation.
    views: ViewMacros,
    /// A partial/erroring patch attempt disables diffs until a complete build resets them.
    valid: bool,
    /// Accepted changes still need compiling before another kind of full reload.
    dirty: bool,
}

impl Markup {
    /// Read a stable successful-build snapshot; parse failures simply disable patching.
    pub fn new(root: &Path, snapshot: &Snapshot) -> Self {
        let paths = snapshot
            .0
            .keys()
            .filter(|p| eligible(p))
            .map(|p| root.join(p))
            .collect::<Vec<_>>();
        let views = ViewMacros::new();
        let stable = paths.iter().all(|p| {
            std::fs::read(p).ok().as_ref() == snapshot.0.get(p.strip_prefix(root).unwrap_or(p))
        });
        let valid = stable && views.update_from_paths(&paths).is_ok();
        Self {
            root: root.into(),
            sources: snapshot.0.clone(),
            views,
            valid,
            dirty: false,
        }
    }

    /// A later full reload must compile every accepted patch into both SSR and WASM.
    pub fn rebuild_work(&self, mut work: Work) -> Work {
        if self.dirty && (work.restart || work.browser || work.native || work.generate) {
            work.browser = true;
            work.native = true;
            work.restart = true;
        }
        work
    }

    /// Return the official serialized Patches only if every changed file is pure markup.
    pub fn patch(&mut self, paths: &BTreeSet<PathBuf>, next: &Snapshot) -> Option<String> {
        if !self.valid || paths.is_empty() {
            return None;
        }
        for path in paths {
            if !eligible(path) || !pure(self.sources.get(path)?, next.0.get(path)?) {
                return None;
            }
            // Source may change again while the debounced batch is being prepared.
            if std::fs::read(self.root.join(path)).ok().as_ref() != next.0.get(path) {
                return None;
            }
        }
        // A failed/partial diff cannot leave the cache trusted for a later edit.
        self.valid = false;
        let mut all = Vec::new();
        for path in paths {
            let absolute = self.root.join(path);
            let patches = self.views.patch(&absolute.try_into().ok()?).ok()??;
            for (id, patches) in patches.0 {
                let (_, line) = id.rsplit_once('-')?;
                let id = format!(
                    "{}-{line}",
                    path.to_string_lossy().replace(['/', '\\'], "-")
                );
                all.push((id, patches));
            }
        }
        if all.is_empty() {
            return None;
        }
        let json = serde_json::to_string(&Patches(all)).ok()?;
        for path in paths {
            self.sources.insert(path.clone(), next.0.get(path)?.clone());
        }
        self.valid = true;
        self.dirty = true;
        Some(json)
    }
}

/// Only the app's shared view source is patched; host, startup, models and config rebuild.
fn eligible(path: &Path) -> bool {
    path.starts_with("plugins/app/src/views") && path.extension().is_some_and(|e| e == "rs")
}

/// Removing macro tokens exposes any changed Rust outside markup as a different AST.
#[derive(Default)]
struct Macros(Vec<String>);

impl VisitMut for Macros {
    /// Freeze each macro's top-level token structure, retaining every embedded Rust group.
    fn visit_macro_mut(&mut self, node: &mut syn::Macro) {
        if node.path.is_ident("view") {
            let frozen = node
                .tokens
                .clone()
                .into_iter()
                .map(|token| {
                    // Only top-level string literals can be text/static HTML attributes.
                    // Literals inside Rust groups and all identifiers remain exact.
                    if syn::parse_str::<syn::LitStr>(&token.to_string()).is_ok() {
                        "\"__blog_static_string__\"".into()
                    } else {
                        token.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            self.0.push(frozen);
            node.tokens = Default::default();
        } else {
            syn::visit_mut::visit_macro_mut(self, node);
        }
    }
}

/// Parse both versions with syn and Leptos, preserving macro counts and starting lines.
fn pure(before: &[u8], after: &[u8]) -> bool {
    (|| -> Option<bool> {
        let before = std::str::from_utf8(before).ok()?;
        let after = std::str::from_utf8(after).ok()?;
        let mut old = syn::parse_file(before).ok()?;
        let mut new = syn::parse_file(after).ok()?;
        let mut old_macros = Macros::default();
        old_macros.visit_file_mut(&mut old);
        let mut new_macros = Macros::default();
        new_macros.visit_file_mut(&mut new);
        if old.into_token_stream().to_string() != new.into_token_stream().to_string() {
            return Some(false);
        }
        let directory = tempfile::tempdir().ok()?;
        let old_path = directory.path().join("before.rs");
        let new_path = directory.path().join("after.rs");
        std::fs::write(&old_path, before).ok()?;
        std::fs::write(&new_path, after).ok()?;
        let old =
            serde_json::to_value(ViewMacros::parse_file(&old_path.try_into().ok()?).ok()?).ok()?;
        let new =
            serde_json::to_value(ViewMacros::parse_file(&new_path.try_into().ok()?).ok()?).ok()?;
        let old = old.as_array()?;
        let new = new.as_array()?;
        if old.is_empty() || old.len() != new.len() || old.len() != old_macros.0.len() {
            return Some(false);
        }
        let mut changed = false;
        for (index, (old, new)) in old.iter().zip(new).enumerate() {
            let line = |value: &Value| {
                value["id"]
                    .as_str()
                    .and_then(|s| s.rsplit_once('-'))
                    .map(|(_, line)| line.to_owned())
            };
            if line(old) != line(new) {
                return Some(false);
            }
            let old = &old["template"];
            let new = &new["template"];
            if old == new {
                // Leptos intentionally omits dynamic attribute expressions from its tree.
                // Their Rust tokens must still match, even in an otherwise unchanged macro.
                if old_macros.0[index] != new_macros.0[index] {
                    return Some(false);
                }
                continue;
            }
            changed = true;
            // Entirely static trees can change structure. Mixed views allow literal text
            // and attribute edits only; component props/children and Rust stay exact.
            if !(static_tree(old) && static_tree(new))
                && (old_macros.0[index] != new_macros.0[index] || protected(old) != protected(new))
            {
                return Some(false);
            }
        }
        Some(changed)
    })()
    .unwrap_or(false)
}

/// Leptos serializes nodes as externally tagged enums; dynamic nodes are never static.
fn static_tree(node: &Value) -> bool {
    if node.get("Text").is_some() {
        return true;
    }
    if let Some(children) = node.get("Fragment").and_then(Value::as_array) {
        return children.iter().all(static_tree);
    }
    if let Some(element) = node.get("Element") {
        return element["attrs"].as_array().is_some_and(|attrs| {
            attrs
                .iter()
                .all(|attr| attr[1].get("Static").is_some() || attr[1] == "Boolean")
        }) && element["children"]
            .as_array()
            .is_some_and(|children| children.iter().all(static_tree));
    }
    false
}

/// Keep every component/prop/dynamic subtree exact, including its position in the tree.
fn protected(node: &Value) -> Vec<Value> {
    let mut values = Vec::new();
    if node.get("Component").is_some() || node.get("DynChild").is_some() {
        values.push(node.clone());
    } else if let Some(children) = node
        .get("Fragment")
        .or_else(|| node.get("Element").and_then(|e| e.get("children")))
        .and_then(Value::as_array)
    {
        for (index, child) in children.iter().enumerate() {
            for value in protected(child) {
                values.push(serde_json::json!([index, value]));
            }
        }
    }
    values
}

#[cfg(test)]
#[path = "../../tests/cases/markup.rs"]
mod tests;
