// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The entertainment classifier's 2 decision tables: [`CHARACTER`] gives a
//! line's character from what was provided and on what occasion, and
//! [`TREATMENT`]'s first matching row gives each attendee class's outcome.

/// Whether a line is entertainment, and of which kind.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Character {
    /// Meal entertainment (FBTAA s 37AD), which Division 9A can value.
    Meal,
    /// Entertainment by way of recreation, or travel for it: never meal
    /// entertainment, so Division 9A does not reach it.
    Recreation,
    NotEntertainment,
}
use Character::{Meal, NotEntertainment, Recreation};

/// TR 97/17's tests of why, what, when and where food or drink is provided
/// (para 23), applied to the occasion the operator states: (provided,
/// occasion, character, authority). Food or drink with no occasion stated is
/// left undetermined.
pub(super) const CHARACTER: [(&str, Option<&str>, Character, &str); 8] = [
    (
        "food_or_drink",
        Some("social_function"),
        Meal,
        "ITAA 1997 s 32-10(1)(a) and FBTAA s 37AD(a): food or drink at a party, reception or other social function is entertainment whatever is served (TR 97/17 paras 20, 50 and 62, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("business_meal"),
        Meal,
        "ITAA 1997 s 32-10(1)(a) and (2) and FBTAA s 37AD(a) and (d): a business lunch, dinner or drinks, or another meal beyond light refreshments for attendees not travelling for work, is entertainment even if business is discussed (TR 97/17 paras 20, 23(b), 58, 93 and 138.1, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("with_other_entertainment"),
        Meal,
        "ITAA 1997 s 32-10(1)(a) and FBTAA s 37AD(a): food or drink with other entertainment, such as a floor show, is entertainment (TR 97/17 paras 71, 77 and 119, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("light_refreshments"),
        NotEntertainment,
        "ITAA 1997 s 32-10(1): morning or afternoon tea, light meals or finger food, not at a social function, confer no entertainment (TR 97/17 paras 18, 19, 23(b), 56, 69 and 114, the Commissioner's view)",
    ),
    (
        "food_or_drink",
        Some("travel_meal"),
        NotEntertainment,
        "ITAA 1997 s 32-10(1) note: a meal of attendees all travelling for work or business, with no other entertainment, is not entertainment (TR 97/17 paras 21, 67, 71, 76, 82 and 86, the Commissioner's view)",
    ),
    (
        "recreation",
        None,
        Recreation,
        "ITAA 1997 s 32-10(1)(a): entertainment by way of recreation, which includes amusement, sport or similar leisure-time pursuits (s 995-1); not meal entertainment, which FBTAA s 37AD limits to food or drink (TR 97/17 para 8(b))",
    ),
    (
        FOOD_TRAVEL,
        None,
        Meal,
        "ITAA 1997 s 32-10(1)(b) and FBTAA s 37AD(b): accommodation or travel to do with entertainment by way of food or drink",
    ),
    (
        "accommodation_or_travel_for_recreation",
        None,
        Recreation,
        "ITAA 1997 s 32-10(1)(b): accommodation or travel to do with entertainment by way of recreation; not meal entertainment (FBTAA s 37AD(b))",
    ),
];

/// Accommodation or travel to do with food or drink: entertainment only when
/// the food or drink is, so it takes the food or drink's occasion.
pub(super) const FOOD_TRAVEL: &str = "accommodation_or_travel_for_food_or_drink";
pub(super) const TRAVEL_NOT_ENTERTAINMENT: &str = "ITAA 1997 s 32-10(1)(b) and FBTAA s 37AD(b): accommodation or travel is entertainment only when it is to do with entertainment by way of food, drink or recreation, and the food or drink it is to do with is not entertainment on the occasion stated (TR 97/17 para 23, the Commissioner's view)";

/// The fact an undetermined food or drink line, or accommodation or travel
/// line to do with food or drink, lacks.
pub(super) const OCCASION_MISSING: &str = "occasion: whether food or drink, or accommodation or travel to do with it, is entertainment turns on why, what, when and where the food or drink is provided (ITAA 1997 s 32-10(1); TR 97/17 paras 7 and 23); state social_function, business_meal, with_other_entertainment, light_refreshments or travel_meal, or split the line until one fits";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Class {
    Employees,
    Associates,
    Others,
}
use Class::{Associates, Employees, Others};

pub(super) const CLASSES: [(&str, Class); 3] = [
    ("employees", Employees),
    ("associates", Associates),
    ("others", Others),
];

/// The employer's method of valuing meal entertainment for the FBT year.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Valuation {
    Actual,
    FiftyFifty,
    Register,
}
use Valuation::{FiftyFifty, Register};

/// Whether FBTAA s 41(1) can reach an employee's share: the employer provided
/// property (food or drink it paid for), consumed on its business premises
/// on a working day; it provided something else there, which may or may not
/// be property; or neither.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Premises {
    Property,
    Unsettled,
    No,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fbt {
    Arises,
    Exempt,
    NotAFringeBenefit,
    InMethodTotal,
    Undetermined,
}

/// The share of a class's cost that is deductible, and so creditable.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Share {
    Whole,
    Nil,
    Half,
    RegisterPercentage,
    Undetermined,
}

/// One row: the line's character, the employer's valuation, the attendee
/// class and the s 41 position it matches (empty matches any), and the
/// outcome it gives.
pub(super) struct Treatment {
    pub(super) characters: &'static [Character],
    pub(super) valuations: &'static [Valuation],
    pub(super) classes: &'static [Class],
    pub(super) premises: &'static [Premises],
    pub(super) fbt: Fbt,
    pub(super) share: Share,
    pub(super) authority: &'static str,
}

const ENTERTAINMENT: &[Character] = &[Meal, Recreation];
const RECIPIENTS: &[Class] = &[Employees, Associates];

/// The first row that matches decides; the order is the order of the
/// questions in TR 97/17 Part B: an election under Division 9A first (para
/// 42), then who received it, then s 41.
const TREATMENT: [Treatment; 11] = [
    Treatment {
        characters: &[Meal],
        valuations: &[FiftyFifty],
        classes: RECIPIENTS,
        premises: &[],
        fbt: Fbt::InMethodTotal,
        share: Share::Half,
        authority: "FBTAA ss 37AF and 37BA: under the 50/50 split method the line's whole cost, whoever attended, counts in the year's meal entertainment expenditure and no other fringe benefit arises, so neither s 41 nor s 58P exempts any of it (TR 97/17 paras 31 and 44; TR 2007/12 para 14, the Commissioner's view). ITAA 1936 s 51AEA(1): a deduction of 50% of the expense and no other; GST Act s 69-5(3A)(a): no GST credit for the other 50%.",
    },
    Treatment {
        characters: &[Meal],
        valuations: &[Register],
        classes: RECIPIENTS,
        premises: &[],
        fbt: Fbt::InMethodTotal,
        share: Share::RegisterPercentage,
        authority: "FBTAA s 37CB: under the 12 week register method the line's whole cost, whoever attended, counts in the year's meal entertainment expenditure, which fbt-meal-entertainment values at the register percentage; s 41 does not apply, and minor benefits count only in the register (TR 97/17 paras 32 and 44; TR 2007/12 para 15, the Commissioner's view). ITAA 1936 s 51AEB(1): a deduction of the expense times income_tax_register_percentage and no other; GST Act s 69-5(3A)(b): no GST credit for the rest.",
    },
    Treatment {
        characters: &[Meal],
        valuations: &[FiftyFifty],
        classes: &[Others],
        premises: &[],
        fbt: Fbt::NotAFringeBenefit,
        share: Share::Half,
        authority: "No fringe benefit: a fringe benefit is provided to an employee or an associate (FBTAA s 136(1)). Under the 50/50 split method the cost still counts in the year's meal entertainment expenditure, which includes meal entertainment provided to all persons (FBTAA s 37BA and its note; TR 97/17 para 31, the Commissioner's view). ITAA 1936 s 51AEA(1): a deduction of 50% of the expense and no other; GST Act s 69-5(3A)(a): no GST credit for the other 50%.",
    },
    Treatment {
        characters: &[Meal],
        valuations: &[Register],
        classes: &[Others],
        premises: &[],
        fbt: Fbt::NotAFringeBenefit,
        share: Share::RegisterPercentage,
        authority: "No fringe benefit: a fringe benefit is provided to an employee or an associate (FBTAA s 136(1)). Under the 12 week register method the cost still counts in the year's total meal entertainment expenditure, the expenses of providing meal entertainment to all persons (FBTAA s 37CB(3) and its note; TR 97/17 para 32, the Commissioner's view). ITAA 1936 s 51AEB(1): a deduction of the expense times income_tax_register_percentage and no other; GST Act s 69-5(3A)(b): no GST credit for the rest.",
    },
    Treatment {
        characters: &[NotEntertainment],
        valuations: &[],
        classes: &[Others],
        premises: &[],
        fbt: Fbt::NotAFringeBenefit,
        share: Share::Whole,
        authority: "No fringe benefit: a fringe benefit is provided to an employee or an associate (FBTAA s 136(1)). Not entertainment, so Division 32 denies no deduction and GST Act s 69-5 no GST credit (ITAA 1997 s 8-1; TR 97/17 paras 41, 48, 84 and 88, the Commissioner's view).",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: &[Others],
        premises: &[],
        fbt: Fbt::NotAFringeBenefit,
        share: Share::Nil,
        authority: "No fringe benefit: a fringe benefit is provided to an employee or an associate (FBTAA s 136(1)), so ITAA 1997 s 32-20 does not lift s 32-5 and none of it is deductible (TR 97/17 paras 52, 96 and 127, the Commissioner's view); no GST credit (GST Act s 69-5(1) and (3)(f)).",
    },
    Treatment {
        characters: &[NotEntertainment],
        valuations: &[],
        classes: &[Employees],
        premises: &[Premises::Property],
        fbt: Fbt::Exempt,
        share: Share::Whole,
        authority: "FBTAA s 41(1): property provided to and consumed by a current employee on the employer's business premises on a working day is an exempt benefit. Not entertainment, so deductible with its GST credit (ITAA 1997 s 8-1; TR 97/17 paras 38, 41 and 56, the Commissioner's view).",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: &[Employees],
        premises: &[Premises::Property],
        fbt: Fbt::Exempt,
        share: Share::Nil,
        authority: "FBTAA s 41(1): property provided to and consumed by a current employee on the employer's business premises on a working day is an exempt benefit (TR 97/17 paras 27, 44 and 50, the Commissioner's view). An exempt benefit is not a fringe benefit (FBTAA s 136(1)), so ITAA 1997 s 32-20 does not lift s 32-5 and none of it is deductible (TR 97/17 paras 36, 46 and 50); no GST credit (GST Act s 69-5(1) and (3)(f)).",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: &[Employees],
        premises: &[Premises::Unsettled],
        fbt: Fbt::Undetermined,
        share: Share::Undetermined,
        authority: "FBTAA s 41(1) exempts only property consumed on the business premises on a working day; if it applies, ITAA 1997 s 32-5 denies the deduction, and if not, s 32-20 allows it.",
    },
    Treatment {
        characters: ENTERTAINMENT,
        valuations: &[],
        classes: RECIPIENTS,
        premises: &[],
        fbt: Fbt::Arises,
        share: Share::Whole,
        authority: "A fringe benefit (FBTAA s 136(1)): s 41 reaches only an employee's property consumed on business premises on a working day, and never an associate's (TR 97/17 paras 45, 51 and 62, the Commissioner's view). Deductible because it is a fringe benefit (ITAA 1997 s 32-20; TR 97/17 para 26), with its GST credit.",
    },
    Treatment {
        characters: &[NotEntertainment],
        valuations: &[],
        classes: RECIPIENTS,
        premises: &[],
        fbt: Fbt::Arises,
        share: Share::Whole,
        authority: "A fringe benefit (FBTAA s 136(1)), whose taxable value the otherwise deductible rule can reduce for an employee (FBTAA ss 24 and 44; TR 97/17 paras 39 and 71, the Commissioner's view). Not entertainment, so deductible with its GST credit (ITAA 1997 s 8-1; TR 97/17 para 41).",
    },
];

/// The fact an undetermined s 41 row lacks.
pub(super) const PROPERTY_MISSING: &str = "whether what the employer provided is property its employees consumed on the business premises (FBTAA ss 40 and 41(1)): if so it is exempt and not deductible, and if not it is a fringe benefit and deductible";

/// Whether a fringe benefit is left out of reporting, as `fbt-return` takes it.
pub(super) const MEAL_EXCLUDED: &str = "An excluded fringe benefit, left out of the employee's individual fringe benefits amount and so not reportable: meal entertainment (FBTAA s 37AD) not provided under a salary packaging arrangement, whether or not Division 9A is elected (FBTAA s 5E(2) and (3)(a); TR 97/17 Part A table, column R, the Commissioner's view).";
pub(super) const NOT_EXCLUDED: &str = "Not an excluded fringe benefit: it is not meal entertainment (FBTAA s 5E(3)(a)), and within this classifier's scope no other paragraph of s 5E(3), nor regulations 6 to 11 of the Fringe Benefits Tax Assessment Regulations 2018, covers it. Its taxable value, after any reduction, counts in the employee's individual fringe benefits amount (s 5E(2)), which is reportable when more than $2,000 (s 135P(1)); TR 97/17 Part A table, column R, the Commissioner's view.";

pub(super) fn treatment(
    character: Character,
    valuation: Valuation,
    class: Class,
    premises: Premises,
) -> &'static Treatment {
    fn has<T: PartialEq>(set: &[T], value: T) -> bool {
        set.is_empty() || set.contains(&value)
    }
    TREATMENT
        .iter()
        .find(|row| {
            has(row.characters, character)
                && has(row.valuations, valuation)
                && has(row.classes, class)
                && has(row.premises, premises)
        })
        .expect("a row for every case")
}

/// The fringe benefit's type: (type, authority). On a reimbursed line only
/// one employee was reimbursed: any other employee's share is a benefit to
/// that employee, not part of the expense payment.
pub(super) fn benefit_type(
    provided: &str,
    reimbursed: bool,
    (class, heads): (Class, Option<i64>),
) -> (&'static str, &'static str) {
    let shared = reimbursed && class == Employees && heads != Some(1);
    match (shared, reimbursed, provided) {
        (true, ..) if heads.is_none() => (
            "undetermined",
            "The number of employees is not stated: the reimbursed employee's own share is an expense payment benefit (FBTAA s 20) and any other employee's share is a benefit to that employee (TR 97/17 para 98, the Commissioner's view). Give the head count, or the reimbursed employee's share as its own line.",
        ),
        (true, _, "food_or_drink") => (
            "expense_payment_and_property",
            "Where more than one employee attended: FBTAA s 20 for the reimbursed employee's own share, its taxable value reduced under s 63A by the part for the others entertained; each other employee's share is a property benefit to that employee (s 40) (TR 97/17 paras 98 and 100, the Commissioner's view). Give the reimbursed employee's share as its own line to separate them.",
        ),
        (true, _, _) => (
            "expense_payment_and_property_or_residual",
            "Where more than one employee attended: FBTAA s 20 for the reimbursed employee's own share, its taxable value reduced under s 63A by the part for the others entertained; each other employee's share is a benefit to that employee, a property benefit (s 40) if what was provided is property, otherwise a residual benefit (s 45) (TR 97/17 para 98, the Commissioner's view). Give the reimbursed employee's share as its own line to separate them.",
        ),
        (false, true, _) => (
            "expense_payment",
            "FBTAA s 20: the employer reimbursed or paid for the attendee who incurred it; s 63A moves the part for others entertained to them (TR 97/17 paras 73 and 98, the Commissioner's view)",
        ),
        (false, false, "food_or_drink") => (
            "property",
            "FBTAA s 40: food or drink is tangible property (s 136(1)); paying by corporate card does not change that (TR 97/17 paras 62 and 129, the Commissioner's view)",
        ),
        _ => (
            "property_or_residual",
            "FBTAA s 40 if what was provided is property (s 136(1)), otherwise a residual benefit (s 45)",
        ),
    }
}
