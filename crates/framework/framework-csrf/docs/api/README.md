# CSRF API reference

Use this index when you know which operation you need. For a first working example, begin with [getting started](../getting-started.md).

| Surface | Lookup |
| --- | --- |
| `Service`, `CsrfConfig`, `DEFAULT_ERROR` | [Mechanism and configuration](service.md) |
| `plugin::register`, `plugin::CsrfSection` | [Optional server integration](plugin.md) |

The crate root re-exports its direct types. The `plugin` module exists only with `server`; there is no session-crate dependency or automatic request guard.
