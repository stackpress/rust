//! Run the same value/default/access matrix against the actual native generated contracts.
#![cfg(feature = "matrix")]
use generated_workspace_app::generated::{contracts, metadata};
#[path = "../../support/matrix_contracts.rs"]
mod cases;
