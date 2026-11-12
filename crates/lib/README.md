# stackpress-lib

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Transport-neutral values, requests, responses and ordered event dispatch form the foundation shared by Stackpress hosts and plugins.

```rust
use stackpress_lib::TemplateEngine;
let engine = TemplateEngine::default();
assert_eq!(engine.render_pairs("Hello {{name}}", &[("name", "Ada")])?, "Hello Ada");
# Ok::<(), stackpress_lib::TemplateError>(())
```

The template renderer runs without a host or executor.

Licence: [Apache-2.0](LICENSE).
