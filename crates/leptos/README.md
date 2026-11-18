# Stackpress Leptos

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Reusable Leptos UI and explicit native rendering. Start with the [documentation](docs/README.md)
or the [independent consumer](../../examples/leptos-independent/README.md), which has its own
Cargo workspace and no view, server, facade, persistence, SQL, schema or admin dependency.

[TextEditor and HTML](docs/api/text-editor.md) provide frui-compatible HTML editing
and verbatim HTML display. Applications own stored-content security; these APIs
perform no sanitization.

The shared [source-role family](docs/api/source-roles.md) also covers source inputs,
choices, file upload hooks, collections and displays, with the G55 native rich-role contract.

Independent `client-sse` and `client-websocket` features expose owner-scoped
`realtime::use_event_source` and `realtime::use_websocket`. They open nothing on
native SSR; browser effects connect after mount and disposal closes transports,
removes listeners and cancels retry timers. See the [realtime guide](docs/api/realtime.md)
for bounded state, backpressure and application security responsibilities.
Facade equivalents are `leptos-sse` and `leptos-websocket`; no native host is pulled in.
