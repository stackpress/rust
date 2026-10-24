//! Cooperative execution outcomes, independent of response/application status.

/// A handler's decision about starting the remaining work in its pipeline.
///
/// Abort preserves completed mutations; it does not roll them back or signal
/// an application error. A handler failure instead travels through Result.
///
/// # Examples
///
/// ```
/// use stackpress_lib::Flow;
///
/// let next_step = Flow::Continue;
/// assert_ne!(next_step, Flow::Abort);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Flow {
    /// Permit execution of the remaining phases/listeners.
    Continue,
    /// Skip remaining phases/listeners without undoing completed work.
    Abort,
}

/// The result of listener dispatch, separate from any response status.
///
/// Missing listeners and cooperative abort are ordinary outcomes; neither
/// supplies an HTTP code or terminal exit code automatically.
///
/// # Examples
///
/// ```
/// use stackpress_lib::DispatchOutcome;
///
/// let outcome = DispatchOutcome::NoListener;
/// assert_ne!(outcome, DispatchOutcome::Completed);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    /// Every selected listener completed and permitted the next step.
    Completed,
    /// A selected listener cooperatively stopped further execution.
    Aborted,
    /// No listener matched this invocation.
    NoListener,
}
