//! Fuzz the CSV readers with fabricated exports: the payroll and super-export
//! importers with vendor detection, and the canonical row parser under the
//! default column mapping. A refusal is a result; a panic is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    payday_super_checker::fuzzing::read_export(data);
});
