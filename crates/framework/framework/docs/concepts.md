# Composition and ownership

An aggregate registrar is a sequence of ordinary Rust calls. It makes the source order convenient to reuse; it does not replace constituent APIs, transport adapters or application policy. The [first event example](getting-started.md) works because registration and execution are separate operations.

## Nine stages, then startup phases

Selected stages run in this exact order. Each stage queues its own existing Config, Listen or Route work; priority and registration order within those phases remain meaningful.

| Stage | Contribution |
| --- | --- |
| 1. server | Queues develop, emit and serve events. |
| 2. schema | Publishes the compiled client in Config at priority 10; queues generate. |
| 3. language | Publishes configured language data and its request hook. |
| 4. CSRF | Reuses the real CSRF registrar with its server feature. |
| 5. SQL | Queues command events, then calls model listen registrars in catalog order. |
| 6. view | Configures source-present view behaviour and build/development hooks. |
| 7. session | Reuses auth → email → identity composite registration. |
| 8. API | Reuses the API registrar with SQL capability. |
| 9. admin | Calls application-generated admin registrars in catalog order, last. |

The aggregate calls a model's `listen` function during composition. That function must queue its actual listeners for Listen to retain source phase order. Account models use `account::events::register_listen`; its immediate `register` remains available for direct composition. See the [catalog reference](api/registration.md).

## Selection, configuration and execution

Cargo features select executable code before compilation. Configuration controls what the selected plugin contributes, and explicit services supply native IO. Removing configuration is not the same as compiling out a dependency; disabling a plugin requires a rebuild. [Feature graphs](api/features.md) prove both forms separately.

`register` supplies no compiled client; `register_with` supplies a static catalog. Missing and present-empty catalogs remain distinct. Metadata `describe` is evaluated only when explicitly requested. Neither registration runs a listener, compiler, schema generator or database command.

## Handlers and security

G42 permits several handlers for a route or event: priority, then registration order, with Abort stopping the current chain. Nested raw event dispatch has its own local Abort outcome. The aggregate adds no route ownership arbitration or global access guard.

Under G43 each plugin and route secures its process as configured. Missing session configuration means no identity service; missing `session.access` means no global request guard; missing auth configuration means no auth routes. An empty access map follows upstream permissive behaviour. Selecting API types does not implicitly register session events. Refer to [source behaviour](source-behavior.md) for the carried questions and mechanics boundaries.

## Application-owned resources

The application supplies an Engine, command Bindings, revision History, renderer/compiler providers and any packaging callback it intends to use. Those objects stay native; deliberate page props/seeds cross into browser code. Cancellation and shutdown belong to the resource owner, including session Hooks and supervised Packaging. The [command](guides/commands.md) and [website](guides/website.md) guides show these boundaries.
