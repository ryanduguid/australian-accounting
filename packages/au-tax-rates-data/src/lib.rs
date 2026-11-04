// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The AU tax rates dataset tool: `rates snapshot`, `validate`, `check` and
//! `ato-tables`, ported from `rates.py` with its output, exit codes and file
//! bytes. The `py*` modules reproduce the Python behaviour it relied on.
pub mod argparse;
pub mod argtheme;
pub mod decimal;
pub mod https;
pub mod py;
pub mod pycase;
pub mod pydatetime;
pub mod pyjson;
pub mod pyos;
pub mod pypath;
pub mod pytext;
pub mod pyunicode;
pub mod pyurl;
pub mod pyvalue;
pub mod rates;
pub mod sre;
pub mod strptime;
pub mod termsize;
pub mod textwrap;
