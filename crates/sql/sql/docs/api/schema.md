# Fields, keys, types, and defaults

These owned schema values are inputs to Create, Alter, and TableDefinition. Source:
[schema.rs](../../src/schema.rs). Constructors retain data; enclosing build/render validates it.

```rust
use stackpress_sql::{DefaultValue, Field, ForeignKey, Index, Length, ReferentialAction, SqlType};

fn main() {
    let id = Field::new("id", SqlType::Integer)
        .not_null()
        .auto_increment();
    let name = Field::new("name", SqlType::Varchar)
        .length(Length::Size(80))
        .default("");
    let created = Field::new("created", SqlType::DateTime).default(DefaultValue::CurrentTimestamp);
    let unique = Index::new("profile_name", ["name"]).unique();
    let mut team = ForeignKey::new("profile_team", ["team_id"], "team", ["id"]);
    team.on_delete = ReferentialAction::SetNull;
    assert!(id.auto_increment && unique.unique);
    assert_eq!(name.nullable, true);
    assert!(created.default.is_some());
}
```

An auto-increment field also needs a compatible sole integer primary key in its enclosing
CREATE. A SET NULL relationship needs nullable, non-primary local fields.

Contents: [Types](#1-sqltype-length-and-defaultvalue) · [Fields](#2-field) ·
[Indexes](#3-index) · [Foreign keys](#4-foreignkey-and-referentialaction)

## 1. SqlType, Length, and DefaultValue

SqlType and DefaultValue are Clone/Debug/PartialEq; Length is also Copy/Eq. Their public variants
are the complete storage-description surface:

| Type | Variants |
| --- | --- |
| `SqlType` | `Integer`, `Real`, `Boolean`, `Text`, `Varchar`, `Blob`, `Json`, `DateTime`, `Decimal`, `Trusted(DialectId, TrustedSql)`. |
| `Length` | `Size(i32)`, `Precision { precision: i32, scale: i32 }`. |
| `DefaultValue` | `Value(SqlValue)`, `CurrentTimestamp`, `Trusted(DialectId, TrustedSql)`. |

A blanket `From<T>` converts `T: Into<SqlValue>` into DefaultValue::Value. No default is
`None`; an explicit NULL default is `Some(DefaultValue::Value(SqlValue::Null))`.
False, zero, empty text, and NULL are retained. NULL defaults require nullable fields.

Size applies only to Varchar and must be positive. Precision applies only to Decimal, with
positive precision and scale between zero and precision. SQLite Varchar length is affinity
metadata, not enforced length; exact Decimal storage is Unsupported. MySQL Varchar requires an
explicit length through 16,383; Decimal precision/scale are bounded to 65/30 there.

Typed defaults must match compatible storage. DateTime text defaults must pass canonical UTC
validation. CurrentTimestamp requires Text or DateTime. Trusted types/defaults require a matching
dialect and audited syntax; they do not bypass all validation. See [TrustedSql](query.md#3-raw-statements-and-trust).

## 2. Field

Field is Clone/Debug/PartialEq, with all configuration public:

| Field | Type | Initial value / meaning |
| --- | --- | --- |
| `name` | `String` | Literal column name. |
| `data_type` | `SqlType` | Explicit storage type. |
| `nullable` | `bool` | true. |
| `length` | `Option<Length>` | None. |
| `default` | `Option<DefaultValue>` | None. |
| `auto_increment` | `bool` | false. |
| `unsigned` | `bool` | false; supported only for MySQL Integer. |
| `comment` | `Option<String>` | None; SQLite rejects comments. |

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(name, data_type)` | `impl Into<String>`, `SqlType` | Construct a nullable field. |
| `not_null(self)` | none | Set nullable=false without inventing a default. |
| `default(self, value)` | `impl Into<DefaultValue>` | Set an explicit default. |
| `auto_increment(self)` | none | Request generated integer identity. |
| `length(self, length)` | `Length` | Set size/precision metadata. |

**Returns** Each returns Self, without IO. Assign `unsigned` or `comment` directly when required.
Rendering rejects unsupported annotations instead of discarding them. Auto-increment requires a
sole Integer primary with no default. MySQL comments containing backslash/NUL are outside the
renderer subset; comment and default input sizes are bounded.

## 3. Index

Index is Clone/Debug/Eq/PartialEq. Fields are `name: String`, `columns: Vec<String>`, and
`unique: bool`. Column order matters for compound indexes.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(name, columns)` | `impl Into<String>`, `impl IntoIterator<Item = impl Into<String>>` | Define a regular named index. |
| `unique(self)` | none | Require uniqueness. |

**Returns** Self. Validation checks nonempty columns, membership, and naming conflicts. For
same-name definitions on identical ordered columns, unique takes precedence; incompatible
same-name definitions fail. MySQL TEXT/BLOB/JSON indexes need a prefix/expression design outside
this API, so rendering rejects them.

## 4. ForeignKey and ReferentialAction

ForeignKey is Clone/Debug/Eq/PartialEq. It exposes `name: String`, `columns: Vec<String>`,
`table: String`, `foreign_columns: Vec<String>`, `on_delete: ReferentialAction`, and
`on_update: ReferentialAction`.

`ForeignKey::new(name, columns, table, foreign_columns)` takes literal names via
`impl Into<String>` and column lists via `impl IntoIterator<Item = impl Into<String>>`.
Both action fields initially use NoAction; set the public fields to choose another policy.

**Returns** Self without validating a live parent table. Enclosing validation checks local
fields, matching nonempty arity, and compatible actions. SET NULL cannot target NOT NULL or
primary fields. Parent existence/keys and existing native state remain execution concerns.

ReferentialAction is Copy/Clone/Debug/Default/Eq/PartialEq, with variants `NoAction` (default),
`Restrict`, `Cascade`, `SetNull`, and `SetDefault`. MySQL InnoDB SET DEFAULT actions are Unsupported.

Related: [Create](create.md), [Alter](alter.md), [schema diff](schema-diff.md),
[schema guide](../guides/schema-changes.md).
