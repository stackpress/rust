# Configuration, transports and owned messages

An email call combines reusable configuration with one message and one transport instance. Separating those lifetimes lets concurrent calls share a factory without sharing mutable transport state.

## One transport per call

`Service::new(config, factory)` stores exact configuration and an `Arc<dyn TransportFactory>`. Both `send` and `send_native` call `factory.create(&config)`, await the returned transport, then release it. The factory must be Send + Sync; each transport needs only Send, with no Clone or Sync requirement.

Think of the JSON [quick start](getting-started.md) as one such invocation. Calling it twice constructs twice; selecting pool options still constructs per call and requires an application binding for source pool recycling.

## Two message carriers

Ordinary `SendMailOptions` is a serde JSON map retaining unknown fields. `NativeSendMailOptions` keeps that map plus owned byte buffers/readers at original content slots, such as `text`, `raw` or `attachments.0.content`.

`SentMessageInfo.fields` retains provider JSON, including scalar values and extension fields. `message` can additionally hold a native Buffer or Readable, and `raw` can hold SES bytes. Serialization projects buffers as Node Buffer JSON; live readers require explicit native consumption and are not silently drained or omitted. See [native content](guides/native-content.md) before passing streams across event snapshots.

## Compiled adapters and complete providers

All source transport variants are compiled into the native email crate. The `server` feature selects host integration, not SMTP versus JSON. `NativeFactory` maps supported options through lettre, process, SES and content seams; a configured complete `TransportFactory` receives options that the built-in library cannot honor.

Unsupported capabilities route to that configured provider or return a named error. The library does not silently drop them. Generated MIME uses native library framing and buffering; it is semantically tested rather than claimed byte-identical to Nodemailer. Raw bytes retain exactness.

## Direct calls and reusable events

The direct service has no activation gate: callers supply an `EmailConfig` and choose when to send. With `server`, the registrar installs `email-send` only for truthy config; the reusable event repeats its source body/status/config checks at invocation time.

A truthy existing body causes email-send to skip before config or input conversion. Empty objects/arrays and byte buffers, including empty buffers, are truthy; absent/null/false/zero/empty-text bodies are falsy. In the preserved shared auth response, auth-update leaves its results body, so the following email-send skips transport construction. A fresh resolve has independent response state and can send. [Source behavior](source-behavior.md#shared-auth-response-and-hook-behavior) explains the resulting OTP/magic behavior without clearing that body automatically.

## Error and task ownership

Direct factory/send errors propagate unchanged. Event errors retain a native cause through the existing framework lifecycle; the event does not manufacture an error status. Cancellation drops owned transports/readers without retry or rollback of remote effects already performed.

`TaskSupervisor` owns source non-awaited jobs. The application supplies a runtime and owned child input, then drains jobs or awaits shutdown. Parent response state is not merged with child responses, and independently scheduled jobs may finish in either order. The [hook guide](guides/events-and-hooks.md) shows the distinct signup hook and explicit application delegation.
