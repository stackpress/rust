# Owned task supervision

`tasks::TaskSupervisor` is the public owner for source non-awaited work. The [submit-and-drain example](../guides/events-and-hooks.md#own-non-awaited-work) shows owned server input, outcome inspection and shutdown.

## Construct and submit

`new() -> TaskSupervisor` and Default create an empty group without acquiring runtime/network resources. Scheduling requires the application's explicit Tokio Handle.

| Method | Parameters | Contract |
| --- | --- | --- |
| `spawn_on` | `&mut self`, `runtime: &Handle`, `job: Future<Output = EmailResult<()>> + Send + 'static` | Schedule without awaiting, retaining ownership in the group |
| `resolve_on` (`server`) | `runtime`, `server: Arc<Server>`, `event: impl Into<String>`, owned `input: IntoRequest + 'static` | Raw child resolve; discard child response; retain thrown errors |

**Returns:** submission methods return (). Child futures cannot borrow a mutable parent request/response. The compiler also checks the scheduled future's Send requirement. A missing hook resolves normally; no missing-listener error or implicit event rename is added.

<!-- example: direct -->
```rust
use stackpress_framework_email::{EmailResult, tasks::TaskSupervisor};
use tokio::runtime::Handle;

pub async fn supervise(runtime: &Handle) -> EmailResult<()> {
    let mut tasks = TaskSupervisor::new();
    let message = String::from("owned child input");
    tasks.spawn_on(runtime, async move {
        assert!(!message.is_empty());
        Ok(())
    });
    while let Some(outcome) = tasks.join_next().await {
        outcome??;
    }
    tasks.shutdown().await;
    Ok(())
}
```

This direct example needs no server feature and later drains the scheduled work explicitly. In a long-lived application, retain the owner instead of creating an owner that immediately drops on submission.

## Observe and close

Finished jobs stay in the group until joined. Submission order does not promise completion order for independent jobs.

| Method | Parameters | Result/behavior |
| --- | --- | --- |
| `is_empty` | `&self` | bool, false for finished-but-unjoined jobs |
| `join_next` | `&mut self` | Await Option<TaskOutcome>; None when empty; cancelling this wait retains jobs |
| `shutdown` | `&mut self` | Await abort and destruction of all remaining jobs |

**Returns:** `TaskOutcome` is `Result<EmailResult<()>, tokio::task::JoinError>`. The outer error represents panic/cancellation; the inner error retains the provider/event failure. The example's `outcome??` checks both layers.

Drop requests abort without detaching unfinished tasks; explicit shutdown is the destruction/reap barrier. Neither completion nor cancellation merges a child response into its parent or rolls back remote effects. The [source notes](../source-behavior.md) describe the owned-snapshot mechanics replacing JavaScript aliases.
