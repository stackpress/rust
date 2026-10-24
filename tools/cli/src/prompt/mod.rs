//! Bounded prompt state, validation, rendering and explicit event drivers.
//!
//! No prompt runs inside host dispatch automatically. Native pipes use an explicit
//! answer/default or fail immediately; use buffered events for supplied pipe data.
//! Async callbacks must cooperate with polling/cancellation. Synchronous callbacks
//! and writers can block; the framework never moves them onto a hidden worker.
//!
//! ```
//! use stackpress_cli::prompt::{
//!     BufferedInput, PromptCancellation, PromptOptions, RenderOptions, prompt,
//! };
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let options = PromptOptions::new("Name?");
//! let mut input = BufferedInput::lines("Ada\n");
//! let mut output = Vec::new();
//! let answer = prompt(&options, &mut input, &mut output,
//!     &PromptCancellation::new(), RenderOptions::default()).await?;
//! assert_eq!(answer, "Ada");
//! # Ok(()) }
//! ```

mod driver;
mod options;
mod render;
mod state;

pub use driver::{BufferedInput, PromptCancellation, PromptInput, prompt, prompt_native};
pub use options::{PromptError, PromptLimits, PromptOptions, PromptResult, Transformer, Validator};
pub use render::{PromptRender, RenderOptions};
pub use state::{
    InvalidMode, Prefill, PromptAction, PromptEvent, PromptState, PromptStatus, Validation,
};
