// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Native candidate. The complete migration acceptance remains in NATIVE.md.
pub mod adapters;
pub mod command_line;
pub mod contracts;
pub mod doctor;
pub mod handler;
mod identifiers;
pub mod prompts;
pub mod resources;
mod retrieval;
mod validation;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
