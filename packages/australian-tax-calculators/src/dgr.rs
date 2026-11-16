// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `dgr-check`: whether the recipient of each donation is shown as a
//! deductible gift recipient (DGR) in one week's ABN Bulk Extract, from what
//! date, and which ITAA 1997 Division 30 conditions the data cannot settle.
//! It never says a payment is deductible: DGR status is one condition of
//! several. The data is `data/dgr-register.jsonl`, built from the extract by
//! `examples/dgr_register.rs`; its first line records the extract it came from.
use crate::api::{Refusal, optional_decimal};
use crate::dates;
use crate::decimal::Decimal;
use crate::engine;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::sync::OnceLock;

const COMMAND: &str = "dgr-check";
const FIELDS: [&str; 4] = ["abn", "name", "date", "amount"];
const MAX_DONATIONS: usize = 10_000;
const MAX_CANDIDATES: usize = 10;

/// Gifts made from this date need no minimum value: Treasury Laws Amendment
/// (Delivering an Efficient and Trusted Tax System) Act 2026 (No. 58, 2026)
/// Sch 1 items 1 to 6 repealed the '$2 or more' special conditions of ITAA 1997
/// s 30-15, and item 7 applies that to a gift or contribution made on or after
/// 1 July 2024.
const NO_MINIMUM_FROM: &str = "2024-07-01";

const ITAA_1997: &str = "https://www.legislation.gov.au/C2004A05138/2026-10-01/2026-10-01/text/original/epub/OEBPS/document_1/document_1.html";
const ACT_58_2026: &str = "https://www.legislation.gov.au/C2026A00058/asmade/text";
const ATO_GIFTS: &str = "https://www.ato.gov.au/individuals-and-families/income-deductions-offsets-and-records/deductions-you-can-claim/gifts-and-donations";
const BULK_EXTRACT: &str = "https://data.gov.au/data/dataset/abn-bulk-extract";
const ABN_LOOKUP_DGR: &str = "https://abr.business.gov.au/Tools/DgrListing";
const ABN_FORMAT: &str = "https://abr.business.gov.au/Help/AbnFormat";
const ABN_LOOKUP_DISCLAIMER: &str = "https://abr.business.gov.au/Home/Disclaimer";

const SOURCES: [&str; 7] = [
    ITAA_1997,
    ACT_58_2026,
    ATO_GIFTS,
    BULK_EXTRACT,
    ABN_LOOKUP_DGR,
    ABN_FORMAT,
    ABN_LOOKUP_DISCLAIMER,
];

const SCOPE: &str = "A gift of money to a fund, authority or institution under ITAA 1997 s 30-15 item 1 or 2, by an ABN or by a name from the ledger. Checks the recipient against the deductible gift recipient entries in one week's ABN Bulk Extract and lists the other conditions of Division 30 the data cannot settle; applies the $2 minimum only to a gift made before 1 July 2024 (Act No. 58, 2026, Sch 1). Gifts of property, shares, cultural and heritage gifts (s 30-15 items 1(b) to (e) and 4 to 6), contributions for fund-raising events and auctions (items 7 and 8), gifts to political parties and independent candidates (Subdivision 30-DA), conservation covenants (Division 31), the amount of a deduction and the limit that stops a gift deduction adding to or creating a tax loss are not worked out.";

/// What one copy of the data cannot show; every result carries these.
const LIMITS: [&str; 6] = [
    "The data is a copy of the ABN Bulk Extract, which holds current details only: it shows the DGR entries current on its extract date, with the date each started. An endorsement that ended before then is not in it, so it cannot show that a recipient was a DGR before an entry's start date, and it gives no end date for an entry.",
    "An ABN not in the data is not shown as a DGR in the data of that date; that does not establish that it is not one. ABN Lookup holds historical details of each ABN.",
    "A DGR named in the income tax law need not be endorsed, and one without an ABN is not in the Australian Business Register at all. ITAA 1997 s 30-229(1), note 1: an entry of the statement, or its lack, 'does not affect whether you can deduct a gift to the fund, authority or institution'.",
    "The data does not give the DGR item or category, so it cannot show the gift conditions that category carries (s 30-15 item 1, special condition (c)). ABN Lookup shows the DGR item.",
    "A name match is a candidate only, never a finding: the ABN on the receipt settles which entity was paid.",
    "The Commissioner's statement on relying on ABN Lookup concerns ABN Lookup itself (https://abr.business.gov.au/Home/Disclaimer); confirm a result there before relying on it.",
];

/// The register: the extract's header and every ABN with a DGR entry, sorted by ABN.
struct Register {
    header: Value,
    data_date: String,
    entries: Vec<Entry>,
    /// Each name word and the (entry, name) pairs it appears in.
    words: HashMap<String, Vec<(usize, usize)>>,
}

/// A data row: ABN, status, status date, name, DGR entries and other names.
type Row = (
    String,
    String,
    String,
    String,
    Vec<(String, Option<String>)>,
    Vec<String>,
);

struct Entry {
    abn: String,
    cancelled: bool,
    status_from: String,
    /// The entity's name, its other names, then its fund names.
    names: Vec<String>,
    other_names: usize,
    /// Each DGR entry's start date and, when the endorsement is only for the
    /// operation of a fund, authority or institution, its name (empty where
    /// the extract gives none).
    dgr: Vec<(String, Option<String>)>,
}

fn register() -> &'static Register {
    static REGISTER: OnceLock<Register> = OnceLock::new();
    REGISTER.get_or_init(|| parse_register(include_str!("../data/dgr-register.jsonl")))
}

/// The bundled data, which `tests/dgr_check.rs` reads in full.
fn parse_register(text: &str) -> Register {
    let mut lines = text.lines();
    let header: Value = serde_json::from_str(lines.next().expect("a header")).expect("JSON header");
    let time = header["extract_time"].as_str().expect("an extract time");
    let data_date = time.get(..10).filter(|d| dates::parse(d).is_some());
    let data_date = data_date.expect("a YYYY-MM-DD extract date").to_string();
    let mut entries: Vec<Entry> = Vec::new();
    let mut words: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for line in lines {
        let row: Row = serde_json::from_str(line).expect("a register row");
        let (abn, status, status_from, name, dgr, others) = row;
        assert!(abn_digits(&abn).is_some(), "{abn}: a valid ABN");
        assert!(entries.last().is_none_or(|e| e.abn < abn), "{abn}: sorted");
        assert!(matches!(status.as_str(), "ACT" | "CAN"), "{abn}: {status}");
        let dated = std::iter::once(&status_from).chain(dgr.iter().map(|(from, _)| from));
        assert!(
            dated.clone().all(|d| dates::parse(d).is_some()),
            "{abn}: dates"
        );
        assert!(!dgr.is_empty(), "{abn}: a DGR entry");
        let other_names = others.len();
        let mut names = vec![name];
        names.extend(others);
        names.extend(dgr.iter().filter_map(|(_, fund)| fund.clone()));
        for (at, name) in names.iter().enumerate() {
            for word in name_words(name) {
                let postings = words.entry(word).or_default();
                if postings.last() != Some(&(entries.len(), at)) {
                    postings.push((entries.len(), at));
                }
            }
        }
        entries.push(Entry {
            abn,
            cancelled: status == "CAN",
            status_from,
            names,
            other_names,
            dgr,
        });
    }
    Register {
        header,
        data_date,
        entries,
        words,
    }
}

/// The ABN's 11 digits, if it passes ABN Lookup's 'Format of the ABN' test
/// (https://abr.business.gov.au/Help/AbnFormat, read 16 November 2026):
/// subtract 1 from the first digit, weight the 11 digits 10, 1, 3, 5, 7, 9, 11,
/// 13, 15, 17 and 19, and the sum must divide by 89. As in gst-review and the
/// MCP's `validate_abn`, only ASCII digits and spaces are accepted: stripping
/// anything else could turn a malformed ABN into a different, valid one.
fn abn_digits(text: &str) -> Option<String> {
    const WEIGHTS: [u32; 11] = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    if !text.bytes().all(|b| b.is_ascii_digit() || b == b' ') {
        return None;
    }
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    let values: Vec<u32> = digits.bytes().map(|b| u32::from(b - b'0')).collect();
    if values.len() != 11 || values[0] == 0 {
        return None;
    }
    let total: u32 = values.iter().zip(WEIGHTS).map(|(d, w)| d * w).sum();
    // Subtracting 1 from the first digit takes its weight off the sum once.
    (total - WEIGHTS[0]).is_multiple_of(89).then_some(digits)
}

/// A name's words for matching: upper case, split at anything not a letter or
/// digit, with `&` read as AND and LTD, INC and PTY as the words they shorten.
// shortcut: whole-word matching after these 4 rewrites, no fuzzy or phonetic
// match; a misspelt or reworded name finds nothing. A ranked fuzzy search would
// find more, at the cost of more candidates to rule out by hand.
fn name_words(name: &str) -> Vec<String> {
    let spaced = name.to_ascii_uppercase().replace('&', " AND ");
    let mut words: Vec<String> = spaced
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| match w {
            "LTD" => "LIMITED",
            "INC" => "INCORPORATED",
            "PTY" => "PROPRIETARY",
            other => other,
        })
        .map(str::to_string)
        .collect();
    words.sort();
    words.dedup();
    words
}

fn spaced_abn(abn: &str) -> String {
    format!("{} {} {} {}", &abn[..2], &abn[2..5], &abn[5..8], &abn[8..])
}

/// A YYYY-MM-DD date in words, as the ATO writes one: 1 July 2024.
fn long_date(date: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let (y, m, d) = dates::to_civil(dates::parse(date).expect("a checked date"));
    format!("{d} {} {y}", MONTHS[(m - 1) as usize])
}

/// One donation's inputs, checked.
struct Donation {
    abn: Option<String>,
    name: Option<String>,
    date: Option<String>,
    amount: Option<Decimal>,
}

fn read_donation(at: usize, value: &Value) -> Result<Donation, Refusal> {
    let field = |key: &str| format!("donations[{at}].{key}");
    let Some(object) = value.as_object() else {
        return Err(Refusal::new(
            "input.donations_invalid",
            format!("donations[{at}] must be an object"),
        ));
    };
    if let Some(key) = object.keys().find(|k| !FIELDS.contains(&k.as_str())) {
        return Err(Refusal::new(
            "input.field_unknown",
            format!("{} is not an input of {COMMAND}", field(key)),
        ));
    }
    let text = |key: &str, code: &'static str, why: &str| match object.get(key) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(Refusal::new(code, format!("{} {why}", field(key)))),
    };
    let abn = text("abn", "input.abn_invalid", "must be a string")?;
    let abn = match abn {
        None => None,
        Some(given) => Some(abn_digits(&given).ok_or_else(|| {
            Refusal::new(
                "input.abn_invalid",
                format!(
                    "{} {given:?} is not 11 digits, optionally spaced, that pass the ABN check digit test ABN Lookup publishes",
                    field("abn")
                ),
            )
        })?),
    };
    let why = "must be a string of 1 to 200 characters with an ASCII letter or digit";
    let name = text("name", "input.name_invalid", why)?;
    if let Some(name) = &name
        && (name.chars().count() > 200 || !name.chars().any(|c| c.is_ascii_alphanumeric()))
    {
        return Err(Refusal::new(
            "input.name_invalid",
            format!("{} {why}", field("name")),
        ));
    }
    if abn.is_none() && name.is_none() {
        return Err(Refusal::new(
            "input.missing_field",
            format!("donations[{at}] needs an abn or a name"),
        ));
    }
    let date = text(
        "date",
        "input.date_invalid",
        "must be a YYYY-MM-DD date from 1900 to 2100",
    )?;
    if let Some(date) = &date
        && dates::parse(date).is_none()
    {
        return Err(Refusal::new(
            "input.date_invalid",
            format!(
                "{} must be a YYYY-MM-DD date from 1900 to 2100",
                field("date")
            ),
        ));
    }
    let prefix = |mut refusal: Refusal| {
        refusal.message = format!("{}: {}", field("amount"), refusal.message);
        refusal
    };
    let amount = match optional_decimal(value, "amount").map_err(prefix)? {
        None => None,
        Some(arg) => {
            Some(engine::money(&arg).map_err(|e| prefix(Refusal::new(e.code(), e.to_string())))?)
        }
    };
    Ok(Donation {
        abn,
        name,
        date,
        amount,
    })
}

/// A condition the data cannot settle, or one the inputs settle.
fn condition(id: &str, status: &str, text: String, source: &str) -> Value {
    json!({"condition": id, "status": status, "text": text, "source": source})
}

/// The conditions of Division 30 that stay with the reviewer for this donation.
fn conditions(donation: &Donation, fund_only: &[&str]) -> Result<Vec<Value>, Refusal> {
    let mut out = vec![
        condition(
            "a_gift",
            "open",
            "The payment is a gift: a voluntary transfer of money with no material benefit or advantage in return. A payment for a ticket to a fund-raising event or for an item won at a charity auction is a contribution under s 30-15 items 7 or 8, with other conditions; one made under a will is not deductible (s 30-15(2)).".into(),
            ATO_GIFTS,
        ),
        condition(
            "gift_conditions",
            "open",
            "The special conditions of the s 30-15 item that applies are met. Under item 1 the fund, authority or institution must be in Australia and any condition of its table item in Subdivision 30-B, such as a gift being for a stated purpose, must be met (special conditions (a) and (c)); under item 2 the giving fund's will or trust must let the trustee invest money from the gift only as an Australian law allows trustees to invest trust money (special condition (b)); other items set their own. ABN Lookup shows the recipient's DGR item.".into(),
            ITAA_1997,
        ),
        condition(
            "record",
            "open",
            "A record of the gift is kept, such as a receipt; a DGR's receipt states the name of the fund, authority or institution, the DGR's ABN (if any) and that it is for a gift (s 30-228(1)).".into(),
            ATO_GIFTS,
        ),
    ];
    if !fund_only.is_empty() {
        out.push(condition(
            "gift_to_the_fund",
            "open",
            format!(
                "The entity is endorsed only for the operation of a fund, authority or institution, so the gift must have been made to it: {} (s 30-17(2)(b) and (c), s 30-229(2)).",
                fund_only.join("; ")
            ),
            ITAA_1997,
        ));
    }
    let before_cutover = donation.date.as_deref().map(|d| d < NO_MINIMUM_FROM);
    if before_cutover != Some(false) {
        let two = engine::d("2");
        let status = match (before_cutover, &donation.amount) {
            (Some(true), Some(amount)) if amount.lt(&two).map_err(signal)? => "fails",
            (Some(true), Some(_)) => "met",
            _ => "open",
        };
        out.push(condition(
            "two_dollar_minimum",
            status,
            "A gift made before 1 July 2024 must be $2 or more; the minimum does not apply to a gift made on or after that date (Act No. 58, 2026, Sch 1 item 7).".into(),
            ACT_58_2026,
        ));
    }
    Ok(out)
}

fn signal(s: crate::decimal::Signal) -> Refusal {
    let error = engine::Error::from(s);
    Refusal::new(error.code(), error.to_string())
}

/// What the data shows about a recipient on the payment date.
fn on_date(entry: &Entry, date: Option<&str>, data_date: &str) -> (&'static str, String) {
    let Some(date) = date else {
        return (
            "no_date",
            "No payment date was given, so the data was not read for one.".into(),
        );
    };
    if date > data_date {
        return (
            "after_data_date",
            format!(
                "The payment on {} is after the data date, {}; ABN Lookup shows the current position.",
                long_date(date),
                long_date(data_date)
            ),
        );
    }
    if entry.cancelled && entry.status_from.as_str() <= date {
        return (
            "abn_cancelled",
            format!(
                "The data shows the ABN cancelled from {}, on or before the payment; it cannot show whether the DGR entries held after that. Check ABN Lookup's history for the ABN.",
                long_date(&entry.status_from)
            ),
        );
    }
    let covering: Vec<&(String, Option<String>)> = entry
        .dgr
        .iter()
        .filter(|(from, _)| from.as_str() <= date)
        .collect();
    if covering.iter().any(|(_, fund)| fund.is_none()) {
        return (
            "shown",
            "The data shows the entity as a DGR in its own right from on or before the payment date.".into(),
        );
    }
    if !covering.is_empty() {
        return (
            "shown_for_a_fund",
            "The data shows the entity endorsed on the payment date only for the operation of a fund, authority or institution; it cannot show an entry for the entity itself that has since ended.".into(),
        );
    }
    let first = entry
        .dgr
        .iter()
        .map(|(from, _)| from)
        .min()
        .expect("an entry");
    (
        "not_shown",
        format!(
            "The data shows DGR entries only from {}, after the payment. It cannot show an earlier endorsement that has since ended; check ABN Lookup's history for the ABN.",
            long_date(first)
        ),
    )
}

/// A fund's name as shown, or what stands for it where the extract has a
/// fund entry with no name.
fn fund_name(name: &str) -> &str {
    if name.is_empty() {
        "a fund, authority or institution the data does not name"
    } else {
        name
    }
}

fn dgr_entries(entry: &Entry) -> Value {
    entry
        .dgr
        .iter()
        .map(|(from, fund)| {
            let shown = fund.as_deref().map_or("the entity itself", fund_name);
            json!({"from": from, "entry_for": shown})
        })
        .collect()
}

fn entity(entry: &Entry) -> Value {
    json!({
        "abn": entry.abn,
        "name": entry.names[0],
        "abn_status": if entry.cancelled { "cancelled" } else { "active" },
        "abn_status_from": entry.status_from,
    })
}

/// The funds an entity is endorsed for when it is not endorsed in its own right
/// on the date (or at all, with no date). Empty when no entry held on the
/// date: an endorsement that has since ended is not in the data, so the funds
/// listed now cannot say where such a gift had to go. Read only for a
/// `shown_for_a_fund` or `no_date` reading.
fn fund_only<'a>(entry: &'a Entry, date: Option<&str>) -> Vec<&'a str> {
    let held = |from: &String| date.is_none_or(|d| from.as_str() <= d);
    if entry
        .dgr
        .iter()
        .any(|(from, fund)| fund.is_none() && held(from))
    {
        return Vec::new();
    }
    let funds = entry.dgr.iter().filter(|(from, _)| held(from));
    funds
        .filter_map(|(_, fund)| fund.as_deref().map(fund_name))
        .collect()
}

fn by_abn(
    register: &Register,
    abn: &str,
    donation: &Donation,
) -> Result<Map<String, Value>, Refusal> {
    let data_date = &register.data_date;
    let mut out = Map::new();
    out.insert("abn".into(), json!(abn));
    let found = register
        .entries
        .binary_search_by(|e| e.abn.as_str().cmp(abn));
    let Ok(at) = found else {
        out.insert("register".into(), json!("not_shown_as_dgr"));
        out.insert(
            "finding".into(),
            json!(format!(
                "ABN {} is not shown as a deductible gift recipient in the data of {}. That does not establish that it is not one: check ABN Lookup ({ABN_LOOKUP_DGR}).",
                spaced_abn(abn),
                long_date(data_date)
            )),
        );
        out.insert("conditions".into(), json!(conditions(donation, &[])?));
        return Ok(out);
    };
    let entry = &register.entries[at];
    let (status, reading) = on_date(entry, donation.date.as_deref(), data_date);
    let first = entry
        .dgr
        .iter()
        .map(|(from, _)| from)
        .min()
        .expect("an entry");
    out.insert("register".into(), json!("shown_as_dgr"));
    out.insert(
        "finding".into(),
        json!(format!(
            "ABN {} ({}) is shown as a deductible gift recipient in the data of {}, with entries from {}. {reading}",
            spaced_abn(abn),
            entry.names[0],
            long_date(data_date),
            long_date(first)
        )),
    );
    if let Some(name) = &donation.name {
        let given = name_words(name);
        let matched = entry.names.iter().any(|n| {
            let words = name_words(n);
            given.iter().all(|w| words.contains(w))
        });
        out.insert("name_matches_register".into(), json!(matched));
    }
    out.insert("entity".into(), entity(entry));
    out.insert(
        "other_names".into(),
        json!(entry.names[1..=entry.other_names]),
    );
    out.insert("dgr_entries".into(), dgr_entries(entry));
    out.insert("on_payment_date".into(), json!(status));
    // A cancelled ABN or a payment after the data date leaves the entries on
    // the payment date unknown, so no fund is named for them.
    let funds = match status {
        "shown_for_a_fund" | "no_date" => fund_only(entry, donation.date.as_deref()),
        _ => Vec::new(),
    };
    out.insert("conditions".into(), json!(conditions(donation, &funds)?));
    Ok(out)
}

fn by_name(
    register: &Register,
    name: &str,
    donation: &Donation,
) -> Result<Map<String, Value>, Refusal> {
    let mut words = name_words(name);
    words.sort_by_key(|w| register.words.get(w).map_or(0, Vec::len));
    let mut hits: Vec<(usize, usize)> = register.words.get(&words[0]).cloned().unwrap_or_default();
    for word in &words[1..] {
        let postings = register.words.get(word).map_or(&[][..], Vec::as_slice);
        hits.retain(|hit| postings.binary_search(hit).is_ok());
    }
    hits.dedup_by_key(|(entry, _)| *entry);
    let candidates: Vec<Value> = hits
        .iter()
        .take(MAX_CANDIDATES)
        .map(|&(at, name_at)| {
            let entry = &register.entries[at];
            json!({
                "status": "unconfirmed",
                "matched_name": entry.names[name_at],
                "entity": entity(entry),
                "dgr_entries": dgr_entries(entry),
            })
        })
        .collect();
    let data_date = long_date(&register.data_date);
    let finding = if hits.is_empty() {
        format!(
            "No deductible gift recipient in the data of {data_date} has a name with every word of {name:?}. That does not establish that the payee is not one: it may be recorded under another name. Find the ABN on the receipt."
        )
    } else {
        format!(
            "{} deductible gift recipients in the data of {data_date} have a name with every word of {name:?}. Each is a candidate only: the ABN on the receipt settles which entity was paid.",
            hits.len()
        )
    };
    let mut out = Map::new();
    out.insert("abn".into(), Value::Null);
    out.insert("register".into(), json!("abn_needed"));
    out.insert("finding".into(), json!(finding));
    out.insert("candidates_total".into(), json!(hits.len()));
    out.insert("candidates".into(), json!(candidates));
    out.insert("conditions".into(), json!(conditions(donation, &[])?));
    Ok(out)
}

/// `dgr-check`: `{"donations": [{"abn", "name", "date", "amount"}, ...]}`, each
/// with an `abn` or a `name`, gives each donation's reading of the data, in order.
pub fn check(v: &Value) -> Result<Value, Refusal> {
    check_with(register(), v)
}

fn check_with(register: &Register, v: &Value) -> Result<Value, Refusal> {
    if let Some(key) = v
        .as_object()
        .into_iter()
        .flat_map(|o| o.keys())
        .find(|k| *k != "donations")
    {
        return Err(Refusal::new(
            "input.field_unknown",
            format!("{key} is not an input of {COMMAND}"),
        ));
    }
    let Some(donations) = v.get("donations") else {
        return Err(Refusal::new("input.missing_field", "donations is required"));
    };
    let donations = match donations.as_array() {
        Some(list) if (1..=MAX_DONATIONS).contains(&list.len()) => list,
        _ => {
            return Err(Refusal::new(
                "input.donations_invalid",
                format!("donations must be a list of 1 to {MAX_DONATIONS} donations"),
            ));
        }
    };
    let read: Vec<Donation> = donations
        .iter()
        .enumerate()
        .map(|(at, d)| read_donation(at, d))
        .collect::<Result<_, _>>()?;
    let mut results = Vec::new();
    for (at, (donation, given)) in read.iter().zip(donations).enumerate() {
        let mut result = match (&donation.abn, &donation.name) {
            (Some(abn), _) => by_abn(register, abn, donation)?,
            (None, Some(name)) => by_name(register, name, donation)?,
            (None, None) => unreachable!("read_donation refuses it"),
        };
        result.insert("index".into(), json!(at));
        result.insert("input".into(), given.clone());
        // Each donation carries what it rests on, so it can be read alone.
        result.insert("data_date".into(), json!(register.data_date));
        result.insert("source_url".into(), json!(BULK_EXTRACT));
        result.insert("limits".into(), json!(LIMITS));
        results.push(Value::Object(result));
    }
    Ok(json!({
        "ok": true,
        "engine": "australian-tax-calculators",
        "engine_version": engine::VERSION,
        "calculation": "dgr_check",
        "data": {
            "source": register.header["source"],
            "source_url": BULK_EXTRACT,
            "extract_time": register.header["extract_time"],
            "data_date": register.data_date,
            "licence": "Creative Commons Attribution 3.0 Australia",
            "attribution": "Contains deductible gift recipient entries from the ABN Bulk Extract, Australian Business Register, licensed under CC BY 3.0 AU.",
            "abns_with_dgr_entries": register.entries.len(),
        },
        "donations": results,
        "scope": SCOPE,
        "limits": LIMITS,
        "sources": SOURCES,
        "source_checked": "2026-11-16",
        "warnings": [
            "DGR status is one condition of a deduction for a gift; this result never calls a payment deductible.",
            "Worksheet only, not advice, an assessment or a lodgment. Obtain human review.",
        ],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The fabricated register's ABNs, kept apart from their labels so the
    // repository's ABN secret scan does not read them as real ones.
    const SCHOOL: &str = "23000000004";
    const SCHOOL_SPACED: &str = "23 000 000 004";
    const CLOSED: &str = "42000000003";
    const RIVER: &str = "61000000002";
    const RED: &str = "80000000001";
    /// Endorsed for a fund the data does not name, then in its own right.
    const HARBOUR: &str = "74000000006";
    /// Passes the check digit test but is not in the register.
    const ABSENT: &str = "93000000005";
    const BAD_CHECK_DIGIT: &str = "23000000005";
    const DASHED: &str = "23-000-000-004";
    const TWELVE_DIGITS: &str = "023000000004";

    /// A fabricated register: ABNs that pass the check digit, with entries
    /// chosen to reach each reading.
    const REGISTER: &str = r#"{"source":"fabricated","extract_time":"2026-11-11T12:20:05","abns":5}
["23000000004","ACT","2000-02-24","EXAMPLE SCHOOL LTD",[["2000-07-01","EXAMPLE SCHOOL BUILDING FUND"],["2010-07-01",null]],["EXAMPLE COLLEGE"]]
["42000000003","CAN","2020-01-01","CLOSED CHARITY LIMITED",[["2001-01-01",null]],[]]
["61000000002","ACT","2015-01-01","RIVER CARE ASSOCIATION INC",[["2015-03-01","RIVER CARE FUND"],["2016-03-01","RIVER CARE LIBRARY"]],[]]
["74000000006","ACT","2000-01-01","HARBOUR AID LTD",[["2000-07-01",""],["2018-01-01",null]],[]]
["80000000001","ACT","2000-01-01","RED EXAMPLE CROSS SOCIETY",[["2000-07-01",null]],["RED CROSS SHOPS"]]"#;

    fn run(donations: Value) -> Result<Value, Refusal> {
        check_with(&parse_register(REGISTER), &json!({"donations": donations}))
    }

    fn one(donation: Value) -> Value {
        run(json!([donation])).unwrap()["donations"][0].clone()
    }

    fn refused(donations: Value) -> (&'static str, String) {
        let r = run(donations).unwrap_err();
        (r.code, r.message)
    }

    fn found<'a>(result: &'a Value, id: &str) -> Option<&'a Value> {
        let conditions = result["conditions"].as_array().unwrap();
        conditions.iter().find(|c| c["condition"] == id)
    }

    fn status(result: &Value, id: &str) -> Option<String> {
        found(result, id).map(|c| c["status"].as_str().unwrap().to_string())
    }

    #[test]
    fn an_entity_endorsed_in_its_own_right_is_shown_on_a_date_inside_its_entry() {
        let r = one(json!({"abn": SCHOOL_SPACED, "date": "2012-05-01", "amount": "50.00"}));
        assert_eq!(r["register"], "shown_as_dgr");
        assert_eq!(r["on_payment_date"], "shown");
        assert_eq!(r["entity"]["name"], "EXAMPLE SCHOOL LTD");
        assert_eq!(r["other_names"], json!(["EXAMPLE COLLEGE"]));
        assert_eq!(status(&r, "gift_to_the_fund"), None);
        assert_eq!(status(&r, "two_dollar_minimum").as_deref(), Some("met"));
        assert_eq!(status(&r, "a_gift").as_deref(), Some("open"));
        let finding = r["finding"].as_str().unwrap();
        assert!(
            finding.starts_with(&format!("ABN {SCHOOL_SPACED} (EXAMPLE SCHOOL LTD) is shown as a deductible gift recipient in the data of 11 November 2026, with entries from 1 July 2000.")),
            "{finding}"
        );
    }

    #[test]
    fn before_the_entity_entry_only_its_fund_is_shown_and_the_gift_must_go_to_it() {
        let r = one(json!({"abn": SCHOOL, "date": "2005-01-01"}));
        assert_eq!(r["on_payment_date"], "shown_for_a_fund");
        let fund = found(&r, "gift_to_the_fund").unwrap()["text"]
            .as_str()
            .unwrap();
        assert!(fund.contains("EXAMPLE SCHOOL BUILDING FUND"), "{fund}");
    }

    #[test]
    fn a_date_before_every_entry_is_not_shown_and_a_later_one_is_after_the_data() {
        let early = one(json!({"abn": SCHOOL, "date": "2000-06-30"}));
        assert_eq!(early["on_payment_date"], "not_shown");
        assert_eq!(early["register"], "shown_as_dgr");
        let finding = early["finding"].as_str().unwrap();
        assert!(
            finding.contains("only from 1 July 2000, after the payment"),
            "{finding}"
        );
        let late = one(json!({"abn": SCHOOL, "date": "2026-11-12"}));
        assert_eq!(late["on_payment_date"], "after_data_date");
        let on = one(json!({"abn": SCHOOL, "date": "2026-11-11"}));
        assert_eq!(on["on_payment_date"], "shown");
        let first_day = one(json!({"abn": SCHOOL, "date": "2000-07-01"}));
        assert_eq!(first_day["on_payment_date"], "shown_for_a_fund");
    }

    #[test]
    fn a_cancelled_abn_is_not_read_as_a_dgr_from_its_cancellation() {
        let after = one(json!({"abn": CLOSED, "date": "2020-01-01"}));
        assert_eq!(after["on_payment_date"], "abn_cancelled");
        assert_eq!(after["entity"]["abn_status"], "cancelled");
        let before = one(json!({"abn": CLOSED, "date": "2019-12-31"}));
        assert_eq!(before["on_payment_date"], "shown");
    }

    #[test]
    fn a_fund_only_entity_names_the_funds_held_on_the_date() {
        let r = one(json!({"abn": RIVER}));
        assert_eq!(r["on_payment_date"], "no_date");
        let text = found(&r, "gift_to_the_fund").unwrap()["text"]
            .as_str()
            .unwrap();
        assert!(
            text.contains("RIVER CARE FUND; RIVER CARE LIBRARY"),
            "{text}"
        );
        let r = one(json!({"abn": RIVER, "date": "2015-06-01"}));
        let text = found(&r, "gift_to_the_fund").unwrap()["text"]
            .as_str()
            .unwrap();
        assert!(
            text.contains("RIVER CARE FUND") && !text.contains("LIBRARY"),
            "{text}"
        );
        // Before either fund the data shows no entry, and an ended one could
        // have been for the entity itself, so no fund is named.
        let r = one(json!({"abn": RIVER, "date": "2015-01-01"}));
        assert_eq!(r["on_payment_date"], "not_shown");
        assert_eq!(found(&r, "gift_to_the_fund"), None);
        assert_eq!(status(&r, "two_dollar_minimum").as_deref(), Some("open"));
        // After the data date the entries on the payment date are unknown too.
        let r = one(json!({"abn": RIVER, "date": "2026-11-12"}));
        assert_eq!(r["on_payment_date"], "after_data_date");
        assert_eq!(found(&r, "gift_to_the_fund"), None);
    }

    #[test]
    fn a_fund_the_extract_does_not_name_is_still_a_fund() {
        let r = one(json!({"abn": HARBOUR, "date": "2010-01-01"}));
        assert_eq!(r["on_payment_date"], "shown_for_a_fund");
        let unnamed = "a fund, authority or institution the data does not name";
        assert_eq!(r["dgr_entries"][0]["entry_for"], unnamed);
        let text = found(&r, "gift_to_the_fund").unwrap()["text"]
            .as_str()
            .unwrap();
        assert!(
            text.ends_with(&format!(
                "made to it: {unnamed} (s 30-17(2)(b) and (c), s 30-229(2))."
            )),
            "{text}"
        );
        let later = one(json!({"abn": HARBOUR, "date": "2018-01-01"}));
        assert_eq!(later["on_payment_date"], "shown");
        assert_eq!(found(&later, "gift_to_the_fund"), None);
    }

    #[test]
    fn an_abn_not_in_the_data_is_not_shown_never_not_a_dgr() {
        let r = one(json!({"abn": ABSENT, "date": "2025-01-01", "amount": "1.00"}));
        assert_eq!(r["register"], "not_shown_as_dgr");
        let finding = r["finding"].as_str().unwrap();
        assert!(
            finding.starts_with(&format!("ABN {} is not shown as a deductible gift recipient in the data of 11 November 2026. That does not establish that it is not one", spaced_abn(ABSENT))),
            "{finding}"
        );
        assert!(r.get("entity").is_none());
        // A gift from 1 July 2024 has no minimum, so no $2 condition at all.
        assert_eq!(status(&r, "two_dollar_minimum"), None);
    }

    #[test]
    fn the_two_dollar_minimum_applies_to_gifts_before_1_july_2024_only() {
        let minimum = |date: &str, amount: &str| {
            let r = one(json!({"abn": RED, "date": date, "amount": amount}));
            status(&r, "two_dollar_minimum")
        };
        assert_eq!(minimum("2024-06-30", "1.99").as_deref(), Some("fails"));
        assert_eq!(minimum("2024-06-30", "2.00").as_deref(), Some("met"));
        assert_eq!(minimum("2024-06-30", "2").as_deref(), Some("met"));
        assert_eq!(minimum("2024-07-01", "1.99"), None);
        let undated = one(json!({"abn": RED, "amount": "1.00"}));
        assert_eq!(
            status(&undated, "two_dollar_minimum").as_deref(),
            Some("open")
        );
        let no_amount = one(json!({"abn": RED, "date": "2020-01-01"}));
        assert_eq!(
            status(&no_amount, "two_dollar_minimum").as_deref(),
            Some("open")
        );
    }

    #[test]
    fn a_name_gives_unconfirmed_candidates_and_never_a_reading() {
        let r = one(json!({"name": "Example School Ltd", "date": "2025-01-01"}));
        assert_eq!(r["register"], "abn_needed");
        assert_eq!(r["candidates_total"], 1);
        let candidate = &r["candidates"][0];
        assert_eq!(candidate["status"], "unconfirmed");
        assert_eq!(candidate["entity"]["abn"], SCHOOL);
        assert_eq!(candidate["matched_name"], "EXAMPLE SCHOOL LTD");
        assert!(r.get("on_payment_date").is_none());
        // Fund names and other names match too, by whole words.
        assert_eq!(
            one(json!({"name": "river care library"}))["candidates_total"],
            1
        );
        assert_eq!(one(json!({"name": "Red Cross"}))["candidates_total"], 1);
        assert_eq!(one(json!({"name": "Red Cros"}))["candidates_total"], 0);
        assert_eq!(one(json!({"name": "example"}))["candidates_total"], 2);
        let none = one(json!({"name": "Nobody Here"}));
        assert_eq!(none["candidates_total"], 0);
        let finding = none["finding"].as_str().unwrap();
        assert!(
            finding.contains("That does not establish that the payee is not one"),
            "{finding}"
        );
    }

    #[test]
    fn candidates_stop_at_the_cap_but_the_total_counts_every_match() {
        let mut text = String::from(
            r#"{"source":"fabricated","extract_time":"2026-11-11T12:20:05","abns":12}"#,
        );
        let mut abns = Vec::new();
        for body in 0..10_000_000u32 {
            let base = format!("{body:09}");
            if let Some(abn) = (10..100)
                .map(|c| format!("{c}{base}"))
                .find(|a| abn_digits(a).is_some())
            {
                abns.push(abn);
            }
            if abns.len() == 12 {
                break;
            }
        }
        abns.sort();
        for abn in &abns {
            text.push_str(&format!(
                "\n[\"{abn}\",\"ACT\",\"2000-01-01\",\"SAME NAME\",[[\"2000-07-01\",null]],[]]"
            ));
        }
        let r = check_with(
            &parse_register(&text),
            &json!({"donations": [{"name": "same name"}]}),
        )
        .unwrap();
        assert_eq!(r["donations"][0]["candidates_total"], 12);
        assert_eq!(
            r["donations"][0]["candidates"].as_array().unwrap().len(),
            MAX_CANDIDATES
        );
    }

    #[test]
    fn a_name_beside_an_abn_is_compared_with_the_register() {
        let r = one(json!({"abn": RED, "name": "Red Cross Shops"}));
        assert_eq!(r["name_matches_register"], true);
        let r = one(json!({"abn": RED, "name": "Blue Cross"}));
        assert_eq!(r["name_matches_register"], false);
        assert_eq!(r["register"], "shown_as_dgr");
    }

    #[test]
    fn results_keep_the_input_order_and_never_say_deductible() {
        let donations = json!([{"abn": ABSENT}, {"name": "river"}, {"abn": RED}]);
        let r = run(donations).unwrap();
        let donations = r["donations"].as_array().unwrap();
        let order: Vec<&Value> = donations.iter().map(|d| &d["index"]).collect();
        assert_eq!(order, [0, 1, 2]);
        assert_eq!(donations[1]["input"], json!({"name": "river"}));
        assert_eq!(r["data"]["data_date"], "2026-11-11");
        assert_eq!(r["data"]["abns_with_dgr_entries"], 5);
        let text = r.to_string();
        assert!(!text.contains("\"deductible\"") && !text.contains("is deductible"));
    }

    #[test]
    fn malformed_input_is_refused() {
        let cases = [
            (json!({"donations": []}), "input.donations_invalid"),
            (json!({"donations": {}}), "input.donations_invalid"),
            (json!({}), "input.missing_field"),
            (
                json!({"donations": [{}], "year": "2025-26"}),
                "input.field_unknown",
            ),
        ];
        for (input, code) in cases {
            let r = check_with(&parse_register(REGISTER), &input).unwrap_err();
            assert_eq!(r.code, code, "{input}");
        }
        let too_many = vec![json!({"abn": SCHOOL}); MAX_DONATIONS + 1];
        assert_eq!(refused(json!(too_many)).0, "input.donations_invalid");
    }

    #[test]
    fn a_malformed_donation_is_refused_naming_it() {
        let long = "x".repeat(201);
        let cases = [
            (
                json!(["23000000004"]),
                "input.donations_invalid",
                "donations[0] must be an object",
            ),
            (
                json!([{}]),
                "input.missing_field",
                "donations[0] needs an abn or a name",
            ),
            (
                json!([{"abn": SCHOOL}, {"abn": BAD_CHECK_DIGIT}]),
                "input.abn_invalid",
                "donations[1].abn \"",
            ),
            (
                json!([{"abn": DASHED}]),
                "input.abn_invalid",
                "donations[0].abn",
            ),
            (
                json!([{"abn": TWELVE_DIGITS}]),
                "input.abn_invalid",
                "donations[0].abn",
            ),
            (
                json!([{"abn": 23000000004_u64}]),
                "input.abn_invalid",
                "donations[0].abn must be a string",
            ),
            (
                json!([{"name": " - "}]),
                "input.name_invalid",
                "donations[0].name",
            ),
            // name_words reads & as AND; the name itself has no letter or digit.
            (
                json!([{"name": "&"}]),
                "input.name_invalid",
                "donations[0].name",
            ),
            (
                json!([{"abn": SCHOOL, "name": "&"}]),
                "input.name_invalid",
                "donations[0].name",
            ),
            (
                json!([{"name": long}]),
                "input.name_invalid",
                "donations[0].name",
            ),
            (
                json!([{"name": 7}]),
                "input.name_invalid",
                "donations[0].name",
            ),
            (
                json!([{"abn": SCHOOL, "date": "2025-02-30"}]),
                "input.date_invalid",
                "donations[0].date",
            ),
            (
                json!([{"abn": SCHOOL, "date": 20250101}]),
                "input.date_invalid",
                "donations[0].date",
            ),
            (
                json!([{"abn": SCHOOL, "amount": "-5.00"}]),
                "input.amount_invalid",
                "donations[0].amount: ",
            ),
            (
                json!([{"abn": SCHOOL, "amount": "5.001"}]),
                "input.amount_invalid",
                "donations[0].amount: ",
            ),
            (
                json!([{"abn": SCHOOL, "amount": "five"}]),
                "input.decimal_unreadable",
                "donations[0].amount: ",
            ),
            (
                json!([{"abn": SCHOOL, "amount": 5}]),
                "input.amount_invalid",
                "donations[0].amount: ",
            ),
            (
                json!([{"abn": SCHOOL, "ref": "INV-1"}]),
                "input.field_unknown",
                "donations[0].ref is not an input of dgr-check",
            ),
        ];
        for (donations, code, message) in cases {
            let (got, text) = refused(donations.clone());
            assert_eq!(got, code, "{donations}");
            assert!(text.starts_with(message), "{donations}: {text}");
        }
    }

    #[test]
    fn the_abn_check_follows_abn_lookups_worked_example() {
        // https://abr.business.gov.au/Help/AbnFormat (version 9.9.7, read 16
        // November 2026): 51 824 753 556 gives 534, which divides by 89.
        assert_eq!(abn_digits("51 824 753 556").as_deref(), Some("51824753556"));
        assert_eq!(abn_digits("51824753557"), None);
        assert_eq!(abn_digits("5182475355"), None);
        assert_eq!(abn_digits("51\u{a0}824\u{a0}753\u{a0}556"), None);
    }

    #[test]
    fn name_words_read_abbreviations_and_ampersands_as_words() {
        let words = name_words("Smith & Co Pty Ltd");
        assert_eq!(words, ["AND", "CO", "LIMITED", "PROPRIETARY", "SMITH"]);
        assert_eq!(
            name_words("The Example Inc."),
            ["EXAMPLE", "INCORPORATED", "THE"]
        );
        assert_eq!(long_date("2024-07-01"), "1 July 2024");
    }
}
