// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

/// The command words after the program name. They are data for the command,
/// not a path and not a security decision. The program name itself is omitted
/// because an operator can set it to any text.
pub fn command_words() -> Vec<String> {
    let read: fn() -> std::env::Args = std::env::args;
    read().skip(1).collect()
}
