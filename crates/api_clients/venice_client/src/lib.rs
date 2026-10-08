//! A Venice.ai wrapper.
//!
//! Venice's API is synchronous for image generation: a single POST returns
//! base64-encoded images. Video generation uses an async queue and is not
//! covered by this client yet.

pub mod client;
pub mod creds;
pub mod error;
pub mod requests;
pub mod utils;
