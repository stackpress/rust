# Compile inputs and parse limits

`CompileInputs` makes environment evaluation explicit; `ParseLimits` bounds the checked parser
and data operations. Source: [lib.rs](../../src/lib.rs).

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, ParseLimits, parse};

fn main() -> Result<(), IdeaError> {
    let mut inputs = CompileInputs {
        limits: ParseLimits {
            max_bytes: 4096,
            max_depth: 8,
        },
        ..Default::default()
    };
    inputs.environment.insert("TITLE".into(), "Profiles".into());
    let schema = parse(
        "settings.idea",
        "prop Settings { title env(\"TITLE\") }",
        &inputs,
    )?;
    assert_eq!(schema.prop["Settings"]["title"], "Profiles");
    Ok(())
}
```

Contents: [CompileInputs](#1-compileinputs) · [ParseLimits](#2-parselimits) ·
[Accounting](#3-accounting-and-boundaries)

## 1. CompileInputs

`CompileInputs` is Debug, Clone, and Default. It has no public inherent methods.

| Public field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `environment` | `IndexMap<String, String>` | Empty | Only these keys are visible to `env("NAME")`. |
| `missing_env_is_empty` | `bool` | `false` | Explicitly allow missing keys to become empty strings. |
| `limits` | `ParseLimits` | See below | Per-operation bounded-resource policy. |

Environment values always compile to strings. Neither parsing nor compilation reads the ambient
environment. Clones own independent ordinary data; invocation state is not shared or cached globally.

## 2. ParseLimits

`ParseLimits` is Debug, Clone, Copy, and Default. All fields are public:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `max_bytes` | `usize` | `1_048_576` | Source bytes and related stage-specific byte budgets. |
| `max_depth` | `usize` | `64` | Caller-selected nesting policy, capped by `HARD_DEPTH`. |

Depth counts containers at the relevant representation. The Idea declaration object contributes
to data depth; JSON additionally counts its schema/envelope containers. Equal numeric policies
can therefore accept one representation and reject another.

### 2.1. HARD_DEPTH

`ParseLimits::HARD_DEPTH: usize = 64`

This associated constant is the non-raiseable recursive ceiling. A larger `max_depth` field
still uses an effective maximum of 64.

### 2.2. max_nodes

`max_nodes(self) -> usize`

Return the derived node budget for the supplied byte limit.

```rust
use stackpress_idea_parser::ParseLimits;

fn main() {
    let limits = ParseLimits::default();
    assert_eq!(limits.max_nodes(), 100_000);
    let small = ParseLimits {
        max_bytes: 512,
        ..limits
    };
    assert_eq!(small.max_nodes(), 512);
}
```

**Returns** `min(max_bytes, 100_000)`. The method only computes policy; it does not inspect data
or return an error.

### 2.3. max_collection

`max_collection(self) -> usize`

Return the maximum entries in one checked collection.

```rust
use stackpress_idea_parser::ParseLimits;

fn main() {
    let limits = ParseLimits::default();
    assert_eq!(limits.max_collection(), 10_000);
    let small = ParseLimits {
        max_bytes: 512,
        ..limits
    };
    assert_eq!(small.max_collection(), 512);
}
```

**Returns** `min(max_bytes, 10_000)`, with no mutation or fallible work. Lower byte limits lower
both this policy and `max_nodes`; callers cannot configure those budgets independently.

## 3. Accounting and boundaries

Checked operations apply budgets before constructing recursive syntax or materializing repeated
expansions. The main accounting boundaries are:

| Boundary | Checked work |
| --- | --- |
| Text parsing/compilation | Source text and logical name bytes, aggregate environment key/value bytes, syntax nodes, collection widths, nesting. |
| Environment evaluation | Aggregate materialized value bytes, charging every occurrence before copying. |
| Reference collection | Repeated owner/path/source metadata before allocating records. |
| Reference finalization | Site shape, dependency depth/cycles, serialized copy bytes, resulting data depth/size. |
| Checked JSON | Input/output bytes, containers, nodes, collection widths, structural validity. |

Compilation rechecks a retained AST against caller-lowered limits. Expansion or encoded JSON can
exceed a budget even if the source fits. A stage need not use its entire budget before a different
constraint fails; inspect the returned diagnostic rather than inferring one global size metric.

`SourceFile::new` owns/indexes text before these checks. Public config fields also permit caller
construction outside the checked boundaries. Validate assembled config before recursive export
or finalization; direct Clone/Serde and destruction of hostile caller-built values are the caller's
responsibility.

Related: [environment guide](../guides/environment-inputs.md), [diagnostics](error.md),
[checked schema methods](schema.md#2-checked-json-operations).
