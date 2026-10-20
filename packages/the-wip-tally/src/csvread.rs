//! `csv.reader(open(path, encoding="utf-8-sig", newline=""))` as CPython runs it.
//!
//! The text layer reads 8,192-byte chunks and decodes each one as it is needed,
//! so a UTF-8 error surfaces only when a row first needs its chunk, and the
//! reported byte position is relative to the bytes that decode call received
//! (the chunk plus any undecoded tail of the previous one, after a BOM).
//! Records are parsed by `_csv.c`'s state machine for the default excel
//! dialect, non-strict, with the default field size limit.

use std::fmt;

const CHUNK_SIZE: usize = 8192;
const FIELD_LIMIT: usize = 131_072;
const BOM: &[u8] = b"\xef\xbb\xbf";

/// Why a row could not be read.
#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    /// `UnicodeDecodeError`: its `start` and `reason`.
    Decode { start: usize, reason: &'static str },
    /// `csv.Error`.
    Csv(String),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Decode { start, reason } => write!(f, "byte {start} ({reason})"),
            ReadError::Csv(message) => f.write_str(message),
        }
    }
}

/// The `utf-8-sig` incremental decoder behind a newline decoder with
/// `translate=False`, fed one chunk at a time.
struct TextReader<'a> {
    data: &'a [u8],
    pos: usize,
    pending: Vec<u8>,
    first: bool,
    pending_cr: bool,
    eof: bool,
    text: Vec<char>,
    cursor: usize,
}

/// `codecs.utf_8_decode(data, "strict", final)`: (text, consumed) or the error.
fn utf8_decode(data: &[u8], final_: bool) -> Result<(String, usize), ReadError> {
    match std::str::from_utf8(data) {
        Ok(text) => Ok((text.to_string(), data.len())),
        Err(e) => {
            let start = e.valid_up_to();
            let valid = std::str::from_utf8(&data[..start]).expect("valid prefix");
            match e.error_len() {
                None if !final_ => Ok((valid.to_string(), start)),
                None => Err(ReadError::Decode {
                    start,
                    reason: "unexpected end of data",
                }),
                Some(_) => Err(ReadError::Decode {
                    start,
                    reason: if (0xc2..=0xf4).contains(&data[start]) {
                        "invalid continuation byte"
                    } else {
                        "invalid start byte"
                    },
                }),
            }
        }
    }
}

impl<'a> TextReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        TextReader {
            data,
            pos: 0,
            pending: Vec::new(),
            first: true,
            pending_cr: false,
            eof: false,
            text: Vec::new(),
            cursor: 0,
        }
    }

    /// `utf_8_sig.IncrementalDecoder._buffer_decode`.
    fn buffer_decode(&mut self, data: &[u8], final_: bool) -> Result<(String, usize), ReadError> {
        if self.first {
            if data.len() < 3 {
                if BOM.starts_with(data) {
                    // Not enough data to decide; even at the end this decodes nothing.
                    return Ok((String::new(), 0));
                }
                self.first = false;
            } else {
                self.first = false;
                if &data[..3] == BOM {
                    let (text, consumed) = utf8_decode(&data[3..], final_)?;
                    return Ok((text, consumed + 3));
                }
            }
        }
        utf8_decode(data, final_)
    }

    /// TextIOWrapper's `read_chunk`: one raw read, decoded through both decoders.
    fn read_chunk(&mut self) -> Result<(), ReadError> {
        let end = (self.pos + CHUNK_SIZE).min(self.data.len());
        let chunk = &self.data[self.pos..end];
        self.pos = end;
        let final_ = chunk.is_empty();
        let mut input = std::mem::take(&mut self.pending);
        input.extend_from_slice(chunk);
        let (mut output, consumed) = self.buffer_decode(&input, final_)?;
        self.pending = input[consumed..].to_vec();
        // IncrementalNewlineDecoder(translate=False) holds back a trailing \r.
        if self.pending_cr && (!output.is_empty() || final_) {
            output.insert(0, '\r');
            self.pending_cr = false;
        }
        if output.ends_with('\r') && !final_ {
            output.pop();
            self.pending_cr = true;
        }
        self.text.extend(output.chars());
        self.eof = final_;
        Ok(())
    }

    /// `readline()` with universal line endings left untranslated; None at the end.
    fn readline(&mut self) -> Result<Option<Vec<char>>, ReadError> {
        loop {
            let rest = &self.text[self.cursor..];
            let end = rest
                .iter()
                .position(|c| *c == '\n' || *c == '\r')
                .and_then(|i| {
                    if rest[i] == '\n' {
                        Some(i + 1)
                    } else if i + 1 < rest.len() {
                        Some(if rest[i + 1] == '\n' { i + 2 } else { i + 1 })
                    } else if self.eof {
                        Some(i + 1)
                    } else {
                        None
                    }
                });
            if let Some(end) = end {
                let line = rest[..end].to_vec();
                self.cursor += end;
                return Ok(Some(line));
            }
            if self.eof {
                if rest.is_empty() {
                    return Ok(None);
                }
                let line = rest.to_vec();
                self.cursor = self.text.len();
                return Ok(Some(line));
            }
            self.read_chunk()?;
            // Keep memory bounded to the unread text.
            if self.cursor > 0 {
                self.text.drain(..self.cursor);
                self.cursor = 0;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    StartRecord,
    StartField,
    InField,
    InQuotedField,
    QuoteInQuotedField,
    EatCrnl,
}

/// A character of a line, or the end-of-line marker the C reader appends.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Input {
    Char(char),
    Eol,
}

/// `csv.reader` over the decoded text.
pub struct Reader<'a> {
    text: TextReader<'a>,
    state: State,
    field: Vec<char>,
    fields: Vec<String>,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader {
            text: TextReader::new(data),
            state: State::StartRecord,
            field: Vec::new(),
            fields: Vec::new(),
        }
    }

    fn save_field(&mut self) {
        self.fields.push(self.field.drain(..).collect());
    }

    fn add_char(&mut self, c: char) -> Result<(), ReadError> {
        if self.field.len() >= FIELD_LIMIT {
            return Err(ReadError::Csv(format!(
                "field larger than field limit ({FIELD_LIMIT})"
            )));
        }
        self.field.push(c);
        Ok(())
    }

    fn end_of_line(&mut self, input: Input) {
        self.save_field();
        self.state = if input == Input::Eol {
            State::StartRecord
        } else {
            State::EatCrnl
        };
    }

    /// `parse_process_char` for the excel dialect.
    fn process(&mut self, input: Input) -> Result<(), ReadError> {
        let newline = matches!(input, Input::Char('\n' | '\r'));
        if self.state == State::StartRecord {
            if input == Input::Eol {
                return Ok(());
            }
            if newline {
                self.state = State::EatCrnl;
                return Ok(());
            }
            self.state = State::StartField;
        }
        match self.state {
            State::StartField => match input {
                Input::Eol | Input::Char('\n' | '\r') => self.end_of_line(input),
                Input::Char('"') => self.state = State::InQuotedField,
                Input::Char(',') => self.save_field(),
                Input::Char(c) => {
                    self.add_char(c)?;
                    self.state = State::InField;
                }
            },
            State::InField => match input {
                Input::Eol | Input::Char('\n' | '\r') => self.end_of_line(input),
                Input::Char(',') => {
                    self.save_field();
                    self.state = State::StartField;
                }
                Input::Char(c) => self.add_char(c)?,
            },
            State::InQuotedField => match input {
                Input::Eol => {}
                Input::Char('"') => self.state = State::QuoteInQuotedField,
                Input::Char(c) => self.add_char(c)?,
            },
            State::QuoteInQuotedField => match input {
                Input::Char('"') => {
                    self.add_char('"')?;
                    self.state = State::InQuotedField;
                }
                Input::Char(',') => {
                    self.save_field();
                    self.state = State::StartField;
                }
                Input::Eol | Input::Char('\n' | '\r') => self.end_of_line(input),
                Input::Char(c) => {
                    self.add_char(c)?;
                    self.state = State::InField;
                }
            },
            State::EatCrnl => match input {
                Input::Char('\n' | '\r') => {}
                Input::Eol => self.state = State::StartRecord,
                Input::Char(_) => {
                    return Err(ReadError::Csv(
                        "new-line character seen in unquoted field - \
                         do you need to open the file with newline=''?"
                            .into(),
                    ));
                }
            },
            State::StartRecord => unreachable!("handled above"),
        }
        Ok(())
    }

    /// `next(reader)`: the next record, or None at the end of the input.
    pub fn next_row(&mut self) -> Result<Option<Vec<String>>, ReadError> {
        // parse_reset
        self.fields.clear();
        self.field.clear();
        self.state = State::StartRecord;
        loop {
            let Some(line) = self.text.readline()? else {
                if !self.field.is_empty() || self.state == State::InQuotedField {
                    // Non-strict: the open field is saved at the end of the data.
                    self.save_field();
                    self.state = State::StartRecord;
                    return Ok(Some(std::mem::take(&mut self.fields)));
                }
                return Ok(None);
            };
            for c in line {
                self.process(Input::Char(c))?;
            }
            self.process(Input::Eol)?;
            if self.state == State::StartRecord {
                return Ok(Some(std::mem::take(&mut self.fields)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(data: &[u8]) -> Result<Vec<Vec<String>>, ReadError> {
        let mut reader = Reader::new(data);
        let mut out = Vec::new();
        while let Some(row) = reader.next_row()? {
            out.push(row);
        }
        Ok(out)
    }

    #[test]
    fn records_follow_the_c_reader() {
        let got =
            rows(b"\xef\xbb\xbfa,\"b,\"\"c\"\"\"\r\n\r\nx\"y,\"q\"z\n\"multi\nline\",end").unwrap();
        let want: Vec<Vec<String>> = vec![
            vec!["a".into(), "b,\"c\"".into()],
            vec![],
            vec!["x\"y".into(), "qz".into()],
            vec!["multi\nline".into(), "end".into()],
        ];
        assert_eq!(got, want);
        assert_eq!(rows(b"\"open").unwrap(), vec![vec!["open".to_string()]]);
    }

    #[test]
    fn decode_errors_are_chunk_relative() {
        let mut data = b"a,b\n".to_vec();
        data.extend(std::iter::repeat_n(b'x', 9000));
        data.extend(b"\xff\n");
        assert_eq!(
            rows(&data),
            Err(ReadError::Decode {
                start: 812,
                reason: "invalid start byte"
            })
        );
        assert_eq!(
            rows(b"\xef\xbb\xbfa,\xff\n"),
            Err(ReadError::Decode {
                start: 2,
                reason: "invalid start byte"
            })
        );
        assert_eq!(
            rows(b"a,b\n\xe2\x82"),
            Err(ReadError::Decode {
                start: 0,
                reason: "unexpected end of data"
            })
        );
        assert_eq!(rows(b"\xef\xbb").unwrap(), Vec::<Vec<String>>::new());
    }

    #[test]
    fn oversized_fields_are_refused() {
        let data = vec![b'x'; FIELD_LIMIT + 1];
        assert_eq!(
            rows(&data),
            Err(ReadError::Csv(
                "field larger than field limit (131072)".into()
            ))
        );
    }
}
