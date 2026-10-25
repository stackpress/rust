# Output report API

Reports expose logical names and relative paths without native roots, source contents,
environment values, or timestamps. Source: [output/report.rs](../../src/output/report.rs).

```rust
use stackpress_idea_transformer::{CheckReport, GeneratedFile, OutputWorkspace, Registry};
let report = CheckReport {
    differences: vec![],
};
```

The snippets below use this setup independently. Native examples additionally require `tempfile`.

## 1. CheckReport

CheckReport implements Debug and Serialize. Its public field `differences: Vec<String>` lists
added, changed, or removed paths in deterministic order when returned by the workspace. It can
include `.stackpress-generated.json` when ownership has not yet been installed. Comparing owners
as well as formatted bytes means an ownership change can be stale even when contents agree.

### 1.1. is_current

`is_current(&self) -> bool`

Ask whether a comparison found differences.

```rust
assert!(report.is_current());
let stale = CheckReport {
    differences: vec!["data/profile.txt".into()],
};
assert!(!stale.is_current());
```

**Returns** True exactly when differences is empty. Since install returns the pre-install
comparison, its report does not represent post-install freshness. Report fields are public;
manual construction does not independently inspect disk.

## 2. GenerationReport

GenerationReport implements Debug, Serialize, PartialEq, and Eq. Obtain it through
`OutputWorkspace::install_report` when a tool needs classified output.

```rust
let parent = std::env::temp_dir().canonicalize()?;
let temp = tempfile::tempdir_in(parent)?;
let workspace = OutputWorkspace::new(temp.path().join("generated"))?;
let plan = Registry::new().plan(
    &Default::default(),
    vec![GeneratedFile::new("index.txt", "Profiles\n")],
)?;
let receipt = workspace.install_report(plan)?;
assert_eq!(receipt.generated, ["index.txt"]);
assert_eq!(receipt.contract_version, 1);
```

All fields are public. The example's first install has one generated path and no selected
transforms because an empty registry supplied only an app aggregate.

| Field | Type | Meaning |
| --- | --- | --- |
| `inputs` | `Vec<String>` | Actually consumed auxiliary names, sorted/deduplicated; not source graph paths. |
| `selected` | `Vec<String>` | Actual transform execution order. |
| `generated` | `Vec<String>` | Added/changed planned paths, sorted. |
| `unchanged` | `Vec<String>` | Byte- and owner-identical planned paths, sorted. |
| `removed` | `Vec<String>` | Previously owned paths omitted from the new plan, sorted. |
| `contract_version` | `u32` | Writer receipt/ownership revision, currently 1. |

The marker is excluded from generated/unchanged/removed. Runtime contract IDs and revisions
are separately supplied through TransformSelection; this report version is not a plugin ABI.

Related: [workspace](output-workspace.md), [selection](transform-selection.md).
