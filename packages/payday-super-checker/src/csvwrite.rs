//! `csv.writer` with the excel dialect: QUOTE_MINIMAL and `\r\n` line ends.

/// One row as `writer.writerow(fields)` writes it.
pub fn row(fields: &[String], out: &mut String) {
    if fields.len() == 1 && fields[0].is_empty() {
        out.push_str("\"\"\r\n");
        return;
    }
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        if field.contains([',', '"', '\r', '\n']) {
            out.push('"');
            out.push_str(&field.replace('"', "\"\""));
            out.push('"');
        } else {
            out.push_str(field);
        }
    }
    out.push_str("\r\n");
}

#[cfg(test)]
mod tests {
    #[test]
    fn quotes_minimally() {
        let mut out = String::new();
        super::row(
            &["a".into(), "b,c".into(), "d\"e".into(), String::new()],
            &mut out,
        );
        super::row(&[String::new()], &mut out);
        assert_eq!(out, "a,\"b,c\",\"d\"\"e\",\r\n\"\"\r\n");
    }
}
