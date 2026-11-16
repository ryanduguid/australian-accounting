// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `dgr-check`: whether the recipient of each donation is shown as a
//! deductible gift recipient (DGR) in one week's ABN Bulk Extract, from what
//! date, and which ITAA 1997 Division 30 conditions the data cannot settle.
//! It never says a payment is deductible: DGR status is one condition of
//! several. The data is `data/dgr-register.jsonl`, built from the extract by
//! `examples/dgr_register.rs`; its first line records the extract it came from.
use crate::api::Refusal;
use crate::dates;
use crate::engine;
use serde_json::{Map, Value, json};

mod input;
mod register;
use input::{Donation, read_donation};
use register::{Entry, Register, name_words, register};

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
    held_on(entry, date)
}

/// What the entries that started on or before a payment date, inside the
/// data and the ABN's active life, show for it.
fn held_on(entry: &Entry, date: &str) -> (&'static str, String) {
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
    let found = register
        .entries
        .binary_search_by(|e| e.abn.as_str().cmp(abn));
    let Ok(at) = found else {
        return not_in_data(abn, data_date, donation);
    };
    let entry = &register.entries[at];
    let mut out = Map::new();
    out.insert("abn".into(), json!(abn));
    let (status, reading) = on_date(entry, donation.date.as_deref(), data_date);
    out.insert("register".into(), json!("shown_as_dgr"));
    out.insert("finding".into(), json!(shown(entry, data_date, &reading)));
    if let Some(name) = &donation.name {
        out.insert(
            "name_matches_register".into(),
            json!(name_matches(entry, name)),
        );
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

/// The finding for an ABN with DGR entries, ending with the payment date's reading.
fn shown(entry: &Entry, data_date: &str, reading: &str) -> String {
    let first = entry
        .dgr
        .iter()
        .map(|(from, _)| from)
        .min()
        .expect("an entry");
    format!(
        "ABN {} ({}) is shown as a deductible gift recipient in the data of {}, with entries from {}. {reading}",
        spaced_abn(&entry.abn),
        entry.names[0],
        long_date(data_date),
        long_date(first)
    )
}

/// The result for an ABN with no DGR entry in the data.
fn not_in_data(
    abn: &str,
    data_date: &str,
    donation: &Donation,
) -> Result<Map<String, Value>, Refusal> {
    let mut out = Map::new();
    out.insert("abn".into(), json!(abn));
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
    Ok(out)
}

/// Whether one of the entity's names holds every word of the ledger's name.
fn name_matches(entry: &Entry, name: &str) -> bool {
    let given = name_words(name);
    entry.names.iter().any(|n| {
        let words = name_words(n);
        given.iter().all(|w| words.contains(w))
    })
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
    let donations = donation_list(v)?;
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
    Ok(envelope(register, results))
}

/// The input's `donations`, the only field, as a list of 1 to the maximum.
fn donation_list(v: &Value) -> Result<&Vec<Value>, Refusal> {
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
    match donations.as_array() {
        Some(list) if (1..=MAX_DONATIONS).contains(&list.len()) => Ok(list),
        _ => Err(Refusal::new(
            "input.donations_invalid",
            format!("donations must be a list of 1 to {MAX_DONATIONS} donations"),
        )),
    }
}

/// The response: the data's provenance, the results, the scope and limits.
fn envelope(register: &Register, results: Vec<Value>) -> Value {
    json!({
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
    })
}

#[cfg(test)]
mod tests;
