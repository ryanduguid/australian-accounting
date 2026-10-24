//! The LodgeiT calculator adapter, ported from `lodgeitadapter` with its
//! output, exit codes, evidence bytes and wire bytes. The `py*` modules
//! reproduce the Python behaviour it relied on.
pub mod argparse;
pub mod argtheme;
pub mod cli;
pub mod config;
pub mod contract;
pub mod decimal;
pub mod errors;
pub mod evidence;
pub mod https;
pub mod py;
pub mod pycase;
pub mod pydatetime;
pub mod pyencode;
pub mod pyjson;
pub mod pyops;
pub mod pyos;
pub mod pypath;
pub mod pytext;
pub mod pyunicode;
pub mod pyurl;
pub mod pyvalue;
pub mod sre;
pub mod strptime;
pub mod termsize;
pub mod textwrap;
pub mod transport;
pub mod wire;

#[cfg(feature = "differential")]
pub mod vectors;
