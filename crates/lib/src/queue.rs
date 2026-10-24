//! Stable priority scheduling and sequential cooperative task execution.
//!
//! ```
//! use stackpress_lib::{Flow, Result, queue::TaskQueue};
//! async fn increment(value: &mut i32) -> Result<Flow> {
//!     *value += 1;
//!     Ok(Flow::Continue)
//! }
//! let mut tasks = TaskQueue::new();
//! tasks.add(increment, 10)?;
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

use crate::{Error, Flow, HandlerFuture, Result};

/// A queued value with explicit signed priority and deterministic insertion order.
pub struct Item<T> {
    /// Owned queued payload.
    pub item: T,
    /// Higher priorities run first.
    pub priority: i64,
    /// Increasing registration sequence, including equal-priority entries.
    pub sequence: u64,
}

/// An owned stable priority queue; draining retains priority high/low watermarks.
pub struct ItemQueue<T> {
    /// Pending items sorted in execution order; pop consumes the first entry.
    entries: Vec<Item<T>>,
    /// Next checked insertion order, never reused after draining.
    sequence: u64,
    /// Retained minimum priority, including already consumed items.
    lower: i64,
    /// Retained maximum priority, including already consumed items.
    upper: i64,
}
impl<T> Default for ItemQueue<T> {
    /// Construct an empty priority queue.
    fn default() -> Self {
        // Use the same owned priority state as explicit construction.
        Self::new()
    }
}
impl<T> ItemQueue<T> {
    /// Construct a queue without shared mutable state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// queue.add("first", 10)?;
    /// assert_eq!(queue.pop(), Some("first"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // Priority watermarks start at zero and survive draining so front/back retain their
        // meaning.
        Self {
            entries: Vec::new(),
            sequence: 0,
            lower: 0,
            upper: 0,
        }
    }
    /// Insert a value, sorting descending priority then ascending insertion order.
    ///
    /// # Errors
    /// Rejects exhausted sequence arithmetic before modifying the queue.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// queue.add("first", 10)?.add("second", 10)?;
    /// assert_eq!(queue.pop(), Some("first"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn add(&mut self, item: T, priority: i64) -> Result<&mut Self> {
        // Reserve insertion order before mutation, then compare priorities without subtraction
        // overflow.
        let next = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| Error::limit("queue sequence exhausted"))?;
        self.entries.push(Item {
            item,
            priority,
            sequence: self.sequence,
        });
        // Advance monotonic order only after the corresponding item is stored.
        self.sequence = next;
        // Retain extremes so front/back remain deterministic after older items are popped.
        self.lower = self.lower.min(priority);
        self.upper = self.upper.max(priority);
        // Higher priorities win; explicit sequence order resolves equal-priority ties consistently.
        self.entries.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(a.sequence.cmp(&b.sequence))
        });
        Ok(self)
    }
    /// Insert below the retained low watermark.
    ///
    /// # Errors
    /// Rejects priority/sequence exhaustion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// queue.add("middle", 0)?.push_back("last")?;
    /// assert_eq!(queue.pop(), Some("middle"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn push_back(&mut self, item: T) -> Result<&mut Self> {
        // Choose a checked priority below the retained low watermark, even after a drain.
        self.add(
            item,
            self.lower
                .checked_sub(1)
                .ok_or_else(|| Error::limit("priority underflow"))?,
        )
    }
    /// Insert above the retained high watermark.
    ///
    /// # Errors
    /// Rejects priority/sequence exhaustion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// queue.add("middle", 0)?.push_front("first")?;
    /// assert_eq!(queue.pop(), Some("first"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn push_front(&mut self, item: T) -> Result<&mut Self> {
        // Choose a checked priority above the retained high watermark without wrapping signed
        // extremes.
        self.add(
            item,
            self.upper
                .checked_add(1)
                .ok_or_else(|| Error::limit("priority overflow"))?,
        )
    }
    /// Consume the highest-priority item, with registration-order ties.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// queue.add("first", 10)?;
    /// assert_eq!(queue.pop(), Some("first"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        // Empty queues return None; consuming the selected item leaves all unstarted work in order.
        if self.entries.is_empty() {
            None
        } else {
            Some(self.entries.remove(0).item)
        }
    }
    /// Inspect remaining length.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// queue.add("first", 10)?;
    /// assert_eq!(queue.len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        // Count pending items, independently of the retained priority watermarks.
        self.entries.len()
    }
    /// Report no pending work.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// assert!(queue.is_empty());
    /// queue.add("first", 10)?;
    /// assert!(!queue.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        // Draining removes pending items but deliberately does not reset insertion/priority
        // history.
        self.entries.is_empty()
    }
    /// Borrow immutable entries in their execution order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::ItemQueue;
    /// let mut queue = ItemQueue::new();
    /// queue.add("first", 10)?;
    /// assert_eq!(queue.entries()[0].item, "first");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn entries(&self) -> &[Item<T>] {
        // Borrow execution order without allowing priority/sequence invariants to be edited.
        &self.entries
    }
}

/// Queue control result, separate from response/application status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueOutcome {
    /// Every selected task ran.
    Completed,
    /// Before/action/after cooperatively stopped the queue.
    Aborted,
    /// No task was pending.
    Empty,
}

// Keep the lifetime-indexed future family out of public registration signatures.
mod contract {
    use crate::{Flow, HandlerFuture, Result};
    use std::future::Future;

    /// Internal borrowed-future family for a concrete mutable state type.
    pub trait TaskFn<'a, C: 'a> {
        /// A future whose own thread safety is required.
        type Future: Future<Output = Result<Flow>> + Send + 'a;
        /// Invoke an ordinary task function for this borrow lifetime.
        fn call(&'a self, state: &'a mut C) -> Self::Future;
    }
    impl<'a, C: 'a, F, Fut> TaskFn<'a, C> for F
    where
        F: Fn(&'a mut C) -> Fut,
        Fut: Future<Output = Result<Flow>> + Send + 'a,
    {
        type Future = Fut;
        /// Return the original concrete future without an extra async layer.
        fn call(&'a self, state: &'a mut C) -> Fut {
            // Return the original concrete future for this state borrow; no lifetime is extended.
            self(state)
        }
    }
    /// Type-erased task storage, with no mutable callback lock.
    pub trait Task<C>: Send + Sync {
        /// Borrow the state and callable only until this task completes.
        fn run<'a>(&'a self, state: &'a mut C) -> HandlerFuture<'a, Result<Flow>>;
    }
    impl<C: Send + 'static, H> Task<C> for H
    where
        H: for<'a> TaskFn<'a, C> + Send + Sync + 'static,
    {
        /// Erase the already-proved Send future.
        fn run<'a>(&'a self, state: &'a mut C) -> HandlerFuture<'a, Result<Flow>> {
            // The future family already proves Send before its object-safe boxing boundary.
            Box::pin(TaskFn::call(self, state))
        }
    }
}
use contract::Task;

/// Sealed adapter automatically implemented by named Send async task functions.
///
/// Callers register functions directly; no manual lifetime/future-family impl is needed.
pub trait TaskHandler<C: Send + 'static>: contract::Task<C> + Send + Sync + 'static {}
impl<C: Send + 'static, H> TaskHandler<C> for H where
    H: for<'a> contract::TaskFn<'a, C> + Send + Sync + 'static
{
}

/// Identity helper for the secondary boxed borrowing-closure task style.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::queue::{TaskQueue, task};
/// # use stackpress_lib::{Flow, Result};
/// async fn increment(value: &mut u64) -> Result<Flow> {
///     *value += 1;
///     Ok(Flow::Continue)
/// }
/// let mut tasks = TaskQueue::<u64>::new();
/// tasks.add(increment, 10)?;
/// // Box pins a future borrowing value; move retains that borrow inside the future.
/// tasks.add(
///     task(|value: &mut u64| {
///         Box::pin(async move {
///             *value += 1;
///             Ok(Flow::Continue)
///         })
///     }),
///     0,
/// )?;
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn task<C, F>(callback: F) -> F
where
    F: for<'a> Fn(&'a mut C) -> HandlerFuture<'a, Result<Flow>> + Send + Sync + 'static,
{
    // This helper provides closure lifetime inference only; it neither registers nor invokes work.
    callback
}

/// Sequential async task flow over caller-owned mutable state.
///
/// The selected task is consumed before its phases; abort/error/cancellation do
/// not replay it. Unstarted tasks remain queued. After is not a finally hook.
pub struct TaskQueue<C: Send + 'static> {
    /// Owned unstarted callbacks; selection removes a task before any await.
    queue: ItemQueue<Box<dyn Task<C>>>,
    /// Single shared callable run before each selected task.
    before: Option<Box<dyn Task<C>>>,
    /// Single success-path hook, skipped after action abort/error.
    after: Option<Box<dyn Task<C>>>,
}
impl<C: Send + 'static> Default for TaskQueue<C> {
    /// Construct an empty task pipeline.
    fn default() -> Self {
        // Default task flow starts with no pending work or hidden hook.
        Self::new()
    }
}
impl<C: Send + 'static> TaskQueue<C> {
    /// Construct owned task state without selecting an executor.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// tasks.add(increment, 10)?;
    /// assert_eq!(tasks.len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // Own callback storage independently; the caller supplies mutable execution state only at
        // run time.
        Self {
            queue: ItemQueue::new(),
            before: None,
            after: None,
        }
    }
    /// Add a named async task at an explicit priority.
    ///
    /// # Errors
    /// Rejects exhausted registration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// tasks.add(increment, 10)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn add<H>(&mut self, action: H, priority: i64) -> Result<&mut Self>
    where
        H: TaskHandler<C>,
    {
        // Erase the accepted Send task only inside queue storage, keeping user registration bounds
        // small.
        self.queue.add(Box::new(action), priority)?;
        Ok(self)
    }
    /// Add below all retained priorities.
    ///
    /// # Errors
    /// Rejects priority/sequence overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// tasks.add(increment, 0)?.push_back(increment)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn push_back<H>(&mut self, action: H) -> Result<&mut Self>
    where
        H: TaskHandler<C>,
    {
        // Use the item queue's checked low watermark rather than assuming pending priorities start
        // at zero.
        self.queue.push_back(Box::new(action))?;
        Ok(self)
    }
    /// Add above all retained priorities.
    ///
    /// # Errors
    /// Rejects priority/sequence overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// tasks.add(increment, 0)?.push_front(increment)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn push_front<H>(&mut self, action: H) -> Result<&mut Self>
    where
        H: TaskHandler<C>,
    {
        // Use the item queue's checked high watermark while preserving the task's actual Send
        // future contract.
        self.queue.push_front(Box::new(action))?;
        Ok(self)
    }
    /// Replace the single optional before callback.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// tasks.before(increment).add(increment, 0)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn before<H>(&mut self, action: H) -> &mut Self
    where
        H: TaskHandler<C>,
    {
        // Replace the one construction-time hook; it runs before each selected action.
        self.before = Some(Box::new(action));
        self
    }
    /// Replace the single optional after callback.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// tasks.after(increment).add(increment, 0)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn after<H>(&mut self, action: H) -> &mut Self
    where
        H: TaskHandler<C>,
    {
        // This hook runs only after successful Continue, never as a finally or rollback action.
        self.after = Some(Box::new(action));
        self
    }
    /// Inspect unstarted task count.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// tasks.add(increment, 0)?;
    /// assert_eq!(tasks.len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        // Count only tasks that have not yet been selected for execution.
        self.queue.len()
    }
    /// Report no unstarted tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// assert!(tasks.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        // A selected/cancelled task is already consumed, so only genuinely unstarted work remains.
        self.queue.is_empty()
    }
    /// Drain sequentially, returning immediately on phase abort or error.
    ///
    /// # Errors
    /// Returns the original task/hook error without replay or rollback.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::queue::{TaskQueue, task};
    /// # use stackpress_lib::{Flow, Result};
    /// async fn increment(value: &mut u64) -> Result<Flow> {
    ///     *value += 1;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut tasks = TaskQueue::<u64>::new();
    /// # async fn example() -> Result<()> {
    /// let mut tasks = TaskQueue::new();
    /// tasks.add(increment, 0)?;
    /// let mut value = 0;
    /// tasks.run(&mut value).await?;
    /// assert_eq!(value, 1);
    /// #     Ok(())
    /// # }
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub async fn run(&mut self, state: &mut C) -> Result<QueueOutcome> {
        // Empty flow has its own outcome; selecting a task consumes it before any phase awaits.
        if self.queue.is_empty() {
            return Ok(QueueOutcome::Empty);
        }
        // Consume before awaiting: cancellation/error never silently replays a selected side
        // effect.
        while let Some(action) = self.queue.pop() {
            // A before-hook failure/abort skips this action and every later phase while retaining
            // queued successors.
            if let Some(before) = &self.before
                && before.run(state).await? == Flow::Abort
            {
                return Ok(QueueOutcome::Aborted);
            }
            // Wait for the action before its after hook; ? preserves the original failure and
            // prefix mutation.
            if action.run(state).await? == Flow::Abort {
                return Ok(QueueOutcome::Aborted);
            }
            // After runs only for a continuing action; its abort/error stops the remaining queue.
            if let Some(after) = &self.after
                && after.run(state).await? == Flow::Abort
            {
                return Ok(QueueOutcome::Aborted);
            }
        }
        Ok(QueueOutcome::Completed)
    }
}
