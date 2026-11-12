# Named no-panic exceptions

Authority: G62 and the coordinator ruling of 2026-11-12 for Task 00053.
Only parsing a compile-time constant literal may use a narrowly scoped
`#[expect(clippy::expect_used, reason = "...")]`. Runtime input, configuration
and state must use errors or non-panicking representations. No `allow` escapes.

Each exception has a unit test that constructs/parses the exact literal.
This is the complete foundation-lane list; other lanes add only approved literal sites.

| Site | Constant literal | Reason | Unit test |
| --- | --- | --- | --- |
| `crates/lib/src/request.rs:145` | `http://unknownhost/` | Fixed valid absolute HTTP URL; Request::new and Default remain infallible. | `default_url_literal_is_valid` |
| `crates/framework/framework-email/src/options.rs:346` | `include_str!("../data/nodemailer-services.json")` | Embedded versioned source JSON; no runtime value is passed to its parser. | `embedded_services_are_valid_json` |
| `crates/framework/framework-email/src/compose/node.rs:138` | `include_str!("../../data/nodemailer-mime.json")` | Embedded versioned source JSON; no runtime value is passed to its parser. | `embedded_mime_catalog_is_valid_json` |
