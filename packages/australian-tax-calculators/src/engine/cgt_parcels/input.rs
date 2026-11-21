// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reading `cgt_parcels`'s parcels and disposals, each fact checked once.

use super::*;

/// The stated facts of one parcel or disposal, each read once.
struct Fields<'a> {
    what: String,
    fields: &'a Map<String, Value>,
    taken: Vec<&'static str>,
}

impl<'a> Fields<'a> {
    fn invalid<T>(&self, key: &str, expected: &str) -> Result<T> {
        refuse(
            "cgt_parcels.fact_invalid",
            format!("{}: {key} must be {expected}.", self.what),
        )
    }

    fn get(&mut self, key: &'static str) -> Option<&'a Value> {
        self.taken.push(key);
        self.fields.get(key)
    }

    fn required(&mut self, key: &'static str) -> Result<&'a Value> {
        let fields = self.fields;
        match fields.get(key) {
            Some(value) => {
                self.taken.push(key);
                Ok(value)
            }
            None => refuse(
                "cgt_parcels.fact_missing",
                format!("{}: {key} is required.", self.what),
            ),
        }
    }

    fn text(&mut self, key: &'static str) -> Result<String> {
        match self.required(key)?.as_str() {
            Some(text) if !text.trim().is_empty() => Ok(text.to_string()),
            _ => self.invalid(key, "a non-empty string"),
        }
    }

    fn date(&mut self, key: &'static str) -> Result<i64> {
        match self.required(key)?.as_str().and_then(dates::parse) {
            Some(day) => Ok(day),
            None => self.invalid(key, "a YYYY-MM-DD date from 1900 to 2100"),
        }
    }

    fn decimal(value: Option<&Value>) -> Option<Decimal> {
        value?.as_str().and_then(|text| Decimal::parse(text).ok())
    }

    /// An amount in AUD; nil when an optional one is absent.
    fn amount(&mut self, key: &'static str, optional: bool) -> Result<Decimal> {
        let value = if optional {
            match self.get(key) {
                None => return Ok(int(0)),
                given => given,
            }
        } else {
            Some(self.required(key)?)
        };
        match Self::decimal(value).map(|v| money(&Arg::Decimal(v))) {
            Some(Ok(amount)) => Ok(amount),
            _ => self.invalid(
                key,
                "a finite non-negative decimal string, at most 2dp and 1e12",
            ),
        }
    }

    /// A number of shares or units: above nil, at most 6dp and 1e12.
    fn units(&mut self, key: &'static str) -> Result<Decimal> {
        let value = Self::decimal(Some(self.required(key)?));
        match value {
            Some(v) if v.is_finite() && v.exponent() >= -6 => {
                if v.gt(&int(0))? && !v.gt(&Decimal::parse("1e12")?)? {
                    return Ok(v);
                }
                self.invalid(key, "a decimal string above 0, at most 6dp and 1e12")
            }
            _ => self.invalid(key, "a decimal string above 0, at most 6dp and 1e12"),
        }
    }

    fn none_left(&self) -> Result<()> {
        match self
            .fields
            .keys()
            .find(|key| !self.taken.contains(&key.as_str()))
        {
            Some(key) => refuse(
                "input.field_unknown",
                format!(
                    "{}: {key} is not a fact it takes beside its other facts.",
                    self.what
                ),
            ),
            None => Ok(()),
        }
    }
}

fn list_invalid<T>(name: &str) -> Result<T> {
    refuse(
        "cgt_parcels.list_invalid",
        format!("{name} must be a list of 1 to {MOST} objects."),
    )
}

/// The objects of a non-empty list of at most [`MOST`].
pub(super) fn objects<'a>(value: &'a Arg, name: &str) -> Result<Vec<&'a Map<String, Value>>> {
    let Arg::Json(Value::Array(list)) = value else {
        return list_invalid(name);
    };
    if list.is_empty() || list.len() > MOST {
        return list_invalid(name);
    }
    list.iter()
        .map(|v| match v.as_object() {
            Some(fields) => Ok(fields),
            None => list_invalid(name),
        })
        .collect()
}

pub(super) fn read_parcel(n: usize, fields: &Map<String, Value>) -> Result<Parcel> {
    let mut f = Fields {
        what: format!("parcel {n}"),
        fields,
        taken: Vec::new(),
    };
    let id = f.text("parcel")?;
    let security = f.text("security")?;
    let acquired = f.date("acquired")?;
    let units = f.units("units")?;
    let mut base = f.amount("acquisition_cost", false)?;
    base = base.add(&f.amount("incidental_costs", false)?)?;
    let owning = f.amount("ownership_costs", true)?;
    base = base.add(&f.amount("capital_expenditure", true)?)?;
    base = base.add(&f.amount("title_costs", true)?)?;
    f.none_left()?;
    Ok(Parcel {
        id,
        security,
        acquired,
        units,
        base,
        owning,
    })
}

fn read_picks(f: &mut Fields) -> Result<Vec<Pick>> {
    let what = f.what.clone();
    let bad = || {
        refuse(
            "cgt_parcels.fact_invalid",
            format!(
                "{what}: parcels must be a non-empty list of objects, each with parcel and units."
            ),
        )
    };
    let Value::Array(list) = f.required("parcels")? else {
        return bad();
    };
    if list.is_empty() {
        return bad();
    }
    let mut picks = Vec::new();
    for (i, value) in list.iter().enumerate() {
        let Some(fields) = value.as_object() else {
            return bad();
        };
        let mut pick = Fields {
            what: format!("{what}, selected parcel {}", i + 1),
            fields,
            taken: Vec::new(),
        };
        let parcel = pick.text("parcel")?;
        let units = pick.units("units")?;
        pick.none_left()?;
        if picks.iter().any(|p: &Pick| p.parcel == parcel) {
            return conflict(format!("{what}: parcel {parcel} is selected twice."));
        }
        picks.push(Pick { parcel, units });
    }
    Ok(picks)
}

pub(super) fn read_disposal(
    n: usize,
    fields: &Map<String, Value>,
    selected: bool,
) -> Result<Disposal> {
    let mut f = Fields {
        what: format!("disposal {n}"),
        fields,
        taken: Vec::new(),
    };
    let security = f.text("security")?;
    let date = f.date("date")?;
    let units = f.units("units")?;
    let proceeds = f.amount("capital_proceeds", false)?;
    let selling = f.amount("incidental_costs", false)?;
    let picks = if selected {
        Some(read_picks(&mut f)?)
    } else {
        None
    };
    f.none_left()?;
    Ok(Disposal {
        n,
        security,
        date,
        units,
        proceeds,
        selling,
        picks,
    })
}
