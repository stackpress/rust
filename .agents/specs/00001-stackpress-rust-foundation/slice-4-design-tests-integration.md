# Slice 4 upstream adapter and framework test inventory

Companion to [framework inventory](slice-4-design-inventory-framework.md) and [decisions](slice-4-design-decisions.md). Each row maps one exact source `it` declaration in source order: **43 adapter + 23 framework = 66** tests. Native SQLite equivalents are proposed now; other packages follow the named later scope. No upstream suite was run, and mocked begin/commit checks are not proof of real affinity/cancellation.

## inquire-mysql2 / Connection.test.ts

Source contains 10 mapped tests.

| # | Exact source test title | Disposition / reason |
| --- | --- | --- |
| 1 | Should return the correct value for lastId when _lastId is set to a positive integer | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. No shared lastId slot; return ID on its statement result. |
| 2 | Should handle a request object with no values property | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 3 | should convert Date objects in the values array to ISO string format | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 4 | Should handle an empty array by converting it to an empty JSON array string | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 5 | Should convert a nested object in the values array to a JSON string | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 6 | Should set _lastId when results[0] is an object with insertId property and not an array | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. No shared lastId slot; return ID on its statement result. |
| 7 | Should return an empty array when results is an empty array | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 8 | Should successfully execute the callback and commit when both operations succeed | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 9 | Should handle and rollback transaction when beginTransaction throws an error | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. Do not reproduce blind rollback after failed begin; prove lease cleanup and original cause. |
| 10 | Should return results as Results<R> when results[0] is an array using lowercase isarray method | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
## inquire-pg / Connection.test.ts

Source contains 9 mapped tests.

| # | Exact source test title | Disposition / reason |
| --- | --- | --- |
| 1 | Should throw an exception when the query string does not contain a question mark | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. Use prepared parameter metadata/native bind nodes, not global question-mark counting. |
| 2 | Should convert a Date object in the values array to an ISO string | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 3 | Should convert an array in the values array to a JSON string | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 4 | Should convert an object with nested properties in the values array to a JSON string | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 5 | Should throw an exception when the query string contains multiple question marks | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. Use prepared parameter metadata/native bind nodes, not global question-mark counting. |
| 6 | Should return an empty array when the query returns no rows | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 7 | Should successfully commit the transaction when the callback resolves without errors | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
| 8 | Should handle and rollback transaction when beginTransaction throws an error | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. Do not reproduce blind rollback after failed begin; prove lease cleanup and original cause. |
| 9 | Should handle a query with no values and return the expected result | Later S4-N: native network driver/service proof; preserve real binds/results/one-client affinity. |
## inquire-pglite / Connection.test.ts

Source contains 12 mapped tests.

| # | Exact source test title | Disposition / reason |
| --- | --- | --- |
| 1 | Should create profile table | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 2 | Should safely replay creation of a table with an index | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 3 | Should alter profile table | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 4 | Should flatten data | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 5 | Should insert profile values | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 6 | Should update profile values | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 7 | Should fetch all profiles | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 8 | Should filter by json object | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 9 | Should filter by json array | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 10 | Should filter by json nested array | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 11 | Should filter by json nested x 2 array | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
| 12 | Should delete profile | Later S4-H: requires embedded PostgreSQL host assessment; SQLite analog now is separate evidence. |
## inquire-sqlite3 / Connection.test.ts

Source contains 12 mapped tests.

| # | Exact source test title | Disposition / reason |
| --- | --- | --- |
| 1 | Should create profile table | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 2 | Should safely replay creation of a table with indexes | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 3 | Should alter profile table | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 4 | Should flatten data | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 5 | Should insert profile values | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 6 | Should update profile values | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 7 | Should fetch all profiles | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 8 | Should filter by json object | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 9 | Should filter by json array | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 10 | Should filter by json nested array | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 11 | Should filter by json nested x 2 array | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
| 12 | Should delete profile | Now: real isolated SQLite execution; per-statement metadata/typed encoding replace mutable format/lastId. |
## stackpress-sql / helpers.test.ts

Source contains 14 mapped tests.

| # | Exact source test title | Disposition / reason |
| --- | --- | --- |
| 1 | should coerce values into SQL-safe primitives | No silent coercion/epoch fallback; Now fallible typed SQL values, explicit date text; model conversion Later S4-F. |
| 2 | should coerce dates and fall back invalid values to epoch time | No silent coercion/epoch fallback; Now fallible typed SQL values, explicit date text; model conversion Later S4-F. |
| 3 | should flatten nested records and optionally expand arrays | Later S4-F: schema-owned flatten/alias traversal; bounded paths and collision checks. |
| 4 | should derive aliases and selector strings for nested store paths | Later S4-F: schema-owned flatten/alias traversal; bounded paths and collision checks. |
| 5 | should collect removed alter members and detect destructive changes | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 6 | should format an aggregate destructive warning message | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 7 | should treat empty removals as safe | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 8 | should detect destructive removals from real builder diffs | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. Pure core diff/rename/render behavior is tested now independently. |
| 9 | should return destructive SQL and warning metadata together | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 10 | should treat real no-op diffs as safe across builder dialects | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. Pure core diff/rename/render behavior is tested now independently. |
| 11 | should plan a clear one-to-one same-semantics field rename | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 12 | should refuse to guess across multiple same-shape candidates | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 13 | should delegate rename SQL to every supported Inquire dialect | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. Pure core diff/rename/render behavior is tested now independently. |
| 14 | should allow equivalent renamed keys and constraints | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
## stackpress-sql / upgrade.test.ts

Source contains 9 mapped tests.

| # | Exact source test title | Disposition / reason |
| --- | --- | --- |
| 1 | should preserve a clear field rename through Inquire | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 2 | should refuse ambiguous field rename candidates | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 3 | should refuse destructive field removals when force is false | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 4 | should execute destructive alter SQL when force is enabled | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 5 | should refuse dropped tables when force is false | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 6 | should report all destructive changes in one warning | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 7 | should write a clear field rename through Inquire | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 8 | should write ambiguous rename candidates as raw SQL | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |
| 9 | should write destructive migration SQL without requiring force | Later S4-M: schema revision/rename/destructive warning and operator refusal/artifact policy, not generic core execution. |

## Generated test templates and behavioral expansion

Source `src/transform/tests/{store,actions,events}.ts` contains commented illustrative `it` declarations and emitted template bodies, not an extra executed suite. All 21 named template declarations (7 per file) are mapped below. Additional helper/relation/scalarization behavior is mapped in the framework inventory. Model-specific generated output must be compiled/executed when S4-F ships; the comments alone prove nothing.

| Source template behavior | Disposition / reason |
| --- | --- |
| Store template: count query, alter query, delete query, insert query, select query, update query, where clause | Later S4-F model consumer tests; underlying generic builders and native SQLite execution now. |
| Actions create, batch, search/findAll, get/find, update, remove, restore | Later S4-F including validation and soft-delete semantics; compile emitted code and exercise public outcomes. |
| Events create, batch, search, get, update, remove, restore | Later S4-F with request/response conventions, denied/malformed input and no implicit transaction inheritance. |
| Template sample scalar/date/JSON/list/optional/relation inputs | Later S4-F after normalized schema semantics; no pretending Profile's two Strings cover them. |

## New tests beyond upstream

[Verification](slice-4-design-verification.md) adds real binding attacks, missing/excess binds, quote/comment/native-placeholder forms, WITH and UPDATE/DELETE RETURNING, NULL/missing/false/zero distinctions, duplicate column aliases, pool acquisition/drop/close, begin/commit cancellation, rollback failure, panic, nested concurrent dispatch, resource budgets, feature trees and independent test files. Upstream adapter tests lack this combined safety evidence.
