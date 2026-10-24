# Queue API

Stable priority scheduling for owned items and sequential async startup work. Exports live under
`stackpress_lib::queue`. Source: [queue.rs](../../src/queue.rs).

```rust
use stackpress_lib::queue::{ItemQueue, QueueOutcome, TaskQueue, task};
use stackpress_lib::{Flow, HandlerFuture, Result};

async fn register(state: &mut Vec<String>) -> Result<Flow> {
    state.push("registered".into());
    Ok(Flow::Continue)
}
```

Examples run independently inside an async function returning `Result<()>`, using these imports and
named task. A task takes `&mut C` and returns `Result<Flow>` through a `Send` future.

Contents: [Scheduling and public values](#1-scheduling-and-public-values) ·
[ItemQueue operations](#2-itemqueue-operations) · [Task callbacks](#3-task-callbacks) ·
[TaskQueue operations](#4-taskqueue-operations)

## 1. Scheduling and public values

Both queues run higher `i64` priorities first, then earlier insertion sequence. Watermarks begin at
zero and retain all prior extrema even after draining. Neither queue implements `Clone`.

| Public value | Meaning |
| --- | --- |
| `Item<T>.item: T` | Owned queued payload. |
| `Item<T>.priority: i64` | Explicit scheduling priority. |
| `Item<T>.sequence: u64` | Stable insertion sequence, starting at zero. |
| `QueueOutcome::Completed` | A nonempty run drained all tasks with Continue. |
| `QueueOutcome::Aborted` | A selected phase returned Abort. |
| `QueueOutcome::Empty` | No task was pending at run entry. |

`Item<T>` is inspectable through `entries`; the returned slice is read-only. `QueueOutcome` is
`Clone`, `Copy`, `Debug`, `Eq`, and `PartialEq`, and is independent of response status.

## 2. ItemQueue operations

An item queue owns `T` without requiring it to be a handler. All arithmetic exhaustion errors occur
before insertion.

### 2.1. ItemQueue::new

`ItemQueue::<T>::new() -> Self`

Create an empty queue with zero watermarks; `Default` is equivalent.

```rust
let queue = ItemQueue::<String>::new();
assert!(queue.is_empty());
```

**Returns** An empty owned queue.

### 2.2. ItemQueue::add

`add(&mut self, item: T, priority: i64) -> Result<&mut Self>`

Insert and order an item with explicit priority.

```rust
let mut queue = ItemQueue::new();
queue.add("register", 10)?.add("routes", 0)?;
assert_eq!(queue.pop(), Some("register"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `item` | `T` | Owned payload. |
| `priority` | `i64` | Higher values run first; ties preserve insertion order. |

**Returns** The mutable queue for chaining, or a limit error if its `u64` insertion sequence is
exhausted.

### 2.3. ItemQueue::push_front

`push_front(&mut self, item: T) -> Result<&mut Self>`

Insert at a checked priority above the retained maximum.

```rust
let mut queue = ItemQueue::new();
queue.push_front("register")?;
assert_eq!(queue.pop(), Some("register"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `item` | `T` | Owned payload. |

**Returns** The mutable queue, or a limit error for priority overflow/underflow or sequence
exhaustion. Draining does not reset the retained watermark.

### 2.4. ItemQueue::push_back

`push_back(&mut self, item: T) -> Result<&mut Self>`

Insert at a checked priority below the retained minimum.

```rust
let mut queue = ItemQueue::new();
queue.push_back("register")?;
assert_eq!(queue.pop(), Some("register"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `item` | `T` | Owned payload. |

**Returns** The mutable queue, or a limit error for priority overflow/underflow or sequence
exhaustion. Draining does not reset the retained watermark.

### 2.5. ItemQueue::pop

`pop(&mut self) -> Option<T>`

Consume the first scheduled payload; keep its watermark history.

```rust
let mut queue = ItemQueue::new();
queue.add("register", 10)?;
assert_eq!(queue.pop(), Some("register"));
```

**Returns** The owned payload, or `None` when empty.

### 2.6. ItemQueue::len

`len(&self) -> usize`

Count pending items.

```rust
let queue = ItemQueue::<String>::new();
assert_eq!(queue.len(), 0);
```

**Returns** Pending count, excluding already consumed items.

### 2.7. ItemQueue::is_empty

`is_empty(&self) -> bool`

Test pending work without resetting history.

```rust
let queue = ItemQueue::<String>::new();
assert!(queue.is_empty());
```

**Returns** True when no item remains.

### 2.8. ItemQueue::entries

`entries(&self) -> &[Item<T>]`

Borrow items in execution order without mutating their priorities.

```rust
let mut queue = ItemQueue::new();
queue.add("register", 10)?;
assert_eq!(queue.entries()[0].priority, 10);
```

**Returns** A borrowing ordered slice with payload/priority/sequence metadata.

## 3. Task callbacks

`TaskHandler<C: Send + 'static>` is the public registration bound, implemented automatically for
compatible shared callbacks. Authors do not implement the private adaptation traits. The callable is
`Send + Sync + 'static`; its lifetime-indexed returned future is itself `Send` and may borrow
`&mut C`.

### 3.1. task

`task<C, F>(callback: F) -> F where F: for<'a> Fn(&'a mut C) -> HandlerFuture<'a, Result<Flow>> + Send + Sync + 'static`

Supply lifetime inference for the secondary closure form. Named async functions register directly.

```rust
let mut queue = TaskQueue::new();
queue.add(
    task(|state: &mut Vec<String>| {
        Box::pin(async move {
            state.push("configured".into());
            Ok(Flow::Continue)
        })
    }),
    10,
)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `callback` | `F` | Shared closure returning a pinned boxed Send future for each state-borrow lifetime. |

**Returns** The same closure, without registration or execution. `Box::pin` supplies the required
erased future shape; it does not make a borrow static.

## 4. TaskQueue operations

`TaskQueue<C>` requires `C: Send + 'static`. Run uses the caller's exact mutable state and never
spawns tasks. A selected task is consumed before before/action/after; abort/error/cancellation
retains only unstarted tasks and preserves completed mutations. After is not a finally hook.

### 4.1. TaskQueue::new

`TaskQueue::<C>::new() -> Self`

Create an empty task flow with no hooks; `Default` is equivalent.

```rust
let queue = TaskQueue::<Vec<String>>::new();
assert!(queue.is_empty());
```

**Returns** An empty queue whose context type is fixed by `C`.

### 4.2. TaskQueue::add

`add<H: TaskHandler<C>>(&mut self, action: H, priority: i64) -> Result<&mut Self>`

Register a named task at an explicit priority.

```rust
let mut queue = TaskQueue::new();
queue.add(register, 10)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `action` | `H: TaskHandler<C>` | Compatible shared task callback. |
| `priority` | `i64` | Descending priority with stable insertion ties. |

**Returns** The mutable task queue, or sequence-exhaustion error before insertion.

### 4.3. TaskQueue::push_front

`push_front<H: TaskHandler<C>>(&mut self, action: H) -> Result<&mut Self>`

Queue a task at a priority above the retained maximum.

```rust
let mut queue = TaskQueue::new();
queue.push_front(register)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `action` | `H: TaskHandler<C>` | Compatible shared task callback. |

**Returns** The mutable task queue, or priority/sequence exhaustion. Watermarks persist after all
pending tasks are consumed.

### 4.4. TaskQueue::push_back

`push_back<H: TaskHandler<C>>(&mut self, action: H) -> Result<&mut Self>`

Queue a task at a priority below the retained minimum.

```rust
let mut queue = TaskQueue::new();
queue.push_back(register)?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `action` | `H: TaskHandler<C>` | Compatible shared task callback. |

**Returns** The mutable task queue, or priority/sequence exhaustion. Watermarks persist after all
pending tasks are consumed.

### 4.5. TaskQueue::before

`before<H: TaskHandler<C>>(&mut self, action: H) -> &mut Self`

Select or replace the single before hook. Runs before each selected task; abort skips its action and
after.

```rust
let mut queue = TaskQueue::new();
queue.before(register);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `action` | `H: TaskHandler<C>` | Same callback contract as an action. |

**Returns** The mutable queue for chaining; hook registration is infallible and does not enqueue the
hook as a task.

### 4.6. TaskQueue::after

`after<H: TaskHandler<C>>(&mut self, action: H) -> &mut Self`

Select or replace the single after hook. Runs after each continuing action; abort stops queued
successors.

```rust
let mut queue = TaskQueue::new();
queue.after(register);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `action` | `H: TaskHandler<C>` | Same callback contract as an action. |

**Returns** The mutable queue for chaining; hook registration is infallible and does not enqueue the
hook as a task.

### 4.7. TaskQueue::len

`len(&self) -> usize`

Inspect pending work, excluding a selected consumed task.

```rust
let queue = TaskQueue::<Vec<String>>::new();
assert_eq!(queue.len(), 0);
```

**Returns** Pending task count.

### 4.8. TaskQueue::is_empty

`is_empty(&self) -> bool`

Inspect pending work, excluding a selected consumed task.

```rust
let queue = TaskQueue::<Vec<String>>::new();
assert!(queue.is_empty());
```

**Returns** True when no unstarted tasks remain.

### 4.9. TaskQueue::run

`async run(&mut self, state: &mut C) -> Result<QueueOutcome>`

Drain sequentially through before/action/after phases until empty, aborted, or failed. Absent hooks
simply continue.

```rust
let mut queue = TaskQueue::new();
queue.add(register, 10)?;
let mut state = Vec::new();
assert_eq!(queue.run(&mut state).await?, QueueOutcome::Completed);
assert_eq!(state, ["registered"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `state` | `&mut C` | Caller-owned shared working state, borrowed exclusively for the run. |

**Returns** `Completed`, `Aborted`, or `Empty` after awaiting. Phase failures return the original
error and skip later phases. A dropped pending run never restores the selected task or rolls back
state; remaining queued tasks may be run explicitly later.

## Related

[Startup guide](../guides/startup-with-task-queues.md) · [Event](event.md) · [Error](error.md) ·
[API index](README.md)
