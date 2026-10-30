# CSRF server integration

With `server`, `stackpress_framework_csrf::plugin` publishes the direct service through ordinary server startup. Use [the configuration example](../guides/register-and-configure.md#publish-a-configured-service) to register it.

## Register

`plugin::register(app: &mut ServerBuilder) -> stackpress_lib::Result<()>` declares a raw `csrf` section and contributes a Config handler. That handler decodes object overrides and registers the service.

| Parameter | Meaning |
| --- | --- |
| `app` | Mutable builder for this application's selected plugins |

**Returns:** `()` on contribution success. Errors preserve duplicate typed-section/service registration and typed override decoding. Registration itself does not validate a submitted request.

## Raw section

`CsrfSection(pub serde_json::Value)` is a transparent serde carrier. Missing configuration uses null; a nonobject supplies no name/error overrides. Its public `.0` retains the original JSON section.

The service is published even when config is missing. The registrar contributes no request listener, route or global guard. Routes call the service themselves; [source behavior](../source-behavior.md) explains how that differs from email's config-gated listener.
