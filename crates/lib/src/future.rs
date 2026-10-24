//! Lifetime-bounded Send future erasure without an executor dependency.

use std::future::Future;
use std::pin::Pin;

/// A handler future that can borrow invocation values and move between threads.
///
/// The lifetime limits the borrow; Send does not imply `'static`. To spawn work
/// on an executor requiring `'static`, move its owned inputs into an outer
/// future. Boxing pins the future without extending its lifetime or changing
/// its thread-safety guarantees. The output type is chosen by the caller.
///
/// # Examples
///
/// ```
/// use stackpress_lib::HandlerFuture;
///
/// let input = String::from("profile-search");
/// let work: HandlerFuture<'_, usize> = Box::pin(async { input.len() });
/// // This future borrows input; it can be moved, but cannot outlive input.
/// fn require_send<T: Send>(_: &T) {}
/// require_send(&work);
/// drop(work);
/// ```
pub type HandlerFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
