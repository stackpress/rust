//! Feature-gated buffered native values; the HTTP adapter is a later M3 milestone.
//!
//! These aliases establish the dependency/type seam without a runtime or listener.

/// Caller-owned already-buffered HTTP input; this type does not bound network reads.
pub type BufferedRequest = http::Request<Vec<u8>>;
/// Adapter-produced buffered native output; no stream/backpressure support is implied.
pub type BufferedResponse = http::Response<Vec<u8>>;
