// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

/// The command words after the program name. They are data for the command,
/// not a path and not a security decision. The program name itself is omitted
/// because an operator can set it to any text.
pub fn command_words() -> Vec<String> {
    let read: fn() -> std::env::Args = std::env::args;
    read().skip(1).collect()
}
