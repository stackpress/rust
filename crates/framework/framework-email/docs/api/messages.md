# Message options and results

The [native round trip](../guides/native-content.md#keep-a-reader-out-of-json) shows owned reader input and output; the [message config example](../guides/configuration.md#keep-message-fields-source-shaped) shows ordinary serializable fields.

## Input carriers

`SendMailOptions` is `serde_json::Map<String, serde_json::Value>`. Unknown application/provider fields remain present; there is no closed sender/recipient DTO.

| Type/member | Ownership and meaning |
| --- | --- |
| `NativeSendMailOptions::default()` | Empty fields and content map |
| `NativeSendMailOptions.fields` | Complete SendMailOptions |
| `NativeSendMailOptions.content` | `BTreeMap<String, Content>` keyed by source field/index path |
| `From<SendMailOptions>` | Lift JSON fields with no native slots or new defaults |
| `Content::Bytes(Vec<u8>)` | Owned Buffer equivalent |
| `Content::Reader(MailReader)` | Owned one-shot Readable equivalent |
| `MailReader` | `Pin<Box<dyn AsyncRead + Send + 'static>>`, no Clone/Sync requirement |

**Returns:** constructing these carriers supplies data only; Service/Transport consume it on send. Slot aliases, raw handling and explicit serialization are explained in [native content](../guides/native-content.md#choose-a-content-slot).

## Results

`SentMessageInfo` keeps provider JSON independently from optional native payloads. `From<Value>` retains even nonobject provider info without filtering.

| Member | Type/meaning |
| --- | --- |
| `fields` | Provider `Value`, including unknown extensions |
| `message` | Optional `MessageOutput::Buffer(Vec<u8>)` or `MessageOutput::Stream(MailReader)` |
| `raw` | Optional SES raw MIME bytes |

`info["key"]` borrows a provider JSON field. Debug formatting reports presence of native outputs without printing their bytes/readers; callers still control the complete public `fields` value.

<!-- example: direct -->
```rust
use serde_json::json;
use stackpress_framework_email::{MessageOutput, SentMessageInfo};

pub fn buffer_result() -> Result<(), serde_json::Error> {
    let mut info = SentMessageInfo::from(json!({"extension": {"kept": true}}));
    info.message = Some(MessageOutput::Buffer(vec![0, 255]));
    let serialized = serde_json::to_value(&info)?;
    assert_eq!(
        serialized["message"],
        json!({"type": "Buffer", "data": [0, 255]})
    );
    assert_eq!(info["extension"]["kept"], true);
    Ok(())
}
```

Buffer outputs serialize to the source Node Buffer JSON shape. Stream serialization errors without consuming the reader; native output attached to a nonobject result is also an error. PartialEq against JSON compares this serializable projection, so a live stream is not equal to an erased JSON snapshot. [Server event results](events.md) retain a stream at the original `message` key through a one-shot native carrier.
