# Slice 4 upstream core test inventory

Companion to [core inventory](slice-4-design-inventory-core.md) and [decisions](slice-4-design-decisions.md). Each row maps one `it` declaration from Inquire `packages/inquire/tests/*.test.ts` at `039a1b5`; exact title order identifies the source test. All **160** declarations are covered. These suites were inspected, not executed. Now means a required Rust behavior test after approval, not a passed result.

`No` explicitly replaces a JS mechanism or accidental behavior; the stated Rust test protects the replacement. Dialect snapshots prove rendering only; network execution is Later S4-N. Full SQLite reconstruction is Later S4-M. Source test titles sometimes overstate assertions; production tests must assert outcomes, not only method calls/mocks.

## Alter.test.ts

Source file contains 13 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should build alter | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 2 | Should replace an added index with a same-named unique key | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 3 | Should preserve an added unique key over a same-named index | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 4 | Should rename a field directly | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 5 | Should replace matching remove and add operations with a rename | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 6 | Should reject a field rename with the same source and target | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 7 | Should reject field renames that share a target | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 8 | Should reject a field rename with only a pending removal | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 9 | Should reject a field rename with only a pending addition | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 10 | Should handle setting and getting the engine | No attached-engine mutation: detached builder independence and explicit executor selection instead. |
| 11 | Should throw an exception when dialect is not provided and engine is undefined | No implicit dialect: render requires an explicit argument; compile/API proof replaces late throw. |
| 12 | Should throw an exception when no engine is provided | No awaitable builder: execution requires Engine/Transaction; compile-fail wrong usage. |
| 13 | Should handle rejection if any query in the transaction fails | Now: observable builder/render/validation result; execute applicable SQLite path. |
## Create.test.ts

Source file contains 7 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should build create | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 2 | Should replace a regular index with a same-named unique key | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 3 | Should preserve a unique key over a later same-named index | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 4 | Should handle setting and getting the engine | No attached-engine mutation: detached builder independence and explicit executor selection instead. |
| 5 | Should throw an exception when dialect is not provided and engine is undefined | No implicit dialect: render requires an explicit argument; compile/API proof replaces late throw. |
| 6 | Should throw an exception when no engine is provided | No awaitable builder: execution requires Engine/Transaction; compile-fail wrong usage. |
| 7 | Should handle multiple queries correctly and return the result of the last query | Now one-lease atomic validated plan with ordered per-statement outcomes, not only final rows. |
## Delete.test.ts

Source file contains 7 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should build delete | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 2 | Should handle setting and getting the engine | No attached-engine mutation: detached builder independence and explicit executor selection instead. |
| 3 | Should throw an exception when dialect is not provided and engine is undefined | No implicit dialect: render requires an explicit argument; compile/API proof replaces late throw. |
| 4 | Should return a promise when query method is called with a valid engine | No JS thenable: explicit Send execution future and real result/error test. |
| 5 | Should throw an exception when no engine is provided | No awaitable builder: execution requires Engine/Transaction; compile-fail wrong usage. |
| 6 | Should handle empty query string in where method without throwing an error | No empty raw predicate acceptance: typed invalid-clause error before execution. |
| 7 | Should store JSON filters and custom selector settings | Now per-query checked JSON parse options; No mutable dialect-global state. |
## Engine.test.ts

Source file contains 29 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should diff table schemas | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 2 | Should replace a diffed remove and add with a field rename | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 3 | Should change a renamed field when its definition also changes | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 4 | Should reject a diffed field rename when the target is absent | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 5 | Should reject a diffed rename with no pending field changes | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 6 | Should create a new Delete instance when delete method is called with a valid table name | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 7 | Should remove a field from "from" schema when it does not exist in "to" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 8 | Should remove primary key from "from" schema when it does not exist in "to" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 9 | Should remove unique key from "from" schema when it does not exist in "to" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 10 | Should update unique key when the unique key definition changes between schemas | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 11 | Should call alter.removeKey and alter.addKey when jsonCompare returns false for key comparison | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 12 | Should remove a foreign key when it exists in "from" schema but not in "to" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 13 | Should call alter.removeForeignKey and alter.addForeignKey when jsonCompare returns false for foreign key comparison with different casing | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 14 | Should add a primary key when the "to" schema has a primary key not present in the "from" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 15 | Should add a unique key when the "to" schema has a unique key not present in the "from" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 16 | Should add a key when the "to" schema has a key not present in the "from" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 17 | Should add a foreign key when the "to" schema has a foreign key not present in the "from" schema | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 18 | Should drop a table when the drop method is called with a valid table name | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 19 | Should rename a table when the rename method is called with valid "from" and "to" table names | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 20 | Should throw an error when truncate is called with an empty table name | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 21 | Should create a new Insert instance when insert method is called with a valid table name | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 22 | Should create a new Select instance when select method is called with valid column names | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 23 | Should create a new Update instance when update method is called with a valid table name | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 24 | Should execute a transaction when the transaction method is called with a valid callback | Now explicit guard commit/rollback/await affinity; callback syntax Later S4-X. |
| 25 | Should execute a query with a QueryObject and return the expected results | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 26 | Should execute a query with a string query and values array and return the expected results | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 27 | Should allow the before hook to short-circuit the database query | Later S4-X: middleware short-circuit needs transaction/order/trust contract. |
| 28 | Should format template string queries with the dialect quote character | No JS tagged-template/global quote rewrite: literal+bind and quoted-token tests instead. |
| 29 | Should pass the cascade flag through truncate queries | Now capability-specific flag: reject SQLite cascade; render native semantics for network dialects. |
## Helpers.test.ts

Source file contains 4 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should escape JSON-sensitive string values | No regex/escaping API: Now typed join/path/JSON encoding and bound-value adversarial tests. |
| 2 | Should expose the shared helper constants and compare JSON values | No regex/escaping API: Now typed join/path/JSON encoding and bound-value adversarial tests. |
| 3 | Should compare field definitions by their schema properties | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 4 | Should expose the custom exception class | Now SqlError kind/source chain; no JS exception inheritance. |
## Insert.test.ts

Source file contains 8 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should build insert | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 2 | Should handle setting and getting the engine | No attached-engine mutation: detached builder independence and explicit executor selection instead. |
| 3 | Should throw an exception when dialect is not provided and engine is undefined | No implicit dialect: render requires an explicit argument; compile/API proof replaces late throw. |
| 4 | Should return a promise when query method is called with a valid engine | No JS thenable: explicit Send execution future and real result/error test. |
| 5 | Should throw an exception when no engine is provided | No awaitable builder: execution requires Engine/Transaction; compile-fail wrong usage. |
| 6 | Should set _returning to the provided array when columns is an array | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 7 | Should set _returning to an array with a single "*" when no argument is provided | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 8 | Should convert a single non-array value to an array when passed to values method | Now: observable builder/render/validation result; execute applicable SQLite path. |
## Mysql.test.ts

Source file contains 21 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should translate alter | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 2 | Should translate a field rename | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 3 | Should translate create | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 4 | Should translate delete | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 5 | Should translate insert | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 6 | Should translate select | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 7 | Should translate select with quotes | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 8 | Should translate update | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 9 | Should verify that "DEFAULT NULL" is correctly added to the column array when nullable is true | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 10 | Should verify that "UNSIGNED" is added when the field is unsigned | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 11 | Should throw an exception when no alterations are made in the Alter builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 12 | Should throw an exception when no fields are provided to the Create builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 13 | Should verify that both "default null" and "DEFAULT NULL" are added to the column array | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 14 | Should throw an exception when no filters are provided in the Delete builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 15 | Should throw an exception when no values are provided in the Insert builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 16 | Should throw an exception when no table is specified in the Select builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 17 | Should handle table names with special characters in the Select builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 18 | Should throw an exception when build.data is an empty object | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 19 | Should translate create with multiple indexes, uniques and FKs | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 20 | Should translate JSON selectors in select queries | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 21 | Should expose configurable JSON dialect helpers | Now per-query checked JSON parse options; No mutable dialect-global state. |
## Pgsql.test.ts

Source file contains 24 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should translate alter | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 2 | Should translate a field rename | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 3 | Should translate create | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 4 | Should make create indexes idempotent without changing alter indexes | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 5 | Should translate delete | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 6 | Should translate insert | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 7 | Should translate select | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 8 | Should translate update | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 9 | Should throw an exception when no alterations are made during an alter operation | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 10 | Should throw an exception when trying to create a table with no fields provided | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 11 | Should handle case where field.default is an empty string and ensure no default value is set | No truthiness bug: empty-string default is preserved alongside false/zero. |
| 12 | Should throw an exception when no filters are provided for a delete query | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 13 | Should throw an exception when no values are provided in the Insert builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 14 | Should handle case where build.returning is an array with a single column and verify correct SQL generation | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 15 | Should throw an exception when no table is specified in the Select builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 16 | Should handle table names with special characters in the Select builder | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 17 | Should throw an exception when build.data is an empty object | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 18 | Should correctly format a column with a type of "boolean" and no default value | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 19 | Should correctly handle a field with an attribute but no type specified | No unvalidated SQL splice: explicit trusted type/expression or typed rejection. |
| 20 | Should correctly format a column with a type of "integer" and nullable set to true | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 21 | Should handle a field with a type of "text" and no length specified | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 22 | Should translate create with multiple indexes, uniques and FKs | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 23 | Should translate JSON selectors in select queries | Now: dialect-specific rendering/capability snapshot; native execution Later S4-N needs service fixture. |
| 24 | Should expose configurable JSON dialect helpers | Now per-query checked JSON parse options; No mutable dialect-global state. |
## Select.test.ts

Source file contains 10 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should build select | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 2 | Should handle setting and getting the engine | No attached-engine mutation: detached builder independence and explicit executor selection instead. |
| 3 | Should initialize with default select value "*" when no select argument is provided | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 4 | Should correctly initialize _columns with multiple column names | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 5 | Should add a sort order with column name and default direction when direction is not provided | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 6 | Should throw an exception when dialect is not provided and engine is undefined | No implicit dialect: render requires an explicit argument; compile/API proof replaces late throw. |
| 7 | Should return a promise when query method is called with a valid engine | No JS thenable: explicit Send execution future and real result/error test. |
| 8 | Should throw an exception when no engine is provided | No awaitable builder: execution requires Engine/Transaction; compile-fail wrong usage. |
| 9 | Should add a filter with an empty query string and no values | No empty raw predicate acceptance: typed invalid-clause error before execution. |
| 10 | Should split comma-separated selectors and preserve JSON filter metadata | No raw comma/function parsing: explicit selector list; Now preserve typed JSON filter paths/order. |
## Sqlite.test.ts

Source file contains 30 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should translate alter | Now native-supported ALTER plan; reject type/key/FK changes before effects. Reconstruction Later S4-M. |
| 2 | Should translate a field rename | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 3 | Should translate create | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 4 | Should make create indexes idempotent without changing alter indexes | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 5 | Should translate delete | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 6 | Should translate insert | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 7 | Should translate select | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 8 | Should translate update | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 9 | Should handle case where field.nullable is true and no default value is provided | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 10 | Should throw an exception when no alterations are made in the Alter builder | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 11 | Should handle case where field.default is an empty string and ensure no default value is set | No truthiness bug: empty-string default is preserved alongside false/zero. |
| 12 | Should throw an exception when no filters are provided for a delete query | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 13 | Should handle case where table name contains special characters and ensure correct SQL generation | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 14 | Should throw an exception when no values are provided in the Insert builder | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 15 | Should handle case where build.returning is an array with a single column and verify correct SQL generation | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 16 | Should handle renaming a table with special characters in the name | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 17 | Should handle case where build.table[0] is an empty string and build.table[1] is a valid string and return an empty query | No silent empty SQL: typed invalid definition/query; no effects. |
| 18 | Should throw an exception when no table is specified in the Select builder | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 19 | Should handle case where build.filters is an empty array and ensure no "where" clause is added | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 20 | Should handle case where build.data is an empty object and ensure no query is generated | No silent empty SQL: typed invalid definition/query; no effects. |
| 21 | Should handle case where table name is an empty string and ensure no query is generated | No silent empty SQL: typed invalid definition/query; no effects. |
| 22 | Should handle case where field.default is a function call string other than "now()" and ensure it is converted to uppercase | No inferred arbitrary expression: typed default or explicitly trusted expression, with tests. |
| 23 | Should handle case where build.fields is an empty object and ensure no columns are generated | No silent empty SQL: typed invalid definition/query; no effects. |
| 24 | Should handle case where field.type is a custom type and ensure it is processed correctly | No unvalidated SQL splice: explicit trusted type/expression or typed rejection. |
| 25 | Should handle case where field.length is a negative number and ensure it is processed correctly | No permissive invalid length: validation errors for nonpositive lengths before effects. |
| 26 | Should handle case where field.attribute is a custom attribute and ensure it is included in the query | No unvalidated SQL splice: explicit trusted type/expression or typed rejection. |
| 27 | Should handle case where field.length is zero and ensure it is processed correctly | No permissive invalid length: validation errors for nonpositive lengths before effects. |
| 28 | Should translate create with multiple indexes, uniques and FKs | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 29 | Should translate JSON selectors in select queries | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 30 | Should expose configurable JSON dialect helpers | Now per-query checked JSON parse options; No mutable dialect-global state. |
## Update.test.ts

Source file contains 7 mapped tests.

| # | Exact source test title | Disposition / reason and Rust proof |
| --- | --- | --- |
| 1 | Should build update | Now: observable builder/render/validation result; execute applicable SQLite path. |
| 2 | Should handle setting and getting the engine | No attached-engine mutation: detached builder independence and explicit executor selection instead. |
| 3 | Should throw an exception when dialect is not provided and engine is undefined | No implicit dialect: render requires an explicit argument; compile/API proof replaces late throw. |
| 4 | Should return a promise when query method is called with a valid engine | No JS thenable: explicit Send execution future and real result/error test. |
| 5 | Should throw an exception when no engine is provided | No awaitable builder: execution requires Engine/Transaction; compile-fail wrong usage. |
| 6 | Should add a filter with an empty query string and no values | No empty raw predicate acceptance: typed invalid-clause error before execution. |
| 7 | Should store JSON filters and selector overrides on the builder | Now per-query checked JSON parse options; No mutable dialect-global state. |
