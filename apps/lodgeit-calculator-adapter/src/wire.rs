//! `decimals.py`: exact decimal handling on the wire, in both directions.
//!
//! A response's money is read from decimal strings and nothing else; a request
//! carries each `Decimal` as its own digits, never through a float.
use crate::decimal::Decimal;
use crate::p;
use crate::pyjson::Json;
use crate::pyops::{S, pts, strip};
use crate::pyvalue::repr;

pub const MAX_SCALE: i128 = 10;

/// `DecimalWireError`, a `ValueError`, by its `str()`.
#[derive(Clone, Debug, PartialEq)]
pub struct WireError(pub S);

/// `parse_decimal(value, field)`: a decimal string to a Decimal, exactly.
pub fn parse_decimal(value: &Json, field: &[u32]) -> Result<Decimal, WireError> {
    let r = repr(value);
    match value {
        Json::Bool(_) | Json::Float(_) => Err(WireError(p!(
            field,
            ": ",
            r,
            " arrived as a JSON number. This adapter reads money and rates as decimal \
             strings, because a float has already lost precision."
        ))),
        Json::Int(i) => Ok(Decimal::from_bigint(i)),
        Json::Str(s) if !strip(&s.code_points()).is_empty() => {
            let points = s.code_points();
            let parsed = Decimal::parse_points(strip(&points))
                .map_err(|_| WireError(p!(field, ": ", r, " is not a decimal")))?;
            if !parsed.is_finite() {
                return Err(WireError(p!(field, ": ", r, " is not finite")));
            }
            Ok(parsed)
        }
        _ => Err(WireError(p!(field, ": ", r, " is not a decimal string"))),
    }
}

/// `check_writable(value, field)`: refuse a Decimal that cannot go on the wire
/// as exact digits.
pub fn check_writable(value: &Decimal, field: &[u32]) -> Result<(), WireError> {
    let text = value.to_sci_string();
    if !value.is_finite() {
        return Err(WireError(p!(field, ": ", text, " is not finite")));
    }
    let limit = Decimal::parse_exact("1e15").expect("a decimal literal");
    if !value.copy_abs().lt(&limit).expect("finite values compare") {
        return Err(WireError(p!(
            field,
            ": ",
            text,
            " exceeds the supported magnitude"
        )));
    }
    if value.exponent() < -MAX_SCALE {
        return Err(WireError(p!(
            field,
            ": ",
            text,
            format!(" has more than {MAX_SCALE} decimal places")
        )));
    }
    Ok(())
}

/// `format_number(value)`: a Decimal as JSON number text, with no exponent.
pub fn format_number(value: &Decimal) -> String {
    let text = value.fixed_format();
    if text == "-0" || text == "-0.0" {
        return "0".into();
    }
    text
}

/// `_escape(text)`: a JSON string with only the escapes JSON requires. A lone
/// surrogate is written as itself, for the UTF-8 encode to refuse.
fn escape(points: &[u32], out: &mut S) {
    out.push(u32::from('"'));
    for &cp in points {
        match cp {
            0x22 => out.extend(pts("\\\"")),
            0x5c => out.extend(pts("\\\\")),
            0x0a => out.extend(pts("\\n")),
            0x0d => out.extend(pts("\\r")),
            0x09 => out.extend(pts("\\t")),
            0..0x20 => out.extend(pts(&format!("\\u{cp:04x}"))),
            _ => out.push(cp),
        }
    }
    out.push(u32::from('"'));
}

/// `dumps(value, field=field)`: JSON with each Decimal as its own exact digits.
pub fn dumps(value: &Json, field: &[u32]) -> Result<S, WireError> {
    let mut out = Vec::new();
    dump(value, field, &mut out)?;
    Ok(out)
}

fn dump(value: &Json, field: &[u32], out: &mut S) -> Result<(), WireError> {
    match value {
        Json::Null => out.extend(pts("null")),
        Json::Bool(b) => out.extend(pts(if *b { "true" } else { "false" })),
        Json::Dec(d) => {
            check_writable(d, field)?;
            out.extend(pts(&format_number(d)));
        }
        Json::Int(i) => out.extend(pts(&i.to_string())),
        Json::Float(_) => {
            return Err(WireError(p!(
                field,
                ": a float reached the serialiser. Use Decimal so the digits on the wire \
                 are the digits you meant."
            )));
        }
        Json::Str(s) => escape(&s.code_points(), out),
        Json::Array(items) => {
            out.push(u32::from('['));
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(u32::from(','));
                }
                dump(item, &p!(field, format!("[{i}]")), out)?;
            }
            out.push(u32::from(']'));
        }
        Json::Object(pairs) => {
            out.push(u32::from('{'));
            for (i, (key, item)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(u32::from(','));
                }
                let key = key.code_points();
                escape(&key, out);
                out.push(u32::from(':'));
                dump(item, &p!(field, ".", key), out)?;
            }
            out.push(u32::from('}'));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pyjson::{Numbers, loads_numbers};

    fn body(text: &str) -> Json {
        loads_numbers(text, Numbers::DecimalFloats).unwrap()
    }

    fn text(points: S) -> String {
        points.into_iter().filter_map(char::from_u32).collect()
    }

    #[test]
    fn decimals_go_on_the_wire_as_their_own_digits() {
        let wire = dumps(
            &body(r#"{"a": 55000.10, "b": 1E+3, "c": -0.0, "d": "x\n", "e": [1, true, null]}"#),
            &pts("body"),
        )
        .unwrap();
        assert_eq!(
            text(wire),
            r#"{"a":55000.10,"b":1000,"c":0,"d":"x\n","e":[1,true,null]}"#
        );
        let refused = |t: &str| text(dumps(&body(t), &pts("body")).unwrap_err().0);
        assert_eq!(
            refused(r#"{"a": 0.00000000000}"#),
            "body.a: 0E-11 has more than 10 decimal places"
        );
        assert_eq!(
            refused(r#"{"a": [1e15]}"#),
            "body.a[0]: 1E+15 exceeds the supported magnitude"
        );
        assert_eq!(
            refused(r#"{"a": NaN}"#),
            "body.a: a float reached the serialiser. Use Decimal so the digits on the wire are the digits you meant."
        );
    }

    #[test]
    fn responses_are_read_from_decimal_strings_only() {
        let field = pts("x");
        let read = |t: &str| parse_decimal(&crate::pyjson::loads_standard(t).unwrap(), &field);
        assert_eq!(read(r#"" 12.50 ""#).unwrap().to_sci_string(), "12.50");
        assert_eq!(
            text(read("1.5").unwrap_err().0),
            "x: 1.5 arrived as a JSON number. This adapter reads money and rates as decimal strings, because a float has already lost precision."
        );
        assert_eq!(
            text(read(r#""  ""#).unwrap_err().0),
            "x: '  ' is not a decimal string"
        );
        assert_eq!(
            text(read(r#""1,0""#).unwrap_err().0),
            "x: '1,0' is not a decimal"
        );
        assert_eq!(
            text(read(r#""Infinity""#).unwrap_err().0),
            "x: 'Infinity' is not finite"
        );
    }
}
