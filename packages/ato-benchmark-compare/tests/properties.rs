//! Seeded properties for account identity and mapping round trips. Seeds, run
//! counts, examples and input domains are the Python fast-check properties'; a
//! local SplitMix64 replaces fast-check's generator.

use ato_benchmark_compare::mapping::{
    BUCKETS, MappingRow, account_key, read_mapping, write_mapping,
};
use ato_benchmark_compare::py::PyPath;
use sha2::{Digest, Sha256};

const RUNS: usize = 500;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn pick<'a, T: ?Sized>(&mut self, items: &[&'a T]) -> &'a T {
        items[self.below(items.len())]
    }

    /// One word: a character of `first`, then up to `max` of `rest`.
    fn word(&mut self, first: &str, rest: &str, max: usize) -> String {
        let first: Vec<char> = first.chars().collect();
        let rest: Vec<char> = rest.chars().collect();
        let mut out = String::from(first[self.below(first.len())]);
        for _ in 0..self.below(max + 1) {
            out.push(rest[self.below(rest.len())]);
        }
        out
    }
}

const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const TAIL: &str = "0123456789&'-";

fn sha(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Python's str.title() for ASCII: a letter after any non-letter is upper case.
fn title(text: &str) -> String {
    let mut out = String::new();
    let mut after_letter = false;
    for c in text.chars() {
        out.push(if after_letter {
            c
        } else {
            c.to_ascii_uppercase()
        });
        after_letter = c.is_ascii_alphabetic();
    }
    out
}

/// A canonical account name and a variant differing only in case and whitespace.
fn equivalent_accounts(rng: &mut Rng) -> (String, String) {
    let letters = format!("{LOWER}{}", LOWER.to_uppercase());
    let words: Vec<String> = (0..1 + rng.below(4))
        .map(|_| rng.word(&letters, &format!("{letters}{TAIL}"), 15))
        .collect();
    let canonical = words.join(" ").to_lowercase();
    let cased = match rng.below(3) {
        0 => canonical.clone(),
        1 => canonical.to_uppercase(),
        _ => title(&canonical),
    };
    let separator = rng.pick(&[" ", "  ", "\t", "\n", "\r\n"]);
    let left = rng.pick(&["", " ", "\t", "\n"]);
    let right = rng.pick(&["", " ", "\t", "\r\n"]);
    let variant = format!(
        "{left}{}{right}",
        cased.split(' ').collect::<Vec<_>>().join(separator)
    );
    (canonical, variant)
}

/// Removing case folding, trimming or whitespace collapse must break this.
#[test]
fn case_and_whitespace_variants_keep_one_logical_identity() {
    let mut rng = Rng(0xA70);
    let mut pairs = vec![
        (
            "motor vehicle".to_string(),
            "  MOTOR\tVEHICLE\r\n".to_string(),
        ),
        ("strasse".to_string(), "StraSSE".to_string()),
    ];
    pairs.extend((0..RUNS).map(|_| equivalent_accounts(&mut rng)));
    for (canonical, variant) in pairs {
        let expected = sha(&canonical);
        assert_eq!(account_key(&canonical), expected, "{canonical:?}");
        assert_eq!(account_key(&variant), expected, "{variant:?}");
    }
}

/// Reviewed mapping specs, unique by the identity each must keep.
fn row_specs(rng: &mut Rng) -> Vec<(MappingRow, String)> {
    let mut buckets: Vec<&str> = BUCKETS.iter().map(|(b, _)| *b).collect();
    buckets.sort();
    let prefixes = [
        ("", ""),
        ("=", "="),
        ("@", "@"),
        ("+ref ", "+ref "),
        ("-ref ", "-ref "),
        ("\t=", "="),
    ];
    let mut specs: Vec<(MappingRow, String)> = Vec::new();
    for _ in 0..1 + rng.below(8) {
        let words: Vec<String> = (0..1 + rng.below(3))
            .map(|_| rng.word(LOWER, &format!("{LOWER}{TAIL}"), 12))
            .collect();
        let name = words.join(" ");
        let (prefix, normalised) = prefixes[rng.below(prefixes.len())];
        let row = MappingRow {
            account: format!("{prefix}{name}"),
            bucket: rng.pick(&buckets).to_string(),
            source: rng.pick(&["reviewed", "suggested"]).to_string(),
            note: rng.pick(&["", "checked", "invented fixture"]).to_string(),
            amount: rng.pick(&["", "0", "1.25", "-50"]).to_string(),
        };
        let identity = format!("{normalised}{name}");
        if !specs.iter().any(|(_, seen)| *seen == identity) {
            specs.push((row, identity));
        }
    }
    specs
}

/// Dropping CSV guarding, keyed identity or a reviewed field must break this.
#[test]
fn generated_reviewed_mappings_round_trip_without_identity_loss() {
    let mut rng = Rng(0xA701D);
    let dir = std::env::temp_dir().join(format!("ato-properties-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = PyPath::new(&dir.join("mapping.csv").to_string_lossy());
    for _ in 0..RUNS {
        let specs = row_specs(&mut rng);
        let rows: Vec<MappingRow> = specs.iter().map(|(row, _)| row.clone()).collect();
        write_mapping(&path, &rows).unwrap();
        let mut expected: Vec<(String, MappingRow)> = specs
            .into_iter()
            .map(|(row, identity)| (sha(&identity), row))
            .collect();
        let mut actual = read_mapping(&path).unwrap();
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        actual.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(actual, expected);
    }
    let _ = std::fs::remove_dir_all(&dir);
}
