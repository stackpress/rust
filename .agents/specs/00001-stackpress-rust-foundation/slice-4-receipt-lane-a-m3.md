# Slice 4 lane A M3 receipt

Task 00007, lane A (SQL core), milestone M3, 2026-10-26. Base `ea760da`, branch `trial/3-s4-lane-a-core`. Implementer evidence for coordinator review; no commit, merge, acceptance or publication is claimed.

## Public documentation and first success

- The crate root opens with a small database-free SELECT, imports `Render`, checks exact SQL/binds, explains inspection, and gives the first runnable command. A task table routes readers to all six builders, expressions/JSON, capabilities, schema diff, plan execution, rows/DTO decoding and trusted SQL.
- All **412 declared public types/traits/aliases, methods, fields and enum variants** have Rustdoc and a runnable local doctest. A separate audit found all 412 items/anchors in generated Rustdoc HTML. The root adds two runnable examples and two intentional compile-fail misuse proofs: **416 core doctests total**, **414 runnable + 2 compile-fail**, no ignored or no_run examples.
- Public reference contracts cover default/input/output budgets, validation timing, deferred builder validation, owned results, strict decoding, duplicate names/ordinals, trusted authorship, JSON number/null/path semantics, native capability limits, transaction ordering/cancellation/completion, and error/rollback preservation. Stale milestone/SQLite-only comments were corrected.
- Four standalone programs were compiled and actually run: `build_inspect` (six builders, expressions/JSON, SQL/binds/data inspection); `capabilities` (every flag and unsupported MySQL RETURNING); `schema_diff` (explicit definitions/no-op/rename); `execute_plan` (ordered outcomes and caller-owned rollback). Run any with `cargo run -p stackpress-sql --example NAME`.
- Async doctests and the execution example use the explicitly labeled recorder in `examples/support/mod.rs`. It records statement order, commit and rollback; it does not interpret SQL or prove native storage. Tokio remains an existing dev-dependency. First-success rendering needs no runtime.

Only `crates/sql/sql/**` source docs/examples and this receipt changed. Removing comments/blank lines from library source yields exactly the base implementation. Database/TransactionSession/Engine/Transaction signatures, manifests and Cargo.lock are unchanged; no new dependency was resolved. The normal-edge tree contains only `stackpress-sql` itself.

## Verification

Compiler/Cargo 1.96.0, LLVM 22.1.2, cargo-llvm-cov 0.8.6, native `aarch64-apple-darwin`; default valid workspace features. Both network URL variables were unset. Normal checks used `CARGO_TARGET_DIR=target CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked --offline
cargo test --workspace --doc --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --offline
cargo +1.96.0 check --workspace --all-targets --locked --offline
cargo tree -p stackpress-sql --edges normal --locked --offline
git diff --check
```

All passed. Workspace: **391 passed, 0 failed, 47 ignored** test/harness entries; **852 doctests passed, 0 failed/ignored**. Core: **69** tests passed, **16 opt-in network cases visibly ignored**, and the 416 doctests above. The 47 workspace ignored entries include network opt-ins and existing watchdog child-entry tests, whose parents run their scenarios. Four standalone example runs passed separately. An initial reference-doc build caught unquoted `Option<T>`; it was fixed and the warning-clean workspace docs build rerun successfully. Initial doctest authoring failures are retained in the evidence, not counted as successful checks.

Fresh coverage cleaned and reused that same target root; no other crate's tests or prior profiles contributed. Both network URLs remained unset:

```sh
cargo clean --target-dir target
CARGO_TARGET_DIR=target CARGO_LLVM_COV_TARGET_DIR=target \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 \
cargo llvm-cov -p stackpress-sql --locked --offline --fail-under-lines 70 \
  --ignore-filename-regex '/tests/' --json \
  --output-path /private/tmp/stackpress-s4-lane-a-m3-evidence/core-coverage.json
```

**2,794/2,926 = 95.49%** executable library lines covered, passing the 70% development floor. Only test source paths are excluded, including internal `src/tests`; no production library source is excluded. Stable doctests run separately. Core coverage does not use SQLite adapter tests or live PostgreSQL/MySQL runs. Exactly one target root (`target`) was used sequentially for normal and instrumented builds. Logs, raw coverage JSON, source hashes, API/HTML audits, exact commands/counts and reconciliation data remain at `/private/tmp/stackpress-s4-lane-a-m3-evidence/`.

## Inventory reconciliation

This is the per-row implementation ledger for [core inventory](slice-4-design-inventory-core.md) and [core tests](slice-4-design-tests-core.md); load those for original disposition/provenance. All **39 core rows containing Now** and **121 test rows containing Now** are accounted for, including mixed No/Now replacement rows. Source test suite plus original row number and exact title preserve identity. Of the 121 test rows, **119 are implemented/tested** and **2 have explicit SQL-text deviation D1**. `binary::function` below names an exact test function in `crates/sql/sql/tests/BINARY.rs`; each reference was checked against source. Repeated names indicate one test deliberately proves several related original declarations, not inferred completion from a sampler.

The existing `No`/`Later S4-*` dispositions remain in the design files. G26 amends their historical native-network deferral for opt-in renderer validation only; production network adapters remain deferred. No accepted design text was rewritten during this evidence reconciliation.

### Core exports, methods and behavior

The numbering below follows only the Now-containing rows, in original document order.

| Core inventory Now row (original order) | Status and exact Rust proof |
| --- | --- |
| 1. `Engine` constructor, `connection`, `dialect` | Implemented/tested: `engine::engine_validates_before_driver_and_forwards_owned_results` |
| 2. `Engine.alter/create/delete/insert/select/update` | Implemented/tested: `engine::engine_conveniences_never_attach_or_execute` |
| 3. `Engine.query(QueryObject)`, `query(sql,values)` | Implemented/tested: `engine::engine_validates_before_driver_and_forwards_owned_results`; `contracts::lexical_gate_distinguishes_code_from_quoted_text` |
| 4. `Engine.drop/rename/truncate` | Implemented/tested: `ddl::table_drop_rename_truncate_and_invalid_names`; `dialects::commands_and_offset_only` |
| 5. `Engine.diff(from,to)` | Implemented/tested: `diff::field_properties_are_structural_not_identity`; `diff::compound_keys_add_remove_change_and_order` |
| 6. `Engine.transaction(callback)` | Implemented/tested: `engine::transaction_forwarding_completion_and_error_preservation` |
| 7. `Engine.sql` template segments/backtick replacement | Implemented/tested: `contracts::lexical_gate_distinguishes_code_from_quoted_text`; `dml::explicit_trusted_expression_boundary` |
| 8. `Create` constructor, `addField/addForeignKey/addKey/addPrimaryKey/addUniqueKey`, `build/query` | Implemented/tested: `ddl::create_full_definition_and_ordered_plan`; `ddl::create_validates_keys_and_foreign_constraints` |
| 9. `Alter` constructor including source fields, all Create add methods; `changeField/renameField/removeField/removeForeignKey/removeKey/removePrimaryKey/removeUniqueKey`, `build/query` | Implemented/tested: `ddl::alter_native_plan_and_inspection`; `dialects::alter_full_native_plan`; `ddl::sqlite_alter_rejects_unsupported_operations_as_a_whole` |
| 10. `Insert` constructor, `values(record/records)`, `returning`, `build/query` | Implemented/tested: `dml::insert_bulk_returning_and_inspection`; `dml::insert_rejects_empty_arity_duplicate_and_conflicting_input` |
| 11. `Select` constructor default `*`; `select/from/join/order/limit/offset/where/whereJson/whereJsonContains/build/query` | Implemented/tested: `dml::select_build_and_complete_sql`; `dml::select_sort_and_pagination`; `json_values::json_membership_and_dml_share_one_expression_contract` |
| 12. `Update` constructor, `set/where/whereJson/whereJsonContains/build/query` | Implemented/tested: `dml::update_and_delete_build_and_returning`; `dml::mutation_scope_never_silently_widens` |
| 13. `Delete` constructor, `where/whereJson/whereJsonContains/build/query` | Implemented/tested: `dml::update_and_delete_build_and_returning`; `dml::mutation_scope_never_silently_widens` |
| 14. Create/Alter multiple statements in transaction, last rows result | Implemented/tested: `engine::engine_plan_success_and_caller_owned_transaction`; `engine::engine_plan_failure_preserves_original_and_cleanup_errors`; `engine::engine_plan_validation_and_cancelled_owner` |
| 15. Unique/index same-name precedence | Implemented/tested: `ddl::create_and_alter_unique_precedence_and_conflicts` |
| 16. Rename remove/add reconciliation | Implemented/tested: `ddl::alter_direct_rename_and_reconciliation`; `ddl::alter_rejects_ambiguous_or_incomplete_renames`; `diff::diff_rename_same_changed_and_missing_halves` |
| 17. `Field` | Implemented/tested: `ddl::fields_validate_types_lengths_and_auto_increment`; `ddl::typed_defaults_preserve_false_zero_empty_and_quotes`; `dialects::fields_foreign_actions_and_invalid_lengths` |
| 18. `ForeignKey` | Implemented/tested: `ddl::create_validates_keys_and_foreign_constraints`; `ddl::foreign_set_null_cannot_target_primary_columns` |
| 19. `AlterFields`, `AlterKeys`, `AlterUnqiues`, `AlterPrimaries`, `AlterForeignKeys` | Implemented/tested: `ddl::alter_native_plan_and_inspection`; `dialects::alter_full_native_plan` |
| 20. `Column`, `Selector`, `Table` | Implemented/tested: `dml::select_defaults_and_selector_boundaries`; `dml::hostile_identifiers_and_values_stay_data` |
| 21. `JoinType`, `Join`, `OrderType`, `Sort` | Implemented/tested: `dml::joins_and_operand_validation`; `dml::select_sort_and_pagination` |
| 22. `Where`, `WhereJson`, `WhereBuilder` | Implemented/tested: `dml::expressions_cover_comparisons_groups_null_in_and_like`; `json_values::json_membership_and_dml_share_one_expression_contract` |
| 23. `StrictValue`, `StrictOptValue`, `FlatValue`, `JSONScalarValue`, `Value` | Implemented/tested: `json_values::value_conversions_are_exact_and_fallible`; `json_values::bounded_explicit_json_encoding` |
| 24. `JsonDialect` (`selector/extract/contains/where`) | Implemented/tested: `json_values::json_equality_preserves_scalar_types_and_null`; `json_values::json_membership_and_dml_share_one_expression_contract`; `dialects::json_native_rendering_types_and_unsupported_nul` |
| 25. `Dialect` (`name/q/separator/splitter`, all render methods, `json` overloads) | Implemented/tested: `dialects::explicit_capability_matrix`; `dialects::all_dml_families_native_snapshots`; `dialects::alter_full_native_plan` |
| 26. `OrQueryObject` | Implemented/tested: `dml::expressions_cover_comparisons_groups_null_in_and_like`; `dml::nested_comparison_tree_preserves_parentheses` |
| 27. `QueryObject` | Implemented/tested: `dml::detached_data_and_dialect_selection`; `contracts::finite_input_budgets_and_real_values` |
| 28. `Transaction` | Implemented/tested: `engine::transaction_forwarding_completion_and_error_preservation`; `engine::engine_plan_validation_and_cancelled_owner` |
| 29. `Connection` including `format/query/resource/transaction`, `lastId/before` | Implemented/tested: `engine::engine_validates_before_driver_and_forwards_owned_results`; `engine::driver_failure_is_not_swallowed` |
| 30. `Sqlite`, `Pgsql`, `Mysql`; `alter/create/delete/drop/insert/json/rename/select/truncate/update` | Implemented/tested: `dialects::all_dml_families_native_snapshots`; `dialects::commands_and_offset_only`; `dialects::create_types_defaults_and_idempotent_indexes`; `dialects::alter_full_native_plan` |
| 31. `Json` base class, public `separator/splitter` setters/getters | Implemented/tested: `json_values::checked_json_path_options_and_limits` |
| 32. Dialect static `parse` helpers | Implemented/tested: `dml::select_defaults_and_selector_boundaries`; `json_values::checked_json_path_options_and_limits` |
| 33. JSON dialect `selector/extract/contains/where` | Implemented/tested: `json_values::json_paths_are_bound_structural_data`; `json_values::json_equality_preserves_scalar_types_and_null`; `dialects::json_native_rendering_types_and_unsupported_nul` |
| 34. `joinTypes` | Implemented/tested: `dml::joins_and_operand_validation` |
| 35. `isIndex` | Implemented/tested: `json_values::json_paths_are_bound_structural_data`; `json_values::checked_json_path_options_and_limits` |
| 36. `escapeBackSlashes`, `escapeDoubleQuotes`, `safeJsonValue` | Implemented/tested: `json_values::bounded_explicit_json_encoding`; `json_values::json_paths_are_bound_structural_data` |
| 37. `fieldCompare` | Implemented/tested: `diff::field_properties_are_structural_not_identity` |
| 38. `jsonCompare` | Implemented/tested: `diff::compound_keys_add_remove_change_and_order`; `diff::foreign_key_add_remove_change_and_case` |
| 39. `Exception` / default `InquireException` | Implemented/tested: `contracts::errors_preserve_sources_without_implicit_disclosure`; `engine::engine_plan_failure_preserves_original_and_cleanup_errors` |

### Original test declarations

Every Now-containing declaration retains its source title below.

| Suite / source row | Exact source test title | Status and exact Rust proof |
| --- | --- | --- |
| Alter.test.ts #1 | Should build alter | Implemented/tested: `ddl::alter_native_plan_and_inspection` |
| Alter.test.ts #2 | Should replace an added index with a same-named unique key | Implemented/tested: `ddl::create_and_alter_unique_precedence_and_conflicts` |
| Alter.test.ts #3 | Should preserve an added unique key over a same-named index | Implemented/tested: `ddl::create_and_alter_unique_precedence_and_conflicts` |
| Alter.test.ts #4 | Should rename a field directly | Implemented/tested: `ddl::alter_direct_rename_and_reconciliation` |
| Alter.test.ts #5 | Should replace matching remove and add operations with a rename | Implemented/tested: `ddl::alter_direct_rename_and_reconciliation` |
| Alter.test.ts #6 | Should reject a field rename with the same source and target | Implemented/tested: `ddl::alter_rejects_ambiguous_or_incomplete_renames` |
| Alter.test.ts #7 | Should reject field renames that share a target | Implemented/tested: `ddl::alter_rejects_ambiguous_or_incomplete_renames` |
| Alter.test.ts #8 | Should reject a field rename with only a pending removal | Implemented/tested: `ddl::alter_rejects_ambiguous_or_incomplete_renames` |
| Alter.test.ts #9 | Should reject a field rename with only a pending addition | Implemented/tested: `ddl::alter_rejects_ambiguous_or_incomplete_renames` |
| Alter.test.ts #13 | Should handle rejection if any query in the transaction fails | Implemented/tested: `engine::engine_plan_failure_preserves_original_and_cleanup_errors` |
| Create.test.ts #1 | Should build create | Implemented/tested: `ddl::create_full_definition_and_ordered_plan` |
| Create.test.ts #2 | Should replace a regular index with a same-named unique key | Implemented/tested: `ddl::create_and_alter_unique_precedence_and_conflicts` |
| Create.test.ts #3 | Should preserve a unique key over a later same-named index | Implemented/tested: `ddl::create_and_alter_unique_precedence_and_conflicts` |
| Create.test.ts #7 | Should handle multiple queries correctly and return the result of the last query | Implemented/tested: `engine::engine_plan_success_and_caller_owned_transaction` |
| Delete.test.ts #1 | Should build delete | Implemented/tested: `dml::update_and_delete_build_and_returning` |
| Delete.test.ts #7 | Should store JSON filters and custom selector settings | Implemented/tested: `json_values::json_membership_and_dml_share_one_expression_contract`; `json_values::checked_json_path_options_and_limits` |
| Engine.test.ts #1 | Should diff table schemas | Implemented/tested: `diff::field_properties_are_structural_not_identity` |
| Engine.test.ts #2 | Should replace a diffed remove and add with a field rename | Implemented/tested: `diff::diff_rename_same_changed_and_missing_halves` |
| Engine.test.ts #3 | Should change a renamed field when its definition also changes | Implemented/tested: `diff::diff_rename_same_changed_and_missing_halves` |
| Engine.test.ts #4 | Should reject a diffed field rename when the target is absent | Implemented/tested: `diff::diff_rename_same_changed_and_missing_halves` |
| Engine.test.ts #5 | Should reject a diffed rename with no pending field changes | Implemented/tested: `diff::diff_rename_same_changed_and_missing_halves` |
| Engine.test.ts #6 | Should create a new Delete instance when delete method is called with a valid table name | Implemented/tested: `engine::engine_conveniences_never_attach_or_execute` |
| Engine.test.ts #7 | Should remove a field from "from" schema when it does not exist in "to" schema | Implemented/tested: `diff::diff_rename_same_changed_and_missing_halves` |
| Engine.test.ts #8 | Should remove primary key from "from" schema when it does not exist in "to" schema | Implemented/tested: `diff::compound_keys_add_remove_change_and_order` |
| Engine.test.ts #9 | Should remove unique key from "from" schema when it does not exist in "to" schema | Implemented/tested: `diff::compound_keys_add_remove_change_and_order` |
| Engine.test.ts #10 | Should update unique key when the unique key definition changes between schemas | Implemented/tested: `diff::compound_keys_add_remove_change_and_order` |
| Engine.test.ts #11 | Should call alter.removeKey and alter.addKey when jsonCompare returns false for key comparison | Implemented/tested: `diff::compound_keys_add_remove_change_and_order` |
| Engine.test.ts #12 | Should remove a foreign key when it exists in "from" schema but not in "to" schema | Implemented/tested: `diff::foreign_key_add_remove_change_and_case` |
| Engine.test.ts #13 | Should call alter.removeForeignKey and alter.addForeignKey when jsonCompare returns false for foreign key comparison with different casing | Implemented/tested: `diff::foreign_key_add_remove_change_and_case` |
| Engine.test.ts #14 | Should add a primary key when the "to" schema has a primary key not present in the "from" schema | Implemented/tested: `diff::compound_keys_add_remove_change_and_order` |
| Engine.test.ts #15 | Should add a unique key when the "to" schema has a unique key not present in the "from" schema | Implemented/tested: `diff::compound_keys_add_remove_change_and_order` |
| Engine.test.ts #16 | Should add a key when the "to" schema has a key not present in the "from" schema | Implemented/tested: `diff::compound_keys_add_remove_change_and_order` |
| Engine.test.ts #17 | Should add a foreign key when the "to" schema has a foreign key not present in the "from" schema | Implemented/tested: `diff::foreign_key_add_remove_change_and_case` |
| Engine.test.ts #18 | Should drop a table when the drop method is called with a valid table name | Implemented/tested: `ddl::table_drop_rename_truncate_and_invalid_names` |
| Engine.test.ts #19 | Should rename a table when the rename method is called with valid "from" and "to" table names | Implemented/tested: `ddl::table_drop_rename_truncate_and_invalid_names` |
| Engine.test.ts #20 | Should throw an error when truncate is called with an empty table name | Implemented/tested: `ddl::table_drop_rename_truncate_and_invalid_names` |
| Engine.test.ts #21 | Should create a new Insert instance when insert method is called with a valid table name | Implemented/tested: `engine::engine_conveniences_never_attach_or_execute` |
| Engine.test.ts #22 | Should create a new Select instance when select method is called with valid column names | Implemented/tested: `engine::engine_conveniences_never_attach_or_execute` |
| Engine.test.ts #23 | Should create a new Update instance when update method is called with a valid table name | Implemented/tested: `engine::engine_conveniences_never_attach_or_execute` |
| Engine.test.ts #24 | Should execute a transaction when the transaction method is called with a valid callback | Implemented/tested: `engine::transaction_forwarding_completion_and_error_preservation` |
| Engine.test.ts #25 | Should execute a query with a QueryObject and return the expected results | Implemented/tested: `engine::engine_validates_before_driver_and_forwards_owned_results` |
| Engine.test.ts #26 | Should execute a query with a string query and values array and return the expected results | Implemented/tested: `engine::engine_validates_before_driver_and_forwards_owned_results` |
| Engine.test.ts #29 | Should pass the cascade flag through truncate queries | Implemented/tested: `dialects::commands_and_offset_only` |
| Helpers.test.ts #1 | Should escape JSON-sensitive string values | Implemented/tested: `json_values::bounded_explicit_json_encoding`; `json_values::json_paths_are_bound_structural_data`; `dml::joins_and_operand_validation` |
| Helpers.test.ts #2 | Should expose the shared helper constants and compare JSON values | Implemented/tested: `json_values::bounded_explicit_json_encoding`; `json_values::json_paths_are_bound_structural_data`; `dml::joins_and_operand_validation` |
| Helpers.test.ts #3 | Should compare field definitions by their schema properties | Implemented/tested: `diff::field_properties_are_structural_not_identity` |
| Helpers.test.ts #4 | Should expose the custom exception class | Implemented/tested: `contracts::errors_preserve_sources_without_implicit_disclosure` |
| Insert.test.ts #1 | Should build insert | Implemented/tested: `dml::insert_bulk_returning_and_inspection` |
| Insert.test.ts #6 | Should set _returning to the provided array when columns is an array | Implemented/tested: `dml::insert_bulk_returning_and_inspection` |
| Insert.test.ts #7 | Should set _returning to an array with a single "*" when no argument is provided | Implemented/tested: `dml::insert_bulk_returning_and_inspection` |
| Insert.test.ts #8 | Should convert a single non-array value to an array when passed to values method | Implemented/tested: `dml::insert_bulk_returning_and_inspection` |
| Mysql.test.ts #1 | Should translate alter | Implemented/tested: `dialects::alter_full_native_plan` |
| Mysql.test.ts #2 | Should translate a field rename | Implemented/tested: `dialects::alter_full_native_plan` |
| Mysql.test.ts #3 | Should translate create | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Mysql.test.ts #4 | Should translate delete | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Mysql.test.ts #5 | Should translate insert | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Mysql.test.ts #6 | Should translate select | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Mysql.test.ts #7 | Should translate select with quotes | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Mysql.test.ts #8 | Should translate update | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Mysql.test.ts #9 | Should verify that "DEFAULT NULL" is correctly added to the column array when nullable is true | Deviation D1: `dialects::create_types_defaults_and_idempotent_indexes` |
| Mysql.test.ts #10 | Should verify that "UNSIGNED" is added when the field is unsigned | Implemented/tested: `dialects::fields_foreign_actions_and_invalid_lengths` |
| Mysql.test.ts #11 | Should throw an exception when no alterations are made in the Alter builder | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Mysql.test.ts #12 | Should throw an exception when no fields are provided to the Create builder | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Mysql.test.ts #13 | Should verify that both "default null" and "DEFAULT NULL" are added to the column array | Deviation D1: `dialects::create_types_defaults_and_idempotent_indexes` |
| Mysql.test.ts #14 | Should throw an exception when no filters are provided in the Delete builder | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Mysql.test.ts #15 | Should throw an exception when no values are provided in the Insert builder | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Mysql.test.ts #16 | Should throw an exception when no table is specified in the Select builder | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Mysql.test.ts #17 | Should handle table names with special characters in the Select builder | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Mysql.test.ts #18 | Should throw an exception when build.data is an empty object | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Mysql.test.ts #19 | Should translate create with multiple indexes, uniques and FKs | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Mysql.test.ts #20 | Should translate JSON selectors in select queries | Implemented/tested: `dialects::json_native_rendering_types_and_unsupported_nul` |
| Mysql.test.ts #21 | Should expose configurable JSON dialect helpers | Implemented/tested: `json_values::checked_json_path_options_and_limits`; `dialects::json_native_rendering_types_and_unsupported_nul` |
| Pgsql.test.ts #1 | Should translate alter | Implemented/tested: `dialects::alter_full_native_plan` |
| Pgsql.test.ts #2 | Should translate a field rename | Implemented/tested: `dialects::alter_full_native_plan` |
| Pgsql.test.ts #3 | Should translate create | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Pgsql.test.ts #4 | Should make create indexes idempotent without changing alter indexes | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Pgsql.test.ts #5 | Should translate delete | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Pgsql.test.ts #6 | Should translate insert | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Pgsql.test.ts #7 | Should translate select | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Pgsql.test.ts #8 | Should translate update | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Pgsql.test.ts #9 | Should throw an exception when no alterations are made during an alter operation | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Pgsql.test.ts #10 | Should throw an exception when trying to create a table with no fields provided | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Pgsql.test.ts #12 | Should throw an exception when no filters are provided for a delete query | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Pgsql.test.ts #13 | Should throw an exception when no values are provided in the Insert builder | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Pgsql.test.ts #14 | Should handle case where build.returning is an array with a single column and verify correct SQL generation | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Pgsql.test.ts #15 | Should throw an exception when no table is specified in the Select builder | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Pgsql.test.ts #16 | Should handle table names with special characters in the Select builder | Implemented/tested: `dialects::all_dml_families_native_snapshots` |
| Pgsql.test.ts #17 | Should throw an exception when build.data is an empty object | Implemented/tested: `dialects::unsupported_returning_full_join_and_invalid_shapes` |
| Pgsql.test.ts #18 | Should correctly format a column with a type of "boolean" and no default value | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Pgsql.test.ts #20 | Should correctly format a column with a type of "integer" and nullable set to true | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Pgsql.test.ts #21 | Should handle a field with a type of "text" and no length specified | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Pgsql.test.ts #22 | Should translate create with multiple indexes, uniques and FKs | Implemented/tested: `dialects::create_types_defaults_and_idempotent_indexes` |
| Pgsql.test.ts #23 | Should translate JSON selectors in select queries | Implemented/tested: `dialects::json_native_rendering_types_and_unsupported_nul` |
| Pgsql.test.ts #24 | Should expose configurable JSON dialect helpers | Implemented/tested: `json_values::checked_json_path_options_and_limits`; `dialects::json_native_rendering_types_and_unsupported_nul` |
| Select.test.ts #1 | Should build select | Implemented/tested: `dml::select_build_and_complete_sql` |
| Select.test.ts #3 | Should initialize with default select value "*" when no select argument is provided | Implemented/tested: `dml::select_defaults_and_selector_boundaries` |
| Select.test.ts #4 | Should correctly initialize _columns with multiple column names | Implemented/tested: `dml::select_defaults_and_selector_boundaries` |
| Select.test.ts #5 | Should add a sort order with column name and default direction when direction is not provided | Implemented/tested: `dml::select_sort_and_pagination` |
| Select.test.ts #10 | Should split comma-separated selectors and preserve JSON filter metadata | Implemented/tested: `dml::select_defaults_and_selector_boundaries`; `json_values::json_membership_and_dml_share_one_expression_contract` |
| Sqlite.test.ts #1 | Should translate alter | Implemented/tested: `ddl::alter_native_plan_and_inspection`; `ddl::sqlite_alter_rejects_unsupported_operations_as_a_whole` |
| Sqlite.test.ts #2 | Should translate a field rename | Implemented/tested: `ddl::alter_direct_rename_and_reconciliation` |
| Sqlite.test.ts #3 | Should translate create | Implemented/tested: `ddl::create_full_definition_and_ordered_plan` |
| Sqlite.test.ts #4 | Should make create indexes idempotent without changing alter indexes | Implemented/tested: `ddl::create_full_definition_and_ordered_plan` |
| Sqlite.test.ts #5 | Should translate delete | Implemented/tested: `dml::update_and_delete_build_and_returning` |
| Sqlite.test.ts #6 | Should translate insert | Implemented/tested: `dml::insert_bulk_returning_and_inspection` |
| Sqlite.test.ts #7 | Should translate select | Implemented/tested: `dml::select_build_and_complete_sql`; `dml::select_defaults_and_selector_boundaries` |
| Sqlite.test.ts #8 | Should translate update | Implemented/tested: `dml::update_and_delete_build_and_returning` |
| Sqlite.test.ts #9 | Should handle case where field.nullable is true and no default value is provided | Implemented/tested: `ddl::create_full_definition_and_ordered_plan` |
| Sqlite.test.ts #10 | Should throw an exception when no alterations are made in the Alter builder | Implemented/tested: `ddl::alter_rejects_ambiguous_or_incomplete_renames` |
| Sqlite.test.ts #12 | Should throw an exception when no filters are provided for a delete query | Implemented/tested: `dml::mutation_scope_never_silently_widens` |
| Sqlite.test.ts #13 | Should handle case where table name contains special characters and ensure correct SQL generation | Implemented/tested: `dml::hostile_identifiers_and_values_stay_data` |
| Sqlite.test.ts #14 | Should throw an exception when no values are provided in the Insert builder | Implemented/tested: `dml::insert_rejects_empty_arity_duplicate_and_conflicting_input` |
| Sqlite.test.ts #15 | Should handle case where build.returning is an array with a single column and verify correct SQL generation | Implemented/tested: `dml::insert_bulk_returning_and_inspection` |
| Sqlite.test.ts #16 | Should handle renaming a table with special characters in the name | Implemented/tested: `ddl::table_drop_rename_truncate_and_invalid_names` |
| Sqlite.test.ts #18 | Should throw an exception when no table is specified in the Select builder | Implemented/tested: `dml::select_defaults_and_selector_boundaries` |
| Sqlite.test.ts #19 | Should handle case where build.filters is an empty array and ensure no "where" clause is added | Implemented/tested: `dml::select_build_and_complete_sql`; `dml::select_defaults_and_selector_boundaries` |
| Sqlite.test.ts #28 | Should translate create with multiple indexes, uniques and FKs | Implemented/tested: `ddl::create_full_definition_and_ordered_plan` |
| Sqlite.test.ts #29 | Should translate JSON selectors in select queries | Implemented/tested: `json_values::json_paths_are_bound_structural_data`; `json_values::json_equality_preserves_scalar_types_and_null` |
| Sqlite.test.ts #30 | Should expose configurable JSON dialect helpers | Implemented/tested: `json_values::checked_json_path_options_and_limits` |
| Update.test.ts #1 | Should build update | Implemented/tested: `dml::update_and_delete_build_and_returning` |
| Update.test.ts #7 | Should store JSON filters and selector overrides on the builder | Implemented/tested: `json_values::json_membership_and_dml_share_one_expression_contract`; `json_values::checked_json_path_options_and_limits` |

## Deviations and boundaries

- **D1 — MySQL source tests #9 and #13:** nullable fields without explicit defaults omit DEFAULT rather than inventing DEFAULT NULL; explicit NULL emits one DEFAULT NULL, never duplicated text. This preserves absence/NULL and avoids the upstream string-generation accident. `dialects::create_types_defaults_and_idempotent_indexes` asserts nullable Boolean without an implicit default and an explicit NULL default. No native semantic default is silently discarded.
- **D2 — optional observation surface:** no read-only observation hook was added. The core inventory's Engine.before row permits read-only observation now but defers mutable/rewrite/short-circuit hooks to S4-X. Current observation is explicit `build()`, `Query::sql/binds/dialect`, plan queries and result inspection; there is no implicit bind logging or middleware. This optional surface is not represented as implemented.
- Rust replacements are explicit: columns plus equal-arity rows replace JS record/array normalization; execute/fetch replace query overloads; explicit begin/commit/rollback replace callback transactions; typed selectors/segments replace ambiguous comma/dot/function parsing. The bounded local JSON encoder replaces the inventory's suggested serde_json mechanism, retaining zero normal dependencies. No missing JS mechanism is claimed as a callable Rust API.
- Native SQLite unsupported alteration/reconstruction remains S4-M. PostgreSQL/MySQL renderers retain the M2 documented limits: no identity-policy conversion, coordinated new identity-key alteration, physical column reordering, or prefix/expression index metadata. PostgreSQL primary removal assumes renderer-generated `<table>_pkey`. MySQL atomic execute_plan is Unsupported. Capability flags do not probe server versions or guarantee every definition is supported.
- SQLite native effects are covered by the merged adapter's `tests::m2::native_execution_under_watchdog` (including ddl/unsupported/json/json_types/queries/transactions/plan_validation/plan_cancel/plan_transaction) in this workspace run. Core-only coverage remains independent of it. PostgreSQL/MySQL M2 native results and exact server versions are in the [M2 receipt](slice-4-receipt-lane-a-m2.md); M3 made no renderer changes and did not rerun the opt-in live suite. Here it reports **0 passed, 16 skipped** visibly.
- Only the installed macOS ARM64 target was checked. No Linux/Windows/WASM/mobile guarantee, production network adapter, migration execution framework or automatic DTO generator is inferred from these docs/examples.
- Per coordinated workflow §5.3, this milestone stops before the separately released dedicated refactoring/soundness rounds. The documentation audit does not claim those later rounds.

Agent Workspace baseline: **146 pre-existing errors and 20 warnings**, largely absent sibling-repository links relative to this isolated worktree. This 237-line receipt keeps the complete per-row ledger with its verification context; it is within the 500-line limit and deliberately kept coherent under the allowed 201–500-line rule. Its length adds one preferred-length warning. Post-write validation must retain exactly the same 146 hard-error diagnostics and the same 20 prior warnings, plus that one new length warning.

Stop at M3 for coordinator re-verification. No commit, listener, cross-lane source edit or agent message was performed.
