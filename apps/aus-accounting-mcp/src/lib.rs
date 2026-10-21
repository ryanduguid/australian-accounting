//! Native candidate. The complete migration acceptance remains in NATIVE.md.
pub mod adapters;
pub mod contracts;
pub mod handler;
pub mod prompts;
pub mod resources;
mod retrieval;
mod validation;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
