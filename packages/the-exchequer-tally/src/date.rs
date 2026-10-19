#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(pub String);
impl Date {
    pub fn parse(text: &str) -> Result<Self, String> {
        let b = text.as_bytes();
        let bad = || "payment date must be YYYY-MM-DD".to_string();
        if b.len() != 10
            || b[4] != b'-'
            || b[7] != b'-'
            || !b
                .iter()
                .enumerate()
                .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        {
            return Err(bad());
        }
        let y: i32 = text[..4].parse().map_err(|_| bad())?;
        let m: u32 = text[5..7].parse().map_err(|_| bad())?;
        let d: u32 = text[8..].parse().map_err(|_| bad())?;
        let days = match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) => 29,
            2 => 28,
            _ => 0,
        };
        if y == 0 || d == 0 || d > days {
            return Err(bad());
        }
        Ok(Self(text.into()))
    }
}
impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
