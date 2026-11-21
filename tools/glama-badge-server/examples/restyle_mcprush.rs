// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin()
        .lock()
        .take(32_769)
        .read_to_string(&mut input)?;
    let svg = glama_badge_server::restyle_mcprush_card(&input)?;
    std::io::stdout().lock().write_all(svg.as_bytes())?;
    Ok(())
}
