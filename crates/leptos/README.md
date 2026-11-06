# Stackpress Leptos

Reusable Leptos UI and explicit native rendering. Start with the [documentation](docs/README.md)
or the [independent consumer](../../examples/leptos-independent/README.md), which has its own
Cargo workspace and no view, server, facade, persistence, SQL, schema or admin dependency.

[TextEditor and HTML](docs/api/text-editor.md) provide frui-compatible HTML editing
and verbatim HTML display. Applications own stored-content security; these APIs
perform no sanitization.

The shared [source-role family](docs/api/source-roles.md) also covers source inputs,
choices, file upload hooks, collections and displays, with the G55 native rich-role contract.
