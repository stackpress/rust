//! Send action futures retain exact executor/input borrows; pooled calls are thin forwards.

use super::{BatchMode, SourceStore};
use crate::DataSession;
use stackpress_framework_schema::source::SourceSchema;
use stackpress_sql::{Create, Engine};
use std::{future::Future, sync::Arc};

/// Full native record associated through the action's matching store/schema.
pub type ActionRecord<A> =
    <<<A as SourceActions>::Store as SourceStore>::Schema as SourceSchema>::Record;
/// Source query associated with this action's store.
pub type ActionQuery<A> = <<A as SourceActions>::Store as SourceStore>::Query;
/// Original generated model error, without strict-envelope conversion.
pub type ActionResult<A, T> = Result<T, <<A as SourceActions>::Store as SourceStore>::Error>;

/// Generated source actions with immutable state and explicit database ownership.
///
/// Implementations must take no model/config lock across await. The exclusive DataSession
/// borrow intentionally persists through database work; only its caller finishes a borrowed
/// transaction. Cancellation releases Rust borrows, not proof of rollback/remote outcome.
/// Every async contract explicitly requires Send. No public method adds a policy or guard.
///
/// Keep the returned record after the caller releases its exclusive executor borrow:
///
/// ```
/// # #[cfg(not(target_arch = "wasm32"))] {
/// use stackpress_framework_sql::{DataSession, source::{ActionQuery, ActionRecord,
///     ActionResult, SourceActions}};
///
/// async fn borrowed_read<A: SourceActions>(
///     actions: &A,
///     db: &mut DataSession<'_>,
///     query: &ActionQuery<A>,
/// ) -> ActionResult<A, Option<ActionRecord<A>>> {
///     actions.find_in(db, query).await
/// }
/// # }
/// ```
pub trait SourceActions: Send + Sync + Sized + 'static {
    /// Immutable generated store, with its schema/query/error types.
    type Store: SourceStore;
    /// Original per-item batch status shape; attempt statuses need not imply commit.
    type BatchStatus: Send + Sync + 'static;

    /// Own the engine handle, without opening a database or submitting a query.
    fn new(engine: Arc<Engine>, store: Self::Store) -> Self;
    /// Borrow the application-owned engine for ordinary pooled operations.
    fn engine(&self) -> &Engine;
    /// Inspect immutable metadata/codecs without taking an async lock.
    fn store(&self) -> &Self::Store;

    /// Source create on the caller's exact executor.
    fn create_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a ActionRecord<Self>,
    ) -> impl Future<Output = ActionResult<Self, ActionRecord<Self>>> + Send + 'a;
    /// Source find on the caller's exact executor, returning an owned optional record.
    fn find_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Option<ActionRecord<Self>>>> + Send + 'a;
    /// Source multi-row read on the caller's exact executor.
    fn find_all_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send + 'a;
    /// Source count on the caller's exact executor.
    fn count_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, u64>> + Send + 'a;
    /// Source update retains mutable input and returns owned original source results.
    fn update_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a ActionQuery<Self>,
        input: &'a mut ActionRecord<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send + 'a;
    /// Physical delete on the caller's exact executor.
    fn delete_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send + 'a;
    /// Source upsert, which may return null instead of creating for a nonexistent ID.
    fn upsert_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        input: &'a mut ActionRecord<Self>,
    ) -> impl Future<Output = ActionResult<Self, Option<ActionRecord<Self>>>> + Send + 'a;
    /// Source soft removal on the caller's exact executor.
    fn remove_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send + 'a;
    /// Source restore on the caller's exact executor.
    fn restore_in<'a, 'db: 'a>(
        &'a self,
        db: &'a mut DataSession<'db>,
        query: &'a ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send + 'a;

    /// Explicit source batch owns its transaction and preserves original status semantics.
    fn batch(
        &self,
        inputs: &mut [ActionRecord<Self>],
        mode: BatchMode,
    ) -> impl Future<Output = Vec<Self::BatchStatus>> + Send;
    /// Explicit source installation; the implementation uses the accepted setup selector.
    fn install(&self) -> impl Future<Output = ActionResult<Self, ()>> + Send;
    /// Explicit adjacent upgrade; revision policy is not inferred from a successful call.
    fn upgrade(&self, to: Option<&Create>) -> impl Future<Output = ActionResult<Self, ()>> + Send;
    /// Explicit source purge with dialect/FK behavior preserved by the implementation.
    fn purge(&self, cascade: bool) -> impl Future<Output = ActionResult<Self, ()>> + Send;
    /// Explicit uninstall; no other model's data or startup registration is affected.
    fn uninstall(&self) -> impl Future<Output = ActionResult<Self, ()>> + Send;

    /// Pooled create is the same implementation as the borrowed operation, without a lock.
    fn create(
        &self,
        input: &ActionRecord<Self>,
    ) -> impl Future<Output = ActionResult<Self, ActionRecord<Self>>> + Send {
        async move {
            self.create_in(&mut DataSession::pooled(self.engine()), input)
                .await
        }
    }
    /// Pooled find retains the same query/result contract.
    fn find(
        &self,
        query: &ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Option<ActionRecord<Self>>>> + Send {
        async move {
            self.find_in(&mut DataSession::pooled(self.engine()), query)
                .await
        }
    }
    /// Pooled multi-row read retains owned results and source query semantics.
    fn find_all(
        &self,
        query: &ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send {
        async move {
            self.find_all_in(&mut DataSession::pooled(self.engine()), query)
                .await
        }
    }
    /// Pooled count delegates without changing paging or filter policy.
    fn count(
        &self,
        query: &ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, u64>> + Send {
        async move {
            self.count_in(&mut DataSession::pooled(self.engine()), query)
                .await
        }
    }
    /// Pooled update preserves the exclusive mutable source input borrow.
    fn update(
        &self,
        query: &ActionQuery<Self>,
        input: &mut ActionRecord<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send {
        async move {
            self.update_in(&mut DataSession::pooled(self.engine()), query, input)
                .await
        }
    }
    /// Pooled delete delegates to the exact same generated delete implementation.
    fn delete(
        &self,
        query: &ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send {
        async move {
            self.delete_in(&mut DataSession::pooled(self.engine()), query)
                .await
        }
    }
    /// Pooled upsert preserves null and source-visible input mutation.
    fn upsert(
        &self,
        input: &mut ActionRecord<Self>,
    ) -> impl Future<Output = ActionResult<Self, Option<ActionRecord<Self>>>> + Send {
        async move {
            self.upsert_in(&mut DataSession::pooled(self.engine()), input)
                .await
        }
    }
    /// Pooled remove adds no session invalidation or permission guard.
    fn remove(
        &self,
        query: &ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send {
        async move {
            self.remove_in(&mut DataSession::pooled(self.engine()), query)
                .await
        }
    }
    /// Pooled restore preserves source selection of inactive rows.
    fn restore(
        &self,
        query: &ActionQuery<Self>,
    ) -> impl Future<Output = ActionResult<Self, Vec<ActionRecord<Self>>>> + Send {
        async move {
            self.restore_in(&mut DataSession::pooled(self.engine()), query)
                .await
        }
    }
}
