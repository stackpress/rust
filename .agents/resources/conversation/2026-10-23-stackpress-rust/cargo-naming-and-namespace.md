# Cargo naming and the Stackpress namespace

Conversation update: 2026-10-23. No packages have been created, reserved, or published. Examples below illustrate the intended structure; they are not imports available from a released Rust Stackpress package.

## Author request

The author wants the namespace `stackpress`, illustrated as `stackpress::inquire-mysql`.

This is achievable as the public Rust API namespace, with an identifier adjustment:

```rust
use stackpress::inquire_mysql;
```

Rust source identifiers do not use hyphens. Cargo package names can use hyphens, and the default library name replaces them with underscores.

## Distinguish the names

| Concern | Illustrative name | Meaning |
| --- | --- | --- |
| Facade Cargo package | `stackpress` | Convenient dependency/application API |
| Adapter Cargo package | `stackpress-inquire-mysql` | Independently implemented/publishable adapter |
| Adapter direct Rust crate path | `stackpress_inquire_mysql` | Default library identifier when depended on directly |
| Facade Rust path | `stackpress::inquire_mysql` | Explicit public re-export of the adapter |
| Optional facade feature | `inquire-mysql` | Build selection for the adapter/export |

An additional path such as `stackpress::inquire::mysql` is technically possible through module organization, but is not necessary to satisfy the author's example. Do not introduce a different hierarchy as a confirmed author decision.

Cargo packages have package names rather than nested `stackpress::...` package scopes. A common `stackpress-` prefix is naming convention, not an automatic exclusive reservation of every prefixed package. Registry availability/ownership has not been checked. Git/path dependencies remain available during development regardless of a future crates.io naming decision.

## Proposed facade implementation

Inside the facade package's Cargo.toml, with an illustrative sibling layout:

```toml
[features]
default = []
inquire-mysql = ["dep:stackpress-inquire-mysql"]

[dependencies]
stackpress-inquire-mysql = { path = "../stackpress-inquire-mysql", optional = true }
```

Inside the facade's Rust library:

```rust
#[cfg(feature = "inquire-mysql")]
pub use stackpress_inquire_mysql as inquire_mysql;
```

Consumers enable the facade feature and can then use `stackpress::inquire_mysql`. The exact exported types/functions belong to the eventual adapter implementation. Avoid inventing an existing `Connection`, `connect`, or other API before it is designed.

Other framework crates can use the same pattern for `stackpress::lib`, `stackpress::ingest`, `stackpress::idea`, and `stackpress::inquire`. These additional paths are illustrative, not an approved complete export catalog.

## Dependency and feature consequences

- The implementation remains in focused crates. A facade re-export does not merge their source or force one large internal library.
- Standalone users can depend on `stackpress-inquire-mysql` directly and use `stackpress_inquire_mysql`, with no facade dependency.
- Registering an application plugin is separate from making its code importable. Enabling a dependency/export does not implicitly activate all its runtime behavior unless an explicit composition rule says so.
- Adding a direct adapter dependency does not automatically inject it into another crate's public namespace. The facade must depend on and re-export it.
- Optional dependencies and feature-gated exports keep unwanted database drivers/platform integrations out of the chosen dependency path, subject to Cargo's additive feature/dependency resolution.
- Build-time plugin selection and public API feature selection should be aligned where appropriate but remain distinct concepts.
- The facade depends on foundation/adapter crates; they should not depend back on the facade and create cycles.
- Cargo dependency aliases can alter local crate identifiers without changing the package's published name. Prefer conventional names unless an alias clarifies a specific boundary.

## Decisions versus open details

Confirmed intent: a convenient `stackpress::` namespace and idiomatic Rust syntax.

Proposed normalization: `stackpress::inquire_mysql` for the author's hyphenated example.

Still open: exact package catalog, feature catalog/defaults, complete module hierarchy, crate availability, ownership, release versions, publication layout, and standalone versus facade documentation strategy.

No implementation or publishing is authorized merely by documenting the namespace.

## Primary references checked

- [Cargo package names](https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field)
- [Library target names](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#library)
- [Rust public re-exports](https://doc.rust-lang.org/reference/items/use-declarations.html#use-visibility)
- [Dependency aliases](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml)
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
