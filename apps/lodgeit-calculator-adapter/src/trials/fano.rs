//! `trials/fano.py`: mapping suggestions, and nothing that follows from
//! them.
//!
//! Experimental and opt-in. Fano has a model in the path; what it returns is
//! a suggestion about which LodgeiT chart code a ledger line might belong to.
//! Nothing is applied, a code means nothing outside its own chart without a
//! reviewed crosswalk, and `accepted_fact` is the classifier's verdict on its
//! own prediction, not an approval.
use crate::client::{Client, Outcome, Status};
use crate::decimal::Decimal;
use crate::errors::CONTRACT;
use crate::py::repr_str;
use crate::pyjson::Json;
use crate::pyops::{Exc, R, S, decimal_signal, json_str, pts};
use crate::pyvalue::{repr, type_name};

pub const ROUTE_CALC_URI: &str = "urn:lodgeit:classifier:fano:trial-balance";
pub const PERIOD_URI: &str = "urn:lodgeit:period:fano:unscoped";
pub const ENTITY_STRUCTURES: [&str; 5] = [
    "company",
    "trust",
    "partnership",
    "sole_trader",
    "super_fund",
];
pub const TOPOLOGIES: [&str; 7] = [
    "current_assets",
    "non_current_assets",
    "current_liabilities",
    "non_current_liabilities",
    "equity",
    "revenue",
    "expenses",
];
pub const MAX_LINES: usize = 500;

/// One fabricated trial-balance line. `predicted_code` is `"sbrm_0000"` and
/// `confidence` `Decimal("0")` by default.
#[derive(Clone, Debug)]
pub struct Line {
    pub description: String,
    pub source_topology: String,
    pub amount: Decimal,
    pub predicted_code: String,
    pub confidence: Decimal,
}

/// `repr()` of a tuple or list of `str`.
fn repr_all(open: &str, names: &[&str], close: &str) -> String {
    let items: Vec<String> = names.iter().map(|name| repr_str(name)).collect();
    format!("{open}{}{close}", items.join(", "))
}

impl Line {
    /// `payload()`.
    pub fn payload(&self) -> R<Json> {
        if !TOPOLOGIES.contains(&self.source_topology.as_str()) {
            return Err(Exc::msg(
                CONTRACT,
                &format!(
                    "{} is not one of {}",
                    repr_str(&self.source_topology),
                    repr_all("(", &TOPOLOGIES, ")")
                ),
            ));
        }
        Ok(Json::obj([
            ("description", Json::str(&self.description)),
            ("source_topology", Json::str(&self.source_topology)),
            ("predicted_code", Json::str(&self.predicted_code)),
            ("confidence", Json::Dec(self.confidence.clone())),
            ("amount", Json::Dec(self.amount.clone())),
        ]))
    }
}

/// One line's suggestion, with everything needed to review it. The provider's
/// fields are kept as it sent them, whatever their type.
#[derive(Clone, Debug)]
pub struct Suggestion {
    pub description: String,
    pub supplied_topology: String,
    pub supplied_code: String,
    pub supplied_amount: Decimal,
    pub proposed_code: Json,
    pub confidence: Json,
    pub fano_status: Json,
    pub reason: Json,
    pub cascade_topology: Json,
    pub classifier: Json,
    pub findings: Vec<S>,
}

impl Suggestion {
    /// Every suggestion needs review. The property exists to be read.
    pub fn needs_review(&self) -> bool {
        true
    }

    /// `topology_disagrees`.
    pub fn topology_disagrees(&self) -> bool {
        self.cascade_topology.truthy()
            && self.cascade_topology.as_str() != Some(self.supplied_topology.as_str())
    }

    /// `to_json_dict()`.
    pub fn to_json(&self) -> Json {
        Json::obj([
            (
                "original",
                Json::obj([
                    ("description", Json::str(&self.description)),
                    ("topology", Json::str(&self.supplied_topology)),
                    ("code", Json::str(&self.supplied_code)),
                    ("amount", Json::str(&self.supplied_amount.fixed_format())),
                ]),
            ),
            (
                "suggestion",
                Json::obj([
                    ("proposed_code", self.proposed_code.clone()),
                    ("confidence", self.confidence.clone()),
                    ("fano_status", self.fano_status.clone()),
                    ("reason", self.reason.clone()),
                    ("cascade_topology", self.cascade_topology.clone()),
                    ("classifier", self.classifier.clone()),
                ]),
            ),
            (
                "review",
                Json::obj([
                    ("applied", Json::Bool(false)),
                    ("approved_by", Json::Null),
                    (
                        "topology_disagreement",
                        Json::Bool(self.topology_disagrees()),
                    ),
                    (
                        "findings",
                        Json::Array(self.findings.iter().cloned().map(json_str).collect()),
                    ),
                    (
                        "boundary",
                        Json::str(
                            "A suggestion from a model. accepted_fact is the classifier's \
                             verdict on its own prediction, not an approval, and it authorises \
                             no posting. A LodgeiT code needs a reviewed crosswalk before it \
                             means anything in another chart of accounts.",
                        ),
                    ),
                ]),
            ),
        ])
    }
}

/// `build_payload(entity_structure, lines)`: the request body, with the
/// provider's own limits checked first.
pub fn build_payload(entity_structure: &str, lines: &[Line]) -> R<Json> {
    if !ENTITY_STRUCTURES.contains(&entity_structure) {
        let mut sorted = ENTITY_STRUCTURES;
        sorted.sort_unstable();
        return Err(Exc::msg(
            CONTRACT,
            &format!(
                "entity_structure {} is not one of {}",
                repr_str(entity_structure),
                repr_all("[", &sorted, "]")
            ),
        ));
    }
    if lines.is_empty() {
        return Err(Exc::msg(
            CONTRACT,
            "a trial balance with no lines has nothing to classify",
        ));
    }
    if lines.len() > MAX_LINES {
        return Err(Exc::msg(
            CONTRACT,
            &format!(
                "{} lines exceeds the provider's limit of {MAX_LINES}; it answers 413",
                lines.len()
            ),
        ));
    }
    let zero = Decimal::from_int(0);
    let mut total = zero.clone();
    for line in lines {
        total = total.add(&line.amount).map_err(decimal_signal)?;
    }
    if !total.equals(&zero).map_err(decimal_signal)? {
        return Err(Exc::msg(
            CONTRACT,
            &format!(
                "the supplied lines net to {}, not nil. The provider refuses an unbalanced \
                 payload with a 400 on equilibrium, and a trial balance that does not balance is \
                 a bookkeeping problem, not a classification problem.",
                total.to_sci_string()
            ),
        ));
    }
    let payloads = lines.iter().map(Line::payload).collect::<R<Vec<Json>>>()?;
    Ok(Json::obj([
        ("entity_structure", Json::str(entity_structure)),
        ("lines", Json::Array(payloads)),
    ]))
}

/// `require_crosswalk(crosswalk, proposed_code)`: translate a LodgeiT code
/// into the local chart, or refuse. There is no default and no passthrough.
pub fn require_crosswalk(crosswalk: Option<&[(String, String)]>, proposed_code: &str) -> R<String> {
    let Some(crosswalk) = crosswalk.filter(|entries| !entries.is_empty()) else {
        return Err(Exc::msg(
            CONTRACT,
            "no reviewed crosswalk was supplied. A LodgeiT chart code is meaningless in another \
             chart of accounts until somebody maps it.",
        ));
    };
    // `dict(crosswalk)`: the last entry for a code is the one that stands.
    crosswalk
        .iter()
        .rfind(|(code, _)| code == proposed_code)
        .map(|(_, local)| local.clone())
        .ok_or_else(|| {
            Exc::msg(
                CONTRACT,
                &format!(
                    "{proposed_code} has no entry in the reviewed crosswalk. Add one, or leave \
                     the line for review."
                ),
            )
        })
}

/// What `classify` and `interpret` return: this trial's own status, the
/// suggestions and the notes.
pub type Interpreted = (String, Vec<Suggestion>, Vec<S>);

/// `classify(entity_structure, lines, client)`: send a fabricated trial
/// balance and return suggestions, never mappings.
pub fn classify(
    entity_structure: &str,
    lines: &[Line],
    client: Option<&mut Client<'_>>,
) -> R<Interpreted> {
    let payload = build_payload(entity_structure, lines)?;
    let Some(client) = client else {
        return Ok((
            "NOT_RUN".to_owned(),
            Vec::new(),
            vec![pts("No client supplied; the classifier was not called.")],
        ));
    };
    let outcome = client.invoke(&pts(ROUTE_CALC_URI), &pts(PERIOD_URI), &payload)?;
    Ok(interpret(lines, &outcome))
}

/// `interpret(lines, outcome)`: turn one response into suggestions.
pub fn interpret(lines: &[Line], outcome: &Outcome) -> Interpreted {
    let failure = |reason: String| {
        (
            "CONTRACT_FAILURE".to_owned(),
            Vec::new(),
            vec![pts(&reason)],
        )
    };
    if outcome.status != Status::Computed {
        return (
            outcome.status.as_str().to_owned(),
            Vec::new(),
            outcome.findings.clone(),
        );
    }
    let empty = Json::Object(Vec::new());
    let body = match &outcome.result {
        result @ Json::Object(_) => result,
        _ => &empty,
    };
    let Some(Json::Array(results)) = body.get("results") else {
        return failure("the response carries no results array".to_owned());
    };
    if results.len() != lines.len() {
        return failure(format!(
            "the provider returned {} results for {} lines; the rows cannot be matched up and \
             nothing is proposed",
            results.len(),
            lines.len()
        ));
    }
    if body.get("equilibrium_valid") == Some(&Json::Bool(false)) {
        return failure("the provider reports equilibrium_valid false".to_owned());
    }
    if let Some((index, result)) = results
        .iter()
        .enumerate()
        .find(|(_, result)| !matches!(result, Json::Object(_)))
    {
        return failure(format!(
            "the result at position {index} is {}, not an object, so there is nothing in it to \
             propose. A row that cannot be read is not a row with empty fields.",
            type_name(result)
        ));
    }

    let mut suggestions = Vec::new();
    for (line, result) in lines.iter().zip(results) {
        let mut findings = Vec::new();
        let echoed = result.at("description");
        if echoed.as_str() != Some(line.description.as_str()) {
            findings.push(pts(&format!(
                "the provider echoed description {} against the supplied {}; the rows may be out \
                 of order",
                repr(echoed),
                repr_str(&line.description)
            )));
        }
        let status = result.at("fano_status");
        if !matches!(status.as_str(), Some("accepted_fact" | "draft_fact")) && status != &Json::Null
        {
            findings.push(pts(&format!(
                "unknown fano_status {}; treat the line as unreviewed",
                repr(status)
            )));
        }
        suggestions.push(Suggestion {
            description: line.description.clone(),
            supplied_topology: line.source_topology.clone(),
            supplied_code: line.predicted_code.clone(),
            supplied_amount: line.amount.clone(),
            proposed_code: result.at("predicted_code").clone(),
            confidence: result.at("confidence").clone(),
            fano_status: status.clone(),
            reason: result.at("quarantine_reason").clone(),
            cascade_topology: result.at("cascade_topology").clone(),
            classifier: result.at("model_architecture").clone(),
            findings,
        });
    }
    // The first element stays in this trial's vocabulary; the provider's own
    // status is recorded, unaltered, as a note.
    let mut notes = outcome.findings.clone();
    notes.push(pts(&format!(
        "the provider's own status field read {}",
        repr(body.at("status"))
    )));
    notes.push(pts(
        "Every line above is a suggestion awaiting review. Nothing has been applied.",
    ));
    ("INTERPRETED".to_owned(), suggestions, notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(description: &str, topology: &str, amount: &str) -> Line {
        Line {
            description: description.to_owned(),
            source_topology: topology.to_owned(),
            amount: Decimal::parse_exact(amount).expect("a literal"),
            predicted_code: "sbrm_0000".to_owned(),
            confidence: Decimal::from_int(0),
        }
    }

    fn lines() -> Vec<Line> {
        vec![
            line("Trade debtors", "current_assets", "42000.00"),
            line("Sales", "revenue", "-42000.00"),
        ]
    }

    fn answered(result: Json) -> Outcome {
        Outcome {
            status: Status::Computed,
            calculator: None,
            period: None,
            request_body: None,
            request_json: None,
            raw_response: Json::Null,
            http_status: Some(200),
            result,
            values: Vec::new(),
            manifest: Json::Null,
            advisory: Json::Null,
            upstream_refusal_class: Json::Null,
            findings: Vec::new(),
            observed_at: String::new(),
            elapsed_ms: None,
            contract_snapshot: Json::Null,
            contract_sha256: String::new(),
            base_url: None,
        }
    }

    fn row(description: &str, status: &str, cascade: &str) -> Json {
        Json::obj([
            ("description", Json::str(description)),
            ("predicted_code", Json::str("sbrm_1200")),
            ("fano_status", Json::str(status)),
            ("cascade_topology", Json::str(cascade)),
        ])
    }

    #[test]
    fn a_trial_balance_that_does_not_balance_is_refused_locally() {
        let mut unbalanced = lines();
        unbalanced[1] = line("Sales", "revenue", "-41999.99");
        let error = build_payload("company", &unbalanced).expect_err("refused");
        assert_eq!(error.class, CONTRACT);
        assert!(
            String::from_iter(error.text.iter().filter_map(|&c| char::from_u32(c)))
                .starts_with("the supplied lines net to 0.01, not nil.")
        );
        let error = build_payload("smsf", &lines()).expect_err("refused");
        assert_eq!(
            error.text,
            pts(
                "entity_structure 'smsf' is not one of ['company', 'partnership', 'sole_trader', \
                 'super_fund', 'trust']"
            )
        );
        let unknown = [line("Bank account", "assets", "0.00")];
        assert_eq!(
            build_payload("company", &unknown)
                .expect_err("refused")
                .class,
            CONTRACT
        );
    }

    #[test]
    fn suggestions_keep_the_original_and_flag_what_disagrees() {
        let result = Json::obj([
            (
                "results",
                Json::Array(vec![
                    row("Trade debtors", "accepted_fact", "current_assets"),
                    row("Sales!", "approved", "expenses"),
                ]),
            ),
            ("status", Json::str("COMPLETE")),
        ]);
        let (status, suggestions, notes) = interpret(&lines(), &answered(result));
        assert_eq!(status, "INTERPRETED");
        assert!(suggestions[0].findings.is_empty() && !suggestions[0].topology_disagrees());
        assert_eq!(suggestions[1].findings.len(), 2);
        assert!(suggestions[1].topology_disagrees() && suggestions[1].needs_review());
        assert_eq!(
            suggestions[1].to_json().at("review").at("applied"),
            &Json::Bool(false)
        );
        assert_eq!(
            notes[0],
            pts("the provider's own status field read 'COMPLETE'")
        );
    }

    #[test]
    fn a_malformed_response_proposes_nothing() {
        let short = Json::obj([(
            "results",
            Json::Array(vec![row("Trade debtors", "draft_fact", "")]),
        )]);
        let failed = |result: Json| {
            let (status, suggestions, _) = interpret(&lines(), &answered(result));
            status == "CONTRACT_FAILURE" && suggestions.is_empty()
        };
        assert!(failed(short));
        assert!(failed(Json::obj([])));
        assert!(failed(Json::obj([(
            "results",
            Json::Array(vec![row("a", "draft_fact", ""), Json::Null])
        ),])));
        let imbalance = Json::obj([
            (
                "results",
                Json::Array(vec![row("a", "draft_fact", ""), row("b", "draft_fact", "")]),
            ),
            ("equilibrium_valid", Json::Bool(false)),
        ]);
        assert!(failed(imbalance));
    }

    #[test]
    fn a_code_needs_a_reviewed_crosswalk_before_it_means_anything() {
        assert_eq!(
            require_crosswalk(None, "sbrm_1200")
                .expect_err("refused")
                .class,
            CONTRACT
        );
        assert!(require_crosswalk(Some(&[]), "sbrm_1200").is_err());
        let crosswalk = [("sbrm_1200".to_owned(), "1-1200".to_owned())];
        assert_eq!(
            require_crosswalk(Some(&crosswalk), "sbrm_1200").unwrap(),
            "1-1200"
        );
        assert!(require_crosswalk(Some(&crosswalk), "sbrm_9999").is_err());
    }

    #[test]
    fn without_a_client_the_classifier_is_not_called() {
        let (status, suggestions, _) = classify("company", &lines(), None).expect("a result");
        assert_eq!((status.as_str(), suggestions.len()), ("NOT_RUN", 0));
    }

    /// `dict(crosswalk)`: the last entry for a code stands.
    #[test]
    fn a_repeated_crosswalk_code_takes_its_last_entry() {
        let crosswalk = [
            ("sbrm_1200".to_owned(), "1".to_owned()),
            ("lodgeit_4000".to_owned(), "4".to_owned()),
            ("sbrm_1200".to_owned(), "2".to_owned()),
            ("sbrm_1200".to_owned(), "3".to_owned()),
        ];
        assert_eq!(
            require_crosswalk(Some(&crosswalk), "sbrm_1200").unwrap(),
            "3"
        );
        assert_eq!(
            require_crosswalk(Some(&crosswalk), "lodgeit_4000").unwrap(),
            "4"
        );
    }
}
