// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

pub mod api;
mod argtheme;
pub mod cli;
pub mod csv_io;
pub mod csvread;
pub mod decimal;
pub mod engine;
#[doc(hidden)]
pub mod fuzzing;
pub mod job_to_cash;
pub mod money;
pub mod py;
mod pyjson;
mod pytext;
mod pyunicode;
pub mod report;
mod tables;
mod termsize;
mod textwrap;
