# CSRF for explicit handlers

Generate a cookie/form token, compare the next request's incoming values, and clear outgoing state after a successful process. The crate supplies the mechanism; each route or event chooses when to call it.

Start with [getting started](docs/getting-started.md) for a complete local round trip. Read [concepts](docs/concepts.md) to understand incoming versus outgoing state, then [protect a process](docs/guides/protect-a-process.md) to compose it with server events.

| Need | Read |
| --- | --- |
| Configure a selected plugin | [Registration and configuration](docs/guides/register-and-configure.md) |
| Look up a method or error | [API reference](docs/api/README.md) |
| Compare with original Stackpress | [Source behavior](docs/source-behavior.md) |
| Check examples and dependency graphs | [Documentation verification](docs/verification.md) |

The default feature graph exposes the direct library. Optional `server` adds the ordinary registrar. The CSRF registrar adds no global guard; routes call the service themselves. Selecting this crate does not select identity sessions or an application access policy.
