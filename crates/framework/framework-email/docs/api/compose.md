# Address and metadata helpers

`compose::addresses` and `compose::metadata` inspect source-shaped JSON before content reads. They are public helpers, not a replacement for the selected transport's compiler/provider behavior.

## Inspect metadata

Parse a message's address/envelope/Message-ID without consuming a body reader. Supplying a Message-ID makes this example's identity deterministic.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::compose::{addresses, metadata};
use stackpress_framework_email::{EmailResult, SendMailOptions};

pub fn inspect_metadata() -> EmailResult<()> {
    let parsed = addresses(&json!({"name": "Reader", "address": "reader@example.test"}))?;
    assert_eq!(parsed[0].email.to_string(), "reader@example.test");
    let fields: SendMailOptions = serde_json::from_value(json!({
        "from": "sender@example.test", "to": "reader@example.test", "messageId": "fixed@example.test"
    }))?;
    let info = metadata(&fields)?;
    assert_eq!(info.message_id, "fixed@example.test");
    assert_eq!(info.smtp_envelope["to"], json!(["reader@example.test"]));
    Ok(())
}
```

The explicit source envelope and parsed SMTP envelope are separate projections. Headers and source fallback rules select their contents; missing Message-ID generation uses OS entropy.

## Contracts

Use the source/compiler path through Service when a complete provider should handle unsupported address syntax.

| Function | Parameter | Result/errors |
| --- | --- | --- |
| `addresses` | `value: &Value`, source string/object/list | EmailResult<Vec<lettre::message::Mailbox>>; native syntax errors propagate |
| `metadata` | `fields: &Map<String, Value>` | EmailResult<MessageMetadata>; address/metadata or entropy errors propagate |

**Returns:** addresses returns `EmailResult<Vec<lettre::message::Mailbox>>`; metadata returns `EmailResult<MessageMetadata>`. MessageMetadata has public `envelope: Value`, `smtp_envelope: Value` and `message_id: String`, with Clone and Debug. Unknown explicit envelope extension fields remain available in the SMTP projection.

Group/permissive address forms outside lettre need the complete provider. NativeFactory detects unsupported syntax before consuming content and routes it or reports its named error; calling these helpers directly does not perform that fallback. See [configuration](../guides/configuration.md) and [implementation](../../src/compose/metadata.rs).
