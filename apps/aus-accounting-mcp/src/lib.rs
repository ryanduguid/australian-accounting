// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Native candidate. The complete migration acceptance remains in NATIVE.md.
pub mod adapters;
pub mod contracts;
pub mod handler;
mod identifiers;
pub mod prompts;
pub mod resources;
mod retrieval;
mod validation;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
