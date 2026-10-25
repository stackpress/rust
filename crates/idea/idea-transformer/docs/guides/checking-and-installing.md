# Check, install, and remove owned files

Use the same fresh plan for preview and installation so a freshness check cannot conceal a
different generator path. A successful install reports what changed; a later check proves that
the installed tree matches the plan.

## Install in a temporary root

Run this with transformer and `tempfile` dependencies. The first check leaves `generated`
absent, installation writes it, and the second check is current.

```rust
use stackpress_idea_transformer::{GeneratedFile, OutputWorkspace, Registry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parent = std::env::temp_dir().canonicalize()?;
    let temp = tempfile::tempdir_in(parent)?;
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root)?;
    let registry = Registry::new();
    let plan = registry.plan(
        &Default::default(),
        vec![GeneratedFile::new("index.txt", "Hello, Ada\n")],
    )?;
    assert!(!workspace.check(&plan)?.is_current());
    assert!(!root.exists());
    let installed = workspace.install_report(plan)?;
    assert_eq!(installed.generated, ["index.txt"]);

    let same = registry.plan(
        &Default::default(),
        vec![GeneratedFile::new("index.txt", "Hello, Ada\n")],
    )?;
    assert!(workspace.check(&same)?.is_current());

    // A new empty selection drops only the file promised by the prior manifest.
    let empty = registry.plan(&Default::default(), vec![])?;
    let removed = workspace.install_report(empty)?;
    assert_eq!(removed.removed, ["index.txt"]);
    assert!(!root.join("index.txt").exists());
    assert!(root.join(".stackpress-generated.json").is_file());
    Ok(())
}
```

The final marker remains even though the plan has no output files. That distinguishes an owned
empty root from an uninitialized one. Removing a model or selected transform works the same way:
omit its old files and registrations from the new plan, then install.

## Handle drift explicitly

A changed schema may produce a valid stale report. An edited installed file or an unknown
manual file instead produces an ownership error in both modes. Preserve and review those edits
before explicitly restoring disposable generated output. Neither check nor install grants
automatic permission to overwrite them.

Rust output requires the installed `rustfmt`; invalid Rust or formatter failure stops before
locking or writing output. Use a consistent toolchain for byte comparisons. Non-Rust output
retains the exact bytes you supply, including newline policy.

You have verified first install, no-write freshness, and stale cleanup. Integrate a check before
the app build using the [app-owned tool guide](../../../idea/docs/guides/app-owned-codegen.md);
an ordinary Cargo build does not check schema freshness.
